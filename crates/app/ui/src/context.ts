import { type Binding, displayKey } from "./keys.js";
import { effectivePick } from "./pick.js";

export type FlagMenuItem = { action: string; label: string; shortcut: string };

const FLAG_ACTIONS: [string, string][] = [
  ["pick", "Pick"],
  ["reject", "Reject"],
  ["unflag", "Unflag"],
];

export function flagMenuItems(bindings: Binding[], sidecarFormat: string): FlagMenuItem[] {
  return FLAG_ACTIONS.filter(
    ([action]) => action !== "pick" || effectivePick(true, sidecarFormat),
  ).map(([action, label]) => {
    const key = bindings.find((binding) => binding.action === action)?.keys[0];
    return { action, label, shortcut: key === undefined ? "" : displayKey(key) };
  });
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
