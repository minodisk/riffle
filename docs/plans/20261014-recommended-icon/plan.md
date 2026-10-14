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

# The recommended icon: mark on the filmstrip the frame the personal ranking puts first in its burst

## Purpose

The personal ranking (`crates/core/src/rank.rs` `rank`, served by
`crates/app/src/rank.rs` `ranking_of`) shows today only in the sort menu's
`Recommended` order and in the meta pane's `Recommended` / `Reason` /
`Confidence` rows (`crates/app/ui/src/meta.ts` `rankingRows`). Which frame of
a burst the model would pick is therefore not visible at a glance: the user
has to switch the sort or read the meta pane frame by frame. This plan puts
an icon on the filmstrip thumbnail of every frame the ranking places first
in its burst, while AI analysis (`aiEnabled`) is on, next to the existing
good-photo face icon of `focus.ts` `photoTier`.

What `rank` returns, which the criterion rests on (`Ranked` in
`crates/core/src/rank.rs`, mirrored by `Ranked` in `meta.ts`):

- `score`: the frame's logit above its burst's mean; a lone scored frame is
  exactly 0, the first of a burst is above 0 unless every frame ties.
  Scores are comparable within one burst only; the model was fit on pairs
  of one burst.
- `place`: `1 + (the burst's scored frames with a strictly greater score)`,
  so tied frames share a place; `of`: the burst's frames with a score (an
  embedding at `EMBED_VERSION`), not the burst's size.
- The gap between the first and the second frame is not in `Ranked`.
- The model's confidence (`RankModel.confidence`: `none` / `low` /
  `medium` / `high`) is one value per model, not per file. The built-in
  prior alone (`none`) put the burst's top-ranked frame on a pick in about
  55% of the author's held-out bursts against 39% for the first frame
  (`docs/plans/_archived/20261012-image-embeddings/ranking.md`).

The criterion is one pure function of a `Ranked` entry, so the
folder-wide "probability of being picked" another session is building can
replace it later by changing that function alone. Nothing else is
abstracted for it.

Decided with the user (2026-10-14):

- No score-gap gate: every burst with two or more scored frames marks its
  first (two on an exact tie).
- No confidence gate: the icon shows at every confidence, `none` included.
- The icon is Lucide `sparkles` in `--foreground`, in the bottom-left row
  right of the face icon. Every corner of the cell is taken (flag, stars,
  face icon, burst count `n/m`, and the burst count is on every frame this
  icon can mark), so the bottom-left row is where it goes. The user may
  later replace the face icon with this one if it proves the more useful;
  that is not part of this plan.

Out of scope: the ranking's learning (`rank.rs` pairs, `Labeled`, `fit`),
the training data range, refit triggers, per-genre weighting, folder-wide
learning, and the sharpness "best" mark of
`docs/plans/20261010-burst-best-mark/plan.md` (a separate, unlearned mark;
this icon does not reuse its `.best` bar).

## Steps

- [x] Step 1: Mark the burst's first-ranked frame with an icon on the strip, and document it
  - Done when: while AI analysis is on, the cell of every file whose
    `ranking_of` entry meets the criterion shows the icon, none while it is
    off; the icons change after `ranking-updated` (a refit) and after a
    folder reopen, a filter or a sort change (the cell index moves); the
    criterion is a pure function with its own unit test; `CLAUDE.md`'s
    frontend layout paragraph describes the new module; `docs/humans/usage.md`
    / `usage.ja.md` and `README.md` / `README.ja.md` describe the icon;
    `mise run ci` passes; a manual check on a folder with bursts and
    `aiEnabled` on confirms the icon sits beside the face icon and
    disappears when AI analysis is turned off in Settings (listed in
    `learnings.md` as pending if not run).
  - Implementation approach:
    - **The criterion**, a new `crates/app/ui/src/recommend.ts` with
      `recommended(ranked: Ranked | undefined): boolean` returning
      `ranked !== undefined && ranked.place === 1 && ranked.of >= 2`, taking
      the `Ranked` type from `meta.ts`. `recommend.test.ts` (vitest, as
      `focus.test.ts`) covers: undefined (no entry, so no embedding or AI
      off), `place 1 of 1` (a lone frame, never marked), `place 1 of 2`,
      `place 2 of 2`, and two tied frames both at `place 1 of 3` (both
      marked, since `rank` gives ties the same place).
    - **The strip** (`crates/app/ui/src/strip.ts`): follow `tiers` /
      `setTier` / `paintTier` exactly: a `recommended` set per index,
      `export function setRecommended(index: number, on: boolean)` that
      repaints the cell when it is on screen, a `recommended:
      HTMLSpanElement` in `Cell` created in `createCell` next to `tier` and
      painted in `createCell`'s paint sequence, and the set cleared in
      `setFiles` with the other per-index maps. The icon is inlined with
      `innerHTML` and hidden with the `hidden` attribute like `cell.tier`.
    - **The icon** (`crates/app/ui/src/icons.ts`): add Lucide `sparkles`
      verbatim from the lucide-static version the file's header names as
      `SPARKLES_SVG`, and add its name to the license header, per
      `docs/agents/ui-styling.md` "Icons".
    - **Placement and color** (`crates/app/ui/style.css`): the bottom-left
      row, face first, then the sparkles: either a `.cell span.marks` flex
      row with `gap: 2px` wrapping both spans (move `.cell span.tier`'s
      placement rules to the wrapper), or a second absolute span that shifts
      left when the tier is hidden (the wrapper is cleaner; the step
      decides). 12 px like the face icon, in `var(--foreground)` with the
      same `drop-shadow(0 0 2px #000)` halo; no new color. Rules go in the
      `.cell` block; run the color and layout greps of `ui-styling.md`
      before the PR (no `margin`).
    - **Wiring** (`crates/app/ui/src/main.ts`): an `applyRecommended()`
      beside `applyTiers()` that does `files.forEach((path, at) =>
      strip.setRecommended(at, recommended(fileRanks.get(path))))`. Call it
      in `refreshRanks`'s `.then` after `fileRanks` is set (and after any
      `refilter` it runs); in `setRanking` on the off branch after
      `fileRanks = new Map()`; and wherever `applyTiers()` runs after the
      file list is rebuilt, since the strip's maps are indexed by position
      and `setFiles` clears them. `ranking-updated` already calls
      `refreshRanks`, so a refit updates the icons through it. While AI is
      off `fileRanks` is empty, so `recommended` returns false everywhere.
    - **Docs**: `docs/humans/usage.md` and `usage.ja.md` say, in
      **Recommended order**, that the strip marks the burst's first-ranked
      frame with the `sparkles` icon (name the Lucide icon and its ISC
      license as the face icon's sentence in **Focus mark** does), that a
      lone frame and a frame without an embedding get none, that the icon
      follows the model (it moves after a refit) and that it is the same
      modest suggestion as the order. `README.md` / `README.ja.md`'s
      **Recommended order (AI analysis)** bullet gets one clause for the
      icon and the Lucide credit line lists the new icon. `CLAUDE.md`: add
      `src/recommend.ts` to the frontend layout paragraph.
    - Not touched: `filter.ts`, `companion.ts` / `mcp.rs`, `sort.ts`.

## Trade-offs and risks

- **Two icons on one frame.** A good frame that is also first in its burst
  shows both; the row keeps the face first. On a narrow cell the row could
  approach the burst count at the bottom-right; check during the manual
  check.
- **Partially embedded bursts.** A burst where only one frame has an
  embedding yet (`of === 1`) is unmarked until the next `ranking_of` read;
  `refreshRanks` runs on `embed-done`, so the icon fills in then.

## Progress

- (none yet)
