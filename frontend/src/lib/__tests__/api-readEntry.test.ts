import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("readEntry", () => {
  beforeEach(() => vi.resetModules());

  it("unwraps number[] to Uint8Array", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue([72, 101, 108, 108, 111]);

    const { readEntry } = await import("../api");
    const result = await readEntry("/test.txt");

    expect(invoke).toHaveBeenCalledWith("read_entry", {
      entryPath: "/test.txt",
    });
    expect(result).toBeInstanceOf(Uint8Array);
    expect(Array.from(result)).toEqual([72, 101, 108, 108, 111]);
  });

  it("handles empty content", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValue([]);

    const { readEntry } = await import("../api");
    const result = await readEntry("/empty.bin");

    expect(result).toBeInstanceOf(Uint8Array);
    expect(result.byteLength).toBe(0);
  });
});
