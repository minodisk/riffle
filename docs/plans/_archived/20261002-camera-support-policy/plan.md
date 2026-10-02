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

# Camera support policy: rolling ten-year cutoff

## Purpose

`docs/humans/cameras.md` states the support rule as a fixed year ("A body
released before 2010, or whose embedded JPEG is under 1280 px on the long
edge, is not supported"). The user's decision (2026-10-02) replaces it with
three cases:

1. A RAW with no embedded JPEG preview is outside Riffle's purpose (Riffle
   culls from that preview) and is not supported at all: `no embedded
   preview` is the correct outcome, with no demosaicing and no rendering of
   tiny uncompressed thumbnails.
2. A body released ten or more years ago, counted from today (a rolling
   cutoff: in 2026, released before 2016), or whose largest embedded JPEG is
   under 1280 px on the long edge, is not actively supported: it is not
   listed and gets no new work.
3. Files from such a body that already open keep opening. Riffle does not
   block them and no code path or test is removed.

The user confirmed (2026-10-02) that bodies released in the cutoff year itself
stay: in 2026 the cutoff is "released before 2016", so 2016 bodies remain
listed.

After this work the policy text says all three, the verified-camera table
lists no body released before 2016, `todo.md` carries no item that exists
only for a body now outside the lists, and the rule stays maintainable
without tooling: the prose names the rule and the cutoff year it currently
yields, and the yearly re-check is written down as a one-line procedure.

Investigation at planning time (2026-10-02):

- The support rule appears only in `docs/humans/cameras.md` (lines 9-13).
  `README.md` / `README.ja.md` "Compatibility" / "対応状況" list only the six
  formats and link to `cameras.md`; they state no rule and no bodies. No
  per-body support list exists in `docs/humans/raw-formats.md`,
  `docs/agents/**`, `CLAUDE.md` or `.github/ISSUE_TEMPLATE/camera.yml`.
- Release years of the listed bodies (to be re-verified by the implementer,
  see Step 1). Every Sony (α1 2021 ... ZV-E1 2023), SIGMA (BF 2025, fp L
  2021), Leica (M11-P 2023), Canon (EOS R 2018 ... R6 Mark III 2025), Nikon
  (Z 6 2018, D850 2017, D500 2016, the rest newer) and OM System / Olympus
  (E-M1 Mark II Dec 2016, PEN-F Feb 2016, the rest newer) body is from 2016
  or later and stays. The Fujifilm bodies released before 2016, to remove:

  | Body | Released |
  |---|---|
  | X-T1 | 2014 |
  | X-T10 | 2015 |
  | X-Pro1 | 2012 |
  | X-E2 | 2013 |
  | X-E1 | 2012 |
  | X-A2 | 2015 |
  | X-A1 | 2013 |
  | X-M1 | 2013 |
  | X100T | 2014 |
  | X100S | 2013 |
  | X30 | 2014 |
  | X20 | 2013 |
  | X10 | 2011 |
  | XF1 | 2012 |
  | XQ2 | 2015 |
  | XQ1 | 2013 |
  | X-S1 | 2011 |
  | FinePix X100 | 2011 |
  | FinePix F770EXR | 2012 |
  | FinePix F550EXR | 2011 |
  | FinePix HS50EXR | 2013 |
  | FinePix HS33EXR | 2012 |
  | FinePix HS30EXR | 2012 |
  | FinePix SL1000 | 2013 |
  | FinePix S1 | 2014 |

  The 2016 edge cases that stay: Nikon D500 (Jan 2016), Olympus PEN-F (Jan
  2016), Fujifilm X-Pro2, X-E2S, X70 (Jan 2016), X-T2 (Jul 2016), X-A3
  (Aug 2016), X-A10 (Dec 2016 / Jan 2017).
- `todo.md` sections whose only subject is a body now outside the lists:
  `### Core: the FinePix SL1000 shows Exif \`Make\` and \`Model\` with
  trailing spaces` (SL1000, 2013) and `### Core: pre-2012 NEFs fall back to
  the full-size JPEG for the preview` (the D7000, 2010; the item exists to
  give 2010-2012 NEFs a smaller preview). Items that name an old body only
  as a test input for an unrelated behavior stay: the malformed-JPEG 1:1
  check (`NIKON_D70_Nikon.nef` is a robustness fixture), the ARW
  160x120-thumbnail item (α9 II, α7R IV, α7C, α6400, α6600, ZV-E10 are all
  2019-2021), and "widen camera support" (categories, not bodies). Re-check
  `todo.md` at implementation time, since other sessions edit it.

## Steps

