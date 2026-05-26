<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { EntrySummary } from "../api";

  export let entry: EntrySummary;
  export let content: Uint8Array;
  export let onClose: () => void;

  const TEXT_SIZE_CAP = 5 * 1024 * 1024; // 5 MB

  let blobUrl: string | null = null;
  let pdfError: string | null = null;
  let pdfLoading = false;

  const mime = entry.mime_type.toLowerCase();
  const isPdf = mime === "application/pdf";
  const isImage = mime.startsWith("image/");
  const isText = mime.startsWith("text/") || mime === "application/json";
  const isLargeText = isText && entry.size > TEXT_SIZE_CAP;
  $: textContent = isText && !isLargeText ? new TextDecoder().decode(content) : null;

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }

  onMount(async () => {
    if (isImage && !isLargeText) {
      const blob = new Blob([content], { type: entry.mime_type });
      blobUrl = URL.createObjectURL(blob);
    } else if (isPdf) {
      pdfLoading = true;
      try {
        const pdfjsLib = await import("pdfjs-dist");
        pdfjsLib.GlobalWorkerOptions.workerSrc = `https://cdnjs.cloudflare.com/ajax/libs/pdf.js/${pdfjsLib.version}/pdf.worker.min.mjs`;
        const loadingTask = pdfjsLib.getDocument({ data: content.buffer });
        const pdf = await loadingTask.promise;
        const page = await pdf.getPage(1);
        const scale = 1.5;
        const viewport = page.getViewport({ scale });
        const canvas = document.getElementById("pdf-canvas") as HTMLCanvasElement;
        if (canvas) {
          canvas.width = viewport.width;
          canvas.height = viewport.height;
          const ctx = canvas.getContext("2d");
          if (ctx) {
            await page.render({ canvasContext: ctx, viewport }).promise;
          }
        }
      } catch (err) {
        pdfError = err instanceof Error ? err.message : "Failed to load PDF";
      } finally {
        pdfLoading = false;
      }
    }
  });

  onDestroy(() => {
    if (blobUrl) {
      URL.revokeObjectURL(blobUrl);
      blobUrl = null;
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="overlay" on:click={handleBackdropClick} role="dialog" aria-modal="true">
  <div class="content">
    <div class="header">
      <span class="filename">{entry.path.split("/").pop()}</span>
      <span class="size">{formatSize(entry.size)}</span>
      <button class="close-btn" on:click={onClose} aria-label="Close preview">&times;</button>
    </div>

    <div class="body">
      {#if isPdf}
        {#if pdfLoading}
          <p class="loading-text">Loading PDF...</p>
        {:else if pdfError}
          <p class="error-text">Failed to load PDF: {pdfError}</p>
        {:else}
          <canvas id="pdf-canvas" class="pdf-canvas"></canvas>
        {/if}
      {:else if isImage}
        <img src={blobUrl} alt={entry.path} class="preview-image" />
      {:else if isText}
        {#if isLargeText}
          <div class="large-file-warning">
            <p>File too large to preview ({formatSize(entry.size)})</p>
            <button class="btn" on:click={onClose}>Use Export instead</button>
          </div>
        {:else}
          <textarea readonly class="text-preview">{textContent}</textarea>
        {/if}
      {:else}
        <div class="unsupported">
          <p>Preview not available for this file type</p>
          <button class="btn" on:click={onClose}>Use Export instead</button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .content {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 8px;
    width: 90vw;
    max-width: 900px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
  }

  .filename {
    font-family: var(--font-mono);
    font-size: 0.875rem;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .size {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0 0.25rem;
    line-height: 1;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .body {
    flex: 1;
    overflow: auto;
    padding: 1rem;
    min-height: 0;
  }

  .preview-image {
    max-width: 100%;
    max-height: 70vh;
    display: block;
    margin: 0 auto;
  }

  .text-preview {
    width: 100%;
    min-height: 400px;
    max-height: 70vh;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.75rem;
    font-family: var(--font-mono);
    font-size: 0.8125rem;
    resize: none;
  }

  .pdf-canvas {
    display: block;
    margin: 0 auto;
    max-width: 100%;
  }

  .loading-text,
  .error-text {
    text-align: center;
    color: var(--text-secondary);
    padding: 2rem;
  }

  .error-text {
    color: var(--danger);
  }

  .large-file-warning,
  .unsupported {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 2rem;
    color: var(--text-secondary);
    text-align: center;
  }

  .btn {
    padding: 0.5rem 1rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    cursor: pointer;
  }

  .btn:hover {
    border-color: var(--accent);
  }
</style>
