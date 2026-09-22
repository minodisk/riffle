// Keep an active comparison frame when it is still displayed. If it has
// disappeared, prefer the filmstrip focus when that is a candidate, then the
// first displayed candidate.
export function reconcileActive(
  candidates: readonly string[],
  active: string | null,
  focused: string | undefined,
): string | null {
  if (active !== null && candidates.includes(active)) return active;
  if (focused !== undefined && candidates.includes(focused)) return focused;
  return candidates[0] ?? null;
}
