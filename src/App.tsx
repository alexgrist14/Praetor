import { useEffect, useState } from "react";
import "./App.css";
import { commands } from "./commands/commands";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { Progress } from "./components/ui/progress";
import { Field, FieldLabel } from "./components/ui/field";
import { Button } from "./components/ui/button";
import { ScanEvent } from "./types/events";
import { Spinner } from "./components/ui/spinner";

function App() {
  const [files, setFiles] = useState<string[] | undefined>();
  const [dirForScan, setDirForScan] = useState<string | null>("");
  const [thumbnailsProgress, setThumbnailsProgress] = useState<
    [number, number] | null
  >();
  const [scanProgress, setScanProgress] = useState(0);
  const [isScan, setIsScan] = useState(false);

  useEffect(() => {
    if (dirForScan) {
      (async () => {
        await commands.scanDirectory({ dir: dirForScan });
      })();
    }
  }, [dirForScan]);

  const handlePickDirectory = async () => {
    await open({ directory: true }).then((res) => {
      if (res) {
        setFiles(undefined);
        setThumbnailsProgress(null);
        setScanProgress(0);
      }

      setDirForScan(res);
    });
  };

  const handleGenerateThumbnails = async () => {
    if (files) {
      commands.generateThumbnails({ files });
    }
  };

  useEffect(() => {
    const promise = listen<[number, number]>("thumbnail-progress", (event) => {
      setThumbnailsProgress(event.payload);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    const promise = listen<ScanEvent>("scan", (event) => {
      const payload = event.payload;
      console.log(payload);
      if (payload === "Started") {
        setIsScan(true);
      } else if ("Progress" in payload) {
        setScanProgress(payload.Progress);
        // payload.Progress — кол-во файлов
      } else if ("Finished" in payload) {
        setIsScan(false);
        setFiles(payload.Finished);
        // payload.Finished — массив путей
      }
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, []);

  return (
    <main className="bg-accent-foreground h-full text-input">
      <form
        className="flex flex-col"
        onSubmit={(e) => {
          e.preventDefault();
        }}
      >
        <div className="flex gap-5">
          <Button type="submit" onClick={handlePickDirectory}>
            Select directory
          </Button>
          {isScan && (
            <div>
              <p>Found media: {scanProgress}</p>
              <Spinner />
            </div>
          )}
          {files && files.length > 0 && (
            <>
              <div>Total files: {files.length}</div>
              <Button onClick={handleGenerateThumbnails}>
                Generate Thumbnails
              </Button>
            </>
          )}
        </div>
        {thumbnailsProgress && (
          <Field className="w-100">
            <FieldLabel>Current progress: </FieldLabel>
            <Progress
              className="rounded-2xl"
              value={
                ((thumbnailsProgress[0] + 1) / thumbnailsProgress[1]) * 100
              }
              max={thumbnailsProgress[1]}
            />
          </Field>
        )}
      </form>
    </main>
  );
}

export default App;
