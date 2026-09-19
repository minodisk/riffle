import { describe, expect, test } from "vitest";
import { placeholderRect } from "./zoom.js";

const focus = { sensor_w: 6000, sensor_h: 4000, x: 1500, y: 1000 };

describe("placeholderRect", () => {
  test("scales to the full JPEG size when the crop header carried it", () => {
    const rect = placeholderRect(1600, 1066, focus, { width: 3000, height: 2000 });
    expect(rect).toEqual({ x: 750, y: 500, width: 3000, height: 2000 });
  });

  test("falls back to the sensor size when no full size is known", () => {
    const rect = placeholderRect(1500, 1000, focus, null);
    expect(rect).toEqual({ x: 1500, y: 1000, width: 6000, height: 4000 });
  });

  test("the focus point lands at the origin in both cases", () => {
    for (const full of [{ width: 3000, height: 2000 }, null]) {
      const rect = placeholderRect(1500, 1000, focus, full);
      // The preview pixel under the focus point, drawn at (-x, -y).
      const px = (focus.x / focus.sensor_w) * 1500;
      const py = (focus.y / focus.sensor_h) * 1000;
      expect(-rect.x + (px * rect.width) / 1500).toBeCloseTo(0);
      expect(-rect.y + (py * rect.height) / 1000).toBeCloseTo(0);
    }
  });
});
