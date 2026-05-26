<script lang="ts">
  import type { EntrySummary } from "../api";
  import FileRow from "./FileRow.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import { deleteEntry, renameEntry, extractEntry } from "../api";
  import { entries, isDirty } from "../stores";
  import { open } from "@tauri-apps/plugin-dialog";

  export let filteredEntries: EntrySummary[];

  let contextMenu: { x: number; y: number; entry: EntrySummary } | null = null;

  async function refreshEntries() {
    const data = await import("../api");
    const updated = await data.listEntries();
    entries.set(updated);
  }

  function handleContextMenu(e: CustomEvent) {
    contextMenu = e.detail;
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  async function handleContextMenuAction(action: "rename" | "delete" | "export") {
    if (!contextMenu) return;
    const entry = contextMenu.entry;

    try {
      if (action === "delete") {
        await deleteEntry(entry.path);
        isDirty.set(true);
        await refreshEntries();
      } else if (action === "rename") {
        const newName = prompt(`New path for "${entry.path}":`, entry.path);
        if (newName && newName !== entry.path) {
          await renameEntry(entry.path, newName);
          isDirty.set(true);
          await refreshEntries();
        }
      } else if (action === "export") {
        const outputDir = await open({
          directory: true,
          multiple: false,
          title: "Choose export directory",
        });
        if (outputDir) {
          await extractEntry(entry.path, outputDir as string);
        }
      }
    } catch (e) {
      console.error(`Failed to ${action} entry:`, e);
    }

    contextMenu = null;
  }
</script>

<div class="file-list-wrapper" on:contextmenu_action={handleContextMenu}>
  {#if filteredEntries.length === 0}
    <div class="empty-list">
      <p>No files in this folder</p>
    </div>
  {:else}
    <table class="file-table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Size</th>
          <th>Modified</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each filteredEntries as entry (entry.id)}
          <FileRow
            {entry}
            on:contextmenu_action={handleContextMenu}
          />
        {/each}
      </tbody>
    </table>
  {/if}

  {#if contextMenu}
    <ContextMenu
      x={contextMenu.x}
      y={contextMenu.y}
      onAction={handleContextMenuAction}
      onClose={closeContextMenu}
    />
  {/if}
</div>

<style>
  .file-list-wrapper {
    position: relative;
    flex: 1;
    overflow-y: auto;
  }

  .file-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.875rem;
  }

  .file-table th {
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-secondary);
    font-weight: 500;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    position: sticky;
    top: 0;
    background: var(--bg-primary);
  }

  .file-table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
  }

  .empty-list {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 3rem 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
    text-align: center;
  }
</style>
