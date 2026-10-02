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
  [["renameFile", "Rename…"]],
];

// A view-only (JPEG) folder takes no judgment and no rename, so only
// `Select All` is left. `Rename…` acts on the focused file alone.
export function contextMenuGroups(
  bindings: Binding[],
  state: MenuState,
  viewOnly = false,
): MenuItem[][] {
  return (viewOnly ? SECTIONS.slice(0, 1) : SECTIONS).map((section) =>
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

// The folder tree's right-click menu: the reveal item, labeled per platform by
// the `reveal_label` command, then copying the folder's path or name, then re-listing it (Refresh), then
// renaming it, then expanding or collapsing every subfolder under it, then
// moving its rejects to the Trash, with or without its subfolders, then the
// sequencing of the folder's JPEGs. Renaming, expanding / collapsing all and
// the recursive trash item are not offered on a root (home or a volume). With
// `count` folders selected, only the trash items are offered, naming the
// count; `root` then says whether any of them is a root.
export function folderMenuGroups(revealLabel: string, root: boolean, count = 1): MenuItem[][] {
  if (count > 1) {
    const label = `Move Rejected in ${count} Folders to Trash`;
    return [
      [
        { action: "trashRejected", label: `${label}…`, shortcut: "", checked: undefined },
        ...(root
          ? []
          : [
              {
                action: "trashRejectedTree",
                label: `${label}, Including Subfolders…`,
                shortcut: "",
                checked: undefined,
              },
            ]),
      ],
    ];
  }
  return [
    [{ action: "revealFolder", label: revealLabel, shortcut: "", checked: undefined }],
    [
      { action: "copyPath", label: "Copy Path", shortcut: "", checked: undefined },
      { action: "copyFolderName", label: "Copy Folder Name", shortcut: "", checked: undefined },
    ],
    [{ action: "refreshFolder", label: "Refresh", shortcut: "", checked: undefined }],
    ...(root
      ? []
      : [
          [{ action: "renameFolder", label: "Rename…", shortcut: "", checked: undefined }],
          [
            { action: "expandAll", label: "Expand All", shortcut: "", checked: undefined },
            { action: "collapseAll", label: "Collapse All", shortcut: "", checked: undefined },
          ],
        ]),
    [
      {
        action: "trashRejected",
        label: "Move Rejected to Trash…",
        shortcut: "",
        checked: undefined,
      },
      ...(root
        ? []
        : [
            {
              action: "trashRejectedTree",
              label: "Move Rejected to Trash, Including Subfolders…",
              shortcut: "",
              checked: undefined,
            },
          ]),
    ],
    [
      {
        action: "sequenceTimestamps",
        label: "Sequence JPEG Timestamps…",
        shortcut: "",
        checked: undefined,
      },
    ],
  ];
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
