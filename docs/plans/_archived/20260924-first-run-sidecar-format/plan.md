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

# First-launch sidecar format dialog

## Purpose

`sidecarFormat` defaults to XMP and is only written to the settings store when
the user changes it in `Riffle > Settings...`. A PhotoLab user who never opens
the settings culls into `.xmp` files PhotoLab does not read, and switching
later writes only the judgments still unwritten in `.dop` (see `docs/usage.md`,
"Ratings and sidecars"), so that culling is wasted. Riffle should ask which
developing software the user runs before the first folder can be opened, and
save the answer under the same `sidecarFormat` key the settings window uses.

Converting sidecars already written in the other format is out of scope.

## Investigation results

- `sidecarFormat` is written only by the `persist` closure inside
  `switch_format` (`crates/app/src/commands.rs`), which returns early when the
  requested format equals the current one, so the key exists only after the
  user actually changed the format. `load_settings` reads it with `store.get`
  and never writes it. A fresh install and an existing user who always kept
  the XMP default are therefore indistinguishable (no key in either case).
- Consequence: the dialog cannot reuse `set_sidecar_format` as-is; choosing
  XMP on a fresh store would hit the early return and save nothing.
- Every folder-open path in `crates/app/ui/src/main.ts` converges on
  `openDirectory`: `reopenLastFolder()` (startup restore of `lastFolder`,
  chained after the `sort_order` invoke), `openFolder()` (the `open-folder`
  menu event, the `open` key, the `#open` button, the empty-state click), and
  the `tauri://drag-drop` handler. The `sidecar-format` and `index-cleared`
  events reopen only when a folder is already open. There is no CLI-args path.

## Decisions (agreed with the user)

- Show the dialog whenever `sidecarFormat` is not saved, to new and existing
  users alike. Users who ever saved a format (either value) never see it.
- The dialog includes one line saying the choice can be changed later "in
  Settings" (not `Riffle > Settings...`, since the menu is
  `File > Settings...` on Windows and Linux).
- Lightroom comes first wherever Lightroom and PhotoLab appear together
  (it has more users): the dialog's buttons and every doc passage touched.
- The Lightroom (XMP) button is focused when the dialog opens, so Enter picks
  Lightroom.

## Steps

