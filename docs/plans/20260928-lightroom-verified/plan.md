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

# Document the verified Lightroom (non-Classic) XMP round-trip

## Purpose

On 2026-09-28 the user verified, by hand on Windows with Adobe Lightroom
9.5.1 (the cloud-based Lightroom, not Lightroom Classic), that the XMP
sidecars Riffle writes and the ones Lightroom writes round-trip in both
directions:

- Riffle -> Lightroom: XMP sidecars written by Riffle (1–5 stars, pick,
  reject, a red color label whose `xmp:Label` was the Japanese preset
  "レッド") showed correctly in Lightroom on import.
- Lightroom -> Riffle: re-rating a photo in Lightroom (3 stars, pick,
  yellow) made Lightroom rewrite the source folder's `.xmp` by itself, with
  no export step and no `Ctrl+S`; reopening the folder in Riffle showed
  3 stars / pick / yellow. Riffle's sidecar format was "Both" at the time
  and the XMP was newer than the `.dop`, so the XMP was the one read back.
- The `todo.md` TODO (Riffle reading Lightroom-written XMP for picks,
  rejects, the five color labels and 5..1 stars) was checked against a copy
  of those files (`D:\photos\tests\lr-check-1-read-lr-xmp`).

The same day the user also verified in Lightroom Classic (Windows, Japanese
UI) that a Riffle-written reject (`xmpDM:good="False"`, no `xmpDM:pick`)
shows as rejected, a Riffle-written pick as picked, and an unflagged file as
unflagged. Picks and the Japanese color label names
(`crates/core/i18n/ja.json`) had been verified earlier, so Lightroom Classic
is ticked too, and Riffle does not need to write `xmpDM:pick`.

The README's compatibility checklist still shows Lightroom as unverified,
the "Working with other software" section only describes Lightroom Classic
(which needs `Ctrl+S` or auto-write to produce XMP), and `todo.md` still
carries the open round-trip item. This work records the verification so
users know the non-Classic Lightroom works without an export step. No code
changes.

## Steps

- [x] Step 1: Tick Lightroom in the README compatibility list, add a short Lightroom note, mirror it in Japanese, and close the todo item
  - Done when:
    - `README.md` "Compatibility > Sidecar formats and software": the line
      `- [ ] Adobe Lightroom (Windows; not Lightroom Classic)` becomes
      `- [x] ...`. The `Adobe Lightroom Classic (Windows, Japanese UI)` box
      is ticked too (scope extension approved by the user). The Capture One
      box stays unticked.
    - `README.md` "Working with other software" gains a `### Lightroom`
      subsection placed directly before the existing `### Lightroom Classic`
      one (same order as the checklist). It is a short bullet list in the
      style of the Lightroom Classic section, saying roughly: Lightroom (not
      Classic) reads the XMP Riffle wrote when the photos are imported;
      a rating, flag or color label changed in Lightroom is written back to
      the `.xmp` next to the RAW by Lightroom itself, with no export step,
      and Riffle picks it up when the folder is opened; verified with
      Lightroom 9.5.1 on Windows. Keep it modest (name the version; do not
      claim more than was tested). Do not repeat the color-label-set
      instructions; that belongs to the Classic section.
    - `README.ja.md` mirrors both changes in Japanese: the checkbox at
      `- [ ] Adobe Lightroom（Windows、Lightroom Classic ではないもの）`
      and `- [ ] Adobe Lightroom Classic（Windows、日本語 UI）` are ticked,
      and a `### Lightroom` subsection with the same content is
      inserted before `### Lightroom Classic`, in the same paragraph-per-
      bullet style the Japanese file uses (its bullets are single long lines,
      not wrapped).
    - `todo.md`: the item `### App: the Lightroom 9.5.1 round-trip check is
      still open` (its heading, the paragraph that starts "From
      `lightroom-xmp-flags-labels`'s implementation", its `#### TODO` heading
      and the one `- [ ] Open D:\Photos\2026\2026-09-05 ...` checkbox) is
      deleted. The item "App/Core: unverified whether a Riffle-written
      pick/reject shows correctly in Lightroom Classic" (heading, paragraph,
      `#### TODO` and its checkbox) is deleted too (scope extension; Riffle
      does not need to write `xmpDM:pick`). The item "Core: `en.json`'s
      Lightroom label preset is unverified against a real Lightroom install"
      is left exactly as it is. The existing `### Lightroom Classic`
      subsection's text is unchanged.
    - No other file changes. `mise run ci` passes (Markdown formatting and
      lychee).
  - Implementation approach:
    - Match the existing README style: `### Lightroom Classic` is a bullet
      list wrapped at ~80 columns in `README.md`; `README.ja.md` uses one
      unwrapped line per bullet. Product names as in the file ("Lightroom
      Classic", "Lightroom").
    - Contrast with the Classic section explicitly but briefly (e.g. "unlike
      Lightroom Classic, no `Ctrl+S` or auto-write setting is needed"), so a
      reader sees why the two sections differ.
    - Commit as `docs(readme): mark Lightroom as verified and note its XMP
      write-back` (Conventional Commits, English), and remove the todo item
      in the same PR.
    - The "Both" format detail (XMP newer than `.dop` wins) is already
      documented under the sidecar format list; do not restate it in the new
      subsection.

## Trade-offs and risks

- **Subsection placement.** The plan puts `### Lightroom` before
  `### Lightroom Classic`, matching the checklist order and reading from the
  simpler case to the one with caveats. Putting it after Classic is equally
  valid; either way both READMEs must use the same order.
- **How much to claim.** Only Lightroom 9.5.1 on Windows was tested, with
  the Japanese label preset. The note names the version and OS so the tick
  is not read as "every Lightroom version, every platform". If the caller
  prefers a version-free sentence, the checklist's "(Windows; not Lightroom
  Classic)" label still bounds the claim.
- **Color label names in Lightroom (non-Classic).** The red label showed
  correctly with `xmp:Label` = "レッド"; whether Lightroom matches labels by
  `photoshop:LabelColor` or by name was not isolated. The note should not
  state a matching rule for non-Classic Lightroom.

## Progress

- 2026-09-28: Step 1 done — Lightroom and Lightroom Classic ticked in both
  READMEs, `### Lightroom` subsection added, two todo.md items removed,
  including the approved Lightroom Classic scope extension.
