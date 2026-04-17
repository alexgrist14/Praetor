export type ScanEvent =
  | "Started"
  | { Progress: number }
  | { Finished: string[] };
