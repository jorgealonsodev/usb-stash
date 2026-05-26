<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { EntrySummary } from "../api";

  const dispatch = createEventDispatcher();

  export let entry: EntrySummary;

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function formatDate(ts: number): string {
    return new Date(ts * 1000).toLocaleDateString();
  }

  function getFileIcon(path: string): string {
    const ext = path.split(".").pop()?.toLowerCase() ?? "";
    const icons: Record<string, string> = {
      txt: "📄",
      md: "📄",
      pdf: "📕",
      doc: "📘",
      docx: "📘",
      jpg: "🖼️",
      jpeg: "🖼️",
      png: "🖼️",
      gif: "🖼️",
      svg: "🖼️",
      mp3: "🎵",
      wav: "🎵",
      mp4: "🎬",
      avi: "🎬",
      zip: "📦",
      tar: "📦",
      gz: "📦",
      js: "⚙️",
      ts: "⚙️",
      py: "⚙️",
      rs: "⚙️",
      go: "⚙️",
    };
    return icons[ext] ?? "📄";
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    dispatch("contextmenu_action", {
      x: e.clientX,
      y: e.clientY,
      entry,
    });
  }

  function handleDblClick() {
    dispatch("preview", entry);
  }
</script>

<tr class="file-row" on:contextmenu={handleContextMenu} on:dblclick={handleDblClick}>
  <td class="path-cell">
    <span class="icon">{getFileIcon(entry.path)}</span>
    <span class="filename" title={entry.path}>{entry.path.split("/").pop()}</span>
  </td>
  <td class="size-cell">{formatSize(entry.size)}</td>
  <td class="date-cell">{formatDate(entry.modified_at)}</td>
  <td class="actions-cell">
    <button class="btn-action" on:click={() => dispatch("contextmenu_action", { x: $event.clientX, y: $event.clientY, entry })} title="Actions">
      ⋯
    </button>
  </td>
</tr>

<style>
  .file-row {
    transition: background-color 0.15s ease;
  }

  .file-row:hover {
    background-color: var(--bg-tertiary);
  }

  .path-cell {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .icon {
    font-size: 1rem;
    flex-shrink: 0;
  }

  .filename {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size-cell {
    font-size: 0.8125rem;
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .date-cell {
    font-size: 0.8125rem;
    color: var(--text-secondary);
  }

  .actions-cell {
    text-align: right;
  }

  .btn-action {
    background: none;
    border: 1px solid transparent;
    color: var(--text-secondary);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
  }

  .btn-action:hover {
    border-color: var(--border);
    color: var(--text-primary);
  }
</style>
