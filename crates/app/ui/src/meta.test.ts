import { describe, expect, test } from "vitest";
import {
  EXIF_HEADING,
  type FocusCue,
  MAKER_NOTE_HEADING,
  type Metadata,
  ANALYSIS_HEADING,
  metaGroups,
  poseValue,
} from "./meta.js";
import type { Eyes } from "./eyes.js";

const mesh: Eyes["mesh"] = { width: 1600, height: 1080, points: [[800, 540]] };

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
  test("splits Sony-like input into EXIF, Maker note and Analysis", () => {
    expect(metaGroups(sony, 303.54)).toEqual([
      {
        heading: EXIF_HEADING,
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
        heading: MAKER_NOTE_HEADING,
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
      {
        heading: ANALYSIS_HEADING,
        rows: [{ label: "Sharpness", value: "303.5" }],
      },
    ]);
  });

  test("puts the Leica focus distance in the Maker note group and drops null rows", () => {
    expect(metaGroups(leica, null)).toEqual([
      {
        heading: EXIF_HEADING,
        rows: [
          { label: "Aperture", value: "f/2.0 (est.)" },
          { label: "Camera", value: "Leica M11-P" },
        ],
      },
      { heading: MAKER_NOTE_HEADING, rows: [{ label: "Focus distance", value: "1.5 m" }] },
    ]);
  });

  test("leaves out the Maker note group when no maker note field is set", () => {
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
    expect(groups.map(({ heading }) => heading)).toEqual([EXIF_HEADING, ANALYSIS_HEADING]);
  });

  test("leaves out the EXIF and Maker note groups when they have no rows", () => {
    expect(metaGroups(empty, 1).map(({ heading }) => heading)).toEqual([ANALYSIS_HEADING]);
  });

  test("shows only the analysis group when the metadata could not be read", () => {
    expect(metaGroups(null, 12)).toEqual([
      {
        heading: ANALYSIS_HEADING,
        rows: [{ label: "Sharpness", value: "12.0" }],
      },
    ]);
  });

  test("leaves out the analysis group without a score", () => {
    expect(metaGroups(sony, null).map(({ heading }) => heading)).toEqual([
      EXIF_HEADING,
      MAKER_NOTE_HEADING,
    ]);
    expect(metaGroups(null, null)).toEqual([]);
  });

  test("shows the AF eye in-focus probability as a percentage after Sharpness", () => {
    const riffle = (focus: FocusCue) => metaGroups(null, 12, focus)[0]?.rows;
    expect(riffle({ candidate: "candidate", eye_focus: 0.8701 })).toEqual([
      { label: "Sharpness", value: "12.0" },
      { label: "AF eye in focus", value: "87%" },
    ]);
    expect(riffle({ candidate: "not_candidate", eye_focus: 0 })).toEqual([
      { label: "Sharpness", value: "12.0" },
      { label: "AF eye in focus", value: "0%" },
    ]);
  });

  test("leaves out the row when there is no value", () => {
    expect(metaGroups(null, 12, { candidate: "unknown", eye_focus: null })[0]?.rows).toEqual([
      { label: "Sharpness", value: "12.0" },
    ]);
  });

  test("shows the open probability after the AF eye row", () => {
    const focus: FocusCue = { candidate: "candidate", eye_focus: 0.87 };
    expect(
      metaGroups(null, 12, focus, { state: "closed", probability: 0.814, pose: null, mesh })[0]
        ?.rows,
    ).toEqual([
      { label: "Sharpness", value: "12.0" },
      { label: "AF eye in focus", value: "87%" },
      { label: "Eyes open", value: "19%" },
    ]);
    expect(
      metaGroups(null, null, null, { state: "open", probability: 0.07, pose: null, mesh }),
    ).toEqual([{ heading: ANALYSIS_HEADING, rows: [{ label: "Eyes open", value: "93%" }] }]);
  });

  test("rounds the open probability itself, whichever state was judged", () => {
    const eyesOpen = (state: Eyes["state"], probability: number) =>
      metaGroups(null, null, null, { state, probability, pose: null, mesh })[0]?.rows;
    expect(eyesOpen("closed", 0.5)).toEqual([{ label: "Eyes open", value: "50%" }]);
    expect(eyesOpen("closed", 0.505)).toEqual([{ label: "Eyes open", value: "50%" }]);
    expect(eyesOpen("open", 0.495)).toEqual([{ label: "Eyes open", value: "51%" }]);
  });

  test("leaves out the open probability when it is unknown or not yet judged", () => {
    const rows = [{ label: "Sharpness", value: "12.0" }];
    expect(metaGroups(null, 12, null, null)[0]?.rows).toEqual(rows);
    expect(metaGroups(null, 12, null, undefined)[0]?.rows).toEqual(rows);
  });

  test("shows the head pose after the open probability", () => {
    const pose = { yaw: 12.4, pitch: -5.2, roll: 3.5 };
    expect(metaGroups(null, null, null, { state: "open", probability: 0.07, pose, mesh })).toEqual([
      {
        heading: ANALYSIS_HEADING,
        rows: [
          { label: "Eyes open", value: "93%" },
          { label: "Head pose", value: "yaw 12°, pitch -5°, roll 4°" },
        ],
      },
    ]);
    expect(poseValue({ yaw: -110.2, pitch: 0, roll: -1.6 })).toBe("yaw -110°, pitch 0°, roll -2°");
  });

  test("rounds a small negative angle to 0°", () => {
    expect(poseValue({ yaw: -0.4, pitch: -0.2, roll: -0.49 })).toBe("yaw 0°, pitch 0°, roll 0°");
  });

  test("leaves out the head pose when it is unknown or the eyes are", () => {
    const rows = [{ label: "Sharpness", value: "12.0" }];
    expect(
      metaGroups(null, 12, null, { state: "open", probability: 0.07, pose: null, mesh })[0]?.rows,
    ).toEqual([...rows, { label: "Eyes open", value: "93%" }]);
    expect(metaGroups(null, 12, null, null)[0]?.rows).toEqual(rows);
    expect(metaGroups(null, 12, null, undefined)[0]?.rows).toEqual(rows);
  });

  test("shows the analysis group for the in-focus probability alone", () => {
    expect(metaGroups(null, null, { candidate: "not_candidate", eye_focus: 0.07 })).toEqual([
      {
        heading: ANALYSIS_HEADING,
        rows: [{ label: "AF eye in focus", value: "7%" }],
      },
    ]);
  });
});
