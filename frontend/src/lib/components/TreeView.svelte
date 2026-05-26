<script lang="ts">
  import type { TreeNode } from "../tree";
  import { expandedPaths, selectedPath } from "../stores";
  import TreeNodeItem from "./TreeNodeItem.svelte";

  export let nodes: TreeNode[];
</script>

{#if nodes.length === 0}
  <div class="empty-tree">
    <p>No files in this stash</p>
  </div>
{:else}
  <div class="tree-root">
    <div
      class="tree-node root"
      class:selected={$selectedPath === null}
      on:click={() => selectedPath.set(null)}
    >
      <span class="icon">📁</span>
      <span class="name">root</span>
    </div>
    {#each nodes as node}
      <TreeNodeItem {node} depth={1} />
    {/each}
  </div>
{/if}

<style>
  .tree-root {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .tree-node {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8125rem;
    color: var(--text-secondary);
    transition: background-color 0.1s ease, color 0.1s ease;
    user-select: none;
  }

  .tree-node:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .tree-node.selected {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-weight: 500;
  }

  .tree-node.root {
    font-weight: 600;
    color: var(--text-primary);
  }

  .tree-node.root:hover {
    background: var(--bg-tertiary);
  }

  .icon {
    font-size: 0.875rem;
    flex-shrink: 0;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-tree {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 0.5rem;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    text-align: center;
  }
</style>
