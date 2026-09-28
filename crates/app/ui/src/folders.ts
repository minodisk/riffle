// The folder tree in the left pane: home and the mounted volumes, expanded
// lazily through `list_subfolders`. A click in it gives it the keyboard (the
// container is the one focusable element; the rows are announced through
// `aria-activedescendant`), and `Escape` or a click elsewhere hands the keys
// back. Meanwhile `main.ts` routes the keys here first and gates the culling
// keymap. A folder is renamed in place: its row's name becomes a text input
// until Enter, Escape or a click away ends the edit. Cmd/Ctrl+click and
// Shift+click select several folders, which a right-click then acts on.

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
  NO_SELECTION,
  type Tree,
  type TreeKey,
  type TreeSelection,
  type Typed,
  addRoots,
  ancestorsWithin,
  appendTyped,
  clickSelect,
  collapse,
  expand,
  markFailed,
  pruneSelection,
  rebase,
  renameFolder,
  rootOf,
  rows,
  selectOnly,
  setChildren,
  step,
  treeKey,
  typeAhead,
  watchedFolders,
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
// The folders a right-click acts on, drawn as `.selected`.
let selection: TreeSelection = NO_SELECTION;
// Bumped every time a click or `menuTargets` assigns `selection` on the
// user's behalf, so `reveal` can tell that apart from the selection merely
// being pruned to what is drawn.
let selectionEdits = 0;
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
  targets: string[],
) => void = () => {};
let rename: (path: string, name: string) => void = () => {};
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
// The path `finishInPlace` just ended the edit of, kept until the next
// `render()` rebuilds that row. The row stays attached (its full rebuild is
// deferred past the pointer sequence), so a plain click or right-click that
// lands on it before that rebuild must still be treated as landing on the
// row that was mid-edit, not reopen the folder or its menu under the
// pre-rename path.
let ended: string | null = null;
// The watched set last sent to `set_tree_watches`, joined, so a render that
// leaves it unchanged sends nothing.
let watched = "";
let syncing: Promise<void> = Promise.resolve();
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
    if (slow.due(path, performance.now())) {
      startRename(path);
    }
  }, SLOW_CLICK_DELAY);
}

const isMac = /Mac/.test(navigator.platform);

// The modifiers of a click in the tree: Cmd (macOS) or Ctrl (elsewhere)
// toggles a folder, Shift selects a range; either leaves the open folder as
// it is.
function modifiers(event: MouseEvent): { toggle: boolean; range: boolean } {
  return { toggle: isMac ? event.metaKey : event.ctrlKey, range: event.shiftKey };
}

// The folders a right-click on `path` acts on, in drawn order: the whole
// selection when `path` is in it, else `path` alone (which becomes the
// selection).
function menuTargets(path: string): { targets: string[]; root: boolean } {
  if (!selection.selected.has(path)) {
    selection = selectOnly(path);
    selectionEdits++;
    render();
  }
  const chosen = rows(tree).filter(({ node }) => selection.selected.has(node.path));
  return {
    targets: chosen.map(({ node }) => node.path),
    root: chosen.some(({ depth }) => depth === 0),
  };
}

