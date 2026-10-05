import { describe, expect, test } from "vitest";
import {
  RenamesInFlight,
  SLOW_CLICK_DELAY,
  SlowClick,
  commit,
  confirmName,
  displayName,
  editKey,
  editOutcome,
  inlineRename,
  pendingOutcome,
  stemLength,
} from "./rename.js";

describe("inlineRename", () => {
  test("starts with the original name as its value, not committed", () => {
    expect(inlineRename("folder", "/a/b", "b")).toEqual({
      kind: "folder",
      path: "/a/b",
      original: "b",
      value: "b",
      committed: false,
    });
  });
});

describe("editKey", () => {
  test("Enter confirms, Escape cancels, anything else is the input's", () => {
    expect(editKey("enter")).toBe("confirm");
    expect(editKey("escape")).toBe("cancel");
    expect(editKey("arrowleft")).toBe("native");
    expect(editKey("a")).toBe("native");
    expect(editKey("shift+enter")).toBe("native");
    expect(editKey(null)).toBe("native");
  });
});

describe("confirmName", () => {
  test("trims the typed name", () => {
    expect(confirmName("old", "  new ")).toBe("new");
  });

  test("an empty or unchanged name renames nothing", () => {
    expect(confirmName("old", "")).toBeNull();
    expect(confirmName("old", "   ")).toBeNull();
    expect(confirmName("old", "old")).toBeNull();
    expect(confirmName("old", " old ")).toBeNull();
  });

  test("a case-only change is a rename", () => {
    expect(confirmName("Photos", "photos")).toBe("photos");
  });
});

describe("stemLength", () => {
  test("selects the name up to its extension", () => {
    expect(stemLength("DSC01234.ARW")).toBe(8);
    expect(stemLength("a.b.dng")).toBe(3);
  });

  test("selects a name with no extension whole", () => {
    expect(stemLength("DSC01234")).toBe(8);
    expect(stemLength(".hidden")).toBe(7);
  });
});

describe("commit", () => {
  test("answers the first decision only", () => {
    const rename = inlineRename("folder", "/a/b", "b");
    expect(commit(rename, "confirm")).toBe("confirm");
    expect(commit(rename, "confirm")).toBeNull();
  });

  test("the blur after an Escape does not confirm", () => {
    const rename = inlineRename("folder", "/a/b", "b");
    expect(commit(rename, "cancel")).toBe("cancel");
    expect(commit(rename, "confirm")).toBeNull();
  });
});

describe("SlowClick", () => {
  test("a first click on an item that is not current does not arm", () => {
    const slow = new SlowClick();
    expect(slow.click("/a", false, 0)).toBe(false);
    expect(slow.due("/a", SLOW_CLICK_DELAY)).toBe(false);
  });

  test("a click on the current item arms, due after the delay", () => {
    const slow = new SlowClick();
    expect(slow.click("/a", true, 100)).toBe(true);
    expect(slow.due("/a", 100 + SLOW_CLICK_DELAY)).toBe(true);
  });

  test("not due before the delay", () => {
    const slow = new SlowClick();
    slow.click("/a", true, 100);
    expect(slow.due("/a", 100 + SLOW_CLICK_DELAY - 1)).toBe(false);
  });

  test("a double-click cancels the arm", () => {
    const slow = new SlowClick();
    slow.click("/a", true, 0);
    slow.cancel();
    expect(slow.due("/a", SLOW_CLICK_DELAY)).toBe(false);
  });

  test("a click on another item replaces the arm", () => {
    const slow = new SlowClick();
    slow.click("/a", true, 0);
    slow.click("/b", true, 10);
    expect(slow.due("/a", SLOW_CLICK_DELAY + 10)).toBe(false);
    expect(slow.due("/b", SLOW_CLICK_DELAY + 10)).toBe(true);
  });

  test("a key press cancels the arm", () => {
    const slow = new SlowClick();
    slow.click("/a", true, 0);
    slow.cancel();
    expect(slow.due("/a", 2 * SLOW_CLICK_DELAY)).toBe(false);
  });

  test("due answers true only once", () => {
    const slow = new SlowClick();
    slow.click("/a", true, 0);
    expect(slow.due("/a", SLOW_CLICK_DELAY)).toBe(true);
    expect(slow.due("/a", SLOW_CLICK_DELAY)).toBe(false);
  });
});

