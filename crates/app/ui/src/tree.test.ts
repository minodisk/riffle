import { describe, expect, test } from "vitest";
import {
  EMPTY_TREE,
  addRoots,
  ancestorsWithin,
  collapse,
  expand,
  rootOf,
  rows,
  setChildren,
  step,
  treeKey,
} from "./tree.js";

const home = { name: "me", path: "/home/me" };
const card = { name: "card", path: "/media/me/card" };

function drawn(tree: ReturnType<typeof addRoots>): string[] {
  return rows(tree).map(({ node, depth }) => `${depth}:${node.name}`);
}

describe("tree state", () => {
  test("roots start unlisted and collapsed, and are not repeated", () => {
    const tree = addRoots(addRoots(EMPTY_TREE, [home, card]), [card]);
    expect(tree.roots).toEqual([home, card]);
    expect(tree.nodes.get("/home/me")).toEqual({
      ...home,
      children: undefined,
      expanded: false,
      rawCount: undefined,
    });
  });

  test("an expanded folder shows its children once listed", () => {
    let tree = expand(addRoots(EMPTY_TREE, [home, card]), "/home/me");
    expect(drawn(tree)).toEqual(["0:me", "0:card"]);
    tree = setChildren(tree, "/home/me", 3, [{ name: "Pictures", path: "/home/me/Pictures" }]);
    expect(drawn(tree)).toEqual(["0:me", "1:Pictures", "0:card"]);
    expect(tree.nodes.get("/home/me")?.rawCount).toBe(3);
  });

  test("collapsing keeps the children for the next expand", () => {
    let tree = expand(addRoots(EMPTY_TREE, [home]), "/home/me");
    tree = setChildren(tree, "/home/me", 0, [{ name: "a", path: "/home/me/a" }]);
    tree = collapse(tree, "/home/me");
    expect(drawn(tree)).toEqual(["0:me"]);
    expect(drawn(expand(tree, "/home/me"))).toEqual(["0:me", "1:a"]);
  });

  test("re-listing keeps the state of a child already known", () => {
    let tree = expand(addRoots(EMPTY_TREE, [home]), "/home/me");
    tree = setChildren(tree, "/home/me", 0, [{ name: "a", path: "/home/me/a" }]);
    tree = setChildren(expand(tree, "/home/me/a"), "/home/me/a", 0, [
      { name: "b", path: "/home/me/a/b" },
    ]);
    tree = setChildren(tree, "/home/me", 0, [
      { name: "a", path: "/home/me/a" },
      { name: "new", path: "/home/me/new" },
    ]);
    expect(drawn(tree)).toEqual(["0:me", "1:a", "2:b", "1:new"]);
  });

  test("changes to an unknown path leave the tree as is", () => {
    const tree = addRoots(EMPTY_TREE, [home]);
    expect(expand(tree, "/nope")).toBe(tree);
    expect(setChildren(tree, "/nope", 1, [])).toBe(tree);
  });
});

