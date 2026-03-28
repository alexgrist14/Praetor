import { invoke } from "@tauri-apps/api/core";

type CommandArgs<T> = Record<string, unknown> & T;

interface GreetArgs {
  name: string;
}

const greet = async (args: CommandArgs<GreetArgs>) => {
  return invoke<string>("greet", args);
};

const scanDirectory = async (args: CommandArgs<{ dir: string }>) => {
  return invoke<string[]>("scan_directory", args);
};

const generateThumbnails = async (args: CommandArgs<{ files: string[] }>) => {
  return invoke<string[]>("generate_thumbnails", args);
};

export const commands = { greet, scanDirectory, generateThumbnails };
