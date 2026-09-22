import { describe, expect, test } from "vitest";
import type { Exif, ExifGroup } from "./exif.js";
import {
  type FilterState,
  type Flag,
  type Judgement,
  type Orientation,
  anchorAfterFilter,
  orientationOf,
  passes,
} from "./filter.js";

function state(
  flags: Flag[] = [],
  stars: number[] = [],
  exif: [ExifGroup, string[]][] = [],
  labels: string[] = [],
  orientations: Orientation[] = [],
): FilterState {
  return {
    flags: new Set(flags),
    stars: new Set(stars),
    labels: new Set(labels),
    orientations: new Set(orientations),
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

const unjudged: Judgement = { rating: null, flag: "none", label: null };
const rejected: Judgement = { rating: null, flag: "reject", label: null };
const threeStars: Judgement = { rating: 3, flag: "none", label: null };
const pickedTwo: Judgement = { rating: 2, flag: "pick", label: null };

describe("passes", () => {
  test("empty groups pass everything", () => {
    for (const judgement of [unjudged, rejected, threeStars, pickedTwo]) {
      expect(passes(state(), judgement, undefined, undefined)).toBe(true);
    }
  });

  test("ORs the checked flags", () => {
    const s = state(["picked", "rejected"]);
    expect(passes(s, pickedTwo, undefined, undefined)).toBe(true);
    expect(passes(s, rejected, undefined, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined, undefined)).toBe(false);
  });

  test("ORs the checked stars", () => {
    const s = state([], [0, 3]);
    expect(passes(s, unjudged, undefined, undefined)).toBe(true);
    expect(passes(s, threeStars, undefined, undefined)).toBe(true);
    expect(passes(s, pickedTwo, undefined, undefined)).toBe(false);
  });

  test("ANDs the groups", () => {
    const s = state(["untagged"], [3]);
    expect(passes(s, threeStars, undefined, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined, undefined)).toBe(false);
    expect(passes(s, pickedTwo, undefined, undefined)).toBe(false);
  });

  test("a reject counts as rejected and 0 stars", () => {
    expect(passes(state(["rejected"], [0]), rejected, undefined, undefined)).toBe(true);
    expect(passes(state(["untagged"]), rejected, undefined, undefined)).toBe(false);
  });

  test("a reject with stars counts as rejected and its stars", () => {
    const rejectedFour = { rating: 4, flag: "reject" as const, label: null };
    expect(passes(state(["rejected"], [4]), rejectedFour, undefined, undefined)).toBe(true);
    expect(passes(state([], [0]), rejectedFour, undefined, undefined)).toBe(false);
  });

  test("a pick with stars counts as picked and its stars", () => {
    expect(passes(state(["picked"], [2]), pickedTwo, undefined, undefined)).toBe(true);
    expect(passes(state(["untagged"]), pickedTwo, undefined, undefined)).toBe(false);
    expect(passes(state([], [0]), pickedTwo, undefined, undefined)).toBe(false);
  });

  test("an EXIF selection matches by label and fails a file without EXIF", () => {
    const s = state([], [], [["camera", ["ILCE-7RM5"]]]);
    expect(passes(s, unjudged, exif, undefined)).toBe(true);
    expect(passes(state([], [], [["camera", ["Other"]]]), unjudged, exif, undefined)).toBe(false);
    expect(passes(s, unjudged, undefined, undefined)).toBe(false);
    expect(passes(s, unjudged, null, undefined)).toBe(false);
  });
});

describe("passes: colour label", () => {
  const red: Judgement = { rating: null, flag: "none", label: "Red" };
  const blue: Judgement = { rating: null, flag: "none", label: "Blue" };
  const foreign: Judgement = { rating: null, flag: "none", label: "Approved" };
  const colours = ["red", "orange", "yellow", "green", "blue", "pink", "purple"];

  test("a labelled file fails none", () => {
    expect(passes(state([], [], [], ["none"]), red, undefined, undefined)).toBe(false);
    expect(passes(state([], [], [], ["none"]), unjudged, undefined, undefined)).toBe(true);
  });

  test("a Red file passes red and fails blue", () => {
    expect(passes(state([], [], [], ["red"]), red, undefined, undefined)).toBe(true);
    expect(passes(state([], [], [], ["blue"]), red, undefined, undefined)).toBe(false);
  });

  test("ORs the checked colours", () => {
    const s = state([], [], [], ["red", "blue"]);
    expect(passes(s, red, undefined, undefined)).toBe(true);
    expect(passes(s, blue, undefined, undefined)).toBe(true);
    expect(passes(s, unjudged, undefined, undefined)).toBe(false);
  });

  test("matches case-insensitively", () => {
    expect(passes(state([], [], [], ["red"]), { ...red, label: "RED" }, undefined, undefined)).toBe(
      true,
    );
  });

  test("a foreign label fails every colour and none", () => {
    for (const key of [...colours, "none"]) {
      expect(passes(state([], [], [], [key]), foreign, undefined, undefined)).toBe(false);
    }
    expect(passes(state(), foreign, undefined, undefined)).toBe(true);
  });

  test("untagged + 0 + none selects exactly the unjudged files", () => {
    const s = state(["untagged"], [0], [], ["none"]);
    expect(passes(s, unjudged, undefined, undefined)).toBe(true);
    expect(passes(s, red, undefined, undefined)).toBe(false);
    expect(passes(s, threeStars, undefined, undefined)).toBe(false);
    expect(passes(s, { rating: null, flag: "pick", label: null }, undefined, undefined)).toBe(
      false,
    );
    expect(passes(s, rejected, undefined, undefined)).toBe(false);
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

describe("a judgement under untagged + 0 + none", () => {
  const s = state(["untagged"], [0], [], ["none"]);
  const judgements: Record<string, Judgement> = {
    rating: { rating: 3, flag: "none", label: null },
    reject: { rating: null, flag: "reject", label: null },
    pick: { rating: null, flag: "pick", label: null },
    label: { rating: null, flag: "none", label: "Red" },
  };

  function after(all: string[], judged: string, judgement: Judgement) {
    const pass = (path: string) =>
      passes(s, path === judged ? judgement : unjudged, undefined, undefined);
    return anchorAfterFilter(all, pass, judged);
  }

  for (const [name, judgement] of Object.entries(judgements)) {
    test(`${name} moves to the next unjudged file after it`, () => {
      expect(after(["a", "b", "c"], "b", judgement)).toBe("c");
    });

    test(`${name} falls back to the last unjudged file before it`, () => {
      expect(after(["a", "b", "c"], "c", judgement)).toBe("b");
    });

    test(`${name} on the only file leaves the empty view`, () => {
      expect(after(["a"], "a", judgement)).toBeUndefined();
    });
  }
});

describe("orientationOf", () => {
  test("a quarter turn is portrait", () => {
    expect(orientationOf(6)).toBe("portrait");
    expect(orientationOf(8)).toBe("portrait");
  });

  test("everything else is landscape", () => {
    expect(orientationOf(1)).toBe("landscape");
    expect(orientationOf(3)).toBe("landscape");
    expect(orientationOf(99)).toBe("landscape");
  });
});

describe("passes: orientation", () => {
  test("an empty group passes both shapes", () => {
    expect(passes(state(), unjudged, undefined, 1)).toBe(true);
    expect(passes(state(), unjudged, undefined, 6)).toBe(true);
  });

  test("portrait passes a quarter turn only", () => {
    const s = state([], [], [], [], ["portrait"]);
    expect(passes(s, unjudged, undefined, 6)).toBe(true);
    expect(passes(s, unjudged, undefined, 8)).toBe(true);
    expect(passes(s, unjudged, undefined, 1)).toBe(false);
    expect(passes(s, unjudged, undefined, 3)).toBe(false);
  });

  test("landscape passes the unrotated tags only", () => {
    const s = state([], [], [], [], ["landscape"]);
    expect(passes(s, unjudged, undefined, 1)).toBe(true);
    expect(passes(s, unjudged, undefined, 3)).toBe(true);
    expect(passes(s, unjudged, undefined, 6)).toBe(false);
    expect(passes(s, unjudged, undefined, 8)).toBe(false);
  });

  test("both checked passes both shapes", () => {
    const s = state([], [], [], [], ["portrait", "landscape"]);
    expect(passes(s, unjudged, undefined, 1)).toBe(true);
    expect(passes(s, unjudged, undefined, 6)).toBe(true);
  });

  test("an unknown orientation fails a non-empty selection", () => {
    expect(passes(state([], [], [], [], ["landscape"]), unjudged, undefined, undefined)).toBe(
      false,
    );
  });

  test("ANDs with the other groups", () => {
    const s = state(["untagged"], [], [], [], ["portrait"]);
    expect(passes(s, unjudged, undefined, 6)).toBe(true);
    expect(passes(s, unjudged, undefined, 1)).toBe(false);
    expect(passes(s, pickedTwo, undefined, 6)).toBe(false);
  });
});
