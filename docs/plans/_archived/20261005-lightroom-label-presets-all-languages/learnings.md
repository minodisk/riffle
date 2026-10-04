# Learnings

## Step 1

- The 14 preset files were generated from the plan's table by a small script
  (`json.dumps(..., ensure_ascii=False, indent=2)` plus a trailing newline),
  which matches the shape and formatting of `ja.json`; `build.rs` and
  `i18n.rs` needed no change.
- `defaultPreset` tries `Intl.Locale(tag).maximize()` first and compares
  `${language}-${script}` case-insensitively with the preset codes; any throw
  (empty tag, no `Intl.Locale`) falls through to the old primary-subtag match.
- `crates/core/i18n/README.md` still uses `pt-BR.json` as an example file
  name although Portuguese ships as `pt.json`; it is only an example of the
  naming scheme, so it was left as is.

## Deferred issues (todo candidates)

- Pending manual check (Windows or macOS, running app): from
  `docs/plans/20261005-lightroom-label-presets-all-languages/plan.md` Step 1.
  Steps: start the app with a fresh settings store (so the first-launch dialog
  shows), choose XMP, and confirm the language dropdown preselects the
  browser locale's preset (e.g. `ja` on a Japanese system); then open the
  settings window and confirm both the first-launch and the settings
  dropdowns list 16 entries in the order English, Deutsch, Español,
  Français, Italiano, 日本語, 한국어, Norsk bokmål, Nederlands, Polski,
  Português (Brasil), Русский, Svenska, ไทย, 简体中文, 繁體中文, and that
  choosing one fills the five label names. The step's checkbox was ticked on
  the automated criteria (`cargo test -p riffle-core`, the `firstrun.test.ts`
  cases, `mise run ci`). Files: `crates/core/i18n/*.json`,
  `crates/app/ui/src/firstrun.ts`.
