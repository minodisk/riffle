// The folder tree in the left pane: home and the mounted volumes, expanded
// lazily through `list_subfolders`. A click in it gives it the keyboard (the
// container is the one focusable element; the rows are announced through
// `aria-activedescendant`), and `Escape` or a click elsewhere hands the keys
// back. Meanwhile `main.ts` routes the keys here first and gates the culling
// keymap. A folder is renamed in place: its row's name becomes a text input
// until Enter, Escape or a click away ends the edit.

import { keyName } from "./keys.js";
import {
  type Decision,
  type InlineRename,
  SLOW_CLICK_DELAY,
  SlowClick,
  commit,
  confirmName,
  editKey,
  inlineRename,
} from "./rename.js";
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
  rebase,
  renameFolder,
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
let contextMenu: (
  path: string,
  name: string,
  x: number,
  y: number,
  root: boolean,
) => void = () => {};
let rename: (path: string, name: string) => void = () => {};
let canRename: () => boolean = () => false;
// The live inline rename, drawn from here on every `render`, so a re-render
// mid-edit (a listing landing) rebuilds the same input.
let editing: InlineRename | null = null;
// Set while `render` swaps the rows, so the input's removal is not taken for
// a click away.
let rendering = false;
// Set while a mouse button is held, so a redraw triggered mid-press (the
// blur-path confirm, or `renamed()` landing while the button is still down)
// waits for the `mouseup`. `click` fires right after `mouseup`, so a render
// during the press would still detach the row before that `click` reaches it.
let pointerDown = false;
// Set when a render was requested while `pointerDown`, so the `mouseup`
// handler runs it once the press ends.
let pendingRender = false;
const slow = new SlowClick();
let slowTimer: ReturnType<typeof setTimeout> | undefined;
// Settles once `folder_roots` has answered (or failed), so a reveal that
// comes first (the reopen of the last folder at launch) waits for the roots.
let rootsSettled: () => void = () => {};
const rootsLoaded = new Promise<void>((resolve) => {
  rootsSettled = resolve;
});

function list(dir: string): Promise<Folder> {
  return window.__TAURI__.core.invoke<Folder>("list_subfolders", { dir });
}

function editor(state: InlineRename): HTMLInputElement {
  const input = document.createElement("input");
  input.type = "text";
  input.className = "name";
  input.value = state.value;
  input.spellcheck = false;
  input.addEventListener("input", () => {
    state.value = input.value;
  });
  // Switching to another app blurs the input too; the edit waits for the
  // window to come back instead of confirming.
  input.addEventListener("blur", () => {
    if (!rendering && document.hasFocus()) {
      finishInPlace(input);
    }
  });
  for (const type of ["mousedown", "click", "contextmenu"]) {
    input.addEventListener(type, (event) => {
      event.stopPropagation();
    });
  }
  return input;
}

// A click on the open folder's name arms a rename instead of reopening it;
// the edit starts `SLOW_CLICK_DELAY` later unless something disarms it.
function armSlowClick(path: string): void {
  slow.click(path, true, performance.now());
  clearTimeout(slowTimer);
  slowTimer = setTimeout(() => {
    if (slow.due(path, performance.now()) && canRename()) {
      startRename(path);
    }
  }, SLOW_CLICK_DELAY);
}

function render(): void {
  const old = container.querySelector<HTMLInputElement>("input.name");
  const range = old === null ? null : ([old.selectionStart ?? 0, old.selectionEnd ?? 0] as const);
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
      if (editing?.path === node.path) {
        return;
      }
      open(node.path);
    });
    row.addEventListener("contextmenu", (event) => {
      if (editing?.path === node.path) {
        return;
      }
      contextMenu(node.path, node.name, event.clientX, event.clientY, depth === 0);
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
    if (editing !== null && node.path === editing.path) {
      row.append(expander, editor(editing));
    } else {
      const name = document.createElement("span");
      name.className = "name";
      name.textContent = node.name;
      name.addEventListener("click", (event) => {
        if (node.path === current && depth > 0 && canRename()) {
          event.stopPropagation();
          armSlowClick(node.path);
        }
      });
      name.addEventListener("dblclick", () => {
        slow.cancel();
      });
      row.append(expander, name);
    }
    if (node.expanded && node.rawCount !== undefined && node.rawCount > 0) {
      const count = document.createElement("span");
      count.className = "count";
      count.textContent = String(node.rawCount);
      row.append(count);
    }
    fragment.append(row);
  }
  rendering = true;
  container.replaceChildren(fragment);
  rendering = false;
  if (active === null) {
    container.removeAttribute("aria-activedescendant");
  } else {
    container.setAttribute("aria-activedescendant", active);
  }
  if (editing !== null) {
    const input = container.querySelector<HTMLInputElement>("input.name");
    if (input === null) {
      // The folder is no longer drawn (a listing dropped it); nothing is
      // left to rename, and a live edit would keep swallowing the keys.
      editing = null;
      return;
    }
    input.focus();
    if (range === null) {
      input.select();
    } else {
      input.setSelectionRange(range[0], range[1]);
    }
  }
}

