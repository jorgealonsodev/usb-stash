<script lang="ts">
  import { entries, isDirty } from "../stores";
  import { saveStash } from "../api";

  let saving = false;

  $: totalSize = $entries.reduce((sum, e) => sum + e.size, 0);
  $: fileCount = $entries.length;

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  async function handleSave() {
    saving = true;
    try {
      await saveStash();
      isDirty.set(false);
    } catch (e) {
      console.error("Failed to save stash:", e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="status-bar">
  <div class="status-info">
    <span class="file-count">{fileCount} {fileCount === 1 ? "file" : "files"}</span>
    <span class="separator">·</span>
    <span class="total-size">{formatSize(totalSize)}</span>
  </div>
  <div class="status-actions">
    {#if $isDirty}
      <span class="dirty-indicator">Unsaved changes</span>
      <button class="btn-save" on:click={handleSave} disabled={saving}>
        {saving ? "Saving..." : "Save"}
      </button>
    {:else}
      <span class="saved-indicator">Saved</span>
    {/if}
  </div>
</div>

<style>
  .status-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border);
    font-size: 0.75rem;
  }

  .status-info {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--text-secondary);
  }

  .file-count {
    font-weight: 500;
  }

  .separator {
    color: var(--border);
  }

  .total-size {
    font-family: var(--font-mono);
  }

  .status-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .dirty-indicator {
    color: #d4c953;
    font-weight: 500;
  }

  .saved-indicator {
    color: var(--success);
    font-weight: 500;
  }

  .btn-save {
    padding: 0.25rem 0.75rem;
    border: 1px solid var(--accent);
    border-radius: 4px;
    background: var(--accent);
    color: #fff;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-save:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-save:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
