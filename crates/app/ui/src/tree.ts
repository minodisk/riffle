// The folder tree's state, kept pure: which folders are known, listed and
// expanded. `folders.ts` draws it and fills it from the backend.

export interface FolderNode {
  name: string;
  path: string;
  // A symlink or junction; `Expand All` lists it without descending, since
  // it can point back at an ancestor.
  is_link?: boolean;
}

export interface TreeNode {
  name: string;
  path: string;
  // `undefined` until `list_subfolders` has answered for this folder.
  children: FolderNode[] | undefined;
  expanded: boolean;
  rawCount: number | undefined;
  // The error of the last `list_subfolders` that failed for this folder,
  // until a later listing succeeds.
  failed?: string;
}

export interface Tree {
  roots: FolderNode[];
  nodes: ReadonlyMap<string, TreeNode>;
}

export interface Row {
  node: TreeNode;
  depth: number;
}

export const EMPTY_TREE: Tree = { roots: [], nodes: new Map() };

function withNodes(nodes: Map<string, TreeNode>, folders: FolderNode[]): Map<string, TreeNode> {
  for (const { name, path } of folders) {
    if (!nodes.has(path)) {
      nodes.set(path, { name, path, children: undefined, expanded: false, rawCount: undefined });
    }
  }
  return nodes;
}

function update(tree: Tree, path: string, change: Partial<TreeNode>): Tree {
  const node = tree.nodes.get(path);
  if (node === undefined) {
    return tree;
  }
  const nodes = new Map(tree.nodes);
  nodes.set(path, { ...node, ...change });
  return { roots: tree.roots, nodes };
}

// Appends top-level folders; one already at the top level is not repeated.
export function addRoots(tree: Tree, roots: FolderNode[]): Tree {
  const fresh = roots.filter((root) => !tree.roots.some((r) => r.path === root.path));
  return {
    roots: [...tree.roots, ...fresh],
    nodes: withNodes(new Map(tree.nodes), fresh),
  };
}

export function expand(tree: Tree, path: string): Tree {
  return update(tree, path, { expanded: true });
}

export function collapse(tree: Tree, path: string): Tree {
  return update(tree, path, { expanded: false });
}

export function markFailed(tree: Tree, path: string, error: string): Tree {
  return update(tree, path, { failed: error });
}

// A child already known keeps its own state, so re-listing a folder does not
// collapse the folders open under it.
export function setChildren(
  tree: Tree,
  path: string,
  rawCount: number,
  children: FolderNode[],
): Tree {
  const node = tree.nodes.get(path);
  if (node === undefined) {
    return tree;
  }
  const nodes = withNodes(new Map(tree.nodes), children);
  nodes.set(path, { ...node, children, rawCount, failed: undefined });
  return { roots: tree.roots, nodes };
}

// The children of `node` the tree draws, or `undefined` until it is listed.
// A folder that is itself a root is drawn as that root only, so home is not
// repeated under the volume it lives on.
export function drawnChildren(tree: Tree, node: TreeNode): FolderNode[] | undefined {
  if (node.children === undefined) {
    return undefined;
  }
  const roots = new Set(tree.roots.map(({ path }) => normalize(path)));
  return node.children.filter(({ path }) => !roots.has(normalize(path)));
}

// The rows to draw, depth first: the roots, and the drawn children of every
// expanded folder that has been listed.
export function rows(tree: Tree): Row[] {
  const out: Row[] = [];
  const walk = (folders: FolderNode[], depth: number): void => {
    for (const { path } of folders) {
      const node = tree.nodes.get(path);
      if (node === undefined) {
        continue;
      }
      out.push({ node, depth });
      const children = drawnChildren(tree, node);
      if (node.expanded && children !== undefined) {
        walk(children, depth + 1);
      }
    }
  };
  walk(tree.roots, 0);
  return out;
}

// The folders the tree watches: the expanded ones among the drawn rows,
// sorted. A node left expanded under a collapsed parent is not drawn, so it
// is not watched.
export function watchedFolders(tree: Tree): string[] {
  return rows(tree)
    .filter(({ node }) => node.expanded)
    .map(({ node }) => node.path)
    .sort();
}