// Turns the folder's name into a text input, its name fully selected.
export function startRename(path: string): void {
  cancelSlowClick();
  const node = tree.nodes.get(path);
  if (node === undefined) {
    return;
  }
  if (editing !== null) {
    finish("confirm");
  }
  editing = inlineRename("folder", path, node.name);
  render();
}

function finish(decision: Decision): void {
  if (editing === null || commit(editing, decision) === null) {
    return;
  }
  const { path, original, value } = editing;
  editing = null;
  render();
  const name = decision === "confirm" ? confirmName(original, value) : null;
  if (name !== null) {
    rename(path, name);
  }
}

// The blur path's confirm: a click on another row moves focus (and so blurs
// the input) before its own `click` fires, and `render`'s synchronous
// `replaceChildren` would detach every row in between, losing that click. So
// this ends the edit by swapping the input for a plain name span in place,
// leaving the rest of the tree untouched, and defers the full `render()`
// (which redraws the row with its usual listeners) past the current pointer
// sequence: while the button is still down, `requestRender` waits for the
// `mouseup` (which comes right before that `click`) instead of a bare
// `setTimeout`, which would only defer past the current task, not past the
// user's still-held button.
function finishInPlace(input: HTMLInputElement): void {
  if (editing === null || commit(editing, "confirm") === null) {
    return;
  }
  const { path, original, value } = editing;
  editing = null;
  const span = document.createElement("span");
  span.className = "name";
  span.textContent = original;
  input.replaceWith(span);
  const name = confirmName(original, value);
  if (name !== null) {
    rename(path, name);
  }
  requestRender();
}

// Renders immediately, unless a mouse button is currently held (the blur or
// the IPC round trip landed mid-press), in which case the render waits for
// the `mouseup` that ends the pointer sequence, right before its `click`.
function requestRender(): void {
  if (pointerDown) {
    pendingRender = true;
    return;
  }
  render();
}

export function isEditing(): boolean {
  return editing !== null;
}

export function cancelSlowClick(): void {
  slow.cancel();
  clearTimeout(slowTimer);
}

// The folder `oldPath` is now `newName` at `newPath`: re-key it in the tree,
// its expansion and everything under it kept, and move the highlight and the
// cursor along when they were on it or under it.
export function renamed(oldPath: string, newPath: string, newName: string): void {
  tree = renameFolder(tree, oldPath, newPath, newName);
  current = current === null ? null : (rebase(current, oldPath, newPath) ?? current);
  cursor = cursor === null ? null : (rebase(cursor, oldPath, newPath) ?? cursor);
  // The `rename_folder` IPC round trip can resolve while the button that
  // started a click elsewhere is still held; defer to `mouseup` then too, for
  // the same reason `finishInPlace` does.
  requestRender();
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
  if (editing !== null) {
    const decision = event.isComposing ? "native" : editKey(key);
    if (decision !== "native") {
      event.preventDefault();
      finish(decision);
      container.focus();
    }
    return true;
  }
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

const isMac = /Mac/.test(navigator.platform);

// A right-click neither gives the tree the keyboard nor takes it away: the
// culling key gate stays as it was, and the cursor stays put. On macOS,
// Control+click is the other standard way to right-click (common on
// trackpads); WebKit reports it as a primary-button mousedown with
// `ctrlKey`, so guard that too. On Windows/Linux, Ctrl+click is an ordinary
// click that should still focus the tree.
container.addEventListener("mousedown", (event) => {
  if (event.button === 2 || (isMac && event.button === 0 && event.ctrlKey)) {
    event.preventDefault();
  }
});

// Any click anywhere disarms a pending slow click; the arming click's own
// `mousedown` comes before its `click`, so it arms after this.
document.addEventListener("mousedown", cancelSlowClick);

// Tracks the pointer sequence so `requestRender` can defer a redraw past it:
// `mouseup` fires right before the `click` that a rebuild mid-press would
// otherwise lose its target for.
document.addEventListener("mousedown", () => {
  pointerDown = true;
});
document.addEventListener("mouseup", () => {
  pointerDown = false;
  if (pendingRender) {
    pendingRender = false;
    // `click` is dispatched right after `mouseup`, before this timer's
    // callback runs, so the render still lands after the click reaches its
    // target.
    setTimeout(render, 0);
  }
});

container.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

export function init(
  onOpen: (path: string) => void,
  onError: (message: string) => void,
  onContextMenu: (path: string, name: string, x: number, y: number, root: boolean) => void,
  onRename: (path: string, name: string) => void,
  renameAllowed: () => boolean,
): void {
  open = onOpen;
  reportError = onError;
  contextMenu = onContextMenu;
  rename = onRename;
  canRename = renameAllowed;
}
