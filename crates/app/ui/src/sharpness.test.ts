import { describe, expect, test } from "vitest";
import { relativeSharpness } from "./sharpness.js";

describe("relativeSharpness", () => {
  test("a lone file is the best with ratio 1", () => {
    expect(relativeSharpness([42])).toEqual([{ ratio: 1, best: true }]);
  });

  test("nulls are skipped and stay null", () => {
    expect(relativeSharpness([null, 10, null, 5])).toEqual([
      null,
      { ratio: 1, best: true },
      null,
      { ratio: 0.5, best: false },
    ]);
  });

  test("the window is clamped at both ends", () => {
    const result = relativeSharpness([4, 1, 1, 1, 1, 1, 8], 2);
    expect(result[0]).toEqual({ ratio: 1, best: true });
    expect(result[1]).toEqual({ ratio: 0.25, best: false });
    expect(result[6]).toEqual({ ratio: 1, best: true });
    expect(result[5]).toEqual({ ratio: 0.125, best: false });
  });

  test("equal scores tie", () => {
    expect(relativeSharpness([3, 3])).toEqual([
      { ratio: 1, best: true },
      { ratio: 1, best: true },
    ]);
  });

  test("a burst of three marks exactly the sharpest", () => {
    const result = relativeSharpness([20, 80, 40]);
    expect(result.map((value) => value?.best)).toEqual([false, true, false]);
    expect(result.map((value) => value?.ratio)).toEqual([0.25, 1, 0.5]);
  });
});
