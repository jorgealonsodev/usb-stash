<script lang="ts">
  import { onMount } from "svelte";
  import { navigate } from "svelte-routing";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { EntrySummary } from "../lib/api";
  import {
    listEntries,
    addEntry,
    lockStash,
  } from "../lib/api";
  import { entries, expandedPaths, selectedPath, searchQuery, isDirty } from "../lib/stores";
  import { buildTree, filterBySearch, type TreeNode } from "../lib/tree";
  import TreeView from "../lib/components/TreeView.svelte";
  import FileList from "../lib/components/FileList.svelte";
  import SearchBar from "../lib/components/SearchBar.svelte";
  import StatusBar from "../lib/components/StatusBar.svelte";
  import DropZone from "../lib/components/DropZone.svelte";

  let loading = true;
  let stashLocked = false;

  $: treeNodes = buildTree($entries);
  $: searchedTree = filterBySearch(treeNodes, $searchQuery);

  // Filter entries by selected path and search
  $: filteredEntries = (() => {
    let result = $entries;

    // Filter by selected folder
    if ($selectedPath !== null) {
      result = result.filter((e) => {
        const entryDir = e.path.includes("/")
          ? e.path.split("/").slice(0, -1).join("/")
          : "";
        return entryDir === $selectedPath;
      });
    }

    // Filter by search query
    if ($searchQuery.trim()) {
      const q = $searchQuery.toLowerCase();
      result = result.filter((e) => e.path.toLowerCase().includes(q));
    }

    return result;
  })();

  // Breadcrumb segments
  $: breadcrumbs = (() => {
    if ($selectedPath === null) return [{ label: "root", path: null }];
    const segments = $selectedPath.split("/").filter(Boolean);
    const result = [{ label: "root", path: null as string | null }];
    let current = "";
    for (const seg of segments) {
      current = current ? `${current}/${seg}` : seg;
      result.push({ label: seg, path: current });
    }
    return result;
  })();

  onMount(async () => {
    try {
      const data = await listEntries();
      entries.set(data);
    } catch (e) {
      console.error("Failed to load entries:", e);
      stashLocked = true;
    } finally {
      loading = false;
    }
  });

  async function handleAddFile() {
    try {
      const selected = await open({
        multiple: true,
        title: "Select files to add",
      });
      if (!selected) return;

      const paths = Array.isArray(selected) ? selected : [selected];
      for (const filePath of paths) {
        // Tauri v2 returns file paths; we need to read them
        // Using the file system API
        const { readFile } = await import("@tauri-apps/plugin-fs");
        const content = await readFile(filePath);
        const fileName = filePath.split("/").pop() ?? filePath;
        await addEntry(fileName, Array.from(content));
        isDirty.set(true);
      }

      // Refresh
      const data = await listEntries();
      entries.set(data);
    } catch (e) {
      console.error("Failed to add file:", e);
    }
  }

  async function handleLock() {
    try {
      await lockStash();
      navigate("/");
    } catch (e) {
      console.error("Failed to lock stash:", e);
    }
  }

  function handleBreadcrumbClick(path: string | null) {
    selectedPath.set(path);
    // Expand folders up to the selected path
    if (path) {
      const segments = path.split("/").filter(Boolean);
      const newExpanded = new Set($expandedPaths);
      let current = "";
      for (const seg of segments) {
        current = current ? `${current}/${seg}` : seg;
        newExpanded.add(current);
      }
      expandedPaths.set(newExpanded);
    }
  }

  function handleFileAdded() {
    listEntries().then((data) => {
      entries.set(data);
      isDirty.set(true);
    });
  }
</script>

<div class="explorer">
  {#if loading}
    <div class="loading">
      <p>Loading stash...</p>
    </div>
  {:else if stashLocked}
    <div class="locked">
      <p>Stash is locked</p>
      <button class="btn" on:click={() => navigate("/")}>Go back</button>
    </div>
  {:else}
    <!-- Top Bar -->
    <div class="top-bar">
      <div class="breadcrumb">
        {#each breadcrumbs as crumb, i}
          {#if i > 0}<span class="separator">&gt;</span>{/if}
          <button
            class="breadcrumb-item"
            class:active={crumb.path === $selectedPath}
            on:click={() => handleBreadcrumbClick(crumb.path)}
          >
            {crumb.label}
          </button>
        {/each}
      </div>
      <SearchBar />
      <div class="top-actions">
        <button class="btn btn-sm" on:click={handleAddFile}>+ Add file</button>
        <button class="btn btn-sm btn-lock" on:click={handleLock}>Lock stash</button>
      </div>
    </div>

    <!-- Main Content -->
    <DropZone isLocked={false} onFileAdded={handleFileAdded}>
      <div class="main-content">
        <!-- Tree Sidebar -->
        <aside class="tree-sidebar">
          <h2 class="sidebar-title">Folders</h2>
          <TreeView nodes={searchedTree} />
        </aside>

        <!-- File List -->
        <main class="file-list-area">
          <FileList {filteredEntries} />
        </main>
      </div>
    </DropZone>

    <!-- Status Bar -->
    <StatusBar />
  {/if}
</div>

<style>
  .explorer {
    display: flex;
    flex-direction: column;
    height: 100vh;
    gap: 0;
  }

  .loading,
  .locked {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  /* Top Bar */
  .top-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--bg-secondary);
    flex-shrink: 0;
  }

  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    flex-shrink: 0;
  }

  .breadcrumb-item {
    background: none;
    border: none;
    color: var(--text-secondary);
    font-size: 0.8125rem;
    padding: 0.25rem 0.375rem;
    border-radius: 4px;
    cursor: pointer;
    font-family: var(--font-mono);
  }

  .breadcrumb-item:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .breadcrumb-item.active {
    color: var(--text-primary);
    font-weight: 500;
  }

  .separator {
    color: var(--border);
    font-size: 0.75rem;
  }

  .top-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: auto;
    flex-shrink: 0;
  }

  .btn {
    padding: 0.5rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    font-size: 0.8125rem;
    font-family: var(--font-sans);
    cursor: pointer;
  }

  .btn:hover {
    border-color: var(--accent);
  }

  .btn-sm {
    padding: 0.375rem 0.625rem;
    font-size: 0.75rem;
  }

  .btn-lock {
    color: var(--text-secondary);
  }

  .btn-lock:hover {
    color: var(--danger);
    border-color: var(--danger);
  }

  /* Main Content */
  .main-content {
    display: flex;
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }

  .tree-sidebar {
    width: 14rem;
    flex-shrink: 0;
    border-right: 1px solid var(--border);
    padding: 0.5rem;
    overflow-y: auto;
    background: var(--bg-secondary);
  }

  .sidebar-title {
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary);
    padding: 0.375rem 0.5rem;
    margin-bottom: 0.25rem;
  }

  .file-list-area {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
