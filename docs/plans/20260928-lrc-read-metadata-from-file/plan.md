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

# Lightroom Classic: Read Metadata from File as the per-photo re-read

## Purpose

The README's `### Lightroom Classic` subsection (under "Working with other
software") tells the user that Lightroom Classic does not re-read a sidecar
Riffle changed after import, and gives `Synchronize Folder...` with
`Scan for metadata updates` as the way to pick it up. That is verified and
stays, but it re-scans a whole folder. On 2026-09-28 the user verified
(Lightroom Classic on Windows, Japanese UI; three Sony ARWs imported, then
Riffle wrote XMP with stars and a red label) that:

- with no action Lightroom Classic picked up nothing and showed no
  metadata-conflict badge;
- `Metadata > Read Metadata from File` (right-click the selected photos in
  the Library grid) picked up the stars and red label of the selected photo;
- `Synchronize Folder...` with `Scan for metadata updates` picked up the
  stars and labels of every photo in the folder.

After this work both READMEs offer the per-photo `Read Metadata from File`
alternative next to the whole-folder `Synchronize Folder...` instruction, so
a user who changed a few files in Riffle does not have to re-scan the folder.

## Steps

- [x] Step 1: Add `Metadata > Read Metadata from File` to the Lightroom Classic section of both READMEs
  - Done when:
    - `README.md` `### Lightroom Classic` keeps the existing
      `Synchronize Folder...` / `Scan for metadata updates` / `Synchronize`
      bullet text and, in the same bullet, also says that for the selected
      photos one can right-click them in the Library grid and choose
      `Metadata > Read Metadata from File`, named first, with the folder-wide
      `Synchronize Folder...` route presented second as the way to pick up a
      whole folder. Menu names stay in backticks as in the surrounding
      bullets.
    - `README.ja.md` `### Lightroom Classic` carries the same content in
      Japanese, in the same one-bullet, English-menu-name style as its
      neighbors (e.g. 「選択中の写真だけなら、ライブラリのグリッドで写真を右クリックして `Metadata > Read Metadata from File` を選びます。フォルダー全体なら、フォルダーを右クリックして `Synchronize Folder...` を選び、`Scan for metadata updates` にチェックを入れて `Synchronize` をクリックしてください。」).
    - No other file changes except this plan's checkbox and Progress entry
      (`docs/usage.md` and the UI text do not describe the procedure;
      archived plans and review history are historical records and stay).
    - `mise run ci` passes (Markdown formatting and lychee).
  - Implementation approach:
    - Edit only the one "After import it does not re-read a sidecar" bullet
      in each README; leave the `Ctrl+S` / auto-write bullet and the color
      label bullet untouched.
    - Keep the wrapped-at-~80-columns style of `README.md` and the
      one-long-line style of `README.ja.md`.
    - Do not claim a version number for this verification; the section's
      existing bullets do not, and the checklist line already says
      "Windows, Japanese UI".
    - Record in `learnings.md` the three verification facts above (including
      that no metadata-conflict badge appeared) so the next plan touching
      this section does not re-verify them.

## Progress

- (2026-09-28) Step 1 complete
