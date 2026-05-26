<script lang="ts">
  import type { EntrySummary } from "../lib/api";

  // Placeholder: will be populated via listEntries() from api.ts
  let entries: EntrySummary[] = [];
</script>

<div class="explorer">
  <div class="toolbar">
    <h1>Explorer</h1>
    <button class="btn btn-sm">+ Add entry</button>
  </div>

  {#if entries.length === 0}
    <p class="empty">No hay entradas en este stash.</p>
  {:else}
    <table class="file-table">
      <thead>
        <tr>
          <th>Path</th>
          <th>Size</th>
          <th>Modified</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each entries as entry (entry.id)}
          <tr>
            <td class="mono">{entry.path}</td>
            <td>{entry.size} B</td>
            <td>{new Date(entry.modified_at * 1000).toLocaleDateString()}</td>
            <td>
              <button class="btn btn-sm">Extract</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .explorer {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  h1 {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 0.8125rem;
  }

  .btn:hover {
    border-color: var(--accent);
  }

  .btn-sm {
    padding: 0.375rem 0.625rem;
    font-size: 0.75rem;
  }

  .empty {
    color: var(--text-secondary);
    font-size: 0.875rem;
    text-align: center;
    padding: 3rem 0;
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
  }

  .file-table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-primary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 0.8125rem;
  }
</style>
