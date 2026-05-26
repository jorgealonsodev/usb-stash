import { describe, it, expect } from "vitest";
import { buildTree, filterBySearch, type TreeNode } from "../tree";
import type { EntrySummary } from "../api";

function makeEntry(overrides: Partial<EntrySummary>): EntrySummary {
  return {
    id: "test-id",
    path: "",
    size: 0,
    mime_type: "application/octet-stream",
    created_at: 0,
    modified_at: 0,
    ...overrides,
  };
}

describe("buildTree", () => {
  it("returns empty array for no entries", () => {
    expect(buildTree([])).toEqual([]);
  });

  it("builds tree from flat list with nested paths", () => {
    const entries = [
      makeEntry({ id: "1", path: "docs/notes.txt", size: 100 }),
      makeEntry({ id: "2", path: "docs/report.pdf", size: 200 }),
      makeEntry({ id: "3", path: "img/logo.png", size: 300 }),
    ];

    const tree = buildTree(entries);

    expect(tree).toHaveLength(2);

    const docs = tree.find((n) => n.name === "docs");
    expect(docs).toBeDefined();
    expect(docs!.type).toBe("folder");
    expect(docs!.path).toBe("docs");
    expect(docs!.children).toHaveLength(2);

    const notes = docs!.children.find((n) => n.name === "notes.txt");
    expect(notes!.type).toBe("file");
    expect(notes!.entry!.size).toBe(100);

    const img = tree.find((n) => n.name === "img");
    expect(img).toBeDefined();
    expect(img!.children).toHaveLength(1);
  });

  it("creates intermediate folders for deeply nested paths", () => {
    const entries = [
      makeEntry({ id: "1", path: "a/b/c/file.txt", size: 50 }),
    ];

    const tree = buildTree(entries);

    expect(tree).toHaveLength(1);
    expect(tree[0].name).toBe("a");
    expect(tree[0].type).toBe("folder");

    const b = tree[0].children[0];
    expect(b.name).toBe("b");
    expect(b.type).toBe("folder");

    const c = b.children[0];
    expect(c.name).toBe("c");
    expect(c.type).toBe("folder");

    const file = c.children[0];
    expect(file.name).toBe("file.txt");
    expect(file.type).toBe("file");
  });

  it("handles file at root level", () => {
    const entries = [
      makeEntry({ id: "1", path: "README.md", size: 1024 }),
    ];

    const tree = buildTree(entries);

    expect(tree).toHaveLength(1);
    expect(tree[0].name).toBe("README.md");
    expect(tree[0].type).toBe("file");
    expect(tree[0].entry!.size).toBe(1024);
  });

  it("handles mixed root files and nested folders", () => {
    const entries = [
      makeEntry({ id: "1", path: "README.md", size: 100 }),
      makeEntry({ id: "2", path: "src/main.ts", size: 200 }),
    ];

    const tree = buildTree(entries);

    expect(tree).toHaveLength(2);
    const readme = tree.find((n) => n.name === "README.md");
    const src = tree.find((n) => n.name === "src");
    expect(readme!.type).toBe("file");
    expect(src!.type).toBe("folder");
  });

  it("skips entries with empty path", () => {
    const entries = [
      makeEntry({ id: "1", path: "", size: 0 }),
      makeEntry({ id: "2", path: "valid.txt", size: 10 }),
    ];

    const tree = buildTree(entries);

    expect(tree).toHaveLength(1);
    expect(tree[0].name).toBe("valid.txt");
  });
});

describe("filterBySearch", () => {
  const sampleTree: TreeNode[] = [
    {
      name: "docs",
      path: "docs",
      type: "folder",
      children: [
        {
          name: "notes.txt",
          path: "docs/notes.txt",
          type: "file",
          children: [],
          entry: makeEntry({ id: "1", path: "docs/notes.txt" }),
        },
        {
          name: "report.pdf",
          path: "docs/report.pdf",
          type: "file",
          children: [],
          entry: makeEntry({ id: "2", path: "docs/report.pdf" }),
        },
      ],
    },
    {
      name: "img",
      path: "img",
      type: "folder",
      children: [
        {
          name: "logo.png",
          path: "img/logo.png",
          type: "file",
          children: [],
          entry: makeEntry({ id: "3", path: "img/logo.png" }),
        },
      ],
    },
  ];

  it("returns original tree for empty query", () => {
    expect(filterBySearch(sampleTree, "")).toBe(sampleTree);
    expect(filterBySearch(sampleTree, "   ")).toBe(sampleTree);
  });

  it("filters by case-insensitive substring match", () => {
    const result = filterBySearch(sampleTree, "notes");

    expect(result).toHaveLength(1);
    expect(result[0].name).toBe("docs");
    expect(result[0].children).toHaveLength(1);
    expect(result[0].children[0].name).toBe("notes.txt");
  });

  it("case-insensitive: lowercase query matches uppercase path", () => {
    const tree: TreeNode[] = [
      {
        name: "README.md",
        path: "README.md",
        type: "file",
        children: [],
        entry: makeEntry({ id: "1", path: "README.md" }),
      },
    ];

    const result = filterBySearch(tree, "readme");

    expect(result).toHaveLength(1);
    expect(result[0].name).toBe("README.md");
  });

  it("returns empty when no matches", () => {
    const result = filterBySearch(sampleTree, "nonexistent");
    expect(result).toHaveLength(0);
  });

  it("keeps ancestor folders of matching files", () => {
    const result = filterBySearch(sampleTree, "logo");

    expect(result).toHaveLength(1);
    expect(result[0].name).toBe("img");
    expect(result[0].children).toHaveLength(1);
    expect(result[0].children[0].name).toBe("logo.png");
  });

  it("matches on folder name itself", () => {
    const result = filterBySearch(sampleTree, "docs");

    expect(result).toHaveLength(1);
    expect(result[0].name).toBe("docs");
    // Folder matches, so all children should be included
    expect(result[0].children).toHaveLength(2);
  });
});
