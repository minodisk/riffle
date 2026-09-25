// The folder tree in the left pane: home and the mounted volumes, expanded
// lazily through `list_subfolders`. It takes no keyboard focus; the keys stay
// with culling.

import {
  EMPTY_TREE,
  type FolderNode,
  type Tree,
  addRoots,
  ancestorsWithin,
  collapse,
  expand,
  rootOf,
  rows,
  setChildren,
} from "./tree.js";

interface Folder {
  raw_count: number;
  children: FolderNode[];
}

const container = document.getElementById("folders") as HTMLDivElement;

let tree: Tree = EMPTY_TREE;
// The open folder, highlighted as `.current`.
let current: string | null = null;
let open: (path: string) => void = () => {};
let reportError: (message: string) => void = () => {};
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
  for (const { node, depth } of rows(tree)) {
    const row = document.createElement("div");
    row.className = "folder";
    row.classList.toggle("current", node.path === current);
    row.setAttribute("role", "treeitem");
    row.setAttribute("aria-level", String(depth + 1));
    row.setAttribute("aria-selected", String(node.path === current));
    row.style.setProperty("--depth", String(depth));
    row.dataset.path = node.path;
    row.title = node.path;
    row.addEventListener("click", () => {
      open(node.path);
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
  render();
  container.querySelector(".folder.current")?.scrollIntoView({ block: "nearest" });
}

export function init(onOpen: (path: string) => void, onError: (message: string) => void): void {
  open = onOpen;
  reportError = onError;
}
