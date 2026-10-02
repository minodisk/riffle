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

# Japanese translations of docs/humans

## Purpose

`README.ja.md` gives Japanese readers the overview, but every link it makes
into `docs/humans/` lands on an English page and is tagged `（英語）`. This
work adds a Japanese counterpart (`<name>.ja.md`) next to each of the four
human-facing docs (`cameras.md`, `performance.md`, `raw-formats.md`,
`usage.md`), makes Japanese documents link only to Japanese documents, and
extends the `CLAUDE.md` sync rule so the pairs stay in step from now on.

Conventions every step follows (taken from `../../../README.ja.md` and the
archived `../_archived/20260924-readme-ja/learnings.md`):

- Tone and terms as in `README.ja.md`: です・ます, full-width parentheses,
  a space between Latin / digits and Japanese, セレクト (culling), 採用 /
  不採用 (pick / reject), スター, レーティング, カラーラベル, サイドカー,
  フォルダー (not フォルダ), フォルダーツリー, ストリップ, ビューア,
  メタペイン, ピント (focus), シャープネス, 連写 (burst), フィルター.
  Key names, identifiers, file paths, code spans, table structure and
  numbers stay as in the English file.
- Language-switch row: the first line of each file of a pair, before the H1,
  exactly as `README.md` / `README.ja.md` do:
  `<p align="center">English | <a href="./cameras.ja.md">日本語</a></p>` in
  the English file and
  `<p align="center"><a href="./cameras.md">English</a> | 日本語</p>` in the
  Japanese one. Use `<a href>`, not Markdown links (not parsed inside a
  one-line HTML block on GitHub).
- Links: a `.ja.md` file links to `.ja.md` counterparts and to
  `../../README.ja.md`; links to files with no Japanese version
  (`../../CONTRIBUTING.md`, `docs/agents/**`) stay as they are. Fragments
  are the GitHub slug of the translated heading in raw (unencoded) Japanese,
  as `README.ja.md#対応状況` already does. Avoid duplicate headings within a
  file, since the slug then gains a `-1` suffix.
- `README.ja.md`: when a step adds `<name>.ja.md`, repoint every
  `./docs/humans/<name>.md` link in `README.ja.md` to the `.ja.md` file,
  translate the fragment, and drop the `（英語）` tag after it.
- Verify with `mise run lint` (lychee `--offline --include-fragments` over
  `*.md` and `docs/**/*.md`) and `mise run ci` before opening the PR. Draft
  the translation in the scratchpad, copy it verbatim, and `diff -q` to be
  sure nothing was lost (the readme-ja plan did the same).

## Steps

- [x] Step 1: Translate `cameras.md` and `raw-formats.md`
  - Done when: `docs/humans/cameras.ja.md` and `docs/humans/raw-formats.ja.md`
    exist as full translations (the camera table in `cameras.md` keeps every
    row and column; body names and model numbers unchanged); both English
    files and both Japanese files carry the language-switch row;
    `cameras.ja.md` links to `../../README.ja.md#対応状況` (x2) and
    `./raw-formats.ja.md`, `raw-formats.ja.md` links to `./cameras.ja.md`
    (x4); `README.ja.md`'s six `cameras.md` links and one `raw-formats.md`
    link point to the `.ja.md` files with no `（英語）`; `mise run ci` passes.
  - Implementation approach:
    - These two are paired because they link to each other four times and
      share vocabulary (MakerNote, AF point, embedded JPEG); translating them
      together keeps the terms consistent.
    - `cameras.ja.md`'s link to `./usage.md` stays English in this step and
      is repointed in Step 3 when `usage.ja.md` lands; note that in
      `learnings.md` so it is not missed.
    - Record the chosen Japanese headings and their slugs in a
      `headings.md` in this plan folder so Steps 2–3 reuse the same terms.

