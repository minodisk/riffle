export const LABEL_COLORS = ["red", "yellow", "green", "blue", "purple"] as const;

export type LabelNames = Record<(typeof LABEL_COLORS)[number], string>;

// The English names, which the backend also falls back to for a blank field.
export function englishLabelNames(): LabelNames {
  return Object.fromEntries(
    LABEL_COLORS.map((color) => [color, color[0].toUpperCase() + color.slice(1)]),
  ) as LabelNames;
}

// The `set_label_names` payload from the fields' values, trimmed.
export function labelNamesPayload(values: (color: string) => string): LabelNames {
  return Object.fromEntries(
    LABEL_COLORS.map((color) => [color, values(color).trim()]),
  ) as LabelNames;
}
