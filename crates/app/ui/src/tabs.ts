// The tab a tablist key moves to among the visible tabs, or `current` for other keys.
export function nextTab(tabs: readonly string[], current: string, key: string): string {
  const index = tabs.indexOf(current);
  switch (key) {
    case "ArrowLeft":
      return tabs[(index - 1 + tabs.length) % tabs.length] ?? current;
    case "ArrowRight":
      return tabs[(index + 1) % tabs.length] ?? current;
    case "Home":
      return tabs[0] ?? current;
    case "End":
      return tabs[tabs.length - 1] ?? current;
    default:
      return current;
  }
}
