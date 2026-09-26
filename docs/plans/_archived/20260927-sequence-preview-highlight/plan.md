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

# Highlight the changed part of the new time in the Sequence JPEG Timestamps preview

## Purpose

The preview list of `File > Sequence JPEG Timestamps…` renders every row as one
plain string (`name  old -> new`) and only dims the unchanged rows to `#777`,
which is too subtle to see at a glance which files get a new time and by how
much. Splitting each row into spans and painting the differing tail of the new
time in an accent color makes the changed rows, and the exact field that
changes (typically the seconds), stand out. Unchanged rows stay listed and
dimmed; the `N of M files get a new time` count line stays.

## Steps

- [x] Step 1: Split preview rows into spans and highlight the differing part of the new time
  - Done when:
    - On changed rows the part of `new` that differs from `old` is wrapped in a span with an accent color; the rest of the row keeps its current color.
    - Unchanged rows still get the `unchanged` class and stay dimmed as today; `changedLine` is unchanged.
    - A pure helper in `crates/app/ui/src/sequence.ts` computes the split and is covered by `crates/app/ui/src/sequence.test.ts`.
    - `mise run ci` passes.
  - Implementation approach:
    - Time format: both `old` and `new` are `ExifDateTime` rendered with `EXIF_DATETIME_FORMAT = "%Y:%m:%d %H:%M:%S"` (`crates/core/src/sequence.rs`, `crates/app/src/sequence.rs` calls `.to_string()`), so they are always 19 chars, `YYYY:MM:DD HH:MM:SS`, fields separated by `:` and one space. `changed` is a separate boolean from Rust; keep using it for the `unchanged` class rather than re-deriving from string equality.
    - `sequence.ts`: replace `rowText(row)` (only used by `main.ts` and its test) with a helper that returns the row's parts, e.g. `rowParts(row): { head: string; same: string; diff: string }` where `head` is `` `${baseName(path)}  ${old} -> ` ``, and `same + diff === new`. Compute the boundary by field, not by character: find the first index where `old` and `new` differ, then move it back to just after the preceding separator (`:` or space, or 0), so `03:04:58 -> 03:04:59` highlights `59`, not `9`, and a minute rollover `03:04:59 -> 03:05:00` highlights `05:00`. For an unchanged row `diff` is `""`. Keep `baseName` as is. No new abstraction beyond this one function.
    - `main.ts` (`showSequencePreview`): build the `li` from the parts: `head` and `same` as text nodes (or one string), `diff` in a `<span class="changed">` (only when non-empty). Keep `item.classList.toggle("unchanged", !row.changed)`.
    - `style.css`: add `#sequence-rows .changed { color: var(--pick-color); }` next to `#sequence-rows .unchanged`. `--pick-color` (`#6bdc8a`) is the existing "positive" accent in `:root`, chosen by the user; `--reject-color` is already used for `#sequence-failed`, so green reads as "this gets a new time" without clashing.
    - `sequence.test.ts`: replace the `rowText` test with tests for the new helper: a seconds-only change, a minute/hour rollover (boundary moves back to the field start), an unchanged row (`diff === ""`), and the Windows-path basename case already covered.
    - No docs change: README only says "A preview shows the new times first", which stays true.

## Trade-offs and risks

- Diff granularity: character-level common prefix is the simplest but highlights `9` for `58 -> 59`, which reads oddly; field-level (back up to the previous `:`/space) highlights `59` / `05:00` and matches "typically the seconds". The plan takes field-level.
- Accent color: `--pick-color` (green), chosen by the user over the inline `#4a9eff` blue.
- Keeping `rowText` exported alongside the new helper would leave dead code; the plan replaces it (its only callers are `main.ts` and the test).
- Another plan in flight (`sequence-reveal-output`) may touch the same Sequence dialog files; expect a possible merge conflict in `main.ts` / `sequence.ts`.

## Progress

- (2026-09-27) Step 1 complete
