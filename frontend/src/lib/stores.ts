import { writable } from "svelte/store";
import type { EntrySummary, Settings } from "./api";

export const currentStashPath = writable<string>("");

// Explorer UI stores
export const entries = writable<EntrySummary[]>([]);
export const expandedPaths = writable<Set<string>>(new Set());
export const selectedPath = writable<string | null>(null);
export const searchQuery = writable<string>("");
export const isDirty = writable<boolean>(false);

// Settings store
export const settings = writable<Settings>({ auto_lock_seconds: 300 });

// Preview overlay stores
export const previewEntry = writable<EntrySummary | null>(null);
export const previewLoading = writable<boolean>(false);
export const previewContent = writable<Uint8Array | null>(null);
