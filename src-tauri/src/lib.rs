use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{AppHandle, Emitter};

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "webm", "mov", "flv", "wmv", "jpg", "jpeg", "png", "gif", "webp", "bmp",
    "svg",
];

fn scan_directory_recursive(dir: &Path) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let path = Path::new(&dir);
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path_to_media = entry.path();
            let file_extension = path_to_media.extension();

            if path_to_media.is_dir() {
                let result = scan_directory_recursive(&path_to_media)?;
                files.extend(result);
            } else if let Some(ext) = file_extension {
                if MEDIA_EXTENSIONS.contains(&ext.to_str().unwrap_or("")) {
                    files.push(path_to_media.to_string_lossy().to_string());
                }
            }
        }
    }
    return Ok(files);
}

#[tauri::command]
async fn scan_directory(dir: String) -> Result<Vec<String>, String> {
    scan_directory_recursive(Path::new(&dir))
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn generate_thumbnails(files: Vec<String>, app: AppHandle) -> Result<(), String> {
    let total = files.len();
    let counter = AtomicUsize::new(0);
    files.par_iter().for_each(|file| {
        if let Ok(img) = image::open(file) {
            let hash = blake3::hash(file.as_bytes());
            let hex_string = hash.to_hex().to_string();
            let file_name = &hex_string[..32];
            let thumb = img.thumbnail(256, 256);
            let _ = thumb.save(format!(
                "/home/alexgrist/Pictures/thumbnails/{file_name}.jpg"
            ));
            let current = counter.fetch_add(1, Ordering::Relaxed);
            app.emit("thumbnail-progress", (current, total)).unwrap();
        }
    });
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
