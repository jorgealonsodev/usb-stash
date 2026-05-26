import { invoke } from "@tauri-apps/api/core";

export interface EntrySummary {
  id: string;
  path: string;
  size: number;
  mime_type: string;
  created_at: number;
  modified_at: number;
}

export interface Settings {
  auto_lock_seconds: number;
}

export interface StashMetadata {
  version: number;
  format: string;
  created_at: number;
  total_entries: number;
  dat_size: number;
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

export const deleteEntry = (entryPath: string): Promise<void> =>
  invoke<void>("delete_entry", { entryPath });

export const renameEntry = (entryPath: string, newPath: string): Promise<void> =>
  invoke<void>("rename_entry", { entryPath, newPath });

export const saveStash = (): Promise<void> => invoke<void>("save_stash");

export const readEntry = (entryPath: string): Promise<Uint8Array> =>
  invoke<number[]>("read_entry", { entryPath }).then((a) => new Uint8Array(a));

// ─── Settings & Password (Phase 8) ─────────────────────────────────────────

export const changePassword = (
  oldPassword: string,
  newPassword: string,
): Promise<void> => invoke<void>("change_password", { oldPassword, newPassword });

export const getSettings = (): Promise<Settings> =>
  invoke<Settings>("get_settings");

export const updateSettings = (settings: Settings): Promise<void> =>
  invoke<void>("update_settings", { settings });

export const getStashMetadata = (): Promise<StashMetadata> =>
  invoke<StashMetadata>("get_stash_metadata");

export const exportStash = (targetPath: string): Promise<void> =>
  invoke<void>("export_stash", { targetPath });