function render(): void {
  ended = null;
  const old = container.querySelector<HTMLInputElement>("input.name");
  const range = old === null ? null : ([old.selectionStart ?? 0, old.selectionEnd ?? 0] as const);
  const fragment = document.createDocumentFragment();
  let active: string | null = null;
  const drawn = rows(tree);
  selection = pruneSelection(selection, drawn);
  for (const [index, { node, depth }] of drawn.entries()) {
    const row = document.createElement("div");
    row.id = `folder-row-${index}`;
    row.className = "folder";
    row.classList.toggle("current", node.path === current);
    row.classList.toggle("selected", selection.selected.has(node.path));
    if (node.path === cursor) {
      row.classList.add("cursor");
      active = row.id;
    }
    row.setAttribute("role", "treeitem");
    row.setAttribute("aria-level", String(depth + 1));
    row.setAttribute("aria-selected", String(selection.selected.has(node.path)));
    row.style.setProperty("--depth", String(depth));
    row.dataset.path = node.path;
    row.title = node.failed ?? node.path;
    row.classList.toggle("failed", node.failed !== undefined);
    row.addEventListener("click", (event) => {
      if (editing?.path === node.path || ended === node.path) {
        return;
      }
      const mods = modifiers(event);
      if (mods.toggle || mods.range) {
        selection = clickSelect(selection, rows(tree), node.path, mods);
        selectionEdits++;
        render();
        return;
      }
      open(node.path);
    });
    row.addEventListener("contextmenu", (event) => {
      if (editing?.path === node.path || ended === node.path) {
        return;
      }
      const { targets, root } = menuTargets(node.path);
      contextMenu(node.path, node.name, event.clientX, event.clientY, root, targets);
    });
    const expander = document.createElement("span");
    expander.className = "expander";
    if (node.children === undefined || node.children.length > 0) {
      row.setAttribute("aria-expanded", String(node.expanded));
      expander.textContent = node.expanded ? "▾" : "▸";
      expander.addEventListener("click", (event) => {
        event.stopPropagation();
        if (ended === node.path) {
          return;
        }
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
        const mods = modifiers(event);
        if (node.path === current && depth > 0 && !mods.toggle && !mods.range) {
          event.stopPropagation();
          if (selection.selected.size !== 1 || !selection.selected.has(node.path)) {
            selection = selectOnly(node.path);
            selectionEdits++;
            render();
          }
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
  syncWatches();
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

// Every expanded, drawn folder follows the disk through a backend watcher,
// which answers with `tree-changed`.
function syncWatches(): void {
  const dirs = watchedFolders(tree);
  const joined = dirs.join("\n");
  if (joined === watched) {
    return;
  }
  watched = joined;
  // Chained onto the previous call so two `set_tree_watches` invokes never
  // run on the backend out of order and leave a stale watch set.
  syncing = syncing.then(async () => {
    try {
      await window.__TAURI__.core.invoke("set_tree_watches", { dirs });
    } catch (err) {
      // The call failed, so the backend never applied `dirs`; forget
      // `watched` so the next render sends the set again.
      watched = "";
      reportError(String(err));
    }
  });
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
  ended = path;
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
  selection = {
    selected: new Set(
      [...selection.selected].map((path) => rebase(path, oldPath, newPath) ?? path),
    ),
    anchor:
      selection.anchor === null
        ? null
        : (rebase(selection.anchor, oldPath, newPath) ?? selection.anchor),
  };
  // The `rename_folder` IPC round trip can resolve while the button that
  // started a click elsewhere is still held; defer to `mouseup` then too, for
  // the same reason `finishInPlace` does.
  requestRender();
}

// Expanding re-lists, so a subfolder created while the folder was collapsed
// shows up; the cached children are drawn meanwhile. While expanded, the
// folder follows the disk through `tree-changed`. Roots themselves
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
      tree = markFailed(collapse(tree, path), path, String(err));
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
// Opening a folder, by whatever route, makes it the selection.
export async function reveal(path: string, stillCurrent: () => boolean): Promise<void> {
  current = path;
  selection = selectOnly(path);
  const editsAtStart = selectionEdits;
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
  if (selectionEdits === editsAtStart) {
    selection = selectOnly(current);
  }
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

// A right-click neither gives the tree the keyboard nor takes it away: the
// culling key gate stays as it was, and the cursor stays put. On macOS,
// Control+click is the other standard way to right-click (common on
// trackpads); WebKit reports it as a primary-button mousedown with
// `ctrlKey`, so guard that too. On Windows/Linux, Ctrl+click is an ordinary
// click that should still focus the tree.
container.addEventListener("mousedown", (event) => {
  if (event.button === 2 || (isMac && event.button === 0 && event.ctrlKey)) {
    event.preventDefault();
    return;
  }
  // Shift+click would otherwise select the rows' text; the tree still takes
  // the keyboard.
  if (event.button === 0 && event.shiftKey) {
    event.preventDefault();
    container.focus();
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

// A folder's watcher's trigger, debounced in Rust. A folder collapsed or
// hidden since (the event and the new watched set can cross) is dropped, and
// a failed re-list leaves the folder expanded with what it showed.
void window.__TAURI__.event.listen<{ dir: string }>("tree-changed", ({ payload }) => {
  const { dir } = payload;
  if (!watchedFolders(tree).includes(dir)) {
    return;
  }
  list(dir).then(
    (folder) => {
      tree = setChildren(tree, dir, folder.raw_count, folder.children);
      requestRender();
    },
    (err: unknown) => {
      // This is a background refresh, not something the user asked for.
      // A permanent delete of the folder itself removes its entries first
      // (`rm -rf`, Shift+Delete), so this re-list often fails with the
      // folder still drawn; the parent's own `tree-changed` is what
      // removes the row. Reporting this as an error would be spurious.
      console.warn(err);
    },
  );
});

export function init(
  onOpen: (path: string) => void,
  onError: (message: string) => void,
  onContextMenu: (
    path: string,
    name: string,
    x: number,
    y: number,
    root: boolean,
    targets: string[],
  ) => void,
  onRename: (path: string, name: string) => void,
): void {
  open = onOpen;
  reportError = onError;
  contextMenu = onContextMenu;
  rename = onRename;
}