- [x] Step 1: Ask for the developing software on launch while `sidecarFormat` is unsaved, gate every folder open on the answer, and update the docs
  - Done when:
    - With a fresh settings store (no `sidecarFormat` key), the main window
      shows a modal dialog offering, in this order, **Lightroom (XMP)** and
      **DxO PhotoLab (.dop)**, with a line saying the choice can be changed
      later in Settings; it cannot be dismissed without choosing. The
      Lightroom (XMP) button has focus when the dialog opens.
    - Until a choice is made, no folder opens: the `lastFolder` restore at
      startup, `File > Open Folder…`, the `open` key, the `Open folder`
      button, the empty-state click and drag-and-drop all do nothing (the
      drop shows no error either), and the deferred startup restore runs once
      the choice is made.
    - The choice is written to `sidecarFormat` in `settings.json` **even when
      it is XMP** (the current default), and the settings window's radio
      shows it; on the next launch the dialog does not appear.
    - With a saved `sidecarFormat` (either value) the dialog never appears
      and folder opening behaves as today.
    - Rust unit tests cover the "saved or not" decision and the
      unconditional persist; vitest tests cover the frontend gate logic.
    - `README.md` and `docs/usage.md` are updated (see approach).
    - `mise run ci` passes.
  - Implementation approach (as far as it is known):
    - Backend (`crates/app/src/commands.rs`, `crates/app/src/main.rs`):
      - Add a sync command that reports whether the store has a
        `sidecarFormat` key (`store.has("sidecarFormat")`, as `load_settings`
        already does for `lastFolder`). A store that cannot be opened should
        count as "saved" so a broken config dir does not lock the app behind
        a dialog whose answer cannot be persisted; log it like `load_settings`
        does. Either expose it as its own command next to `sidecar_format`
        or extend `load_settings` to return the `Option` and keep it in
        managed state; prefer whichever is smaller.
      - Add a `choose_sidecar_format` command (async, `spawn_blocking`, like
        `set_sidecar_format` in `main.rs`) that runs `switch_sidecar_format`
        and then persists `sidecarFormat` unconditionally, because
        `switch_format` returns early (without calling `persist`) when the
        format is unchanged, which is exactly the XMP-on-fresh-store case.
        Emit `sidecar-format` as `set_sidecar_format` does when the value
        changed, so an open settings window follows. Do not change
        `switch_format`'s early return: its tests and the settings window
        rely on the no-op semantics.
      - Register the new commands in `generate_handler!`.
    - Frontend (`crates/app/ui/index.html`, `crates/app/ui/style.css`,
      `crates/app/ui/src/main.ts`, a new small module such as
      `crates/app/ui/src/firstrun.ts` with `firstrun.test.ts`):
      - Add a modal overlay to `index.html` (a full-window `<div>` is the safe
        choice for the WebView targets in `vite.config.ts`) with two buttons,
        Lightroom first, and the "can be changed later in Settings" line.
        Style it in `style.css` next to `#empty` / `#context-menu`. No close
        button, no Escape dismissal. Focus the Lightroom button when shown,
        and keep app keyboard shortcuts from acting while the dialog is up.
      - Gate in `main.ts`: a single `formatChosen` flag (or a `Promise`)
        checked at the top of `openDirectory`, in `openFolder` (so the native
        picker is not even shown) and in the drag-drop handler (so the drop is
        silently ignored). Do not run `reopenLastFolder` until the flag is
        set: invoke the "is it saved" command before the `sort_order` →
        `reopenLastFolder` chain and either continue at once (saved) or after
        the dialog's button click (unsaved). The extracted pure module holds
        the decision logic so vitest can cover it as `empty.test.ts` covers
        `emptyState`.
      - On a button click: invoke `choose_sidecar_format`, hide the dialog,
        set the flag, then run the deferred `reopenLastFolder`. A failed
        invoke keeps the dialog up and shows the error, rather than unlocking
        the app with nothing saved.
      - The settings window needs no change: it reads `sidecar_format` on
        open and listens to `sidecar-format`.
    - Docs:
      - `README.md` "First steps": make step 1 "Choose your developing
        software (Lightroom or DxO PhotoLab) in the dialog Riffle shows on its
        first launch; it can be changed later in Settings", then the existing
        steps. Reorder the two sentences that list PhotoLab before Lightroom
        so Lightroom comes first: the intro bullet ("DxO PhotoLab and tools
        that read XMP sidecars, such as Lightroom") and the "First steps"
        closing paragraph. Keep all other README text as is.
      - `docs/usage.md` "Ratings and sidecars": say the format is chosen on
        first launch and can be changed in Settings; keep the list order
        (Lightroom already first) and the note that switching writes only
        unwritten judgments. Make the `(default)` marker on XMP (here and in
        README "Working with other software") mean one consistent thing: it
        remains the fallback for an unknown stored value, so either keep it
        with that meaning or drop it; do not leave it implying "what you get
        without choosing".
      - Wherever Lightroom and PhotoLab appear together in text this step
        touches, list Lightroom first.

## Risks

- The gate lives in the frontend. A future open path added to `main.ts` that
  calls `openDirectory` without the check bypasses it; putting the check at
  the top of `openDirectory` (not only in its callers) limits that.
- Native alternative not taken: `tauri-plugin-dialog`'s `ask` could show the
  question from Rust, but its button labels and ordering are
  platform-dependent, it cannot be tested with vitest, and the frontend gate is
  needed anyway since menu events and drops reach the webview regardless.

## Progress

- (2026-09-24) Step 1 complete
