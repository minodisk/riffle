import { describe, expect, test } from "vitest";
import { ErrorList } from "./errors.js";

describe("ErrorList", () => {
  test("lists added entries in insertion order", () => {
    const errors = new ErrorList();
    errors.add("a", "one");
    errors.add("b", "two");
    expect(errors.list()).toEqual([
      { key: "a", message: "one" },
      { key: "b", message: "two" },
    ]);
  });

  test("a later error for the same key replaces it in its original position", () => {
    const errors = new ErrorList();
    errors.add("a", "old");
    errors.add("b", "two");
    errors.add("a", "new");
    expect(errors.list()).toEqual([
      { key: "a", message: "new" },
      { key: "b", message: "two" },
    ]);
  });

  test("dismiss removes only that key", () => {
    const errors = new ErrorList();
    errors.add("a", "one");
    errors.add("b", "two");
    errors.dismiss("a");
    expect(errors.list()).toEqual([{ key: "b", message: "two" }]);
  });

  test("clear empties the list", () => {
    const errors = new ErrorList();
    errors.add("a", "one");
    errors.clear();
    expect(errors.list()).toEqual([]);
  });
});
