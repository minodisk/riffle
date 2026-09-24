// Mirrors `Metadata` in `crates/app/src/commands.rs`: already formatted for
// display, so a field is either a string to show or null to leave out.
export interface Metadata {
  name: string;
  camera: string | null;
  lens: string | null;
  aperture: string | null;
  shutter: string | null;
  shutter_type: string | null;
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
export const MAKER_NOTE_LABEL = "Maker note";
export const RIFFLE_HEADING = "Riffle";

export type FaceCatch = "caught" | "missed" | "unknown";

function faceCatchLabel(state: FaceCatch | null | undefined): string | null {
  switch (state) {
    case "caught":
      return "Caught";
    case "missed":
      return "Missed";
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
// analysis. The Riffle group needs no `meta`, so a file whose metadata could
// not be read still shows its score. The face-catch state gets a `Face` row
// only when it is known.
export function metaGroups(
  meta: Metadata | null,
  sharpness: number | null,
  faceCatch?: FaceCatch | null,
): MetaGroup[] {
  const groups: MetaGroup[] = [];
  if (meta !== null) {
    groups.push(
      group(EXIF_HEADING, [
        section(null, [
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
          ["Focus distance", meta.focus_distance],
        ]),
      ]),
    );
  }
  groups.push(
    group(RIFFLE_HEADING, [
      section(null, [
        ["Sharpness", sharpness?.toFixed(1) ?? null],
        ["Face", faceCatchLabel(faceCatch)],
      ]),
    ]),
  );
  return groups.filter(({ sections }) => sections.length > 0);
}
