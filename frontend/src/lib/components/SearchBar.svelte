<script lang="ts">
  import { searchQuery } from "../stores";

  let input: HTMLInputElement;

  function handleClear() {
    searchQuery.set("");
    input?.focus();
  }
</script>

<div class="search-bar">
  <span class="search-icon">🔍</span>
  <input
    bind:this={input}
    type="text"
    bind:value={$searchQuery}
    placeholder="Buscar archivos..."
    class="search-input"
  />
  {#if $searchQuery}
    <button class="clear-btn" on:click={handleClear} aria-label="Limpiar búsqueda">
      ✕
    </button>
  {/if}
</div>

<style>
  .search-bar {
    display: flex;
    align-items: center;
    position: relative;
    width: 100%;
    max-width: 24rem;
  }

  .search-icon {
    position: absolute;
    left: 0.625rem;
    font-size: 0.875rem;
    pointer-events: none;
  }

  .search-input {
    width: 100%;
    padding: 0.5rem 2rem 0.5rem 2rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-secondary);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-family: var(--font-sans);
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
  }

  .search-input::placeholder {
    color: var(--text-secondary);
  }

  .clear-btn {
    position: absolute;
    right: 0.375rem;
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.75rem;
    padding: 0.25rem 0.375rem;
    border-radius: 4px;
    cursor: pointer;
    line-height: 1;
  }

  .clear-btn:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }
</style>
