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

# Menu icon template tinting via a patched muda fork

## Purpose

macOS draws a custom `NSMenuItem` image in its own colours unless the image is
marked a template image. muda never calls `setTemplate:` on a custom menu
image, so Riffle's bundled menu PNGs (`Settings...`, `Undo`, `Open Folder…`,
`Open Log Folder`) are exported with a baked-in neutral grey (`#8E8E93`) that
reads differently from every neighbouring item: the OS-provided `Cut` /
`Copy` / `Paste` and the `NativeIcon`-backed items tint with the menu
appearance, ours do not. This is the `todo.md` item
`### App: custom menu-item icons don't tint for dark mode`.

A spike verified in the running app that one line in muda
(`nsimage.setTemplate(true)` in `menuitem_set_icon`) makes every bundled PNG
tint white in dark mode and black in light mode exactly like the OS-provided
items. Two artifacts exist and are **not** to be redone:

- Upstream PR https://github.com/tauri-apps/muda/pull/413 (opt-in
  `IconMenuItem::set_icon_as_template`, against muda `dev`, open).
- Bridge fork `minodisk/muda`, branch `riffle/0.19.3-icon-template`, commit
  `ef4fbfda53416382a2eeca7da683a64a8fe0e8ed`: muda 0.19.3 (the version
  `tauri 2.11.6` resolves for `muda = "^0.19"`) with `setTemplate(true)`
  applied unconditionally. Builds, `cargo fmt --check` passes.

The unconditional fork was chosen deliberately over backporting the opt-in
API: the opt-in route would also need a Tauri fork to wire the flag through,
whereas the unconditional one needs no Tauri change and no
`crates/app/src/main.rs` change. That decision is settled.

Once done, the bundled icons tint with the menu, the grey fill in the export
script is gone, and the fork patch is documented as a temporary bridge with a
clear exit condition.

## Investigation results

- A `[patch.crates-io]` source must be `git` or `path`; a crates.io-to-crates.io
  patch is rejected (`patch for 'muda' points to the same source`). The patched
  version must satisfy `^0.19`, hence 0.19.3, not 0.20.
- `cargo tree -i muda` shows a single dependent path (`tauri 2.11.6`); no other
  crate pulls muda.
- Release workflow (`.github/workflows/release.yml`): `tauri-apps/tauri-action`
  runs a plain `cargo build` through `pnpm tauri`; there is no `--offline`,
  `--locked` or vendor directory, runners have GitHub network access, and the
  cargo cache already includes `~/.cargo/git/db`. The cache key is
  `hashFiles('Cargo.lock')`, so this PR just produces a new cache entry.
- `release-please-config.json` bumps `Cargo.lock` only for packages matching
  `$.package[?(@.source === undefined)]` (the workspace members). A git-sourced
  muda entry carries `source = "git+…"` and is untouched.
- `crates/app/tauri.conf.json` has nothing dependency-related; the bundler does
  not care where a crate came from. `ci.yml` has the same network and cache
  shape as the release workflow.
- Conclusion: the git dependency does not break CI or the release.

## Steps

