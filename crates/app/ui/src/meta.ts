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

export interface MetaSection {
  label: string | null;
  rows: MetaRow[];
}

export interface MetaGroup {
  heading: string;
  sections: MetaSection[];
}

export const EXIF_HEADING = "EXIF";
export const STANDARD_LABEL = "Standard";
export const MAKER_NOTE_LABEL = "Maker note";
export const ANALYSIS_HEADING = "Analysis";

export type FocusCandidate = "candidate" | "not_candidate" | "unknown";

// The second scan pass's result for one file, as `Focus` in `main.ts` carries
// it.
export interface FocusCue {
  candidate: FocusCandidate;
  eye_sharpness: number | null;
}

function candidateLabel(state: FocusCandidate | null | undefined): string | null {
  switch (state) {
    case "candidate":
      return "Candidate";
    case "not_candidate":
      return "Not a candidate";
    default:
      return null;
  }
}

function section(label: string | null, rows: [string, string | null][]): MetaSection {
  return {
    label,
    rows: rows.flatMap(([rowLabel, value]) => (value === null ? [] : [{ label: rowLabel, value }])),
  };
}

function group(heading: string, sections: MetaSection[]): MetaGroup {
  return { heading, sections: sections.filter(({ rows }) => rows.length > 0) };
}

// The meta pane's rows grouped by where each value comes from: standard
// EXIF/TIFF tags, the vendor MakerNote (itself an EXIF tag), and Riffle's own
// analysis. The analysis group needs no `meta`, so a file whose metadata could
// not be read still shows its score. The focus candidate state gets a `Focus`
// row only when it is known, and the eye sharpness a row only when there is
// one.
export function metaGroups(
  meta: Metadata | null,
  sharpness: number | null,
  focus?: FocusCue | null,
): MetaGroup[] {
  const groups: MetaGroup[] = [];
  if (meta !== null) {
    groups.push(
      group(EXIF_HEADING, [
        section(STANDARD_LABEL, [
          ["Aperture", meta.aperture],
          ["Shutter", meta.shutter],
          ["ISO", meta.iso],
          ["Focal length", meta.focal_length],
          ["Exposure", meta.exposure_bias],
          ["Camera", meta.camera],
          ["Lens", meta.lens],
          ["Captured", meta.captured_at],
        ]),
        section(MAKER_NOTE_LABEL, [
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
      ]),
    );
  }
  groups.push(
    group(ANALYSIS_HEADING, [
      section(null, [
        ["Sharpness", sharpness?.toFixed(1) ?? null],
        ["Focus", candidateLabel(focus?.candidate)],
        ["Eye sharpness", focus?.eye_sharpness?.toFixed(1) ?? null],
      ]),
    ]),
  );
  return groups.filter(({ sections }) => sections.length > 0);
}
