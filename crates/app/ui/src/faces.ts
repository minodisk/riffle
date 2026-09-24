// The faces the focus mark draws, detected on demand per file and kept for
// the session. Free of DOM and Tauri so it is tested without mocks; `main.ts`
// only makes the `faces_of` call this decides on.

import type { Faces } from "./focus.js";

// What a file whose detection failed is cached as, so it is not retried on
// every draw.
export const NO_FACES: Faces = { width: 0, height: 0, faces: [] };

export class FaceCache {
  private readonly found = new Map<string, Faces>();
  private readonly inFlight = new Set<string>();
  private generation = 0;

  get(path: string): Faces | undefined {
    return this.found.get(path);
  }

  // The token to settle the request with, or `null` when `path` is cached or
  // already being detected, so a path is never detected twice.
  request(path: string): number | null {
    if (this.found.has(path) || this.inFlight.has(path)) {
      return null;
    }
    this.inFlight.add(path);
    return this.generation;
  }

  // Whether the response was kept: one issued before the last `clear` is
  // dropped, since the file may have changed since. It is kept by path, so a
  // response for a file the user paged away from still saves a detection
  // when they come back.
  settle(path: string, token: number, faces: Faces): boolean {
    if (token !== this.generation) {
      return false;
    }
    this.inFlight.delete(path);
    this.found.set(path, faces);
    return true;
  }

  clear(): void {
    this.found.clear();
    this.inFlight.clear();
    this.generation += 1;
  }
}
