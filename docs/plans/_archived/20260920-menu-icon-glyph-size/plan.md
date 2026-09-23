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

# Menu icon glyph size

## Purpose

On macOS the three PNG-backed menu items (`Settings...`, `Undo`, `Open Log
Folder`) look visibly bigger than the two `NativeIcon` items (`Open in DxO
PhotoLab`, `Check for Updates…`). The five icons should read as the same size.

The explanation committed during the dependency refresh (PR #229) — that muda's
`to_nsimage(Some(18.))` stretches our PNGs to 18pt while natives stay at
~14–16pt — is wrong and must be corrected. Measured on macOS 26.6 (alpha
bounding box of the drawn glyph):

| image | canvas | glyph ink |
|---|---|---|
| `NSFollowLinkFreestandingTemplate` | 19pt | 16.0 x 16.0 |
| `NSRefreshTemplate` | 20pt | 13.5 x 16.5 |
| `gearshape.png` | 18pt | 18.0 x 18.0 |
| `arrow.uturn.backward.png` | 18pt | 17.0 x 18.0 |
| `folder.png` | 18pt | 18.0 x 17.5 |

The canvases are comparable (ours is the smaller one). The difference is the
content: the native template images carry internal padding, so their glyph
covers ~16pt of a 19–20pt canvas, while `tools/macos/export-menu-icons.swift`
draws the SF Symbol at `pointSize: 18` into an 18pt canvas. Worse, the
configured symbol's reported `size` at `pointSize: 18` is 24x24 (gearshape),
21x21 (arrow), 25x21 (folder), so the centered draw rect overflows the canvas
and the committed PNGs are clipped at the edges (folder ink is exactly
edge-to-edge). muda does still resize every `Image`-backed item to an 18pt
height, so the PNG's pixel size is irrelevant to the rendered size — but that
resize is not what makes the icons look bigger.

The fix is entirely in the export script and its outputs: keep the 18pt
canvas, draw the glyph smaller so transparent padding absorbs the difference
and nothing clips. `crates/app/src/main.rs` does not change.

## Steps

- [x] Step 1: Shrink the glyph inside the 18pt canvas, regenerate the PNGs, and correct the root-cause text
  - Done when:
    - `tools/macos/export-menu-icons.swift` separates the canvas size (stays
      18pt, 36x36 px @2x) from the glyph `pointSize`, which becomes **14**
      (shared across all three symbols — see Decisions), and its header comment
      explains why the glyph is drawn smaller than the canvas (native template
      images pad their glyph to ~16pt; the previous 18pt draw also overflowed
      and clipped) and carries an accurate "Last run on macOS …" line
      (`sw_vers`).
    - The three regenerated PNGs under `crates/app/icons/menu/` are committed,
      and a throwaway alpha-bounding-box measurement (scratchpad, not
      committed) shows **no glyph touches the canvas edge** and the ink lands
      near the expected values below, recorded in `learnings.md` as a table
      like the one in Purpose.
    - `docs/agents/tauri-app.md`: the `**Hit**` bullet under
      `### Menu icons: native where one exists, a bundled SF Symbol otherwise (Hit)`
      no longer attributes the mismatch to muda's 18pt hardcode. It states the
      measured cause (native templates pad to ~16pt ink; our PNGs drew the
      glyph at `pointSize: 18` and clipped), notes muda's 18pt resize as a
      fact that makes the PNG pixel size irrelevant but is not the cause, and
      describes the fix as done (glyph drawn smaller inside the same canvas,
      measured with an alpha bounding box), not outstanding.
    - `todo.md`: the item
      `### App: PNG-backed menu icons render larger than native ones` (and its
      `#### TODO` checkbox) is handed to the wrap-up's todo curation for
      **deletion**, not rewording — this PR closes it. The neighboring items
      `### App: custom menu-item icons don't tint for dark mode` and
      `### App: `Open Folder…` has no macOS menu icon` are untouched.
    - `mise run ci` passes.
    - The app is relaunched with `mise run tauri:dev` (after confirming port
      1420 is free: `lsof -nP -iTCP:1420 -sTCP:LISTEN`, and killing any stale
      dev server first) and the user confirms from a screenshot that the five
      menu icons read as the same size.
  - Implementation approach:
    - Only these files change: `tools/macos/export-menu-icons.swift`,
      `crates/app/icons/menu/{gearshape,arrow.uturn.backward,folder}.png`,
      `docs/agents/tauri-app.md`, plus the plan files. **Amended mid-step:**
      `crates/app/src/main.rs` also changes, to give `Open Folder…` the
      `folder.png` icon (see Decisions).
    - Script restructuring: introduce a `canvasSize: CGFloat = 18` used for
      `pixels` and `rep.size`, and a separate glyph `pointSize` used only in
      `NSImage.SymbolConfiguration(pointSize:weight:)`. Keep the current
      approach of centering the configured symbol's reported `size` rect in
      the canvas, keep `weight: .medium`, the `#8E8E93` fill and the
      `.sourceAtop` tint. Keep it a few lines; no options or CLI arguments.
    - Expected ink at the chosen `pointSize: 14` (measured on this machine
      before implementation; confirm against the regenerated PNGs):

      | symbol | expected ink |
      |---|---|
      | gearshape | 15.0 x 15.0 |
      | arrow.uturn.backward | 14.5 x 13.0 |
      | folder | 17.0 x 13.0 |

      Do not pick the size from the symbol's reported `size`, which is larger
      than the ink (19x19 at `pointSize: 16` for gearshape). If the
      regenerated PNGs measure materially differently, re-measure and record
      the real numbers rather than restating these.
    - Measurement: a throwaway swift script in the scratchpad that loads each
      committed PNG with `NSBitmapImageRep(data:)` and reports the bounding
      box of pixels with alpha > ~0.05, divided by the 2x scale. It must also
      report **edge contact** (ink touching pixel 0 or 35) explicitly, not
      just the size — clipping is what went unnoticed last time. Run it on the
      regenerated PNGs (not just on an in-memory render). Do not commit it
      unless there is a clear reason.
    - Verify the header comment's "Last run on macOS …" against `sw_vers`
      on the machine that regenerates the PNGs.
    - The doc bullet currently cites
      `docs/plans/_archived/20260920-dependency-refresh/learnings.md` for the
      observation; keep that pointer as history if useful, but the bullet's
      own text must carry the corrected cause. Do not edit the archived
      learnings file.
    - Verification order: measurement first, then `mise run ci`, then the
      `tauri:dev` visual pass with the port-1420 check. The agent cannot judge
      the final look; end the step by asking the user to eyeball the menu and
      report back. If the user finds the PNG glyphs now too small (or still
      too large), adjust `pointSize` and repeat from the measurement.

## Decisions (settled with the user before implementation)

- **One shared glyph `pointSize` of 14**, not per-symbol sizes. SF Symbols are
  designed to be optically balanced with each other at the same `pointSize`,
  so a shared value is the intended usage and keeps the script simple. 14 is
  also the largest shared value that does not clip `folder` (which is
  inherently wide, ~1.3:1). The cost is that gearshape lands at 15pt ink
  rather than matching the native 16.0 exactly; the user accepted that in
  exchange for the simpler, optically-balanced approach. Revisit only if the
  visual pass says a specific glyph reads off.
- **Mid-step addition: `Open Folder…` gets the folder icon in this PR.** The
  user asked to fold the separate `todo.md` item
  (`### App: \`Open Folder…\` has no macOS menu icon`) into this PR instead of
  handling it on its own, and chose to reuse the existing `folder.png` for
  both `Open Folder…` and `Open Log Folder` rather than exporting a second,
  differentiated symbol: the two items live in different menus, which are
  never open at the same time. The `todo.md` item is deleted in this step's
  commit.
- **The archived learnings keep the wrong explanation.**
  `docs/plans/_archived/20260920-dependency-refresh/learnings.md` records the
  muda-18pt cause. The user chose to leave it as a historical record of what
  was known at the time; the current, correct description lives in
  `docs/agents/tauri-app.md`. Do not edit the archived file.

## Trade-offs and risks

- **Clipping went unnoticed before.** The measurement script must check
  edge contact (ink touching pixel 0 or 35) explicitly, not just size,
  otherwise the same defect can recur in a different shape.
- **`todo.md` handling.** The item is deleted via the wrap-up's todo curation
  rather than hand-edited in the implementation commit. In single-PR mode the
  wrap-up lands in the same PR, so the item disappears in this PR either way;
  the implementer should let `todo-curator` do it.
- **Rasterization drift.** Ink sizes were measured on macOS 26.6; a
  different OS release may rasterize differently, which is why the committed
  PNGs (not the script) are the source of truth. Regenerate and re-measure on
  the same machine in one go.
- **The visual pass depends on the user.** The agent can prove the ink
  measurements and the absence of clipping, but not that the result looks
  right. The step is not done until the user has looked.

## Progress

- (2026-09-20) Step 1 complete: shrank the glyph `pointSize` inside the
  unchanged 18pt canvas in `tools/macos/export-menu-icons.swift`, regenerated
  the three PNGs, and corrected the root-cause explanation in
  `docs/agents/tauri-app.md`. The first attempt at `pointSize: 14` still read
  too large against the OS-provided menu items, so the user chose 12, which
  they then confirmed visually — `Open Folder…`'s newly assigned `folder.png`
  included. A mid-step scope addition gave `Open Folder…` that icon, closing
  its own todo item. A side spike (reverted, not committed) additionally
  proved that a one-line `setTemplate(true)` in muda fixes the dark-mode
  tinting; that is recorded in `learnings.md` and folded into the existing
  todo item rather than done here.
