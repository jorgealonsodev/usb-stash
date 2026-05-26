<script lang="ts">
  import type { TreeNode } from "../tree";
  import { expandedPaths, selectedPath } from "../stores";

  export let node: TreeNode;
  export let depth: number;

  function toggleFolder() {
    if (node.type !== "folder") return;
    const newSet = new Set($expandedPaths);
    if (newSet.has(node.path)) {
      newSet.delete(node.path);
    } else {
      newSet.add(node.path);
    }
    expandedPaths.set(newSet);
  }

  function selectNode() {
    if (node.type === "folder") {
      selectedPath.set(node.path);
    }
  }

  $: isExpanded = $expandedPaths.has(node.path);
  $: isSelected = $selectedPath === node.path;
  $: folderIcon = node.type === "folder" ? (isExpanded ? "📂" : "📁") : "📄";
</script>

{#if node.type === "folder"}
  <div
    class="tree-node"
    class:expanded={isExpanded}
    class:selected={isSelected}
    style="padding-left: {depth * 1}rem"
    on:click={toggleFolder}
  >
    <span class="icon">{folderIcon}</span>
    <span class="name">{node.name}</span>
  </div>
  {#if isExpanded}
    {#each node.children as child}
      <TreeNodeItem node={child} depth={depth + 1} />
    {/each}
  {/if}
{:else}
  <div
    class="tree-node file-node"
    class:selected={isSelected}
    style="padding-left: {depth * 1}rem"
    on:click={selectNode}
  >
    <span class="icon">{folderIcon}</span>
    <span class="name">{node.name}</span>
  </div>
{/if}

<style>
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

  .tree-node.file-node {
    color: var(--text-secondary);
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
</style>
