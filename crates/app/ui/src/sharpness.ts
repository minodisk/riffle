// The strip's sharpness cue: each file's score relative to its neighbors.
// Free of DOM and Tauri so it is tested without mocks.

// How many cells on each side a file is compared with (up to 5 frames).
export const SHARPNESS_RADIUS = 2;

export interface RelativeSharpness {
  ratio: number;
  best: boolean;
}

// Per index, `null` when the file has no score; otherwise its score over the
// maximum non-null score within `radius` cells on either side, and whether it
// holds that maximum (every tied index does).
export function relativeSharpness(
  scores: (number | null)[],
  radius: number = SHARPNESS_RADIUS,
): (RelativeSharpness | null)[] {
  return scores.map((score, at) => {
    if (score === null) {
      return null;
    }
    let max = score;
    const last = Math.min(scores.length - 1, at + radius);
    for (let other = Math.max(0, at - radius); other <= last; other += 1) {
      const value = scores[other];
      if (value !== null && value > max) {
        max = value;
      }
    }
    return { ratio: max > 0 ? score / max : 1, best: score === max };
  });
}
