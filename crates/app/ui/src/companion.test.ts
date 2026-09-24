import { describe, expect, test } from "vitest";
import { type ViewApi, getView, respond } from "./companion.js";

function view(overrides: Partial<ViewApi> = {}): ViewApi {
  return {
    folder: null,
    files: [],
    index: 0,
    selection: new Set(),
    bursts: new Map(),
    sharpness: new Map(),
    ratings: new Map(),
    flags: new Map(),
    labels: new Map(),
    zoomed: false,
    comparing: false,
    compareActive: null,
    sort: "name",
    filtered: false,
    ...overrides,
  };
}

describe("getView", () => {
  test("reports an empty view with no folder open", () => {
    expect(getView(view())).toEqual({
      folder: null,
      count: 0,
      current: null,
      selected: [],
      mode: "normal",
      compare_active: null,
      burst: [],
      sort: "name",
      filtered: false,
    });
  });

  test("reports a single file with no burst", () => {
    const state = getView(
      view({
        folder: "/d",
        files: ["/d/a.ARW"],
        selection: new Set(["/d/a.ARW"]),
        bursts: new Map([["/d/a.ARW", { burst: 0, position: 0, size: 1 }]]),
        zoomed: true,
        sort: "capture",
        filtered: true,
      }),
    );
    expect(state).toEqual({
      folder: "/d",
      count: 1,
      current: { path: "/d/a.ARW", position: 1 },
      selected: ["/d/a.ARW"],
      mode: "zoom",
      compare_active: null,
      burst: [],
      sort: "capture",
      filtered: true,
    });
  });

  test("lists the current file's burst in capture order with its judgments", () => {
    const state = getView(
      view({
        folder: "/d",
        files: ["/d/a", "/d/b", "/d/c", "/d/x"],
        index: 1,
        selection: new Set(["/d/c", "/d/b"]),
        bursts: new Map([
          ["/d/c", { burst: 0, position: 2, size: 3 }],
          ["/d/a", { burst: 0, position: 0, size: 3 }],
          ["/d/b", { burst: 0, position: 1, size: 3 }],
          ["/d/x", { burst: 1, position: 0, size: 1 }],
        ]),
        sharpness: new Map([
          ["/d/a", 0.5],
          ["/d/b", 0.8],
        ]),
        ratings: new Map([["/d/b", 3]]),
        flags: new Map([["/d/c", "reject"]]),
        labels: new Map([["/d/a", "Red"]]),
      }),
    );
    expect(state.current).toEqual({ path: "/d/b", position: 2 });
    expect(state.selected).toEqual(["/d/b", "/d/c"]);
    expect(state.burst).toEqual([
      {
        path: "/d/a",
        position: 1,
        sharpness: 0.5,
        rating: null,
        flag: "none",
        label: "Red",
        visible: true,
      },
      {
        path: "/d/b",
        position: 2,
        sharpness: 0.8,
        rating: 3,
        flag: "none",
        label: null,
        visible: true,
      },
      {
        path: "/d/c",
        position: 3,
        sharpness: null,
        rating: null,
        flag: "reject",
        label: null,
        visible: true,
      },
    ]);
  });

  test("marks a burst frame the filter hides as not visible", () => {
    const state = getView(
      view({
        folder: "/d",
        // The filter hides "/d/c", so it is missing from `files` even
        // though `bursts` still groups it with "/d/a" and "/d/b".
        files: ["/d/a", "/d/b"],
        bursts: new Map([
          ["/d/a", { burst: 0, position: 0, size: 3 }],
          ["/d/b", { burst: 0, position: 1, size: 3 }],
          ["/d/c", { burst: 0, position: 2, size: 3 }],
        ]),
      }),
    );
    expect(state.burst.map(({ path, visible }) => ({ path, visible }))).toEqual([
      { path: "/d/a", visible: true },
      { path: "/d/b", visible: true },
      { path: "/d/c", visible: false },
    ]);
  });

  test("reports compare mode and its active pane", () => {
    const state = getView(
      view({
        folder: "/d",
        files: ["/d/a", "/d/b"],
        selection: new Set(["/d/a", "/d/b"]),
        comparing: true,
        compareActive: "/d/b",
      }),
    );
    expect(state.mode).toBe("compare");
    expect(state.compare_active).toBe("/d/b");
  });
});

describe("respond", () => {
  test("answers get_view", async () => {
    const reply = await respond({ id: 7, kind: "get_view", args: {} }, view());
    expect(reply.id).toBe(7);
    expect(reply.ok).toBe(true);
    expect(reply.value).toMatchObject({ folder: null, count: 0 });
  });

  test("turns an unknown request into an error reply", async () => {
    expect(await respond({ id: 8, kind: "nope", args: {} }, view())).toEqual({
      id: 8,
      ok: false,
      value: "unknown request: nope",
    });
  });
});
