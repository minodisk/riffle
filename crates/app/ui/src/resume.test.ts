import { describe, expect, test } from "vitest";
import { firstEntriesAnchor, type LastViewed, lastViewedWriter, resumeTarget } from "./resume.js";

describe("resumeTarget", () => {
  const all = ["/a.ARW", "/b.ARW", "/c.ARW"];

  test("nothing remembered", () => {
    expect(resumeTarget(null, all)).toBeUndefined();
  });

  test("the remembered file is gone", () => {
    expect(resumeTarget("/z.ARW", all)).toBeUndefined();
  });

  test("the remembered file is still listed", () => {
    expect(resumeTarget("/b.ARW", all)).toBe("/b.ARW");
  });
});

describe("firstEntriesAnchor", () => {
  test("a pending resume target takes over from the provisional anchor", () => {
    expect(firstEntriesAnchor("/b.ARW", "/a.ARW")).toBe("/b.ARW");
  });

  test("no pending resume target falls back to the provisional anchor", () => {
    expect(firstEntriesAnchor(undefined, "/a.ARW")).toBe("/a.ARW");
  });

  test("neither a pending target nor a provisional anchor", () => {
    expect(firstEntriesAnchor(undefined, undefined)).toBeUndefined();
  });
});

describe("lastViewedWriter", () => {
  function recorder(): {
    sent: LastViewed[];
    settle: () => Promise<void>;
    send: (value: LastViewed) => Promise<unknown>;
  } {
    const sent: LastViewed[] = [];
    const resolvers: (() => void)[] = [];
    return {
      sent,
      settle: async () => {
        resolvers.shift()?.();
        await new Promise((resolve) => setTimeout(resolve, 0));
      },
      send: (value) => {
        sent.push(value);
        return new Promise((resolve) => {
          resolvers.push(() => resolve(undefined));
        });
      },
    };
  }

  test("sends at once when idle", () => {
    const { sent, send } = recorder();
    const push = lastViewedWriter(send);
    push({ dir: "/d", path: "/d/a.ARW" });
    expect(sent).toEqual([{ dir: "/d", path: "/d/a.ARW" }]);
  });

  test("keeps only the latest value while one is in flight", async () => {
    const { sent, settle, send } = recorder();
    const push = lastViewedWriter(send);
    push({ dir: "/d", path: "/d/a.ARW" });
    push({ dir: "/d", path: "/d/b.ARW" });
    push({ dir: "/d", path: "/d/c.ARW" });
    expect(sent).toHaveLength(1);
    await settle();
    expect(sent).toEqual([
      { dir: "/d", path: "/d/a.ARW" },
      { dir: "/d", path: "/d/c.ARW" },
    ]);
    await settle();
    expect(sent).toHaveLength(2);
    push({ dir: "/e", path: "/e/a.ARW" });
    expect(sent[2]).toEqual({ dir: "/e", path: "/e/a.ARW" });
  });

  test("a failed send does not stall the next one", async () => {
    const sent: LastViewed[] = [];
    const push = lastViewedWriter((value) => {
      sent.push(value);
      return Promise.reject(new Error("failed"));
    });
    push({ dir: "/d", path: "/d/a.ARW" });
    push({ dir: "/d", path: "/d/b.ARW" });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(sent).toEqual([
      { dir: "/d", path: "/d/a.ARW" },
      { dir: "/d", path: "/d/b.ARW" },
    ]);
  });
});