describe("RenamesInFlight", () => {
  test("blocks the renamed folder and everything under it", () => {
    const renames = new RenamesInFlight();
    renames.start("D:\\photos");
    expect(renames.blocks("D:\\photos", false)).toBe(true);
    expect(renames.blocks("D:\\photos\\2026", false)).toBe(true);
    expect(renames.blocks("D:\\photos\\2026\\01", false)).toBe(true);
  });

  test("leaves a sibling sharing the name's prefix and an unrelated folder", () => {
    const renames = new RenamesInFlight();
    renames.start("D:\\photos");
    expect(renames.blocks("D:\\photos2", false)).toBe(false);
    expect(renames.blocks("D:\\photos2\\a", false)).toBe(false);
    expect(renames.blocks("E:\\other", false)).toBe(false);
  });

  test("a case-only difference in a folder name blocks only with ignoreCase", () => {
    const renames = new RenamesInFlight();
    renames.start("/Users/me/Photos");
    expect(renames.blocks("/Users/me/photos/a", true)).toBe(true);
    expect(renames.blocks("/Users/me/photos/a", false)).toBe(false);
  });

  test("settling the rename clears the block", () => {
    const renames = new RenamesInFlight();
    renames.start("/a/photos");
    renames.settle("/a/photos");
    expect(renames.blocks("/a/photos", false)).toBe(false);
    expect(renames.blocks("/a/photos/b", false)).toBe(false);
  });

  test("settling one of two renames leaves the other's block", () => {
    const renames = new RenamesInFlight();
    renames.start("/a/one");
    renames.start("/a/two");
    renames.settle("/a/one");
    expect(renames.blocks("/a/one/x", false)).toBe(false);
    expect(renames.blocks("/a/two/x", false)).toBe(true);
  });

  test("settling a path never started is a no-op", () => {
    const renames = new RenamesInFlight();
    renames.start("/a/one");
    renames.settle("/a/other");
    expect(renames.blocks("/a/one", false)).toBe(true);
  });
});

describe("displayName", () => {
  test("shows the pending name for the path it is held for", () => {
    expect(displayName({ path: "/a/b", name: "c" }, "/a/b", "b")).toBe("c");
  });

  test("shows the real name for any other path or with nothing held", () => {
    expect(displayName({ path: "/a/b", name: "c" }, "/a/x", "x")).toBe("x");
    expect(displayName(null, "/a/b", "b")).toBe("b");
  });
});

describe("pendingOutcome", () => {
  test("an empty or unchanged pending name keeps the held rename", () => {
    expect(pendingOutcome("b", "c", "")).toBe("keep");
    expect(pendingOutcome("b", "c", "  ")).toBe("keep");
    expect(pendingOutcome("b", "c", " c ")).toBe("keep");
  });

  test("the original name cancels the held rename", () => {
    expect(pendingOutcome("b", "c", "b")).toBe("cancel");
    expect(pendingOutcome("b", "c", " b ")).toBe("cancel");
  });

  test("any other name replaces it", () => {
    expect(pendingOutcome("b", "c", " d ")).toEqual({ rename: "d" });
  });
});

describe("editOutcome", () => {
  const pending = { path: "/a/b", name: "c" };

  test("follows pendingOutcome on the cell whose rename is held", () => {
    expect(editOutcome(pending, "/a/b", "b", "c")).toBe("keep");
    expect(editOutcome(pending, "/a/b", "b", "b")).toBe("cancel");
    expect(editOutcome(pending, "/a/b", "b", "d")).toEqual({ rename: "d" });
  });

  test("follows confirmName on any other cell", () => {
    expect(editOutcome(pending, "/a/x", "x", "x")).toBe("keep");
    expect(editOutcome(null, "/a/x", "x", "")).toBe("keep");
    expect(editOutcome(null, "/a/x", "x", " y ")).toEqual({ rename: "y" });
  });
});