- [x] Step 2: Translate `performance.md`
  - Done when: `docs/humans/performance.ja.md` exists as a full translation
    (every table, every number and every heading level kept); both files
    carry the language-switch row; its two `./cameras.md` links point to
    `./cameras.ja.md`; `README.ja.md`'s performance link points to
    `performance.ja.md` with no `（英語）`; `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged (`cameras.ja.md` exists).
    - Headings mix prose and parameters (`## Sony α7 V ARW (Apple Silicon
      Mac, n=20)`, `### Real folders on Windows`, `#### Real files (Linux
      WSL2)`): translate the prose part, keep product names, `n=20` and the
      platform in parentheses. Nothing links into these headings today, so
      the slugs only have to be unique within the file.
    - Translate table headers (`Step`, `Target`, `Measured (median)`) and
      the step descriptions; leave cell values, units and command lines as is.

- [ ] Step 3: Translate `usage.md`
  - Done when: `docs/humans/usage.ja.md` exists as a full translation of all
    five sections (`Features`, `Keys`, `Ratings and sidecars`,
    `MCP companion`, `Installing`), with key names, tool names, JSON
    examples, menu labels and settings keys unchanged; both files carry the
    language-switch row; it links to `../../README.ja.md`, `./cameras.ja.md`
    (x3) and keeps `../../CONTRIBUTING.md`; its two in-file fragment links
    (`#keys`, `#ratings-and-sidecars`) point at the translated headings;
    `cameras.ja.md`'s `./usage.md` link now points to `./usage.ja.md`;
    `README.ja.md`'s `usage.md` links (plain, `#installing`, `#keys`,
    `#mcp-companion`) point to `usage.ja.md` with translated fragments and
    no `（英語）`; `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1–2 are merged.
    - This is the largest file (~710 lines) and is kept as one PR because
      its sections cross-reference each other and splitting it would leave a
      half-translated file on `main`; the reviewer can go section by section.
    - `README.ja.md` already translates many of the same features (focus
      mark, sharpness bar, bursts, MCP companion, sidecars); reuse its
      sentences and terms where the English text is the same, so the two
      do not drift.
    - The `Keys` section is a table of default bindings; keep the key
      strings exactly (`Shift+x`, `Alt+ArrowLeft`, ...) and translate only
      the action column.

- [ ] Step 4: Extend the sync rule in `CLAUDE.md`
  - Done when: the Language section of `CLAUDE.md` says the Japanese
    exception covers `README.ja.md` and `docs/humans/*.ja.md`, and the sync
    rule reads that a PR changing `README.md` or `docs/humans/<name>.md`
    updates `README.ja.md` / `docs/humans/<name>.ja.md` in the same PR and
    vice versa; a final grep shows no `（英語）` in `README.ja.md` and no
    link from a `.ja.md` file to an English file that has a Japanese
    counterpart; `mise run ci` passes.
  - Implementation approach:
    - Assumes Steps 1–3 are merged (the rule is only true once all four
      pairs exist).
    - `CLAUDE.md` lines 118–120 are the only place the README-only rule is
      stated; CONTRIBUTING.md, `.claude/agents/*`, `.claude/skills/*` and
      `.github` do not repeat it (verified by grep at planning time; re-grep
      `README.ja` and `in sync` before closing the step).
    - Keep the edit to the three sentences of the exception; do not touch the
      Layout section.

## Trade-offs and risks

- One PR per file vs one PR for all: the four files total ~1650 lines of
  English; a single translation PR is not reviewable, so the plan splits by
  file. The cost is that `README.ja.md` flips its links one file at a time,
  so between Step 1 and Step 3 some links still carry `（英語）`. That is the
  state today, so nothing regresses.
- Step 4 is a small PR, kept separate because it is a process rule rather
  than content, and because it becomes true only when all pairs exist.
- Language-switch rows follow `README.md`'s convention, which means each step
  also edits the English file by one line.
- Japanese anchors: raw Japanese in the fragment (as `README.ja.md#対応状況`)
  vs percent-encoded. Follow the existing raw form; lychee resolves it today.
  If a translated heading ends up duplicated within a file, GitHub appends
  `-1` and the link breaks, so headings must stay unique.
- Translation drift: `README.ja.md` and `usage.ja.md` will describe the same
  features in two places. The Step 4 sync rule covers the pairs, but a change
  to a feature still needs both Japanese files updated; the Step 3 approach of
  reusing `README.ja.md`'s sentences reduces the chance they diverge in terms.

## Progress

- (2026-10-03) Step 1 complete
- (2026-10-03) Step 2 complete
