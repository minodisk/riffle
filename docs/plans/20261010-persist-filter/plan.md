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

# Persist the strip's filter across restarts

## Purpose

The filter menu's checks live only in memory, so every launch starts
unfiltered and a culling session narrowed to, say, `Good` + `Picked` has to be
narrowed again. This work stores the one app-wide filter (flags, stars, color
labels, orientations, `AF eye` and `Eyes`; the EXIF groups stay per session
as today, cleared when a folder opens) under a `filter` key of the settings
store, the way `sortOrder` and `panels` are stored, restores it before the
last folder opens, and keeps the menu's checks and the filter button's lit
state in step with the restored value. A stored value that is missing,
malformed or names states that no longer exist (the `AF eye` states were
renamed in #766) degrades to an empty, all-pass filter and never blocks
launch.

## Steps

- [ ] Step 1: `filter` / `set_filter` commands and the stored value's parser
  - Done when: `crates/app/src/commands.rs` has `filter_setting(Option<&Value>) -> Value` that normalizes a stored value into the canonical shape (every section present, unknown members dropped, non-object input or a non-array section treated as empty, extra keys dropped), a `filter` command that reads the `filter` key through it, and a `set_filter(app, filter: Value)` command that normalizes and saves (a save failure is logged, not returned, like `set_sort_order`); both are registered in `generate_handler!` in `crates/app/src/main.rs`; unit tests next to `panels_setting_falls_back_to_shown` cover `None`, `"x"` and `[]` giving the empty filter, each section dropping unknown members (`"ok"` in candidates, `7` / `"3"` / `2.5` in stars, `"Red"` in labels), a valid value round-tripping unchanged, and an extra key (`"exif"`) being dropped. `mise run ci` passes.
  - Implementation approach:
    - Canonical JSON shape, also the wire shape of both commands:
      `{"flags": [..], "stars": [..], "labels": [..], "orientations": [..], "candidates": [..], "eyes": [..]}`, arrays of strings except `stars` (integers 0..=5).
    - Allowed values, from `ui/index.html`'s `data-*` items and the types in `ui/src/filter.ts`, `ui/src/focus.ts` (`MarkState`) and `ui/src/eyes.ts` (`EyeState`): flags `picked|untagged|rejected`; labels `red|orange|yellow|green|blue|pink|purple|none`; orientations `portrait|landscape`; candidates `good|not_candidate|unknown`; eyes `open|closed|unknown`. Keep them as `const` slices with a comment naming those files as the source of truth, so a renamed state (as in #766) is updated in both places.
    - Dedupe members, keep the stored order, do not sort.
    - Follow the doc-comment and `log::warn!` style of `set_sort_order` (`commands.rs` ~853-886) and `set_panels` / `panels_setting` (~1870-1913). No managed state struct is needed: nothing native reads the filter.
- [ ] Step 2: Save the filter on every change, restore it at launch, and document it
  - Done when: `crates/app/ui/src/filter.ts` exports a `StoredFilter` type matching Step 1's shape plus `toStored(state: FilterState): StoredFilter` (the six sets as arrays, `exif` not included) and `applyStored(state: FilterState, stored: StoredFilter): void` (replaces the six sets' contents, leaves `exif` alone), with vitest cases in `filter.test.ts` for the round-trip, the empty filter, and `applyStored` not touching `exif`; `main.ts` invokes `set_filter` with `toStored(...)` from the change path of `filterChanged()` (every item click and `Reset` persist); at launch it invokes `filter`, applies the value to the `shown*` sets, mirrors the menu's `aria-checked` and the filter button's `active` class without rendering, and only then reopens the last folder; `docs/humans/usage.md`'s Filter menu bullet (~line 305) and `docs/humans/usage.ja.md` (line 20) say the flag, star, label, orientation, `AF eye` and `Eyes` checks are remembered across restarts and apply to every folder, while the EXIF checks are not (they clear when a folder opens, as today), and that `Reset` clears the remembered filter too. Manual check: set a filter, quit, relaunch -> same files hidden, checks and lit button shown; `Reset`, relaunch -> no filter; a bogus `filter` value in `settings.json` -> launches unfiltered. `mise run ci` passes.
  - Implementation approach:
    - Assumes Step 1 is merged.
    - Split `filterChanged()` (`main.ts` ~3876) into the menu-mirroring part (checks + `filterToggle.classList.toggle("active", filterActive())`), reused by the restore, and the change path (mirror + `refilter()` + `set_filter`). The restore must not call `refilter()` before a folder is open and must not write the value back.
    - Join the restore to the existing launch chain: `sortLoaded = invoke("sort_order")...finally(() => formatGate.whenOpen(reopenLastFolder))` (~4154). Use `Promise.allSettled([sort, filter])` before `formatGate.whenOpen(reopenLastFolder)`, so the first `files = order.filter(passes)` in `openDirectory` sees the restored sets; a rejected `filter` read is ignored like `sort_order` / `panels` (`() => {}`) and still proceeds. Keep `Promise.allSettled([sortLoaded, keymapLoaded]).then(folders.loadRoots)` working.
    - `toStored` emits `stars` as numbers and keeps set order; the backend is the validator, so the TS helpers do no validation beyond the typed shape.
    - `README.md` / `README.ja.md` do not enumerate what persists for the filter (checked at planning time), so they need no change; `CLAUDE.md` gets a mention of the `filter` key only if the implementer touches the sentence that lists the store's keys (keep it minimal).

## Trade-offs and risks

- **EXIF groups not persisted (decided).** Today the EXIF checks are cleared by `openDirectory` and stale labels are dropped by `rebuildExifMenu`; persisting them would have changed that in-session behavior, so they stay session-only. The stored shape has no `exif` key, and `filter_setting` drops one if present.
- **Validation lives in Rust only.** The TS helpers trust the value the `filter` command returns, matching `parse_sort_order` / `panels_setting`. The allowed-value lists are duplicated between `index.html` / `filter.ts` and `commands.rs` and must change together when a state is added or renamed; a TS-side guard would avoid that failure mode at the cost of duplicated tests.
- **Write frequency.** `set_filter` saves the store on every click, like `set_panels` and `set_sort_order`; the write is small and synchronous on the calling thread, which `docs/agents/tauri-app.md` allows for small settings. No coalescing.
- **Launch ordering.** The reopen now waits on one more store read; `sort_order` already gates it the same way, and `allSettled` keeps a failed read from blocking the reopen.
- **Companion / MCP.** `get_view` reports "whether a filter is on" from the same sets, so a restored filter reports as on at launch; no change expected, but glance at `companion.test.ts` if it asserts the launch state.

## Progress

- (none yet)
