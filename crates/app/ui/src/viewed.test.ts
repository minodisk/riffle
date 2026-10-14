import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import {
  dwellTimer,
  switchesFolder,
  VIEW_DWELL_MS,
  type ViewedBatch,
  viewedWriter,
} from "./viewed.js";

describe("viewedWriter", () => {
  function recorder(): {
    sent: ViewedBatch[];
    settle: (ok?: boolean) => Promise<void>;
    send: (batch: ViewedBatch) => Promise<unknown>;
  } {
    const sent: ViewedBatch[] = [];
    const settlers: ((ok: boolean) => void)[] = [];
    return {
      sent,
      settle: async (ok = true) => {
        settlers.shift()?.(ok);
        await new Promise((resolve) => setTimeout(resolve, 0));
      },
      send: (batch) => {
        sent.push(batch);
        return new Promise((resolve, reject) => {
          settlers.push((ok) => (ok ? resolve(undefined) : reject(new Error("write failed"))));
        });
      },
    };
  }

  test("sends at once when idle", () => {
    const { sent, send } = recorder();
    const push = viewedWriter(send);
    push("/d", "/d/a.ARW");
    expect(sent).toEqual([{ dir: "/d", paths: ["/d/a.ARW"] }]);
  });

  test("collects the paths pushed while a batch is in flight into one call", async () => {
    const { sent, send, settle } = recorder();
    const push = viewedWriter(send);
    push("/d", "/d/a.ARW");
    push("/d", "/d/b.ARW");
    push("/d", "/d/c.ARW");
    push("/d", "/d/b.ARW");
    expect(sent).toHaveLength(1);
    await settle();
    expect(sent).toEqual([
      { dir: "/d", paths: ["/d/a.ARW"] },
      { dir: "/d", paths: ["/d/b.ARW", "/d/c.ARW"] },
    ]);
    await settle();
    expect(sent).toHaveLength(2);
  });

  test("keeps folders apart", async () => {
    const { sent, send, settle } = recorder();
    const push = viewedWriter(send);
    push("/d", "/d/a.ARW");
    push("/d", "/d/b.ARW");
    push("/e", "/e/a.ARW");
    await settle();
    await settle();
    expect(sent).toEqual([
      { dir: "/d", paths: ["/d/a.ARW"] },
      { dir: "/d", paths: ["/d/b.ARW"] },
      { dir: "/e", paths: ["/e/a.ARW"] },
    ]);
  });

  test("keeps going after a rejected send", async () => {
    const { sent, send, settle } = recorder();
    const push = viewedWriter(send);
    push("/d", "/d/a.ARW");
    push("/d", "/d/b.ARW");
    await settle(false);
    expect(sent).toEqual([
      { dir: "/d", paths: ["/d/a.ARW"] },
      { dir: "/d", paths: ["/d/b.ARW"] },
    ]);
    await settle();
    push("/d", "/d/c.ARW");
    expect(sent).toHaveLength(3);
  });
});

describe("switchesFolder", () => {
  test("opening another folder switches", () => {
    expect(switchesFolder("/photos/a", "/photos/b")).toBe(true);
  });

  test("the first open after launch does not", () => {
    expect(switchesFolder(null, "/photos/a")).toBe(false);
  });

  test("reopening the same folder does not", () => {
    expect(switchesFolder("/photos/a", "/photos/a")).toBe(false);
    expect(switchesFolder("/photos/a", "/Photos/A", true)).toBe(false);
    expect(switchesFolder("/photos/a", "/Photos/A")).toBe(true);
  });

  test("opening a subfolder switches", () => {
    expect(switchesFolder("/photos", "/photos/a")).toBe(true);
  });
});

describe("dwellTimer", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  function recorder(): { recorded: string[]; dwell: ReturnType<typeof dwellTimer> } {
    const recorded: string[] = [];
    const viewed = new Set<string>();
    const dwell = dwellTimer(
      (path) => viewed.has(path),
      (path) => {
        viewed.add(path);
        recorded.push(path);
      },
    );
    return { recorded, dwell };
  }

  test("a display cancelled before the dwell records nothing", () => {
    const { recorded, dwell } = recorder();
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS - 1);
    dwell.cancel();
    vi.advanceTimersByTime(VIEW_DWELL_MS);
    expect(recorded).toEqual([]);
  });

  test("a display replaced before the dwell records only the one that stays", () => {
    const { recorded, dwell } = recorder();
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(30);
    dwell.shown("/d/b.ARW");
    vi.advanceTimersByTime(30);
    dwell.shown("/d/c.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS);
    expect(recorded).toEqual(["/d/c.ARW"]);
  });

  test("a display past the dwell records once", () => {
    const { recorded, dwell } = recorder();
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS);
    expect(recorded).toEqual(["/d/a.ARW"]);
    vi.advanceTimersByTime(VIEW_DWELL_MS * 4);
    expect(recorded).toEqual(["/d/a.ARW"]);
  });

  test("showing a recorded path again does not record it again", () => {
    const { recorded, dwell } = recorder();
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS);
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS);
    expect(recorded).toEqual(["/d/a.ARW"]);
  });

  test("a new path restarts the clock", () => {
    const { recorded, dwell } = recorder();
    dwell.shown("/d/a.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS - 100);
    dwell.shown("/d/b.ARW");
    vi.advanceTimersByTime(VIEW_DWELL_MS - 1);
    expect(recorded).toEqual([]);
    vi.advanceTimersByTime(1);
    expect(recorded).toEqual(["/d/b.ARW"]);
  });
});