- [ ] Step 1: Patch muda to the template-image fork, drop the grey fill from the icon export, regenerate the PNGs, update the guide
  - Done when:
    - The workspace `Cargo.toml` has a `[patch.crates-io]` entry pointing
      `muda` at `https://github.com/minodisk/muda` with a comment stating what
      it is (muda 0.19.3 + unconditional `setTemplate(true)` in
      `menuitem_set_icon`), why it exists (custom menu icons do not tint with
      the menu appearance otherwise), and the exit condition (remove once
      muda#413 ships in a muda release Tauri resolves and Tauri exposes the
      template flag for menu items).
    - `Cargo.lock` records muda from the fork; `cargo tree -i muda` prints a
      `git+https://github.com/minodisk/muda…` source, not the registry.
    - `tools/macos/export-menu-icons.swift` no longer defines the `#8E8E93`
      `color` or performs the `.sourceAtop` fill; the header comment explains
      that the PNGs are alpha-only because muda (via the fork patch) marks them
      as template images. Canvas 18pt and glyph `pointSize` 12 are unchanged.
    - The three PNGs under `crates/app/icons/menu/` are regenerated with the
      script and committed; sampling a non-transparent pixel shows no baked
      colour (alpha-only glyph).
    - `docs/agents/tauri-app.md` "Menu icons" section describes the fork patch,
      why it exists and when it goes away, instead of the grey compromise;
      the one-line-fix note and the pointer to muda#413 carrying it opt-in
      stay.
    - `mise run ci` passes.
    - The user confirms visually in `mise run tauri:dev` that the four
      PNG-backed items tint white in dark mode and black in light mode.
    - The PR body states explicitly that this adds a git dependency (third-party
      code now fetched from `github.com/minodisk/muda` at build time), and the
      Cargo comment says the same.
  - Implementation approach:
    - `Cargo.toml` (workspace root): add

      ```toml
      [patch.crates-io]
      muda = { git = "https://github.com/minodisk/muda", rev = "ef4fbfda53416382a2eeca7da683a64a8fe0e8ed" }
      ```

      with the explanatory comment above it. Use `rev`, not `branch` (see
      Trade-offs). Run `cargo update -p muda` (or a plain `cargo build`) so
      `Cargo.lock` picks up the git source; commit the lockfile change. No
      change to `crates/app/Cargo.toml` or `crates/app/src/main.rs`.
    - `tools/macos/export-menu-icons.swift`: delete the `let color = …` line
      and the `color.set()` / `rect.fill(using: .sourceAtop)` pair; leave the
      drawing, centring, `canvasSize`, `pointSize`, `scale` and `symbols`
      untouched. Rewrite the last header paragraph (currently "The symbols are
      drawn in a fixed neutral grey because …") to say the glyphs are drawn
      alpha-only because the muda fork marks the image as a template, so only
      the alpha channel is used. Also drop "or colour" from the "rerun this
      only when a symbol, its size, weight or colour changes" sentence and
      update the "Last run on macOS …" line to the OS the regeneration runs
      on (`sw_vers`).
    - Regenerate: `swift tools/macos/export-menu-icons.swift`. Verify the
      bounding box did not move (only colour should differ) and that a
      non-transparent pixel has no colour: e.g. a throwaway Swift snippet
      reading `NSBitmapImageRep.colorAt(x:y:)` on an opaque pixel. Do not
      commit the check script.
    - `docs/agents/tauri-app.md`, section "Menu icons: native where one exists,
      a bundled SF Symbol otherwise": replace the "Limitation" and "The fix is
      a one-line gap" bullets with the new reality: muda is patched via
      `[patch.crates-io]` to `minodisk/muda` (0.19.3 + unconditional
      `setTemplate(true)`); why (`crates.io` patches rejected, `^0.19` bound,
      the opt-in upstream route would need a Tauri fork too); the exit
      condition; that muda#413 carries the change opt-in upstream. Keep the
      "only when a symbol, its size or weight changes" regeneration note
      (drop "colour") and the existing "Hit" bullet on glyph size. Keep the
      link to the archived learnings heading so `lychee --include-fragments`
      still resolves.
    - Verification: `mise run ci`; then check port 1420 is free
      (`lsof -nP -iTCP:1420 -sTCP:LISTEN`; a stale dev server from an earlier
      worktree is a known trap in this repo, kill it first), run
      `mise run tauri:dev`, and ask the user to confirm the tint in both
      appearances. The agent cannot judge this visually.
    - Do not touch `todo.md` in this step; see Trade-offs for the wrap-up.

## Trade-offs and risks

- **`rev` vs `branch` in the patch.** Settled: `rev =
  "ef4fbfda53416382a2eeca7da683a64a8fe0e8ed"`. The fork is a bridge whose
  exit is deletion, not update; nothing else will push fixes to it, so the
  only thing a `branch` buys is exposure to the branch moving, being
  force-pushed or deleted under CI, and `cargo update` silently moving the
  pin. `Cargo.lock` pins the commit in both cases, so the practical
  difference is only under `cargo update`.
- **New git dependency.** Third-party code is now fetched from a personal
  GitHub fork (`github.com/minodisk/muda`) at build time on every developer
  machine and CI/release runner, instead of only from crates.io. The rev pin
  makes the content reproducible, but a human should see this in the PR body.
  The fork must stay reachable for as long as the patch exists; deleting the
  fork repo would break every build.
- **Exit condition has two halves.** muda#413 landing in a muda release is not
  enough on its own: Tauri must resolve that release under its `^0.19` (or a
  later Tauri must bump) *and* expose the template flag on menu items. If muda
  releases but Tauri never wires it, the fork stays (rebased onto the newer
  muda base) rather than going away. The rewritten todo item tracks this.
- **`todo.md` wrap-up.** `### App: custom menu-item icons don't tint for dark
  mode` is only partly closed by this step. The wrap-up's todo curation
  should rewrite it to the remainder ("muda#413 lands upstream and Tauri
  exposes the flag, then drop the `[patch.crates-io]` entry and any
  now-unneeded workaround; verify the icons still tint") rather than delete
  it, because the patch is explicitly temporary and something must track its
  removal.
- **Visual acceptance is manual.** The agent cannot see the menu; the user
  confirms in the running app. Port 1420 must be free before
  `mise run tauri:dev`.
- **Doc link check.** `mise run lint` runs `lychee --include-fragments`; keep
  the existing fragment link to
  `docs/plans/_archived/20260920-menu-icon-glyph-size/learnings.md` intact
  or remove it cleanly.

## Progress
