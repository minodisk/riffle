import { describe, expect, test } from "vitest";
import { type LastViewed, lastViewedWriter, resumeTarget } from "./resume.js";

describe("resumeTarget", () => {
  const all = ["/a.ARW", "/b.ARW", "/c.ARW"];
  const every = (): boolean => true;

  test("nothing remembered", () => {
    expect(resumeTarget(null, all, all, every)).toBeUndefined();
  });

  test("the remembered file is gone", () => {
    expect(resumeTarget("/z.ARW", all, all, every)).toBeUndefined();
  });

  test("the remembered file passes the filter", () => {
    expect(resumeTarget("/b.ARW", all, all, every)).toBe("/b.ARW");
  });

  test("a filtered-out file resumes at the next passing file", () => {
    expect(resumeTarget("/b.ARW", all, all, (path) => path !== "/b.ARW")).toBe("/c.ARW");
  });

  test("a filtered-out last file resumes at the previous passing file", () => {
    expect(resumeTarget("/c.ARW", all, all, (path) => path === "/a.ARW")).toBe("/a.ARW");
  });

  test("the neighbor follows the strip order", () => {
    const order = ["/c.ARW", "/b.ARW", "/a.ARW"];
    expect(resumeTarget("/b.ARW", all, order, (path) => path !== "/b.ARW")).toBe("/a.ARW");
  });

  test("nothing passes the filter", () => {
    expect(resumeTarget("/b.ARW", all, all, () => false)).toBeUndefined();
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
