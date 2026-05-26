import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@testing-library/svelte";
import FileRow from "../components/FileRow.svelte";
import type { EntrySummary } from "../api";

function makeEntry(overrides: Partial<EntrySummary>): EntrySummary {
  return {
    id: "test-id",
    path: "test.txt",
    size: 100,
    mime_type: "text/plain",
    created_at: 0,
    modified_at: 0,
    ...overrides,
  };
}

describe("FileRow preview integration", () => {
  it("dispatches preview event on double-click", async () => {
    const entry = makeEntry({ path: "docs/notes.txt", size: 512 });
    const previewHandler = vi.fn();

    const { component } = render(FileRow, { props: { entry } });
    component.$on("preview", previewHandler);

    const row = document.querySelector(".file-row");
    expect(row).not.toBeNull();

    await fireEvent.dblClick(row!);

    expect(previewHandler).toHaveBeenCalledTimes(1);
    expect(previewHandler.mock.calls[0][0].detail).toEqual(entry);
  });

  it("still dispatches contextmenu_action on right-click", async () => {
    const entry = makeEntry();
    const contextHandler = vi.fn();

    const { component } = render(FileRow, { props: { entry } });
    component.$on("contextmenu_action", contextHandler);

    const row = document.querySelector(".file-row");
    await fireEvent.contextMenu(row!, { clientX: 100, clientY: 200 });

    expect(contextHandler).toHaveBeenCalledTimes(1);
    expect(contextHandler.mock.calls[0][0].detail.entry).toEqual(entry);
  });
});
