import type { EntrySummary } from "./api";

export interface TreeNode {
  name: string;
  path: string;
  type: "folder" | "file";
  children: TreeNode[];
  entry?: EntrySummary;
}

/**
 * Build a tree from flat entries by splitting virtual paths on `/`.
 * Root-level folders and files appear as top-level nodes.
 * Empty path segments are skipped.
 */
export function buildTree(entries: EntrySummary[]): TreeNode[] {
  const root: TreeNode = {
    name: "",
    path: "",
    type: "folder",
    children: [],
  };

  // Map from full path -> TreeNode for O(1) lookups
  const pathMap = new Map<string, TreeNode>();
  pathMap.set("", root);

  for (const entry of entries) {
    const segments = entry.path.split("/").filter((s) => s.length > 0);
    if (segments.length === 0) continue;

    // Ensure all intermediate folders exist
    let currentPath = "";
    for (let i = 0; i < segments.length - 1; i++) {
      const segment = segments[i];
      const parentPath = currentPath;
      currentPath = currentPath ? `${currentPath}/${segment}` : segment;

      if (!pathMap.has(currentPath)) {
        const folder: TreeNode = {
          name: segment,
          path: currentPath,
          type: "folder",
          children: [],
        };
        pathMap.set(currentPath, folder);

        // Attach to parent
        const parent = pathMap.get(parentPath)!;
        parent.children.push(folder);
      }
    }

    // Create the file leaf
    const fileName = segments[segments.length - 1];
    const fileNode: TreeNode = {
      name: fileName,
      path: entry.path,
      type: "file",
      children: [],
      entry,
    };
    pathMap.set(entry.path, fileNode);

    // Attach to parent folder
    const parentPath = segments.slice(0, -1).join("/");
    const parent = pathMap.get(parentPath)!;
    parent.children.push(fileNode);
  }

  return root.children;
}

/**
 * Filter tree nodes by a case-insensitive substring match on their path.
 * Returns a new tree containing only matching nodes and their ancestors.
 * Non-matching branches are pruned.
 */
export function filterBySearch(nodes: TreeNode[], query: string): TreeNode[] {
  if (!query || query.trim() === "") {
    return nodes;
  }

  const lowerQuery = query.toLowerCase();

  function filterNode(node: TreeNode): TreeNode | null {
    const matches = node.path.toLowerCase().includes(lowerQuery);

    if (node.type === "file") {
      return matches ? { ...node, children: [] } : null;
    }

    // Folder: recurse into children
    const filteredChildren: TreeNode[] = [];
    for (const child of node.children) {
      const filtered = filterNode(child);
      if (filtered) {
        filteredChildren.push(filtered);
      }
    }

    // Keep folder if it matches directly OR has matching descendants
    if (matches || filteredChildren.length > 0) {
      return { ...node, children: filteredChildren };
    }

    return null;
  }

  const result: TreeNode[] = [];
  for (const node of nodes) {
    const filtered = filterNode(node);
    if (filtered) {
      result.push(filtered);
    }
  }

  return result;
}