describe("step", () => {
  // me > (Pictures > 2026), card
  function tree(): ReturnType<typeof addRoots> {
    let t = expand(addRoots(EMPTY_TREE, [home, card]), "/home/me");
    t = setChildren(t, "/home/me", 0, [{ name: "Pictures", path: "/home/me/Pictures" }]);
    t = expand(t, "/home/me/Pictures");
    return setChildren(t, "/home/me/Pictures", 0, [
      { name: "2026", path: "/home/me/Pictures/2026" },
    ]);
  }

  test("down and up walk the visible rows across depth", () => {
    const drawnRows = rows(tree());
    expect(step(drawnRows, "/home/me", "down")).toBe("/home/me/Pictures");
    expect(step(drawnRows, "/home/me/Pictures", "down")).toBe("/home/me/Pictures/2026");
    expect(step(drawnRows, "/home/me/Pictures/2026", "down")).toBe("/media/me/card");
    expect(step(drawnRows, "/media/me/card", "up")).toBe("/home/me/Pictures/2026");
    expect(step(drawnRows, "/home/me/Pictures", "up")).toBe("/home/me");
  });

  test("down and up between siblings", () => {
    const drawnRows = rows(addRoots(EMPTY_TREE, [home, card]));
    expect(step(drawnRows, "/home/me", "down")).toBe("/media/me/card");
    expect(step(drawnRows, "/media/me/card", "up")).toBe("/home/me");
  });

  test("clamps at both ends", () => {
    const drawnRows = rows(tree());
    expect(step(drawnRows, "/home/me", "up")).toBe("/home/me");
    expect(step(drawnRows, "/media/me/card", "down")).toBe("/media/me/card");
  });

  test("home and end are the first and last rows", () => {
    const drawnRows = rows(tree());
    expect(step(drawnRows, "/home/me/Pictures", "home")).toBe("/home/me");
    expect(step(drawnRows, "/home/me/Pictures", "end")).toBe("/media/me/card");
    expect(step(drawnRows, null, "end")).toBe("/media/me/card");
  });

  test("a missing or stale cursor lands on the first row", () => {
    const collapsed = rows(collapse(tree(), "/home/me"));
    for (const key of ["up", "down"] as const) {
      expect(step(collapsed, null, key)).toBe("/home/me");
      expect(step(collapsed, "/home/me/Pictures/2026", key)).toBe("/home/me");
    }
  });

  test("an empty tree has no row", () => {
    for (const key of ["up", "down", "home", "end"] as const) {
      expect(step([], null, key)).toBeNull();
      expect(step([], "/home/me", key)).toBeNull();
    }
  });
});

describe("treeKey", () => {
  const pictures = { name: "Pictures", path: "/home/me/Pictures" };
  const empty = { name: "empty", path: "/home/me/empty" };
  const y2026 = { name: "2026", path: "/home/me/Pictures/2026" };

  // me > (Pictures > 2026, empty), card
  function tree(): ReturnType<typeof addRoots> {
    let t = expand(addRoots(EMPTY_TREE, [home, card]), home.path);
    t = setChildren(t, home.path, 0, [pictures, empty]);
    t = setChildren(t, empty.path, 0, []);
    t = expand(t, pictures.path);
    return setChildren(t, pictures.path, 0, [y2026]);
  }

  test("right expands a collapsed row that can expand, listed or not", () => {
    const collapsed = collapse(tree(), home.path);
    expect(treeKey(rows(collapsed), home.path, "right")).toEqual({
      kind: "expand",
      path: home.path,
    });
    expect(treeKey(rows(tree()), card.path, "right")).toEqual({
      kind: "expand",
      path: card.path,
    });
  });

  test("right on an expanded row enters its first child", () => {
    expect(treeKey(rows(tree()), home.path, "right")).toEqual({
      kind: "focus",
      path: pictures.path,
    });
  });

  test("right does nothing on a leaf or an expanded row still listing", () => {
    expect(treeKey(rows(tree()), empty.path, "right")).toBeNull();
    const listing = rows(expand(tree(), card.path));
    expect(treeKey(listing, card.path, "right")).toBeNull();
  });

  test("left collapses an expanded row, even one still listing", () => {
    expect(treeKey(rows(tree()), pictures.path, "left")).toEqual({
      kind: "collapse",
      path: pictures.path,
    });
    expect(treeKey(rows(expand(tree(), card.path)), card.path, "left")).toEqual({
      kind: "collapse",
      path: card.path,
    });
  });

  test("left on a collapsed row or a leaf goes to the parent", () => {
    expect(treeKey(rows(tree()), y2026.path, "left")).toEqual({
      kind: "focus",
      path: pictures.path,
    });
    expect(treeKey(rows(tree()), empty.path, "left")).toEqual({
      kind: "focus",
      path: home.path,
    });
    const collapsed = rows(collapse(tree(), pictures.path));
    expect(treeKey(collapsed, pictures.path, "left")).toEqual({
      kind: "focus",
      path: home.path,
    });
  });

  test("left on an expanded leaf goes to the parent", () => {
    const opened = rows(expand(tree(), empty.path));
    expect(treeKey(opened, empty.path, "left")).toEqual({ kind: "focus", path: home.path });
  });

  test("left does nothing on a collapsed root", () => {
    expect(treeKey(rows(tree()), card.path, "left")).toBeNull();
    expect(treeKey(rows(collapse(tree(), home.path)), home.path, "left")).toBeNull();
  });

  test("enter opens the cursor row", () => {
    expect(treeKey(rows(tree()), y2026.path, "enter")).toEqual({
      kind: "open",
      path: y2026.path,
    });
  });

  test("a missing or stale cursor does nothing", () => {
    const collapsed = rows(collapse(tree(), home.path));
    for (const key of ["left", "right", "enter"] as const) {
      expect(treeKey(collapsed, null, key)).toBeNull();
      expect(treeKey(collapsed, y2026.path, key)).toBeNull();
      expect(treeKey([], null, key)).toBeNull();
    }
  });
});

