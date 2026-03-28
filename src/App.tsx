import { useEffect, useState } from "react";
import "./App.css";
import { commands } from "./commands/commands";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [files, setFiles] = useState<string[] | undefined>();
  const [dirForScan, setDirForScan] = useState<string | null>("");

  useEffect(() => {
    if (dirForScan) {
      (async () => {
        setFiles(await commands.scanDirectory({ dir: dirForScan }));
      })();
    }
  }, [dirForScan]);

  const handlePickDirectory = async () => {
    await open({ directory: true }).then((res) => {
      setDirForScan(res);
    });
  };

  const handleGenerateThumbnails = async () => {
    if (files) {
      commands.generateThumbnails({ files });
    }
  };

  useEffect(() => {
    const promise = listen("thumbnail-progress", (event) => {
      console.log(event.payload);
    });

    return () => {
      promise.then((unlisten) => unlisten());
    };
  }, []);

  console.log(files);

  return (
    <main className="container">
      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
        }}
      >
        <button type="submit" onClick={handlePickDirectory}>
          Select directory
        </button>
        <button onClick={handleGenerateThumbnails}>Generate Thumbnails</button>
      </form>
    </main>
  );
}

export default App;
