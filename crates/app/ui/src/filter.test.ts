import { describe, expect, test } from "vitest";
import type { Exif, ExifGroup } from "./exif.js";
import { type FilterState, type Flag, anchorAfterFilter, passes } from "./filter.js";

function state(
  flags: Flag[] = [],
  stars: number[] = [],
  exif: [ExifGroup, string[]][] = [],
  labels: string[] = [],
): FilterState {
  return {
    flags: new Set(flags),
    stars: new Set(stars),
    labels: new Set(labels),
    exif: new Map(exif.map(([group, values]) => [group, new Set(values)])),
  };
}

const exif: Exif = {
  camera: "ILCE-7RM5",
  lens: null,
  aperture: null,
  shutter: null,
  iso: null,
  focal_length: null,
};

const unjudged = { rating: null, pick: false, label: null };
const rejected = { rating: -1, pick: false, label: null };
const threeStars = { rating: 3, pick: false, label: null };
const pickedTwo = { rating: 2, pick: true, label: null };

describe("passes", () => {
  test("empty groups pass everything", () => {
    for (const judgement of [unjudged, rejected, threeStars, pickedTwo]) {
      expect(passes(state(), judgement, undefined)).toBe(true);
    }
  });

  test("ORs the checked flags", () => {
    const s = state(["picked", "rejected"]);
    expect(passes(s, pickedTwo, undefined)).toBe(true);
    expect(passes(s, rejected, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined)).toBe(false);
  });

  test("ORs the checked stars", () => {
    const s = state([], [0, 3]);
    expect(passes(s, unjudged, undefined)).toBe(true);
    expect(passes(s, threeStars, undefined)).toBe(true);
    expect(passes(s, pickedTwo, undefined)).toBe(false);
  });

  test("ANDs the groups", () => {
    const s = state(["untagged"], [3]);
    expect(passes(s, threeStars, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined)).toBe(false);
    expect(passes(s, pickedTwo, undefined)).toBe(false);
  });

  test("a reject counts as rejected and 0 stars", () => {
    expect(passes(state(["rejected"], [0]), rejected, undefined)).toBe(true);
    expect(passes(state(["untagged"]), rejected, undefined)).toBe(false);
  });

  test("a pick with stars counts as picked and its stars", () => {
    expect(passes(state(["picked"], [2]), pickedTwo, undefined)).toBe(true);
    expect(passes(state(["untagged"]), pickedTwo, undefined)).toBe(false);
    expect(passes(state([], [0]), pickedTwo, undefined)).toBe(false);
  });

  test("an EXIF selection matches by label and fails a file without EXIF", () => {
    const s = state([], [], [["camera", ["ILCE-7RM5"]]]);
    expect(passes(s, unjudged, exif)).toBe(true);
    expect(passes(state([], [], [["camera", ["Other"]]]), unjudged, exif)).toBe(false);
    expect(passes(s, unjudged, undefined)).toBe(false);
    expect(passes(s, unjudged, null)).toBe(false);
  });
});

describe("passes: colour label", () => {
  const red = { rating: null, pick: false, label: "Red" };
  const blue = { rating: null, pick: false, label: "Blue" };
  const foreign = { rating: null, pick: false, label: "Approved" };
  const colours = ["red", "orange", "yellow", "green", "blue", "pink", "purple"];

  test("a labelled file fails none", () => {
    expect(passes(state([], [], [], ["none"]), red, undefined)).toBe(false);
    expect(passes(state([], [], [], ["none"]), unjudged, undefined)).toBe(true);
  });

  test("a Red file passes red and fails blue", () => {
    expect(passes(state([], [], [], ["red"]), red, undefined)).toBe(true);
    expect(passes(state([], [], [], ["blue"]), red, undefined)).toBe(false);
  });

  test("ORs the checked colours", () => {
    const s = state([], [], [], ["red", "blue"]);
    expect(passes(s, red, undefined)).toBe(true);
    expect(passes(s, blue, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined)).toBe(false);
  });

  test("matches case-insensitively", () => {
    expect(passes(state([], [], [], ["red"]), { ...red, label: "RED" }, undefined)).toBe(true);
  });

  test("a foreign label fails every colour and none", () => {
    for (const key of [...colours, "none"]) {
      expect(passes(state([], [], [], [key]), foreign, undefined)).toBe(false);
    }
    expect(passes(state(), foreign, undefined)).toBe(true);
  });

  test("untagged + 0 + none selects exactly the unjudged files", () => {
    const s = state(["untagged"], [0], [], ["none"]);
    expect(passes(s, unjudged, undefined)).toBe(true);
    expect(passes(s, red, undefined)).toBe(false);
    expect(passes(s, threeStars, undefined)).toBe(false);
    expect(passes(s, { rating: null, pick: true, label: null }, undefined)).toBe(false);
    expect(passes(s, rejected, undefined)).toBe(false);
  });
});

describe("anchorAfterFilter", () => {
  const all = ["a", "b", "c", "d"];

  test("stays on the anchor if it passes", () => {
    expect(anchorAfterFilter(all, () => true, "b")).toBe("b");
  });

  test("moves to the next passing file after it", () => {
    expect(anchorAfterFilter(all, (path) => path !== "b", "b")).toBe("c");
    expect(anchorAfterFilter(all, (path) => path === "a" || path === "d", "b")).toBe("d");
  });

  test("falls back to the last passing file before it", () => {
    expect(anchorAfterFilter(all, (path) => path === "a" || path === "b", "c")).toBe("b");
  });

  test("returns undefined when nothing passes", () => {
    expect(anchorAfterFilter(all, () => false, "b")).toBeUndefined();
    expect(anchorAfterFilter(all, () => true, undefined)).toBeUndefined();
  });
});
