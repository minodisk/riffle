import { describe, expect, test } from "vitest";
import {
  EXIF_HEADING,
  MAKER_NOTE_LABEL,
  type Metadata,
  RIFFLE_HEADING,
  metaGroups,
} from "./meta.js";

const empty: Metadata = {
  name: "DSC00001.ARW",
  camera: null,
  lens: null,
  aperture: null,
  shutter: null,
  shutter_type: null,
  focus_mode: null,
  af_area: null,
  af_tracking: null,
  drive: null,
  stabilization: null,
  exposure_mode: null,
  metering: null,
  creative_style: null,
  dro: null,
  raw_type: null,
  iso: null,
  focal_length: null,
  exposure_bias: null,
  focus_distance: null,
  captured_at: null,
};

const sony: Metadata = {
  ...empty,
  camera: "SONY ILCE-1",
  lens: "FE 50mm F1.2 GM",
  aperture: "f/2.8",
  shutter: "1/500",
  shutter_type: "Electronic front curtain",
  focus_mode: "AF-C",
  af_area: "Custom AF Area",
  af_tracking: "Face tracking",
  drive: "Continuous, frame 2",
  stabilization: "On",
  exposure_mode: "Manual",
  metering: "Multi-segment",
  creative_style: "Standard",
  dro: "Auto",
  raw_type: "Lossless Compressed RAW",
  iso: "ISO 100",
  focal_length: "50 mm",
  exposure_bias: "+0.3 EV",
  captured_at: "2026-09-24 12:00:00",
};

const leica: Metadata = {
  ...empty,
  name: "L1000001.DNG",
  camera: "Leica M11-P",
  aperture: "f/2.0 (est.)",
  focus_distance: "1.5 m",
};

describe("metaGroups", () => {
  test("splits Sony-like input into standard EXIF, maker note and Riffle", () => {
    expect(metaGroups(sony, 303.54)).toEqual([
      {
        heading: EXIF_HEADING,
        sections: [
          {
            label: null,
            rows: [
              { label: "Aperture", value: "f/2.8" },
              { label: "Shutter", value: "1/500" },
              { label: "ISO", value: "ISO 100" },
              { label: "Focal length", value: "50 mm" },
              { label: "Exposure", value: "+0.3 EV" },
              { label: "Camera", value: "SONY ILCE-1" },
              { label: "Lens", value: "FE 50mm F1.2 GM" },
              { label: "Captured", value: "2026-09-24 12:00:00" },
            ],
          },
          {
            label: MAKER_NOTE_LABEL,
            rows: [
              { label: "Shutter type", value: "Electronic front curtain" },
              { label: "Focus mode", value: "AF-C" },
              { label: "AF area", value: "Custom AF Area" },
              { label: "AF tracking", value: "Face tracking" },
              { label: "Drive", value: "Continuous, frame 2" },
              { label: "Stabilization", value: "On" },
              { label: "Exposure mode", value: "Manual" },
              { label: "Metering", value: "Multi-segment" },
              { label: "Creative style", value: "Standard" },
              { label: "DRO", value: "Auto" },
              { label: "RAW type", value: "Lossless Compressed RAW" },
            ],
          },
        ],
      },
      {
        heading: RIFFLE_HEADING,
        sections: [{ label: null, rows: [{ label: "Sharpness", value: "303.5" }] }],
      },
    ]);
  });

  test("puts the Leica focus distance in the maker note section and drops null rows", () => {
    const [exif] = metaGroups(leica, null);
    expect(exif).toEqual({
      heading: EXIF_HEADING,
      sections: [
        {
          label: null,
          rows: [
            { label: "Aperture", value: "f/2.0 (est.)" },
            { label: "Camera", value: "Leica M11-P" },
          ],
        },
        { label: MAKER_NOTE_LABEL, rows: [{ label: "Focus distance", value: "1.5 m" }] },
      ],
    });
  });

  test("leaves out the maker note section when no maker note field is set", () => {
    const groups = metaGroups(
      {
        ...sony,
        shutter_type: null,
        focus_mode: null,
        af_area: null,
        af_tracking: null,
        drive: null,
        stabilization: null,
        exposure_mode: null,
        metering: null,
        creative_style: null,
        dro: null,
        raw_type: null,
      },
      1,
    );
    expect(groups[0]?.sections.map(({ label }) => label)).toEqual([null]);
  });

  test("leaves out the EXIF group when it has no rows", () => {
    expect(metaGroups(empty, 1).map(({ heading }) => heading)).toEqual([RIFFLE_HEADING]);
  });

  test("shows only the Riffle group when the metadata could not be read", () => {
    expect(metaGroups(null, 12)).toEqual([
      {
        heading: RIFFLE_HEADING,
        sections: [{ label: null, rows: [{ label: "Sharpness", value: "12.0" }] }],
      },
    ]);
  });

  test("leaves out the Riffle group without a score", () => {
    expect(metaGroups(sony, null).map(({ heading }) => heading)).toEqual([EXIF_HEADING]);
    expect(metaGroups(null, null)).toEqual([]);
  });

  test("shows the focus candidate state as a Focus row, and none when unknown", () => {
    const riffle = (state: "candidate" | "not_candidate" | "unknown") =>
      metaGroups(null, 12, state)[0]?.sections[0]?.rows;
    expect(riffle("candidate")).toEqual([
      { label: "Sharpness", value: "12.0" },
      { label: "Focus", value: "Candidate" },
    ]);
    expect(riffle("not_candidate")).toEqual([
      { label: "Sharpness", value: "12.0" },
      { label: "Focus", value: "Not a candidate" },
    ]);
    expect(riffle("unknown")).toEqual([{ label: "Sharpness", value: "12.0" }]);
  });
});
