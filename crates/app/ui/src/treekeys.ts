// Which keymap actions still run while the folder tree has the keyboard:
// only those about the window or the folder, never about a photo. Keyed by
// action name so it holds however the keys are rebound; an action missing
// here (including one added later) is swallowed.

export const TREE_PASSTHROUGH: ReadonlySet<string> = new Set([
  "open",
  "toggleLeft",
  "toggleRight",
  "toggleStrip",
  "toggleSides",
]);

export function treeGate(action: string | undefined): "run" | "swallow" {
  return action !== undefined && TREE_PASSTHROUGH.has(action) ? "run" : "swallow";
}
