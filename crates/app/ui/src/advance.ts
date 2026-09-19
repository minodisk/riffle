const ADVANCING = new Set(["rate1", "rate2", "rate3", "rate4", "rate5", "reject", "pick"]);

// Whether a judgement action moves on to the next file when Auto-advance is
// on. Colour labels are toggles and the clearing actions are corrections, so
// they stay on the file.
export function advancesAfter(action: string): boolean {
  return ADVANCING.has(action);
}
