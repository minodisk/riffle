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

# Add an "Example workflow" section to the READMEs

## Purpose

`README.md` describes each feature on its own under `## Key features`, but
never shows how they chain into one job: cull the camera's RAW folder, clear
the rejects, develop the keepers in Lightroom / DxO PhotoLab, sequence the
exported JPEGs, check them, upload. A short numbered "Example workflow" right
after `### First steps` makes the use case clear at a glance, with each step
linking to where the details already are. `README.ja.md` gets the same
section in Japanese in the same PR, as `CLAUDE.md` requires.

## Steps

- [x] Step 1: Add `### Example workflow` to `README.md` and `### ワークフローの例` to `README.ja.md`
  - Done when:
    - `README.md` has `### Example workflow` directly after the `### First
      steps` block (after its closing paragraph "Judgments are saved to the
      sidecars as you go ...", before `## Key features`), and `README.ja.md`
      has `### ワークフローの例` at the matching position (after the
      "判定はその場でサイドカーに保存されるので ..." paragraph, before
      `## 主な機能`)
    - The section is six numbered steps, one or two lines each, in this
      order: (1) open the camera's RAW folder and cull with stars / rejects,
      (2) clear the rejects with `Move Rejected to Trash…`, (3) the judgments
      come in from the sidecars in Lightroom / DxO PhotoLab; develop the
      keepers and export JPEGs, (4) right-click the export folder in the
      folder tree and run `Sequence JPEG Timestamps…` to make
      `<folder>-sequenced/`, (5) open `<folder>-sequenced/` in Riffle (it
      opens view-only) to check the order, (6) upload `<folder>-sequenced/`
      to Google Photos
    - Every link resolves to an existing heading (see the anchor list below);
      no new headings are added elsewhere and no Key features text is copied
    - `mise run ci` passes (its `lychee --offline --include-fragments` run
      checks every anchor)
  - Implementation approach:
    - Files: `README.md` (insert after the "Judgments are saved ..."
      paragraph) and `README.ja.md` (insert after the matching paragraph).
      Nothing else changes.
    - Wording: reuse the menu item names exactly as `README.md` writes them
      (`Move Rejected to Trash…`, `Sequence JPEG Timestamps…`,
      `<folder>-sequenced/`, "view-only"); in `README.ja.md` keep the menu
      items in English in backticks as its Key features bullets do, and its
      terms (`不採用`, `スター`, `フォルダーツリー`, `書き出し先のフォルダー`,
      `<フォルダー>-sequenced/`, `閲覧専用`, `Google フォト`, `サイドカー`).
      Keep `README.md` wrapped at the existing ~80 columns; `README.ja.md`
      uses one long line per item.
    - Links (decided with the user): section-level anchors that stay in the
      README. Steps 1, 2, 4 and 5 → `#key-features` (ja: `#主な機能`);
      step 3 → `#working-with-other-software` (ja: `#他のソフトとの連携`);
      step 6 → no link (external service). Keep the two READMEs symmetric.
    - Do not add a per-feature heading to the READMEs to get finer anchors;
      that would change the Key features layout, which is out of scope.
    - Run `mise run ci` before opening the PR (lychee checks fragments
      offline; the Japanese fragment must match GitHub's slug of the heading
      exactly).

## Trade-offs and risks

- Link granularity: Key features bullets are bold list items, not headings,
  so a link cannot point at one feature. The user chose section-level README
  anchors over `./docs/usage.md#features` and over adding headings.
- Position: `### Example workflow` inside `## Getting started`, after
  `### First steps`, so it reads as the sequel to the first steps.
- The ja terminology for "keepers" has no fixed term in `README.ja.md`
  (`残した写真` appears once, in the sequencer bullet); reuse that rather than
  coining a new one.

## Progress

- (2026-10-01) Step 1 complete
