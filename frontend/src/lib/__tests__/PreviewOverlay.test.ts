import { describe, it, expect, vi } from "vitest";
import { render, fireEvent, screen } from "@testing-library/svelte";
import PreviewOverlay from "../components/PreviewOverlay.svelte";
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

describe("PreviewOverlay", () => {
  it("renders header with filename and size", () => {
    const entry = makeEntry({ path: "docs/notes.txt", size: 2048 });
    render(PreviewOverlay, {
      props: { entry, content: new Uint8Array([65]), onClose: () => {} },
    });

    expect(screen.getByText("notes.txt")).toBeTruthy();
    expect(screen.getByText("2.0 KB")).toBeTruthy();
  });

  it("renders textarea for text MIME types", () => {
    const entry = makeEntry({ mime_type: "text/plain" });
    const text = "Hello, world!";
    const { container } = render(PreviewOverlay, {
      props: { entry, content: new TextEncoder().encode(text), onClose: () => {} },
    });

    const textarea = container.querySelector("textarea");
    expect(textarea).toBeTruthy();
    expect(textarea!.hasAttribute("readonly")).toBe(true);
    // The text content is rendered inside the textarea element
    // In jsdom, verify the element exists and is properly configured
    expect(textarea!.className).toContain("text-preview");
  });

  it("shows warning for large text files (>5MB)", () => {
    const entry = makeEntry({
      mime_type: "text/plain",
      size: 6 * 1024 * 1024, // 6 MB
    });
    render(PreviewOverlay, {
      props: { entry, content: new Uint8Array([65]), onClose: () => {} },
    });

    expect(screen.getByText(/File too large/i)).toBeTruthy();
    expect(screen.getByText(/Use Export instead/i)).toBeTruthy();
  });

  it("shows unsupported fallback for unknown MIME types", () => {
    const entry = makeEntry({ mime_type: "application/octet-stream" });
    render(PreviewOverlay, {
      props: { entry, content: new Uint8Array([0]), onClose: () => {} },
    });

    expect(screen.getByText(/Preview not available/i)).toBeTruthy();
    expect(screen.getByText(/Use Export instead/i)).toBeTruthy();
  });

  it("calls onClose when X button is clicked", async () => {
    const onClose = vi.fn();
    const entry = makeEntry();
    render(PreviewOverlay, {
      props: { entry, content: new Uint8Array([65]), onClose },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Close preview" }));
    expect(onClose).toHaveBeenCalled();
  });
});
