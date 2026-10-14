import type { Ranked } from "./meta.js";

// Whether the strip marks a file as its burst's recommended frame: first
// among two or more scored frames of its burst under the personal ranking.
// Tied frames share a place, so a tie for first marks each of them; a lone
// scored frame and a file without a ranking entry get none.
export function recommended(ranked: Ranked | undefined): boolean {
  return ranked !== undefined && ranked.place === 1 && ranked.of >= 2;
}
