export const LABEL_COLORS = ["red", "yellow", "green", "blue", "purple"] as const;

export type LabelNames = Record<(typeof LABEL_COLORS)[number], string>;

// A language's Lightroom default color label set, as `label_names` returns it.
export type LabelPreset = { code: string; name: string; names: LabelNames };

// The `set_label_names` payload from the fields' values, trimmed.
export function labelNamesPayload(values: (color: string) => string): LabelNames {
  return Object.fromEntries(
    LABEL_COLORS.map((color) => [color, values(color).trim()]),
  ) as LabelNames;
}
