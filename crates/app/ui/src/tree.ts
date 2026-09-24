// The folder tree's state, kept pure: which folders are known, listed and
// expanded. `folders.ts` draws it and fills it from the backend.

export interface FolderNode {
  name: string;
  path: string;
}

export interface TreeNode {
  name: string;
  path: string;
  // `undefined` until `list_subfolders` has answered for this folder.
  children: FolderNode[] | undefined;
  expanded: boolean;
  rawCount: number | undefined;
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
  nodes.set(path, { ...node, children, rawCount });
  return { roots: tree.roots, nodes };
}

// The rows to draw, depth first: the roots, and the children of every
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
      if (node.expanded && node.children !== undefined) {
        walk(node.children, depth + 1);
      }
    }
  };
  walk(tree.roots, 0);
  return out;
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
// through the volume home lives on.
export function ancestorsWithin(roots: string[], path: string): string[] | null {
  const target = normalize(path);
  let best: string | null = null;
  for (const root of roots) {
    const prefix = normalize(root);
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
