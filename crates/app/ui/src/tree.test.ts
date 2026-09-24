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
