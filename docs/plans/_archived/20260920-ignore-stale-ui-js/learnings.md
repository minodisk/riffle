# Learnings

## Step 1: Ignore `.js` under `crates/app/ui/src/`

### Root cause: the `tsc` era left `.js` files that Vite kept serving

The user reported that the settings window's shortcut capture registered the
bare modifier (`control`, `alt`, `shift`) instead of `ctrl+a` / `alt+a` /
`shift+a`, and that the "Press a key..." prompt vanished on modifier-down. The
vanishing prompt is only a consequence: registering a key calls
`updateShortcuts()`, which clears `capturing`.

PR #185 (`46ff37a`) had already added the lone-modifier guard to
`crates/app/ui/src/keys.ts`, and that guard is correct for the real event
shape. Console logging in the settings window confirmed the macOS WebKit
events: a lone Control is `{key: "Control", code: "ControlLeft", ctrlKey:
true}`, and the following letter is `{key: "a", code: "KeyA", ctrlKey: true}`.
Both are handled correctly by the current `keyName()`.

The discrepancy showed up in what the dev server served:

- `curl http://localhost:1420/src/keys.ts` returned the guarded code.
- `curl http://localhost:1420/src/keys.js` returned a *different, much older*
  implementation (pre-#166: only `ctrl+alt+` combinations named,
  `return event.key.toLowerCase()` otherwise, plus an `isUnboundModifier`
  export that no longer exists).

`crates/app/ui/src/` in the developer's main checkout held untracked `.js`
files from the pre-Vite+ `tsc` build: `exif.js`, `exif.test.js`, `keys.js`,
`main.js`, `settings.js`, `strip.js`, `worker.js`. Since `settings.ts` imports
`./keys.js`, Vite resolved the real `keys.js` and served it instead of
`keys.ts`. The old `keyName()` returns `event.key.toLowerCase()` with no guard,
so a lone Control keydown yielded `"control"`. So the bogus binding and the
apparently ineffective #185 both followed from the stale file, not from a code
defect.

Deleting those `.js` files fixed it — but only after restarting the dev server,
because Vite had cached the old transform. The user confirmed: Ctrl+A now
registers as `ctrl+a`.

### What tripped us up

- The stale files were untracked and only on the developer's machine, so CI
  (which checks out a clean tree) could never have caught this.
- `git status` noise made them easy to overlook. Now that `.gitignore` hides
  them, the guide note in `docs/agents/tauri-app.md` is the only signal, which
  is why it tells you to run `ls crates/app/ui/src/*.js` before debugging.

### Scope decisions

- The ignore pattern is narrow (`crates/app/ui/src/**/*.js`): a repo-wide
  `*.js` would ignore `.claude/skills/merge-settings/scripts/merge.js`, the
  only tracked `.js` in the repository.
- Not ignored: `*.d.ts` (`crates/app/ui/src/tauri.d.ts` is tracked and
  hand-written) and `*.js.map` (not emitted).
- No change to `crates/app/ui/src/keys.ts`: it is already correct.

## Deferred issues (todo candidates)

- Bogus modifier-only entries (`control`, `alt`, `shift`) already persisted in
  the user's settings store (`shortcuts` key) are not cleaned up by this
  change. Basis: this plan's implementation approach explicitly leaves it out
  of scope; an existing `todo.md` item already covers it. Related files:
  `crates/app/src/shortcuts.rs`, `crates/app/ui/src/settings.ts`.
