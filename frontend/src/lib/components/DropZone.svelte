<script lang="ts">
  import { addEntry } from "../api";

  export let isLocked: boolean;
  export let onFileAdded: () => void;

  let isDragging = false;

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;

    if (isLocked) return;

    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    for (const file of Array.from(files)) {
      try {
        const buffer = await file.arrayBuffer();
        const content = Array.from(new Uint8Array(buffer));
        await addEntry(file.name, content);
        onFileAdded();
      } catch (err) {
        console.error(`Failed to add file "${file.name}":`, err);
      }
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (!isLocked) {
      isDragging = true;
    }
  }

  function handleDragLeave() {
    isDragging = false;
  }
</script>

<div
  class="drop-zone"
  class:active={isDragging && !isLocked}
  class:locked={isLocked}
  on:drop={handleDrop}
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
>
  {#if isDragging && !isLocked}
    <div class="drop-overlay">
      <span class="drop-icon">📥</span>
      <p>Drop files here to add to stash</p>
    </div>
  {:else if isLocked}
    <div class="drop-overlay locked-message">
      <span class="drop-icon">🔒</span>
      <p>Stash is locked — unlock to add files</p>
    </div>
  {/if}
  <slot />
</div>

<style>
  .drop-zone {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    background: rgba(10, 10, 11, 0.85);
    border: 2px dashed var(--accent);
    border-radius: 8px;
    pointer-events: none;
  }

  .drop-overlay.locked-message {
    border-color: var(--text-secondary);
    background: rgba(10, 10, 11, 0.7);
  }

  .drop-icon {
    font-size: 2rem;
  }

  .drop-overlay p {
    color: var(--text-primary);
    font-size: 0.875rem;
    font-weight: 500;
  }

  .locked-message p {
    color: var(--text-secondary);
  }
</style>
