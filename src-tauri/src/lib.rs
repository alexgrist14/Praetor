use ffmpeg_sidecar::command::FfmpegCommand;
use rayon::prelude::*;
use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tauri::Manager;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
enum ScanEvent {
    Started,
    Progress(usize),
    Finished(Vec<String>),
}

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "webm", "mov", "flv", "wmv", "jpg", "jpeg", "png", "gif", "webp", "bmp",
    "svg",
];

const VIDEOS_EXTENSIONS: &[&str] = &["mp4", "mkv", "avi", "webm", "mov", "flv", "wmv"];

struct AppState {
    db: Mutex<Connection>,
}

fn scan_directory_recursive(
    dir: &Path,
    app: &AppHandle,
    counter: &mut usize,
) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let path = Path::new(&dir);
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            *counter += 1;
            app.emit("scan", ScanEvent::Progress(*counter)).unwrap();
            let entry = entry.map_err(|e| e.to_string())?;
            let path_to_media = entry.path();
            let file_extension = path_to_media.extension();

            if path_to_media.is_dir() {
                let result = scan_directory_recursive(&path_to_media, app, counter)?;
                files.extend(result);
            } else if let Some(ext) = file_extension {
                if MEDIA_EXTENSIONS.contains(&ext.to_str().unwrap_or("")) {
                    files.push(path_to_media.to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
async fn scan_directory(dir: String, app: AppHandle) -> Result<(), String> {
    let mut counter: usize = 0;
    let _ = app.emit("scan", ScanEvent::Started);
    let result = scan_directory_recursive(Path::new(&dir), &app, &mut counter)?;
    let _ = app.emit("scan", ScanEvent::Finished(result));
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn generate_thumbnails(
    files: Vec<String>,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let total = files.len();
    let counter = AtomicUsize::new(0);
    files.par_iter().for_each(|file| {
        let hash = blake3::hash(file.as_bytes());
        let hex_string = hash.to_hex().to_string();
        let extention = Path::new(file).extension();
        let file_name = &hex_string[..32];

        if let Some(ext) = extention {
            let media_type = if VIDEOS_EXTENSIONS.contains(&ext.to_str().unwrap_or("")) {
                "video"
            } else {
                "image"
            };
            let thumbnail_ok = if media_type == "video" {
                generate_video_thumbnail(file, file_name).is_ok()
            } else {
                image::open(file)
                    .map(|img| {
                        let thumb = img.thumbnail(256, 256);
                        thumb
                            .save(format!(
                                "/home/alexgrist/Pictures/thumbnails/{file_name}.jpg"
                            ))
                            .is_ok()
                    })
                    .unwrap_or(false)
            };
            if thumbnail_ok {
                if let Err(e) = write_media_to_db(&state, file, file_name, media_type) {
                    eprintln!("Failed to write to DB: {}", e);
                }
            }
        }

        let current = counter.fetch_add(1, Ordering::Relaxed);
        let _ = app.emit("thumbnail-progress", (current, total));
    });
    Ok(())
}

fn write_media_to_db(
    state: &tauri::State<'_, AppState>,
    file: &String,
    file_name: &str,
    media_type: &str,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT OR IGNORE INTO media (path, hash, media_type) VALUES (?1,?2,?3)",
        &[file, file_name, media_type],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn generate_video_thumbnail(file: &String, file_name: &str) -> Result<(), String> {
    FfmpegCommand::new()
        .args(&["-ss", "00:00:05"]) // Seek to 5 seconds
        .input(file)
        .args(&["-vframes", "1"]) // Grab one frame
        .args(&["-vf", "scale=256:-1"]) // Scale width to 640, keep aspect ratio
        .output(format!(
            "/home/alexgrist/Pictures/thumbnails/{file_name}.jpg"
        ))
        .overwrite() // -y
        .spawn()
        .map_err(|e| e.to_string())? // Spawn process
        .wait()
        .map_err(|e| e.to_string())?; // Wait for exit
    println!("Thumbnail generated: {}", file_name);
    Ok(())
}

fn initiate_db(app: AppHandle) -> Result<Connection, String> {
    const INIT_SQL: &str = include_str!("../db/media.sql");
    let data_dir = app.path().app_data_dir().unwrap();
    fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    let conn = Connection::open(data_dir.join("praetor.db")).map_err(|e| e.to_string())?;
    conn.execute_batch(INIT_SQL).map_err(|e| e.to_string())?;
    Ok(conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    ffmpeg_sidecar::download::auto_download().unwrap();
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            let conn = initiate_db(handle).expect("Failed to init DB");
            app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_directory,
            generate_thumbnails
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
