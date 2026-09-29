export type Orientation = "horizontal" | "vertical";

// The tab a tablist key moves to among the visible tabs, or `current` for other keys.
// A horizontal tablist moves on ArrowLeft / ArrowRight, a vertical one on
// ArrowUp / ArrowDown.
export function nextTab(
  tabs: readonly string[],
  current: string,
  key: string,
  orientation: Orientation,
): string {
  const index = tabs.indexOf(current);
  const [previous, next] =
    orientation === "vertical" ? ["ArrowUp", "ArrowDown"] : ["ArrowLeft", "ArrowRight"];
  switch (key) {
    case previous:
      return tabs[(index - 1 + tabs.length) % tabs.length] ?? current;
    case next:
      return tabs[(index + 1) % tabs.length] ?? current;
    case "Home":
      return tabs[0] ?? current;
    case "End":
      return tabs[tabs.length - 1] ?? current;
    default:
      return current;
  }
}
