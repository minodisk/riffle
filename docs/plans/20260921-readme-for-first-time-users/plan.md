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

# README for first-time users

## Purpose

`README.md` today is a single 490-line file that mixes the first-time user's
questions (what is this, how do I install it, what do I press) with exhaustive
behaviour descriptions and the whole performance log. A newcomer has to scroll
past filter-menu semantics and symlink benchmarks to find the install table.

The user reviewed a Japanese draft of the new README at
`/private/tmp/claude-501/-Users-mino--herdr-worktrees-riffle-worktree-lucky-meadow-222c/99090927-c783-432d-8850-78e69ad21ba5/scratchpad/README.ja.md`
(outside the repository; **every implementer must read it from that absolute
path** — it is the agreed structure and content, to be rendered in English).
After this work:

- `README.md` is short and user-facing: intro, Getting started, Features (one
  line each), Keys, Working with other software, Compatibility, For developers.
- The detailed behaviour moves, unchanged in substance, to `docs/usage.md`, and
  the Performance section to `docs/performance.md`.
- Compatibility reports have somewhere to go: three GitHub issue forms
  (`os.yml`, `camera.yml`, `software.yml`) and `config.yml` pointing at
  Discussions "works" threads.

Note: the review-only `README.ja.md` must **not** be committed; read the draft
from the scratchpad path above.

## Decisions made at approval

- Two PRs (Step 2 touches `.github/**`, which needs user approval to merge).
- Lightroom: Getting started says judgements are picked up by DxO PhotoLab and
  tools that read XMP sidecars, without asserting Lightroom works. The intro may
  name Lightroom only as an example of an XMP-reading tool.
- `blank_issues_enabled: true`.
- Discussions threads are placeholders with visible `TODO` markers; creating
  them is a follow-up that needs the user's confirmation.

## Steps

- [ ] Step 1: Split README.md into `docs/usage.md`, `docs/performance.md` and a rewritten user-facing README
  - Done when:
    - `docs/usage.md` holds the current README's "Features" (the whole
      bulleted list plus its lead-in paragraph about opening a folder),
      "Keys" (both tables and the rebinding paragraph), "Ratings and sidecars"
      (all paragraphs, including the `-1` / Lightroom note) and "Installing"
      (full table with `.msi` / `.deb` / `.rpm`, the three-OS first-launch
      list including the `xattr` command, the full Updating paragraph with the
      Windows and Linux notes). Wording is preserved; only headings, the
      intra-document links (`#ratings-and-sidecars`) and a short lead-in line
      pointing back to `README.md` change.
    - `docs/performance.md` holds the current "Performance" section to the end
      of the file, including footnote `[^1]`, with headings promoted one level
      (`###` -> `##`, `####` -> `###`) and wording preserved.
    - `README.md` follows the draft section by section, in English (see the
      approach below for the content rules).
    - Existing prose pointers to README sections that moved are updated so
      they are not stale: `CONTRIBUTING.md` (the "Performance" in the README
      pointer -> `docs/performance.md`), and the `todo.md` items that name
      README's "Per-page preview read", "Measuring on your own folder", "End to
      end, keypress to pixels", "The 1:1 focus check path" and "Updating
      paragraph" (point them at `docs/performance.md` / `docs/usage.md`).
      `docs/agents/tauri-app.md` historical narrative may stay.
    - `mise run lint` passes (lychee runs `--offline --include-fragments` over
      `*.md` and `docs/**/*.md`, so every relative link and `#anchor` in the
      three files must resolve).
    - `README.ja.md` is not part of the commit.
  - Implementation approach:
    - Content rules for `README.md`:
      - Intro: three benefit bullets from the draft (embedded JPEG only, so no
        waiting; judgements go to sidecars that DxO PhotoLab and XMP-reading
        tools such as Lightroom read, RAW never written; no developing /
        editing) and a pointer to `#compatibility`.
      - Getting started > Install: table trimmed to one file per OS
        (`aarch64.dmg`, `x64.dmg`, `x64-setup.exe`, `amd64.AppImage`);
        first-launch permission for macOS (right-click -> Open) and Windows
        (SmartScreen More info -> Run anyway); auto-update in one line; "see
        docs/usage.md for the other packages and details".
      - Getting started > First steps: the draft's three numbered steps
        (drop a folder / `Cmd+O`; `↑` `↓` + `1`-`5` + `x`; `z` for the 1:1
        check), then the "saved as you go" line. Do not assert Lightroom picks
        the judgements up: say they are picked up by DxO PhotoLab and tools that
        read XMP sidecars.
      - Features: one line each for filmstrip, 1:1 focus check, focus mark,
        sharpness cue, shooting info, filter and sort, Move Rejected to Trash,
        undo/redo, auto-advance; end with "Detailed behaviour: docs/usage.md".
      - Keys: the draft's condensed single table (`↑`/`↓`, `1`-`5`, `0`, `x`,
        `p` (.dop only), `u`, `Ctrl+Alt+1`-`7`, `Ctrl+Alt+0`, `z`, `f`,
        `Cmd+O`/`Ctrl+O`) followed by "Keys can be changed in
        `Riffle > Settings...`".
      - Working with other software: XMP (default, `FOO.xmp`, no pick) vs DxO
        PhotoLab (`FOO.ARW.dop`, has pick, `File > Open in DxO PhotoLab`);
        other tools' sidecars edited in place, everything but stars / flag /
        label untouched.
      - Compatibility: intro line "unchecked = not verified yet"; `### OS`
        (macOS: 26 (Apple Silicon) [x]; Windows: 11 [x]; Linux: Ubuntu [ ],
        Fedora [ ]); `### RAW formats and cameras` (ARW: Sony α7 V [x]; DNG:
        Leica M11-P [x]; the Sigma BF / fp L rows are dropped);
        `### Sidecar formats and software` (XMP: Adobe Lightroom Classic [ ],
        Capture One [ ]; DOP: DxO PhotoLab 10 [x]). Each subsection ends with
        the two-sentence call to action: "works -> post in the <X> works
        report thread in Discussions; does not -> open an issue from the <X>
        template", linking
        `https://github.com/minodisk/riffle/issues/new?template=os.yml`,
        `...?template=camera.yml`, `...?template=software.yml`. The cameras
        subsection also asks for a sample file (attach or link; one is enough,
        landscape and portrait if possible so rotation and the focus mark can
        be checked). Discussions threads do not exist yet: link
        `https://github.com/minodisk/riffle/discussions` and mark the three
        links with a visible `TODO` (e.g. an HTML comment
        `<!-- TODO: replace with the OS works-report thread -->`); record the
        follow-up in `learnings.md`.
      - For developers: `CONTRIBUTING.md` (build from source) and
        `docs/performance.md`.
      - No phases, verification logs or personal test environment in README.
    - Link discipline: README links to `./docs/usage.md`,
      `./docs/performance.md`, `./CONTRIBUTING.md`; `docs/usage.md` links back
      with `../README.md`; internal anchors in `docs/usage.md` stay
      `#ratings-and-sidecars`. External `github.com` links are not checked by
      the offline lychee run, so verify the template query strings by eye
      against the file names chosen in Step 2.
    - Files: `README.md`, `docs/usage.md` (new), `docs/performance.md` (new),
      `CONTRIBUTING.md` (one pointer), `todo.md` (pointers only).

