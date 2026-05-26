<script lang="ts">
  import { onMount, onDestroy } from "svelte";

  export let x: number;
  export let y: number;
  export let onAction: (action: "rename" | "delete" | "export") => void;
  export let onClose: () => void;

  let menuEl: HTMLElement;

  function handleClickOutside(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  onMount(() => {
    document.addEventListener("click", handleClickOutside);
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener("click", handleClickOutside);
    document.removeEventListener("keydown", handleKeydown);
  });

  function handleAction(action: "rename" | "delete" | "export") {
    onAction(action);
    onClose();
  }
</script>

<div
  bind:this={menuEl}
  class="context-menu"
  style="left: {x}px; top: {y}px;"
  role="menu"
>
  <button class="menu-item" on:click={() => handleAction("rename")} role="menuitem">
    <span class="menu-icon">✏️</span>
    <span>Rename</span>
  </button>
  <button class="menu-item danger" on:click={() => handleAction("delete")} role="menuitem">
    <span class="menu-icon">🗑️</span>
    <span>Delete</span>
  </button>
  <button class="menu-item" on:click={() => handleAction("export")} role="menuitem">
    <span class="menu-icon">⬇️</span>
    <span>Export</span>
  </button>
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 1000;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0.375rem 0;
    min-width: 10rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: none;
    border: none;
    color: var(--text-primary);
    font-size: 0.8125rem;
    cursor: pointer;
    text-align: left;
  }

  .menu-item:hover {
    background: var(--bg-tertiary);
  }

  .menu-item.danger:hover {
    color: var(--danger);
  }

  .menu-icon {
    font-size: 0.875rem;
    flex-shrink: 0;
  }
</style>
