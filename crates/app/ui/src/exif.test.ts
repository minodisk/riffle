import { describe, expect, test } from "vitest";
import { type Exif, exifKey, focalRange, focalRanges } from "./exif.js";

const exif: Exif = {
  camera: "ILCE-7RM5",
  lens: "FE 35mm F1.4 GM",
  aperture: { value: 1.4, label: "f/1.4" },
  shutter: { value: 0.004, label: "1/250 s" },
  iso: { value: 100, label: "ISO 100" },
  focal_length: { value: 35, label: "35 mm" },
};

describe("focalRange", () => {
  test.each([
    [0, "<24 mm"],
    [23.9, "<24 mm"],
    [24, "24–35 mm"],
    [35, "35–50 mm"],
    [50, "50–85 mm"],
    [85, "85–135 mm"],
    [135, "135–200 mm"],
    [199.9, "135–200 mm"],
    [200, ">200 mm"],
    [600, ">200 mm"],
  ])("%s mm falls in %s", (mm, label) => {
    expect(focalRanges[focalRange(mm)].label).toBe(label);
  });
});

describe("exifKey", () => {
  test("returns null without EXIF", () => {
    expect(exifKey(null, "camera")).toBeNull();
    expect(exifKey(undefined, "iso")).toBeNull();
  });

  test("orders camera and lens by their text", () => {
    expect(exifKey(exif, "camera")).toEqual({ label: "ILCE-7RM5", order: "ILCE-7RM5" });
    expect(exifKey(exif, "lens")).toEqual({ label: "FE 35mm F1.4 GM", order: "FE 35mm F1.4 GM" });
  });

  test("orders labelled values by their numeric value", () => {
    expect(exifKey(exif, "aperture")).toEqual({ label: "f/1.4", order: 1.4 });
    expect(exifKey(exif, "shutter")).toEqual({ label: "1/250 s", order: 0.004 });
    expect(exifKey(exif, "iso")).toEqual({ label: "ISO 100", order: 100 });
  });

  test("buckets the focal length into its range", () => {
    expect(exifKey(exif, "focal")).toEqual({ label: "35–50 mm", order: 2 });
  });

  test("returns null for a missing field", () => {
    const empty: Exif = {
      camera: null,
      lens: null,
      aperture: null,
      shutter: null,
      iso: null,
      focal_length: null,
    };
    for (const group of ["camera", "lens", "aperture", "shutter", "iso", "focal"] as const) {
      expect(exifKey(empty, group)).toBeNull();
    }
  });
});
