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

# Delist pre-2010 and tiny-preview bodies, drop the `OLYMP\0` compacts todo

## Purpose

User decision (2026-09-30 / 10-01): a camera body released before 2010, or
whose RAW carries no embedded JPEG or only one too small to cull with, is
not a supported body, compact or not. **Delisting is a docs change only**:
files that the parsers already open keep opening (the E-300 and its
siblings fixed by PR #618 included); no code path is removed or disabled.
Bodies from 2010 on with a proper preview stay listed, fixed-lens ones
included (Fujifilm X100V, X100VI, GFX100RF).

Two things follow:

- The old Olympus compacts with an `OLYMP\0` MakerNote and no CameraSettings
  (C5050Z, C5060WZ, C7070WZ, C8080WZ, E-10, E-20, the SP-series) carry only
  a 160x120 thumbnail and predate 2010, so the `todo.md` item PR #618 filed
  to open them anyway is dropped. None of them is listed.
- A sweep of every listed body against the local samples (the JPEG
  `reader::read_preview` / `read_full` hand back, measured 2026-09-30)
  found every listed ARW, CR3, DNG, NEF and ORF body at a preview of at
  least 1616x1080 and a full JPEG of at least 3200x2400, all from 2016 on
  or later (the oldest being the D500 and PEN-F). The listed bodies to
  remove are old Fujifilm ones listed only in `docs/cameras.md`:
  - FinePix S2 Pro, S3 Pro, S5 Pro (1440x960), DBP for GX680 (1344x960),
    FinePix S5000, S5500, S7000 (1280x960);
  - FinePix S100FS, S9500, S9600, S5200, S6000fd, S6500fd (1600x1200);
  - FinePix S200EXR (2048x1536, released 2009).

## Steps

- [x] Step 1: Delete the `OLYMP\0` compacts item from `todo.md` and delist
      the pre-2010 bodies from `docs/cameras.md`
  - Done when:
    - The whole section
      `### Core: the old \`OLYMP\0\` Olympus compacts still open with "no embedded preview"`
      (its `#### Background` and `#### TODO`) is deleted from `todo.md`,
      together with the blank line separating it from the previous section.
      Nothing else in `todo.md` changes.
    - The release year of every body listed in `README.md`,
      `README.ja.md` and `docs/cameras.md` is checked (announcement /
      release year from the maker or a reliable spec site); any body before
      2010 is delisted. At planning time that is exactly the 14 Fujifilm
      bodies above; if the check finds another, it is delisted too and
      recorded in `learnings.md`.
    - In `docs/cameras.md`, the rows of the delisted bodies are gone from
      the feature table and from the "Embedded JPEG size" table (a size row
      whose every body is delisted goes entirely), and the prose that names
      them is adjusted: the "manual focus" sentence (drop the delisted
      bodies it names) and the sentence "The FinePix S2 Pro, S5 Pro and DBP
      for GX680 write no `FocusPixel` at all, so they have no AF point."
      (delete it). Sentences that stay true are left alone.
    - (Added during implementation at the user's request.)
      `docs/cameras.md` states the support rule in a short paragraph where
      it introduces the camera list: a body released before 2010, or whose
      embedded JPEG is under 1280 px on the long edge, is not supported,
      because the preview Riffle culls from cannot be good enough; its files
      may still open (Riffle does not block them), but it is not listed.
      Every body left listed is checked against the rule (none has a preview
      under 1280 px on the long edge).
    - `README.md` and `README.ja.md`: the RAF paragraph ("Older X-series
      and FinePix bodies (... the FinePix S and HS lines, and others) also
      open ... at most 2176x1448") is re-read and changed only if it is no
      longer true ("or smaller" no longer was, so it now says "1920x1280 to
      2176x1448") (the S1, 2014, keeps the S line listed). Any change to
      `README.md` is mirrored in `README.ja.md`.
    - No file under `crates/` changes; `docs/agents/**`,
      `docs/raw-formats.md` and `docs/usage.md` are left as they are (they
      describe what the parsers handle, not what is supported).
    - `mise run ci` passes (lychee, formatting).
  - Implementation approach:
    - Docs only. Record the delisted bodies with their measured preview
      size and release year in `learnings.md`, so the record survives the
      delisting.

## Trade-offs and risks

- A user of a delisted body still gets a working app (the files open as
  before) with no statement that the body is unsupported. Accepted: the
  user wants existing behavior kept and only the claim dropped.
- The `todo.md` deletion loses the list of the 12 failing ORF sample files;
  the archived `orf-old-maker-note-preview` plan and its learnings still
  hold it.

## Progress

- (none yet)
