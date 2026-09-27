<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Show `position/count` on every burst member cell

## Purpose

The strip's burst badge (`.cell span.count`, painted by `paintBurst` in
`crates/app/ui/src/strip.ts`) shows the burst's size on the first displayed
cell of a run and `position/size` on the current cell only; every other member
shows nothing. Checked on a real Windows machine, that reads poorly: the
member cells give no clue where they sit in the burst until they are selected.
After this change every visible member of a burst shows its own `i/n`
(`1/3`, `2/3`, `3/3`) at all times, regardless of focus.

## Steps

- [x] Step 1: Badge every burst member with `position/size`; update tests and docs
  - Done when:
    - Every strip cell in a burst of two or more shows `${position + 1}/${size}`
      whether or not it is the current or a selected cell; a cell not in a
      burst shows nothing. The badge no longer changes when `current` moves.
    - Split-burst behavior is defined as: `position` and `size` are the
      file's place in the **whole** burst in capture order (what `burstMarks`
      already returns), not in the displayed run. So a burst of 5 with two
      members hidden by a filter shows `1/5`, `3/5`, `5/5` on the three visible
      cells, and under a non-capture sort a cell may read `3/3` before `1/3`.
      The band still opens and closes per displayed run (`first` / `last`
      unchanged).
    - A unit test in `crates/app/ui/src/burst.test.ts` covers the badge text
      for a member (any position, including the first), and for a split burst
      (whole-burst numbering on non-contiguous / partially hidden members),
      and `null` → empty.
    - `docs/usage.md` Bursts bullet and the `todo.md` by-hand check item
      describe the new behavior; no repository text still says the first cell
      shows the size or that the badge follows the selection (archived plans,
      `CHANGELOG.md` and `docs/plans/review-history/**` are history and stay).
    - `mise run ci` passes.
  - Implementation approach:
    - `crates/app/ui/src/burst.ts`: add a small pure exported
      `burstBadge(mark: BurstMark | null): string` returning
      `""` for `null`, else `` `${mark.position + 1}/${mark.size}` ``.
      Keep `BurstMark`, `burstMarks`, `groupBursts` unchanged; the
      whole-burst `position` / `size` they already carry are the defined
      split-burst semantics (see Trade-offs).
    - `crates/app/ui/src/burst.test.ts`: a `describe("burstBadge")` using the
      existing `members` map, e.g. `burstBadge(burstMarks(["a","e","b"], members)[2])`
      is `"2/3"`, `burstMarks(["a","b","c",...])[0]` is `"1/3"`, `null` is `""`.
    - `crates/app/ui/src/strip.ts`: `paintBurst` sets
      `cell.count.textContent = burstBadge(value ?? null)` and its comment is
      rewritten (no more "first displayed cell carries the size / current cell
      `position/size` instead"). Remove the `paintBurst(index, cell)` call from
      `highlight()` (it existed only so the badge followed `current`;
      `createCell` and `setBurst` still paint it). `current` is otherwise
      untouched.
    - `crates/app/ui/style.css`: no change expected; `.cell span.count` already
      has `width: auto`, `right: 2px`. Verify in `mise run tauri:dev` on a
      burst folder that a two-digit badge like `12/15` does not collide with
      the sharpness bar (left edge) or the file name below; adjust only if it
      does.
    - `docs/usage.md` (~line 356-359): replace "The first cell of the band
      shows the burst's size (`7`), and the current cell shows its position in
      the burst instead (`3/7`)." with a sentence saying every cell of the band
      shows the frame's position in the burst and the burst's size (`3/7`),
      counted over the whole burst in capture order, so a filter or sort that
      hides or separates frames leaves the numbers as they are.
    - `todo.md` (~line 319-325): reword the pending by-hand check to the new
      expectation (every member cell shows `position/size`, unchanged by the
      selection). `README.md` / `README.ja.md` only say "a count badge" and
      remain accurate; do not touch them.
    - Commit as `feat(app): show position/count on every burst member cell`.

## Trade-offs and risks

- **Split-burst numbering.** Chosen: whole-burst `position` / `size` (capture
  order over the whole folder), as `burstMarks` already supplies and as the
  archived burst-frame-nav plan decided for the size badge; it also matches
  what `Shift+x` (`rejectRest`) acts on and the MCP companion's 1-based
  `position`. Under a filter the visible numbers skip (`1/5`, `3/5`, `5/5`),
  which tells the user frames are hidden. Alternative: number relative to the
  displayed run (`1/3`, `2/3`, `3/3` for the three visible). Rejected: it
  would disagree with `Shift+x` scope and the companion, and needs a new
  per-run computation; the caller may override.
- **Where the badge text lives.** Chosen: a pure `burstBadge` in `burst.ts`
  so it is unit-tested (vitest runs in `environment: "node"`, so `strip.ts`
  has no tests). Alternative: inline the template literal in `paintBurst` and
  rely on the existing `burstMarks` split test alone; smaller but leaves the
  "regardless of focus" acceptance criterion untested.
- **Dropping `paintBurst` from `highlight()`.** It is the orphan this change
  creates (added only so the badge tracked `current`). Removing it is a small
  per-keystroke saving; keeping it is harmless. Chosen: remove.
- **Visual noise.** A badge on every member is more text on the strip; the
  user asked for this after a real-machine check, so it is taken as the
  intended trade.

## Progress

- (2026-09-28) Step 1 complete
