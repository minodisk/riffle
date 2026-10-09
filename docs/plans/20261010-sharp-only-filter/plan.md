<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Make the filter's Sharp item mean "in focus but not Good"

## Purpose

#754 folded the good and fair tiers into one `good` tier, so the strip's
filter menu's `AF eye` section is now `Good` / `Sharp` / `Soft` / `Unknown`.
`Sharp` (`data-candidate="candidate"`) still means every focus candidate, and
`Good` is a subset of it (`photoTier` in `crates/app/ui/src/focus.ts` returns
`good` only for a candidate), so the two items overlap: checking `Sharp`
shows the good frames too, and there is no way to see only the in-focus
frames that miss `Good` (profile faces, closed eyes, probability under the
Good bar). The filter ORs within a group and cannot exclude, so the item
cannot be dropped; it changes meaning instead, to exactly the frames
`markState` colors in the dim green `candidate_only` (`#8b8`), which is
already the color of the item's face icon. After this, the four items
partition the frames and the menu reads like the mark's colors.

## Steps

- [x] Step 1: Narrow the `Sharp` item to candidates outside the `good` tier, rename it, and bring the docs in line
  - Done when:
    - With the renamed item checked, `passes()` lets through a frame with
      `candidate === "candidate"` and no tier, and rejects a `good` frame, a
      `not_candidate` frame, an `unknown` frame and a frame the pass has not
      reached. `Good`, the renamed item, `Soft` and `Unknown` each let through
      exactly one of those classes (no overlap), and checking several still
      ORs them; the other groups still AND.
    - `crates/app/ui/src/filter.test.ts` pins this: the test
      `"Sharp keeps passing every candidate, in the tier or not"` is replaced
      by one asserting a good frame does not pass the renamed item, and a
      test asserts the four items are disjoint over the four classes
      (good / candidate-not-good / not_candidate / unknown-or-undefined).
    - `crates/app/ui/index.html` shows the new label on the
      `data-candidate="candidate"` item; the item keeps its face icon in
      `FOCUS_MARK_COLORS.candidate_only`.
    - `docs/humans/usage.md` **Filter menu** bullet lists the current
      `AF eye` items: `Good` for a bright green focus mark (the good tier),
      the renamed item for a dim green mark (AF eye in focus but not good),
      `Soft` for orange, `Unknown` for white including files the second pass
      has not reached, and says checking `Good` and the renamed item together
      shows every in-focus frame. `docs/humans/usage.ja.md` in sync (Japanese
      body). `README.md` (`AF eye` section item list, and "also on the filter
      menu's `Sharp` item") and the matching sentences in `README.ja.md` name
      the new label.
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/ui/src/filter.ts` `passes()`: replace the two-clause OR
      with one lookup of the item the frame belongs to, mirroring
      `markState` in `focus.ts`: `state.candidates.has(tier ?? candidate ?? "unknown")`
      (a non-null `tier` implies `candidate === "candidate"`, so the fallback
      order is safe). Update the top comment to say the `candidate` item
      means a focus candidate not in the tier, and that the four items
      partition the frames. Keep `AfEye`, the `candidate` data key and
      `passes()`'s signature so `main.ts` (the `passes` wrapper, the generic
      `data-candidate` handlers) needs no change beyond the comment near the
      AF eye face icon coloring that names `Sharp`.
    - Do not rename the `data-candidate="candidate"` key: the filter state
      is in-memory only (`shownCandidates` in `main.ts`, no store key), and
      the MCP companion (`companion.ts`, `mcp.rs`) exposes only a
      `filtered` boolean, so nothing persists or reads the key.
    - Label: `Sharp only`. Use the same label in all four docs.
    - `docs/plans/20261008-burst-keep-score/plan.md` Step 6 (open) says the
      filter menu's `AF eye` text "stays"; add a one-line note to its Step 6
      that this plan rewrote the filter bullet so Step 6 does not revert it
      (that plan's other content is untouched), and record it in this plan's
      `learnings.md`.
    - Out of scope: `focus.ts` thresholds, `crates/core/src/eyes.rs`,
      `crates/cli/src/main.rs`, `docs/plans/20261010-mesh-roll/`.

## Trade-offs and risks

- **Label.** `Sharp only` is short and sits under `Good` as "sharp, and only
  that". It can be misread as "show only sharp frames", i.e. the old meaning;
  the docs bullet spells out the meaning. `Sharp, not Good` was rejected as
  longer and repeating the item above it.
- **Rewriting `passes()` vs. subtracting the tier.** The chosen form
  (`has(tier ?? candidate ?? "unknown")`) makes the partition structural,
  the same as `markState`. Keeping the OR and adding `&& tier === null` on the
  candidate clause keeps the diff smaller but leaves two code paths that must
  agree.
- **Doc overlap with the open burst-keep-score plan.** Its Step 6 rewrites
  the Focus mark paragraph of `usage.md` / README and states the filter text
  stays. If both land, the later PR must not revert the filter bullet; the
  step above leaves a note for it. No code conflict: Step 6 is docs only.
- **Behavior change for a user who checks `Sharp` to mean "all in focus".**
  Checking `Good` and the renamed item together restores that set; the docs
  bullet says so.

## Progress

- Step 1 done: `passes()` in `crates/app/ui/src/filter.ts` now does a single
  `tier ?? candidate ?? "unknown"` lookup, so the Sharp item passes only
  in-focus frames outside Good. The item is renamed `Sharp only`, the docs
  (README, `docs/humans/usage`) list all four `AF eye` items, and the
  burst-keep-score plan's Step 6 carries a note about the change.