// The listed folders under `path`, depth first, through the children the tree
// draws: a folder not yet listed ends its branch, and a root drawn elsewhere
// is left to its own row. Empty for an unknown path.
export function descendants(tree: Tree, path: string): string[] {
  const out: string[] = [];
  const walk = (dir: string): void => {
    const node = tree.nodes.get(dir);
    const children = node === undefined ? undefined : drawnChildren(tree, node);
    for (const child of children ?? []) {
      if (tree.nodes.has(child.path)) {
        out.push(child.path);
        walk(child.path);
      }
    }
  };
  walk(path);
  return out;
}

// Collapses every folder under `path`, leaving `path` itself as it is.
export function collapseAll(tree: Tree, path: string): Tree {
  const under = descendants(tree, path);
  if (under.length === 0) {
    return tree;
  }
  const nodes = new Map(tree.nodes);
  for (const dir of under) {
    const node = nodes.get(dir);
    if (node !== undefined) {
      nodes.set(dir, { ...node, expanded: false });
    }
  }
  return { roots: tree.roots, nodes };
}

// The folders a right-click acts on together, apart from the open folder
// (`current` in `folders.ts`) and the keyboard cursor. The anchor is where a
// Shift+click range starts.
export interface TreeSelection {
  selected: ReadonlySet<string>;
  anchor: string | null;
}

export const NO_SELECTION: TreeSelection = { selected: new Set(), anchor: null };

export function selectOnly(path: string): TreeSelection {
  return { selected: new Set([path]), anchor: path };
}

// A click on the row of `path`: `range` selects the drawn rows from the
// anchor to it, `toggle` adds or removes it, and a plain click selects it
// alone. A range whose anchor is not drawn selects `path` alone.
export function clickSelect(
  selection: TreeSelection,
  rows: Row[],
  path: string,
  { toggle, range }: { toggle: boolean; range: boolean },
): TreeSelection {
  if (range) {
    const paths = rows.map(({ node }) => node.path);
    const from = selection.anchor === null ? -1 : paths.indexOf(selection.anchor);
    const to = paths.indexOf(path);
    if (from === -1 || to === -1) {
      return selectOnly(path);
    }
    return {
      selected: new Set(paths.slice(Math.min(from, to), Math.max(from, to) + 1)),
      anchor: selection.anchor,
    };
  }
  if (toggle) {
    const selected = new Set(selection.selected);
    if (selected.has(path)) {
      selected.delete(path);
    } else {
      selected.add(path);
    }
    return { selected, anchor: path };
  }
  return selectOnly(path);
}

// Drops the folders no longer drawn (a parent collapsed, or a re-listing
// removed them); the anchor goes too when it is dropped.
export function pruneSelection(selection: TreeSelection, rows: Row[]): TreeSelection {
  const drawn = new Set(rows.map(({ node }) => node.path));
  const selected = [...selection.selected].filter((path) => drawn.has(path));
  const anchor = selection.anchor !== null && drawn.has(selection.anchor) ? selection.anchor : null;
  if (selected.length === selection.selected.size && anchor === selection.anchor) {
    return selection;
  }
  return { selected: new Set(selected), anchor };
}

export type StepKey = "up" | "down" | "home" | "end";

// The path of the row the keyboard cursor moves to, or `null` when there is
// no row. `up` / `down` stop at the ends; a cursor not among `rows` (none
// yet, or its parent collapsed) lands on the first row.
export function step(rows: Row[], cursor: string | null, key: StepKey): string | null {
  if (rows.length === 0) {
    return null;
  }
  const last = rows.length - 1;
  const at = rows.findIndex((row) => row.node.path === cursor);
  let to: number;
  if (key === "home") {
    to = 0;
  } else if (key === "end") {
    to = last;
  } else if (at === -1) {
    to = 0;
  } else {
    to = key === "up" ? Math.max(at - 1, 0) : Math.min(at + 1, last);
  }
  return rows[to].node.path;
}

export type TreeKey = "left" | "right" | "enter";

export type TreeCommand =
  | { kind: "focus"; path: string }
  | { kind: "expand"; path: string }
  | { kind: "collapse"; path: string }
  | { kind: "open"; path: string };

function canExpand(tree: Tree, node: TreeNode): boolean {
  const children = drawnChildren(tree, node);
  return children === undefined || children.length > 0;
}