describe("ancestorsWithin", () => {
  test("walks from the root down to the folder itself", () => {
    expect(ancestorsWithin(["/home/me", "/mnt/c"], "/home/me/Pictures/2026")).toEqual([
      "/home/me",
      "/home/me/Pictures",
      "/home/me/Pictures/2026",
    ]);
  });

  test("a root is its own chain, trailing separator or not", () => {
    expect(ancestorsWithin(["/home/me"], "/home/me/")).toEqual(["/home/me"]);
  });

  test("a sibling sharing a name prefix is not held", () => {
    expect(ancestorsWithin(["/home/me"], "/home/meg/x")).toBeNull();
  });

  test("the deepest root wins", () => {
    expect(ancestorsWithin(["/", "/home/me"], "/home/me/a")).toEqual(["/home/me", "/home/me/a"]);
  });

  test("the filesystem root holds every absolute path", () => {
    expect(ancestorsWithin(["/"], "/data/raw")).toEqual(["/", "/data", "/data/raw"]);
  });

  test("a drive root joins without doubling the separator", () => {
    expect(ancestorsWithin(["C:\\Users\\me", "D:\\"], "D:\\Photos\\2026")).toEqual([
      "D:\\",
      "D:\\Photos",
      "D:\\Photos\\2026",
    ]);
  });

  test("a drive letter matches in either case, spelled as the root has it", () => {
    expect(ancestorsWithin(["C:\\"], "c:\\Users")).toEqual(["C:\\", "C:\\Users"]);
  });

  test("forward slashes in a Windows path still match", () => {
    expect(ancestorsWithin(["C:\\Users\\me"], "C:/Users/me/Pictures")).toEqual([
      "C:\\Users\\me",
      "C:\\Users\\me\\Pictures",
    ]);
  });

  test("a UNC share", () => {
    expect(ancestorsWithin(["\\\\nas\\photos"], "\\\\nas\\photos\\2026")).toEqual([
      "\\\\nas\\photos",
      "\\\\nas\\photos\\2026",
    ]);
  });

  test("no root holds the folder", () => {
    expect(ancestorsWithin(["C:\\"], "\\\\nas\\photos")).toBeNull();
    expect(ancestorsWithin([], "/home/me")).toBeNull();
  });
});

describe("rootOf", () => {
  test("a drive, a UNC share, or the filesystem root", () => {
    expect(rootOf("E:\\DCIM\\100MSDCF")).toBe("E:\\");
    expect(rootOf("\\\\nas\\photos\\2026")).toBe("\\\\nas\\photos");
    expect(rootOf("/srv/photos")).toBe("/");
  });

  test("the added root holds the folder", () => {
    for (const path of ["E:\\DCIM", "\\\\nas\\photos\\2026", "/srv/photos"]) {
      expect(ancestorsWithin([rootOf(path)], path)?.at(-1)).toBe(path);
    }
  });
});