- [ ] Step 2: Add the GitHub issue forms and the template chooser config
  - Done when:
    - `.github/ISSUE_TEMPLATE/os.yml`, `camera.yml`, `software.yml` and
      `config.yml` exist and are valid GitHub issue-form YAML (each form has
      `name`, `description`, `title` prefix, `labels` as judged, and a `body`
      whose first element is a `markdown` note; every `input` / `textarea` /
      `dropdown` has a unique `id` and `attributes.label`).
    - `os.yml` asks for: OS and version (input), Riffle version (input), what
      happened (textarea), `Riffle.log` contents or attachment (textarea, with
      the log folder path per OS in the description:
      `%LOCALAPPDATA%\com.minodisk.riffle\logs\`,
      `~/Library/Logs/com.minodisk.riffle/`,
      `~/.local/share/com.minodisk.riffle/logs/`, or `Help > Open Log Folder`).
    - `camera.yml` asks for: camera model (input), firmware version (input),
      sample file (textarea: attach or share a link; a landscape and a
      portrait shot if possible; note GitHub's 25 MB limit on non-image
      attachments, so a RAW usually has to be a link), what happened
      (textarea).
    - `software.yml` asks for: software and version (input), sidecar format
      (dropdown: XMP / DxO PhotoLab `.dop`), the judgement applied in Riffle
      (textarea: stars / pick / reject / colour label), how the software shows
      it (textarea).
    - `config.yml` sets `blank_issues_enabled: true` and `contact_links` to the
      three Discussions "works" report threads (OS / camera / software), each
      with a `name`, `url` and `about`; URLs are the Discussions placeholder
      with a `TODO` comment until the threads exist.
    - The `?template=` names in README (Step 1) match the file names exactly.
    - `mise run ci` passes (actionlint only checks `workflows/`, so the forms
      are validated by review against the GitHub issue-forms syntax and, after
      merge, by opening `https://github.com/minodisk/riffle/issues/new/choose`
      once; record the result in `learnings.md`).
  - Implementation approach:
    - `.github/**` is a release-shaping path: the merge needs the user's
      approval. Keep this PR to the four YAML files (and the plan.md update).
    - Everything in the forms is English.
    - Files: `.github/ISSUE_TEMPLATE/{os,camera,software,config}.yml`.

## Trade-offs and risks

- **Discussions threads are placeholders.** README and `config.yml` link
  `https://github.com/minodisk/riffle/discussions` with TODO markers. Follow-up
  (not a step): enable Discussions if needed, create the three "works"
  threads, replace the links. lychee's offline run does not check them.
- **Issue-form validation is only review-based before merge.** A syntax error
  surfaces as the template silently missing from the chooser. After Step 2
  merges, open the chooser once and each form once.
- **Substance drift when moving text.** `docs/usage.md` and
  `docs/performance.md` must be a move, not a rewrite: diff the moved blocks
  against the pre-change README in review. Known stale text in the moved
  Performance section (`Debug > Timing logs`, tracked in `todo.md`) is left
  as is.
- **Dropping the Sigma rows** removes two unchecked cameras from the public
  list; the Compatibility section now invites reports for any camera.

## Progress

- (none yet)
