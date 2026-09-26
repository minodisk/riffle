// The folder tree in the left pane: home and the mounted volumes, expanded
// lazily through `list_subfolders`. A click in it gives it the keyboard (the
// container is the one focusable element; the rows are announced through
// `aria-activedescendant`), and `Escape` or a click elsewhere hands the keys
// back. Meanwhile `main.ts` routes the keys here first and gates the culling
// keymap.

import { keyName } from "./keys.js";
import {
  EMPTY_TREE,
  type FolderNode,
  type StepKey,
  type Tree,
  type TreeKey,
  type Typed,
  addRoots,
  ancestorsWithin,
  appendTyped,
  collapse,
  expand,
  rootOf,
  rows,
  setChildren,
  step,
  treeKey,
  typeAhead,
} from "./tree.js";

interface Folder {
  raw_count: number;
  children: FolderNode[];
}

const container = document.getElementById("folders") as HTMLDivElement;

let tree: Tree = EMPTY_TREE;
// The open folder, highlighted as `.current`.
let current: string | null = null;
// The keyboard cursor, drawn as `.cursor` while the tree has focus.
let cursor: string | null = null;
const NOTHING_TYPED: Typed = { text: "", at: -Infinity };
let typed = NOTHING_TYPED;
let open: (path: string) => void = () => {};
let reportError: (message: string) => void = () => {};
let contextMenu: (path: string, x: number, y: number) => void = () => {};
// Settles once `folder_roots` has answered (or failed), so a reveal that
// comes first (the reopen of the last folder at launch) waits for the roots.
let rootsSettled: () => void = () => {};
const rootsLoaded = new Promise<void>((resolve) => {
  rootsSettled = resolve;
});

function list(dir: string): Promise<Folder> {
  return window.__TAURI__.core.invoke<Folder>("list_subfolders", { dir });
}

function render(): void {
  const fragment = document.createDocumentFragment();
  let active: string | null = null;
  for (const [index, { node, depth }] of rows(tree).entries()) {
    const row = document.createElement("div");
    row.id = `folder-row-${index}`;
    row.className = "folder";
    row.classList.toggle("current", node.path === current);
    if (node.path === cursor) {
      row.classList.add("cursor");
      active = row.id;
    }
    row.setAttribute("role", "treeitem");
    row.setAttribute("aria-level", String(depth + 1));
    row.setAttribute("aria-selected", String(node.path === current));
    row.style.setProperty("--depth", String(depth));
    row.dataset.path = node.path;
    row.title = node.path;
    row.addEventListener("click", () => {
      open(node.path);
    });
    row.addEventListener("contextmenu", (event) => {
      contextMenu(node.path, event.clientX, event.clientY);
    });
    const expander = document.createElement("span");
    expander.className = "expander";
    if (node.children === undefined || node.children.length > 0) {
      row.setAttribute("aria-expanded", String(node.expanded));
      expander.textContent = node.expanded ? "▾" : "▸";
      expander.addEventListener("click", (event) => {
        event.stopPropagation();
        toggle(node.path);
      });
    }
    const name = document.createElement("span");
    name.className = "name";
    name.textContent = node.name;
    row.append(expander, name);
    if (node.expanded && node.rawCount !== undefined && node.rawCount > 0) {
      const count = document.createElement("span");
      count.className = "count";
      count.textContent = String(node.rawCount);
      row.append(count);
    }
    fragment.append(row);
  }
  container.replaceChildren(fragment);
  if (active === null) {
    container.removeAttribute("aria-activedescendant");
  } else {
    container.setAttribute("aria-activedescendant", active);
  }
}

// Expanding always re-lists, so a subfolder created since the last look
// shows up; the cached children are drawn meanwhile. Roots themselves
// (`loadRoots`) are read once at launch and not refreshed here.
function toggle(path: string): void {
  if (tree.nodes.get(path)?.expanded) {
    tree = collapse(tree, path);
    render();
    return;
  }
  tree = expand(tree, path);
  render();
  list(path).then(
    (folder) => {
      tree = setChildren(tree, path, folder.raw_count, folder.children);
      render();
    },
    (err: unknown) => {
      tree = collapse(tree, path);
      render();
      reportError(String(err));
    },
  );
}

