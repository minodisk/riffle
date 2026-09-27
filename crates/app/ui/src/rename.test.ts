import { describe, expect, test } from "vitest";
import {
  SLOW_CLICK_DELAY,
  SlowClick,
  commit,
  confirmName,
  editKey,
  inlineRename,
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