// What `Right` / `Left` / `Enter` do on the cursor row, following the
// WAI-ARIA tree pattern, or `null` when the key does nothing there (or the
// cursor is not among the drawn rows).
export function treeKey(tree: Tree, cursor: string | null, key: TreeKey): TreeCommand | null {
  const drawn = rows(tree);
  const at = drawn.findIndex((row) => row.node.path === cursor);
  if (at === -1) {
    return null;
  }
  const { node, depth } = drawn[at];
  if (key === "enter") {
    return { kind: "open", path: node.path };
  }
  if (key === "right") {
    if (!canExpand(tree, node)) {
      return null;
    }
    if (!node.expanded) {
      return { kind: "expand", path: node.path };
    }
    const child = drawn[at + 1];
    return child !== undefined && child.depth > depth
      ? { kind: "focus", path: child.node.path }
      : null;
  }
  if (node.expanded && canExpand(tree, node)) {
    return { kind: "collapse", path: node.path };
  }
  for (let i = at - 1; i >= 0; i--) {
    if (drawn[i].depth < depth) {
      return { kind: "focus", path: drawn[i].node.path };
    }
  }
  return null;
}

export interface Typed {
  text: string;
  at: number;
}

export const TYPE_AHEAD_TIMEOUT = 500;

// The type-ahead buffer after typing `char` at `now`: appended while the
// keys come within `timeout` of each other, else started over.
export function appendTyped(
  buffer: Typed,
  char: string,
  now: number,
  timeout = TYPE_AHEAD_TIMEOUT,
): Typed {
  return { text: now - buffer.at <= timeout ? buffer.text + char : char, at: now };
}

// The path of the row whose name starts with `prefix` (case-insensitive),
// searching down from the cursor and wrapping to the top, or `null` when
// none does. A single character (or one repeated, as a tapped letter gives)
// starts past the cursor, so tapping cycles through the matches; a longer
// prefix starts at the cursor, so it keeps the row its first letter found.
export function typeAhead(rows: Row[], cursor: string | null, prefix: string): string | null {
  if (rows.length === 0 || prefix === "") {
    return null;
  }
  let needle = prefix.toLowerCase();
  const repeated = needle === needle.charAt(0).repeat(needle.length);
  if (repeated) {
    needle = needle.charAt(0);
  }
  const at = rows.findIndex((row) => row.node.path === cursor);
  const start = at === -1 ? 0 : repeated ? at + 1 : at;
  for (let i = 0; i < rows.length; i++) {
    const { node } = rows[(start + i) % rows.length];
    if (node.name.toLowerCase().startsWith(needle)) {
      return node.path;
    }
  }
  return null;
}

function isWindowsPath(path: string): boolean {
  return /^[A-Za-z]:/.test(path) || path.startsWith("\\\\");
}

function segments(path: string): string[] {
  return path.split(/[\\/]+/).filter((s) => s !== "");
}

// Forward slashes, no trailing separator, and a lower-case drive letter, so
// `C:\Users` and `c:/Users/` compare equal. `/` becomes the empty string.
function normalize(path: string): string {
  const slashed = path.replace(/\\/g, "/").replace(/\/+$/, "");
  return isWindowsPath(path) ? slashed.replace(/^[A-Za-z]:/, (d) => d.toLowerCase()) : slashed;
}

function fold(path: string, ignoreCase: boolean): string {
  return ignoreCase ? normalize(path).toLowerCase() : normalize(path);
}

function join(parent: string, name: string): string {
  if (parent.endsWith("/") || parent.endsWith("\\")) {
    return parent + name;
  }
  return parent + (isWindowsPath(parent) ? "\\" : "/") + name;
}

// The paths from the root holding `path` down to `path` itself, spelled the
// way `list_subfolders` spells its children (the root's own spelling, then
// one separator per level), or `null` when no root holds it. The deepest
// root wins, so a folder under home is reached through home rather than
// through the volume home lives on. With `ignoreCase` (macOS, Windows) a root
// holds the path whatever the case of either; the chain still takes the
// caller's segments, which `respell` corrects from the listings.
export function ancestorsWithin(
  roots: string[],
  path: string,
  ignoreCase = false,
): string[] | null {
  const target = fold(path, ignoreCase);
  let best: string | null = null;
  for (const root of roots) {
    const prefix = fold(root, ignoreCase);
    const holds = target === prefix || target.startsWith(`${prefix}/`);
    if (holds && (best === null || prefix.length > normalize(best).length)) {
      best = root;
    }
  }
  if (best === null) {
    return null;
  }
  const chain = [best];
  for (const name of segments(path).slice(segments(best).length)) {
    chain.push(join(chain[chain.length - 1], name));
  }
  return chain;
}

