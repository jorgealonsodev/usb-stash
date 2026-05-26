import { invoke } from "@tauri-apps/api/core";

export interface EntrySummary {
  id: string;
  path: string;
  size: number;
  mime_type: string;
  created_at: number;
  modified_at: number;
}

export const stashExists = (path: string): Promise<boolean> =>
  invoke<boolean>("stash_exists", { path });

export const createStash = (path: string, password: string): Promise<void> =>
  invoke<void>("create_stash", { path, password });

export const openStash = (path: string, password: string): Promise<void> =>
  invoke<void>("open_stash", { path, password });

export const lockStash = (): Promise<void> => invoke<void>("lock_stash");

export const listEntries = (): Promise<EntrySummary[]> =>
  invoke<EntrySummary[]>("list_entries");

export const addEntry = (path: string, content: number[]): Promise<string> =>
  invoke<string>("add_entry", { path, content });

export const extractEntry = (
  entryPath: string,
  output: string,
): Promise<void> => invoke<void>("extract_entry", { entryPath, output });
