// Mirrors `Metadata` in `crates/app/src/commands.rs`: already formatted for
// display, so a field is either a string to show or null to leave out.
export interface Metadata {
  name: string;
  camera: string | null;
  lens: string | null;
  aperture: string | null;
  shutter: string | null;
  shutter_type: string | null;
  focus_mode: string | null;
  af_area: string | null;
  af_tracking: string | null;
  drive: string | null;
  stabilization: string | null;
  exposure_mode: string | null;
  metering: string | null;
  creative_style: string | null;
  dro: string | null;
  raw_type: string | null;
  iso: string | null;
  focal_length: string | null;
  exposure_bias: string | null;
  focus_distance: string | null;
  captured_at: string | null;
}

export interface MetaRow {
  label: string;
  value: string;
}

export interface MetaGroup {
  heading: string;
  rows: MetaRow[];
}

export const EXIF_HEADING = "EXIF";
export const MAKER_NOTE_HEADING = "Maker note";
export const ANALYSIS_HEADING = "Analysis";

export type FocusCandidate = "candidate" | "not_candidate" | "unknown";

// The second scan pass's result for one file, as `Focus` in `main.ts` carries
// it.
export interface FocusCue {
  candidate: FocusCandidate;
  eye_sharpness: number | null;
}

function group(heading: string, rows: [string, string | null][]): MetaGroup {
  return {
    heading,
    rows: rows.flatMap(([label, value]) => (value === null ? [] : [{ label, value }])),
  };
}

// The meta pane's rows grouped by where each value comes from: standard
// EXIF/TIFF tags, the vendor MakerNote (itself an EXIF tag), and Riffle's own
// analysis. The analysis group needs no `meta`, so a file whose metadata could
// not be read still shows its score. The AF eye sharpness gets a row only when
// there is one.
export function metaGroups(
  meta: Metadata | null,
  sharpness: number | null,
  focus?: FocusCue | null,
): MetaGroup[] {
  const groups: MetaGroup[] = [];
  if (meta !== null) {
    groups.push(
      group(EXIF_HEADING, [
        ["Aperture", meta.aperture],
        ["Shutter", meta.shutter],
        ["ISO", meta.iso],
        ["Focal length", meta.focal_length],
        ["Exposure", meta.exposure_bias],
        ["Camera", meta.camera],
        ["Lens", meta.lens],
        ["Captured", meta.captured_at],
      ]),
      group(MAKER_NOTE_HEADING, [
        ["Shutter type", meta.shutter_type],
        ["Focus mode", meta.focus_mode],
        ["AF area", meta.af_area],
        ["AF tracking", meta.af_tracking],
        ["Drive", meta.drive],
        ["Stabilization", meta.stabilization],
        ["Exposure mode", meta.exposure_mode],
        ["Metering", meta.metering],
        ["Creative style", meta.creative_style],
        ["DRO", meta.dro],
        ["RAW type", meta.raw_type],
        ["Focus distance", meta.focus_distance],
      ]),
    );
  }
  groups.push(
    group(ANALYSIS_HEADING, [
      ["Sharpness", sharpness?.toFixed(1) ?? null],
      ["AF eye sharpness", focus?.eye_sharpness?.toFixed(1) ?? null],
    ]),
  );
  return groups.filter(({ rows }) => rows.length > 0);
}
