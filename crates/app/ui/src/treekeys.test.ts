import { describe, expect, test } from "vitest";
import { TREE_PASSTHROUGH, treeGate } from "./treekeys.js";

describe("treeGate", () => {
  test("lets only the window and folder actions through", () => {
    expect([...TREE_PASSTHROUGH].sort()).toEqual(
      ["open", "toggleLeft", "toggleRight", "toggleSides", "toggleStrip"].sort(),
    );
    for (const action of TREE_PASSTHROUGH) {
      expect(treeGate(action)).toBe("run");
    }
  });

  test("swallows the culling actions", () => {
    for (const action of ["pick", "next", "undo", "rate1", "red"]) {
      expect(treeGate(action)).toBe("swallow");
    }
  });

  test("swallows an unknown or unbound action", () => {
    expect(treeGate("somethingNew")).toBe("swallow");
    expect(treeGate(undefined)).toBe("swallow");
  });
});
