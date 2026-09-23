<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Burst-aware sharpness cue

## Purpose

The strip's sharpness bar (`crates/app/ui/src/sharpness.ts`, applied by
`applySharpness()` in `crates/app/ui/src/main.ts`) compares each displayed
file with the files within `SHARPNESS_RADIUS` (2) cells on either side of the
displayed (filtered, sorted) list, regardless of bursts. So a burst of ten
frames is scored in sliding windows of five, a burst's edge frames are
compared with unrelated frames next to them, a single between two bursts is
compared with burst frames, and changing the filter or sort changes every
bar. The cue should instead answer "is this the sharpest frame of its burst?"
for burst members and "is this sharper than the singles shot around it?" for
lone frames, and the answer must not depend on the filter or the sort.

## Decisions (agreed with the user)

- A burst member (burst size >= 2 in `bursts`) is compared with all members
  of its burst over `allFiles`, hidden (filtered-out) members included. If
  the burst's best frame is hidden, no displayed member is `best`; the bar
  still shows the ratio to that hidden best.
- A single (not in a burst of >= 2) is compared with up to
  `SHARPNESS_RADIUS` neighboring singles on each side in capture order over
  all files; bursts are skipped entirely, filter and sort are ignored. A
  `null`-scored single still takes a window slot (as today).
- The display stays as it is: ratio bar plus accent on the best, every tied
  frame is best, `null` for a file without a score.

## Steps

- [x] Step 1: Compute the sharpness cue per path over all files in capture order, by burst
  - Done when:
    - `crates/app/ui/src/sharpness.ts` exposes a function that, given all
      paths, each path's score (or `null`), and each path's burst membership,
      returns per-path `RelativeSharpness | null` following the decisions
      above; it stays free of DOM and Tauri.
    - `applySharpness()` in `crates/app/ui/src/main.ts` computes (or looks
      up) the result per displayed path from that function and hands it to
      `strip.setSharpness(at, value)`; the result is refreshed whenever
      scores or `bursts` change (both happen only in `refreshEntries`, which
      already calls `applySharpness()`; `refilter` calls it after
      `strip.setFiles` — verify no other call is needed).
    - `crates/app/ui/src/sharpness.test.ts` covers: a burst whose best
      member is not displayed (the others get its ratio and none is `best`),
      singles skipping a burst between them in capture order, clamping at
      both ends, `null` scores (returned as `null`, and a `null` single still
      taking a slot), ties all `best`, a lone file `ratio 1 / best`, and that
      the result does not change with the order of the input paths (capture
      order is applied inside, or by the caller if that shape is chosen).
    - The comments in `sharpness.ts` and above `applySharpness()` no longer
      say the window runs over `files` / the filtered view.
    - `mise run ci` passes.
  - Implementation approach:
    - Suggested signature (decide the final shape in the step): take all
      paths plus lookups for score and burst id and sort into capture order
      inside with `orderFiles("capture", ...)` from `sort.ts` (as
      `groupBursts` in `burst.ts` does), or take an already capture-ordered
      list from `main.ts`. Prefer the variant that keeps `sharpness.ts`
      simple and testable with plain arrays.
    - Burst membership input: `null` for a single, the burst id otherwise;
      in `main.ts` derive it as
      `const m = bursts.get(path); m !== undefined && m.size > 1 ? m.burst : null`,
      the same `size > 1` rule `burstMarks` uses so the cue and the band
      agree.
    - Bursts: one pass collecting the max non-null score per burst id over
      all paths; per member `ratio = max > 0 ? score / max : 1`,
      `best = score === max`. Since the max is over all members, a hidden
      best member leaves the displayed ones with `best: false`.
    - Singles: from the capture-ordered list keep only the singles (a
      compacted array), then apply the current positional ±radius window
      over that array (this keeps "a `null` single still takes a slot" and
      the clamping behavior).
    - `applySharpness()`: recompute over `allFiles` on each call (`refilter`
      and `refreshEntries` only; one sort plus a linear pass), then
      `files.forEach((path, at) => strip.setSharpness(at, result.get(path) ?? null))`.
      Do not add a cache speculatively.
    - Rewrite the existing five tests to the new signature; the "window
      clamped", "nulls", "ties" and "lone file" cases carry over with all
      paths as singles.
    - README.md already says "which frame of a burst is sharpest"; leave it
      unless a phrase for singles is wanted. No `docs/agents` change needed.
    - Commit as `feat(app): compare the sharpness cue within bursts`.

## Trade-offs and risks

- Recompute in `applySharpness()` vs cache when `bursts` change: recompute
  is simpler and keeps one source of truth; it adds a capture-order sort to
  every `refilter`. If it ever matters, cache the per-path map next to
  `bursts` in `refreshEntries`.
- Files without a usable capture time: `groupBursts` makes each a burst of
  its own (size 1, so a single here), and `orderFiles("capture")` sorts them
  last, so they are compared with each other at the end of the capture order.
- A burst whose best member is hidden shows no accent among displayed
  frames. This is the user's decision; note it in the function comment so it
  is not later mistaken for a bug.
- `compare.ts` (`v`, "current shot beside the sharpest frame in its burst")
  reads `sharpness` directly and is unaffected.

## Progress

- (2026-09-23) Step 1 complete
