// A pick is PhotoLab's flag and lives only in a `.dop` sidecar, so the
// backend's `set_rating` drops it under any other format. The frontend
// applies the same rule before a pick reaches `picks` or the strip.
export function effectivePick(pick: boolean, format: string): boolean {
  return pick && format === "dop";
}