- [x] Step 1: Rewrite the support policy, delist the pre-2016 bodies and drop their `todo.md` items
  - Done when:
    - `docs/humans/cameras.md` lines 9-13 are replaced by a policy paragraph
      that states all three cases of the Purpose: (a) a RAW with no embedded
      JPEG preview is outside Riffle's purpose and not supported (`no
      embedded preview` is the intended outcome; Riffle does not decode
      sensor data or render tiny uncompressed thumbnails); (b) a body
      released ten or more years ago, counted from the current year, or
      whose largest embedded JPEG is under 1280 px on the long edge, is not
      actively supported: not listed here, no new work; (c) files from such
      a body that already open keep opening, since Riffle does not block
      them. The rolling rule is stated in prose together with the cutoff it
      yields now, in a form such as "released before 2016, as of 2026", so a
      reader sees both the rule and the current year's number. One sentence
      records the maintenance procedure: once a year, re-read the table
      against the new cutoff and delist what fell below it (a docs-only
      change). No tooling, script or CI check is added.
    - The 25 Fujifilm rows listed in the Purpose are removed from the table
      in `docs/humans/cameras.md`. Before removing, the implementer verifies
      each listed body's release year (maker announcement or a reliable spec
      site; record the source and year per removed body in `learnings.md`),
      and also confirms the 2016 edge cases stay. If the check finds another
      listed body released before 2016, it is delisted too and noted in
      `learnings.md`; if one of the 25 turns out to be 2016 or later, it
      stays.
    - The prose of `docs/humans/cameras.md` that refers to removed bodies is
      updated: the manual-focus sentence under the Fujifilm paragraph drops
      the X-A2; "none of the older X-series and FinePix bodies records it"
      (sub-second) is reworded to the bodies that remain (the 1920x1280
      X bodies); the paragraph "The older X-series and FinePix bodies also
      open, but their embedded JPEG is 1920x1280 to 2176x1448" is reworded to
      the remaining older X bodies at 1920x1280; in the Fujifilm embedded
      JPEG size table the `2176x1448` and `2048x1536` rows (now empty) are
      deleted and the `1920x1280` row keeps only X-H1, X-T2, X-T20, X-T200,
      X-T100, X-Pro2, X-E3, X-E2S, X-A7, X-A5, X-A3, X-A10, X100F, X70, XF10.
    - `docs/humans/raw-formats.md` no longer describes sizes only the
      removed bodies had: the Preview JPEG cell of the format table ("1920x1280
      or 2048x1536 on older X bodies and at most 2176x1448 on the FinePix
      bodies") and the RAF section ("2048x1536 on the small-sensor compacts
      ... FinePix bodies at most 2176x1448") are reduced to "1920x1280 on the
      older X bodies", still pointing at `cameras.md` for the per-body size.
    - `todo.md`: the two sections named in the Purpose (`### Core: the
      FinePix SL1000 ...` and `### Core: pre-2012 NEFs ...`) are deleted
      whole (heading, `#### Background` / body, `#### TODO`, and the blank
      line separating each from the previous section). Nothing else in
      `todo.md` changes. Keep the file's line endings when rewriting
      (the earlier `todo-unsupported-bodies` plan hit a whole-file diff
      from a newline change).
    - `README.md` and `README.ja.md` are not changed (no body list or rule
      there), so they stay in sync.
    - No file under `crates/` changes; no test or fixture is removed.
    - `docs/agents/raw-metadata-parsing.md` is left as is: its RAF bullet
      documents what the parser does with the older bodies' files, which
      remain openable, so it is still true.
    - `mise run ci` passes (lychee: links in the edited files still
      resolve).
  - Implementation approach:
    - Keep the sentence structure of the existing paragraph ("Riffle culls
      from that preview ...") where it still applies; the change is surgical.
    - Verification of release years is done by the implementer with public
      sources; the samples under `D:\Photos\samples` are not reorganized and
      not needed for this step.
    - Another session may be translating `docs/humans/` into Japanese
      (`docs/plans/20261002-humans-docs-ja/`). If a Japanese copy of
      `cameras.md` / `raw-formats.md` exists on `origin/main` at
      implementation time, apply the same change to it.
    - Suggested commit: `docs(cameras): state the rolling ten-year support
      cutoff and delist pre-2016 bodies`.

## Trade-offs and risks

- **Inclusive cutoff year (decided).** Bodies of the cutoff year stay
  (in 2026: released before 2016 are delisted), per the user's decision.
- **README.md / README.ja.md left untouched.** They link to `cameras.md`
  and state no rule; adding a sentence would create a second bilingual copy.
- **Rolling rule without tooling.** The cutoff year written in prose goes
  stale each January; the mitigation is the recorded yearly re-check and
  the "as of 2026" wording, which signals the staleness rather than hiding
  it.
- **Release-year accuracy.** The years above are from planning knowledge;
  the implementer re-verifies each before removing, which is why the step
  requires the source per body in `learnings.md`. Announcement and shipping
  dates can straddle a year end (X-A10: announced Dec 2016, shipped Jan
  2017); use the announcement year, and note the choice in `learnings.md`.
- **`docs/agents/raw-metadata-parsing.md` keeps the older-body sizes.** It
  is engineering knowledge about files that still open, not a support list.

## Progress

- 2026-10-02: Step 1 done. Delisted the 25 pre-2016 bodies from `docs/humans/cameras.md`, removed the two `todo.md` sections, left README untouched.
