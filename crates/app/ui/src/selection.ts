// The strip's multi-selection: which files a judgement applies to. Keyed by
// path, like `ratings` and `picks`, because `refilter` rebuilds `files`. The
// focused file (`files[index]`) is always a member.

export type Selection = {
  selected: Set<string>;
  anchor: string | undefined;
};

export type Modifiers = { toggle: boolean; range: boolean };

export function single(path: string | undefined): Selection {
  return { selected: new Set(path === undefined ? [] : [path]), anchor: path };
}

function between(files: readonly string[], from: number, to: number): Set<string> {
  return new Set(files.slice(Math.min(from, to), Math.max(from, to) + 1));
}

// A click on `files[at]` while `files[focused]` is focused.
export function click(
  selection: Selection,
  files: readonly string[],
  focused: number,
  at: number,
  { toggle, range }: Modifiers,
): Selection {
  const path = files[at];
  if (path === undefined) return selection;
  if (range) {
    const from = selection.anchor === undefined ? -1 : files.indexOf(selection.anchor);
    if (from === -1) return single(path);
    return { selected: between(files, from, at), anchor: selection.anchor };
  }
  if (toggle) {
    if (at === focused) return selection;
    const selected = new Set(selection.selected);
    if (selected.has(path)) selected.delete(path);
    else selected.add(path);
    return { selected, anchor: path };
  }
  return single(path);
}

// Moves the focus by `delta` and selects the range from the anchor to it.
export function extend(
  selection: Selection,
  files: readonly string[],
  focused: number,
  delta: number,
): { selection: Selection; index: number } {
  const index = Math.min(Math.max(focused + delta, 0), files.length - 1);
  const anchor =
    selection.anchor !== undefined && files.includes(selection.anchor)
      ? selection.anchor
      : files[focused];
  if (anchor === undefined) return { selection, index: focused };
  return { selection: { selected: between(files, files.indexOf(anchor), index), anchor }, index };
}

// Drops the paths no longer in `files`; the anchor falls back to the focused
// file when it is dropped.
export function prune(selection: Selection, files: readonly string[], focused: number): Selection {
  const visible = new Set(files);
  const selected = new Set([...selection.selected].filter((path) => visible.has(path)));
  const path = files[focused];
  if (path !== undefined) selected.add(path);
  const anchor =
    selection.anchor !== undefined && visible.has(selection.anchor) ? selection.anchor : path;
  return { selected, anchor };
}

// The files a judgement applies to, in `files` order.
export function targets(selection: Selection, files: readonly string[], focused: number): string[] {
  const path = files[focused];
  if (path === undefined) return [];
  if (selection.selected.size <= 1) return [path];
  return files.filter((file) => selection.selected.has(file));
}