// `chain` with `chain[at]` and everything after it rebased onto the listed
// child that is `chain[at]` (ignoring case when asked), so the rest of the
// chain is spelled the way the tree keys its nodes; unchanged when no child
// matches.
export function respell(
  chain: string[],
  at: number,
  children: FolderNode[],
  ignoreCase: boolean,
): string[] {
  if (at >= chain.length) {
    return chain;
  }
  const dir = chain[at];
  const child = children.find((c) => fold(c.path, ignoreCase) === fold(dir, ignoreCase));
  if (child === undefined) {
    return chain;
  }
  return [
    ...chain.slice(0, at),
    ...chain.slice(at).map((p) => rebase(p, dir, child.path, ignoreCase) ?? p),
  ];
}

// The root to add for a folder no root holds: its drive (`D:\`), its UNC
// share (`\\server\share`), or `/`.
export function rootOf(path: string): string {
  const drive = /^[A-Za-z]:/.exec(path);
  if (drive !== null) {
    return `${drive[0]}\\`;
  }
  if (path.startsWith("\\\\")) {
    return `\\\\${segments(path).slice(0, 2).join("\\")}`;
  }
  return "/";
}

// Whether `path` is `dir` itself or a folder under it, compared the way
// `ancestorsWithin` compares: with `ignoreCase` (macOS, Windows) the
// comparison ignores case.
export function relation(path: string, dir: string, ignoreCase = false): "same" | "under" | null {
  const target = fold(path, ignoreCase);
  const prefix = fold(dir, ignoreCase);
  if (target === prefix) {
    return "same";
  }
  return target.startsWith(`${prefix}/`) ? "under" : null;
}

// The path under `newDir` that `path` had under `oldDir`, or `null` when
// `path` is neither `oldDir` nor under it. With `ignoreCase` (macOS,
// Windows) the comparison ignores case; the rest of `path` is joined onto
// `newDir` in `path`'s own spelling.
export function rebase(
  path: string,
  oldDir: string,
  newDir: string,
  ignoreCase = false,
): string | null {
  const target = fold(path, ignoreCase);
  const prefix = fold(oldDir, ignoreCase);
  if (target === prefix) {
    return newDir;
  }
  if (!target.startsWith(`${prefix}/`)) {
    return null;
  }
  let rebased = newDir;
  for (const name of segments(path).slice(segments(oldDir).length)) {
    rebased = join(rebased, name);
  }
  return rebased;
}

// As `list_subfolders` sorts: case-insensitively by name.
function byName(a: FolderNode, b: FolderNode): number {
  const x = a.name.toLowerCase();
  const y = b.name.toLowerCase();
  return x < y ? -1 : x > y ? 1 : 0;
}

// The tree after the folder `oldPath` was renamed to `newName` at `newPath`:
// it and every node under it re-keyed with their state (expansion, listing,
// count) kept, and its entry in its parent's children replaced and
// re-sorted. A stale node already under `newPath` is dropped.
export function renameFolder(
  tree: Tree,
  oldPath: string,
  newPath: string,
  newName: string,
  ignoreCase = false,
): Tree {
  const renamed = fold(oldPath, ignoreCase);
  const stale = (path: string): boolean =>
    rebase(path, oldPath, newPath, ignoreCase) === null &&
    rebase(path, newPath, newPath, ignoreCase) !== null;
  const moved = (folder: FolderNode): FolderNode => {
    const path = rebase(folder.path, oldPath, newPath, ignoreCase);
    if (path === null) {
      return folder;
    }
    return { name: fold(folder.path, ignoreCase) === renamed ? newName : folder.name, path };
  };
  const list = (folders: FolderNode[]): FolderNode[] => {
    const out = folders.filter((folder) => !stale(folder.path)).map(moved);
    return folders.some((folder) => fold(folder.path, ignoreCase) === renamed)
      ? out.sort(byName)
      : out;
  };
  const nodes = new Map<string, TreeNode>();
  for (const node of tree.nodes.values()) {
    if (stale(node.path)) {
      continue;
    }
    const { name, path } = moved(node);
    const children = node.children === undefined ? undefined : list(node.children);
    nodes.set(path, { ...node, name, path, children });
  }
  return { roots: tree.roots.map(moved), nodes };
}
