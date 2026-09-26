import { type Binding, displayKey } from "./keys.js";
import type { PickFlag } from "./selection.js";

// `checked` is undefined for a plain command, which has no checked state.
export type MenuItem = {
  action: string;
  label: string;
  shortcut: string;
  checked: boolean | undefined;
};

export type MenuState = { rating: number | null; flag: PickFlag; label: string | null };

type Entry = [action: string, label: string, checked?: (state: MenuState) => boolean];

const LABEL_ENTRIES: Entry[] = ["Red", "Orange", "Yellow", "Green", "Blue", "Pink", "Purple"].map(
  (name) => [name.toLowerCase(), name, (state) => state.label === name],
);

const SECTIONS: Entry[][] = [
  [["selectAll", "Select All"]],
  [
    ["pick", "Pick", (state) => state.flag === "pick"],
    ["reject", "Reject", (state) => state.flag === "reject"],
    ["unflag", "Unflag", (state) => state.flag === "none"],
  ],
  [
    ...[1, 2, 3, 4, 5].map((stars): Entry => [
      `rate${stars}`,
      stars === 1 ? "1 star" : `${stars} stars`,
      (state) => state.rating === stars,
    ]),
    ["clear", "No stars", (state) => !state.rating],
  ],
  [...LABEL_ENTRIES, ["clearlabel", "No label", (state) => state.label === null]],
];

export function contextMenuGroups(bindings: Binding[], state: MenuState): MenuItem[][] {
  return SECTIONS.map((section) =>
    section.map(([action, label, checked]) => {
      const key = bindings.find((binding) => binding.action === action)?.keys[0];
      return {
        action,
        label,
        shortcut: key === undefined ? "" : displayKey(key),
        checked: checked?.(state),
      };
    }),
  );
}

// The folder tree's right-click menu: one item, labeled per platform by the
// `reveal_label` command.
export function folderMenuGroups(revealLabel: string): MenuItem[][] {
  return [[{ action: "revealFolder", label: revealLabel, shortcut: "", checked: undefined }]];
}

export function menuPosition(
  x: number,
  y: number,
  width: number,
  height: number,
  viewportWidth: number,
  viewportHeight: number,
): { left: number; top: number } {
  return {
    left: Math.max(0, x + width > viewportWidth ? x - width : x),
    top: Math.max(0, y + height > viewportHeight ? y - height : y),
  };
}
