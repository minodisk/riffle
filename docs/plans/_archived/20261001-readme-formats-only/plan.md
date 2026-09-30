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

# README lists RAW formats only; cameras live in docs/cameras.md

## Purpose

`README.md`'s `Compatibility > RAW formats and cameras` section has grown into a
76-line per-camera checklist plus three per-format paragraphs that name bodies
and preview sizes. `docs/cameras.md` already carries every one of those bodies
in its per-camera table and the RAF preview-size table. Keeping both means two
lists to update on every new body (the `todo.md` items and the
`raw-metadata-parsing.md` rule already say "add it to the README list").
After this change the README names only the six RAW formats and points to
`docs/cameras.md` as the single list of verified cameras, and the body- and
size-specific notes live next to the tables they describe.

## Decisions (approved by the user)

- Rename the heading to `### RAW formats` (`### RAW 形式` in `README.ja.md`).
  No link targets the old anchors.
- Reword the README intro line ("For the supported cameras and formats, see
  Compatibility") to link the formats to `#compatibility` and the verified
  cameras directly to `docs/cameras.md`; mirror it in `README.ja.md`.
- Drop the RAF older-body range enumeration (X-T1 to X-T20, ...) from prose;
  the cameras.md size table lists each body.
- Include the three `todo.md` wording fixes in this change.

## Steps

- [x] Step 1: Move the camera list and the per-format body notes from the README to `docs/cameras.md`, and redirect every doc that pointed at the README list
  - Done when:
    - `README.md`'s `RAW formats` section names no camera model: it lists ARW, CR3, DNG, NEF, ORF, RAF (as a flat list, no `[x]` checklists), keeps the JPEG-only view-only paragraph, keeps the "What each camera records ... / Why support is listed per camera ..." pointer paragraph, and keeps the works-report / issue-template paragraph reworded to "a camera not listed in `[docs/cameras.md](./docs/cameras.md)`".
    - The CR3 HDR PQ, RAF and ORF paragraphs are gone from `README.md` and their content is in `docs/cameras.md`, merged with what it already says (no duplicated sentence).
    - `docs/cameras.md` still lists every body that was in the README checklist (verified at planning: all 76 bodies already have a row; re-check with a diff during implementation).
    - `README.ja.md`'s section mirrors the new `README.md` section sentence for sentence (Japanese body, `（英語）` marker on links to English docs as the file already does), and its intro line mirrors the new README intro line.
    - No doc anywhere in the repository still tells the reader or an agent to add or find a body in the README list; they point at `docs/cameras.md` instead.
    - `mise run lint` (lychee `--include-fragments` over `*.md`, `docs/**/*.md`, `.claude/**/*.md`) and `mise run ci` pass.
  - Implementation approach:
    - `README.md` `### RAW formats and cameras` section: rename the heading per Decisions, replace the checklists with a flat format list, remove the CR3 / RAF / ORF paragraphs, reword the report paragraph. Reword the intro line per Decisions.
    - `docs/cameras.md`:
      - The intro currently says "For the tested cameras, see `[Compatibility](../README.md#compatibility)`", which becomes circular once cameras.md is the list. Reword to state that the table below is the list of bodies verified on a real file, and that the RAW formats are in the README's Compatibility section (keep the `../README.md#compatibility` link).
      - Add the moved notes near the existing per-maker paragraphs rather than in a new far-away section:
        - CR3 HDR PQ: a new paragraph after the Canon/Nikon AF paragraph: HEIF files hold HEVC instead of JPEG; Riffle decodes the 1620x1080 HEVC preview and tone-maps it to sRGB, so thumbnail / preview / sharpness work; the full-size image is not decoded, so 1:1 (`z`) shows a crop of that preview.
        - RAF: extend the existing sentence that introduces the size table ("The embedded JPEG is the preview and the 1:1 view on every Fujifilm body ...") with the README's facts the table does not state in prose: one embedded JPEG, below the sensor's resolution, 1:1 shows it at its own size; older X-series and FinePix bodies open but their 1:1 view is limited to a 1920x1280-or-smaller preview (at most 2176x1448). Do not enumerate the body ranges.
        - ORF: extend the OM System / Olympus paragraph with: one embedded JPEG, 3200x2400 on every listed body, below the sensor's resolution; 1:1 shows it at its own size.
    - Cross-reference fixes (re-grep for `README` and `Compatibility` before finishing):
      - `docs/raw-formats.md` ("why the `[Compatibility](../README.md#compatibility)` list names bodies rather than formats", and "why the [Compatibility] list names each body verified on a real file"): point at `[What the camera records](./cameras.md)` as the per-body list.
      - `docs/performance.md` ("the ones listed in `[Compatibility](../README.md#compatibility)`", "22 Fujifilm bodies listed in ..."): point at `./cameras.md`.
      - `docs/agents/raw-metadata-parsing.md`: "do not add a Sony body to the README's supported list" → "to the `docs/cameras.md` table".
      - `todo.md`: "add each working body to the README "RAW formats and cameras" list", "are listed in the README ... left off the README", "Add the seven bodies to `README.md`, `README.ja.md` and `docs/cameras.md`" → refer to `docs/cameras.md` only. Minimal wording edits; do not touch other todo text.
      - `CHANGELOG.md` is history; do not edit.
    - Verify with `mise run lint` locally.

## Trade-offs and risks

- `docs/raw-formats.md`'s premise ("why support is listed per camera rather than per format") is unchanged in substance: support is still listed per camera, just in cameras.md. Only the pointer moves.
- A "missing body" gap: planning found none. If the implementer's diff finds one, add it to the cameras.md table with `–` cells and a note that its features are unverified rather than guessing values.

## Progress

- (2026-10-01) Step 1 complete