export function loadRoots(): void {
  window.__TAURI__.core
    .invoke<FolderNode[]>("folder_roots")
    .then(
      (roots) => {
        tree = addRoots(tree, roots);
        render();
      },
      (err: unknown) => {
        reportError(String(err));
      },
    )
    .finally(rootsSettled);
}

// Expands the chain down to the open folder, highlights it and scrolls it
// into view. `stillCurrent` is the caller's folder-token check: the listings
// along the chain can outlive a second open, which then owns the tree.
export async function reveal(path: string, stillCurrent: () => boolean): Promise<void> {
  current = path;
  render();
  await rootsLoaded;
  if (!stillCurrent()) {
    return;
  }
  let chain = ancestorsWithin(
    tree.roots.map((root) => root.path),
    path,
  );
  if (chain === null) {
    const root = rootOf(path);
    tree = addRoots(tree, [{ name: root, path: root }]);
    chain = ancestorsWithin([root], path);
  }
  for (const dir of chain ?? []) {
    if (!tree.nodes.has(dir)) {
      break;
    }
    tree = expand(tree, dir);
    let folder: Folder;
    try {
      folder = await list(dir);
    } catch (err) {
      tree = collapse(tree, dir);
      if (stillCurrent()) {
        reportError(String(err));
      }
      break;
    }
    if (!stillCurrent()) {
      return;
    }
    tree = setChildren(tree, dir, folder.raw_count, folder.children);
  }
  // `chain`'s last element is spelled the way the tree's nodes are keyed
  // (`ancestorsWithin` normalizes separators, case and trailing slashes),
  // while `path` is the caller's raw spelling; `render` compares `current`
  // against node keys, so it must be the chain's, not the raw one.
  if (chain !== null) {
    current = chain.at(-1) ?? current;
  }
  cursor = current;
  render();
  container.querySelector(".folder.current")?.scrollIntoView({ block: "nearest" });
}

export function hasFocus(): boolean {
  return container.contains(document.activeElement);
}

export function blur(): void {
  container.blur();
}

const STEPS: Record<string, StepKey> = {
  arrowup: "up",
  arrowdown: "down",
  home: "home",
  end: "end",
};

const TREE_KEYS: Record<string, TreeKey> = {
  arrowleft: "left",
  arrowright: "right",
  enter: "enter",
};

function moveCursor(to: string): void {
  cursor = to;
  render();
  container.querySelector(".folder.cursor")?.scrollIntoView({ block: "nearest" });
}

// A printable key, Shift or not, jumps the cursor by name.
function typeKey(event: KeyboardEvent): boolean {
  if (event.key.length !== 1 || event.ctrlKey || event.altKey || event.metaKey) {
    return false;
  }
  event.preventDefault();
  typed = appendTyped(typed, event.key, Date.now());
  const to = typeAhead(rows(tree), cursor, typed.text);
  if (to !== null) {
    moveCursor(to);
  }
  return true;
}

// True when the tree consumed the key.
export function keydown(event: KeyboardEvent): boolean {
  const key = keyName(event);
  if (key === "escape") {
    container.blur();
    event.preventDefault();
    return true;
  }
  if (key === null) {
    return false;
  }
  const move = STEPS[key];
  if (move !== undefined) {
    event.preventDefault();
    const to = step(rows(tree), cursor, move);
    if (to !== null) {
      moveCursor(to);
    }
    return true;
  }
  const treeMove = TREE_KEYS[key];
  if (treeMove === undefined) {
    return typeKey(event);
  }
  event.preventDefault();
  const command = treeKey(rows(tree), cursor, treeMove);
  if (command?.kind === "focus") {
    moveCursor(command.path);
  } else if (command?.kind === "expand" || command?.kind === "collapse") {
    toggle(command.path);
  } else if (command?.kind === "open") {
    open(command.path);
  }
  return true;
}

container.addEventListener("blur", () => {
  typed = NOTHING_TYPED;
});

// A right-click neither gives the tree the keyboard nor takes it away: the
// culling key gate stays as it was, and the cursor stays put.
container.addEventListener("mousedown", (event) => {
  if (event.button === 2) {
    event.preventDefault();
  }
});

container.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

export function init(
  onOpen: (path: string) => void,
  onError: (message: string) => void,
  onContextMenu: (path: string, x: number, y: number) => void,
): void {
  open = onOpen;
  reportError = onError;
  contextMenu = onContextMenu;
}
