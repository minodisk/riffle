import { describe, expect, test } from "vitest";
import {
  type Entry,
  type TrashEntry,
  History,
  isJudgments,
  mapJudgments,
  undoneTrash,
} from "./undo.js";

describe("History", () => {
  test("pops in reverse push order and is empty afterwards", () => {
    const history = new History<number>(10);
    history.push(1);
    history.push(2);
    expect(history.pop()).toBe(2);
    expect(history.pop()).toBe(1);
    expect(history.pop()).toBeUndefined();
  });

  test("drops the oldest entry past the limit", () => {
    const history = new History<number>(2);
    history.push(1);
    history.push(2);
    history.push(3);
    expect(history.pop()).toBe(3);
    expect(history.pop()).toBe(2);
    expect(history.pop()).toBeUndefined();
  });

  test("removes an entry by identity, keeping later ones", () => {
    const history = new History<{ n: number }>(10);
    const a = { n: 1 };
    const b = { n: 2 };
    history.push(a);
    history.push(b);
    history.remove(a);
    history.remove({ n: 2 });
    expect(history.pop()).toBe(b);
    expect(history.pop()).toBeUndefined();
  });

  test("removeWhere drops every matching entry", () => {
    const history = new History<{ path: string }>(10);
    history.push({ path: "a" });
    history.push({ path: "b" });
    history.push({ path: "a" });
    history.removeWhere((entry) => entry.path === "a");
    expect(history.pop()).toEqual({ path: "b" });
    expect(history.pop()).toBeUndefined();
  });

  test("a batch pushes and pops as one entry", () => {
    const history = new History<{ path: string }[]>(10);
    const single = [{ path: "a" }];
    const batch = [{ path: "b" }, { path: "c" }];
    history.push(single);
    history.push(batch);
    expect(history.pop()).toBe(batch);
    expect(history.pop()).toBe(single);
    expect(history.pop()).toBeUndefined();
  });

  test("removes a batch by identity and removeWhere drops batches it matches", () => {
    const history = new History<{ path: string }[]>(10);
    const kept = [{ path: "a" }, { path: "b" }];
    const removed = [{ path: "c" }, { path: "d" }];
    const gone = [{ path: "e" }];
    history.push(kept);
    history.push(removed);
    history.push(gone);
    history.remove(removed);
    history.removeWhere((batch) => batch.every((entry) => entry.path === "e"));
    expect(history.pop()).toBe(kept);
    expect(history.pop()).toBeUndefined();
  });

  test("clear empties it", () => {
    const history = new History<number>(10);
    history.push(1);
    history.clear();
    expect(history.pop()).toBeUndefined();
  });
});

describe("undo entries", () => {
  type J = { path: string };
  const run: TrashEntry = { kind: "trash", runId: 1, count: 2, dirs: ["/a"], recursive: false };

  test("pruning judgments keeps the trash entries", () => {
    const history = new History<Entry<J>>(10);
    history.push([{ path: "/a/1.ARW" }]);
    history.push(run);
    history.push([{ path: "/a/2.ARW" }]);
    history.removeWhere(isJudgments);
    expect(history.pop()).toBe(run);
    expect(history.pop()).toBeUndefined();
  });

  test("mapping judgments leaves a trash entry alone", () => {
    const history = new History<Entry<J>>(10);
    history.push([{ path: "/a/1.ARW" }, { path: "/a/2.ARW" }]);
    history.push(run);
    history.map(mapJudgments((j) => (j.path === "/a/1.ARW" ? { path: "/a/9.ARW" } : j)));
    expect(history.pop()).toBe(run);
    expect(history.pop()).toEqual([{ path: "/a/9.ARW" }, { path: "/a/2.ARW" }]);
  });

  test("peek shows the top without popping it", () => {
    const history = new History<Entry<J>>(10);
    expect(history.peek()).toBeUndefined();
    history.push(run);
    expect(history.peek()).toBe(run);
    expect(history.pop()).toBe(run);
  });

  test("an undone run is redone over the RAWs that came back only", () => {
    expect(undoneTrash(run, ["/a/1.ARW"])).toEqual({ ...run, count: 1 });
    expect(undoneTrash(run, [])).toBeNull();
  });
});
