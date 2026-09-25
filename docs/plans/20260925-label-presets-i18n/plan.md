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

# Lightroom color label presets from per-language JSON

## Purpose

The Lightroom color label presets the settings window offers are hard-coded in
Rust: English via `COLORS` in `impl Default for LabelNames` and Japanese in
`LabelNames::japanese()` (`crates/core/src/xmp.rs:158-170`), and the
`label_names` command (`crates/app/src/commands.rs:1323`) returns exactly those
two as `{"names", "japanese"}`, which the settings window binds to two buttons
("Lightroom default (Japanese)" and "Reset to English").

Moving the presets into one JSON file per language (`crates/core/i18n/*.json`,
embedded at build time) lets a contributor add a language by adding one JSON
file and nothing else, with tests validating it against `en.json`. The settings
window replaces the two buttons with a language `<select>` plus one Reset
button. The namespaced layout (`meta`, `lightroom.colorLabels`) leaves room for
other per-language values later; only `lightroom.colorLabels` is populated in
this work, and the stored `labelNames` setting is untouched (no migration).

Decisions made with the user:

- The files and the parser live in `crates/core` (adding `serde` /
  `serde_json` to `riffle-core`).
- A `build.rs` in `crates/core` generates the registry, so adding a language
  needs no Rust change.
- `meta.name` is the language's name in that language (`English`, `日本語`).
- The dropdown defaults to English.

## Steps

- [x] Step 1: Add the per-language JSON files, a build.rs-generated registry and a `riffle_core::i18n` module that exposes the presets, and drop `LabelNames::japanese()`
  - Done when:
    - `crates/core/i18n/en.json` and `crates/core/i18n/ja.json` exist with the agreed shape (`meta.name`, `lightroom.colorLabels.{verified,red,yellow,green,blue,purple}`); `ja.json` carries exactly today's names (レッド/イエロー/グリーン/ブルー/パープル), `meta.name: "日本語"` and `verified: "Lightroom Classic 2026 (Windows, Japanese UI)"`; `en.json` carries Red/Yellow/Green/Blue/Purple and `meta.name: "English"`.
    - `crates/core/build.rs` lists `crates/core/i18n/*.json`, emits `cargo:rerun-if-changed=i18n` (and per file as needed so adding, removing or editing a file triggers a rebuild), and writes into `OUT_DIR` a generated registry of `(code, include_str!(<absolute path>))` entries sorted by code. Adding a language requires only a new JSON file.
    - `crates/core/src/i18n.rs` includes the generated registry (no runtime file IO), parses it with serde, and exposes a `presets()`-style function returning, in a stable order (English first, then the rest by language code), one entry per language that has `lightroom.colorLabels`: language code, `meta.name`, `verified`, and a `xmp::LabelNames`.
    - `LabelNames::japanese()` is gone; no Japanese label name remains in non-test Rust. The tests in `crates/app/src/sidecar.rs` and `crates/app/src/commands.rs` that used it now take the names from the `ja` preset (or a small test helper doing that lookup).
    - Validation tests in `i18n.rs`: every key path in every file exists in `en.json` (`en.json` is the reference); `lightroom.colorLabels`, when present, has all five colors non-empty and a non-empty `verified`; `en.json` has `lightroom.colorLabels`; `meta.name` is non-empty; `en.json`'s names equal `LabelNames::default()`; each file name is a plausible language code.
    - `cargo clippy --all-targets -- -D warnings` and `cargo test` pass (`mise run test`).
  - Implementation approach:
    - Add `serde = { version = "1", features = ["derive"] }` and `serde_json = "1"` to `crates/core/Cargo.toml` (same versions as `crates/app/Cargo.toml`). Parse at first use through a `std::sync::OnceLock`; the files are tiny.
    - Implement the key-path check on `serde_json::Value` (walk each file's object tree, assert each path is present in `en.json`'s tree); it doubles as the "no typo'd key" guard and carries the fallback rule forward for future namespaces.
    - Keep `impl Default for LabelNames` as it is (English from `COLORS`); the test asserting `en.json == LabelNames::default()` keeps the two from drifting.
    - Add the `i18n` module to `crates/core/src/lib.rs` next to `xmp` / `dop`, with a module doc comment describing the file layout and the fallback rule (a missing key reads as `en.json`'s value; with only `colorLabels` populated this reduces to "a language without `colorLabels` offers no preset").
    - Keep the build script minimal (std only, no build-dependencies).

- [ ] Step 2: Return the preset list from `label_names` and replace the two preset buttons with a language dropdown and a Reset button
  - Done when:
    - `label_names` returns `{"names": {...}, "presets": [{"code": "en", "name": "English", "names": {...}}, {"code": "ja", "name": "日本語", "names": {...}}]}` (the `names` object in the shape stored under `labelNames`, produced by `label_names_value`); the command's doc comment is updated.
    - `crates/app/ui/settings.html` `#label-names` has a `<select id="label-names-language">` filled from `presets` (option value = code, text = name) and one `<button id="label-names-reset">Reset</button>`; the two old buttons and their ids are gone; the explanatory `<p>` no longer says "the buttons below fill in Lightroom's built-in sets" but describes the dropdown + Reset, and still names the Lightroom Classic menu (Metadata > Color Label Set > Edit).
    - `crates/app/ui/src/settings.ts`: `japaneseLabelNames` is replaced by the preset list; Reset calls `saveLabelNames` with the selected preset's names. If `englishLabelNames()` in `labels.ts` becomes unused, remove it and its test; otherwise leave both alone.
    - The dropdown defaults to `en` (first entry). Selecting a language alone saves nothing; only Reset writes.
    - `pnpm exec vp check`, `pnpm exec vp test`, `cargo test` pass.
  - Implementation approach:
    - Assumes Step 1 is merged (`riffle_core::i18n::presets()`).
    - Keep the command synchronous (no IO).
    - Keep the Tauri `label-names` event and `set_label_names` unchanged.
    - The block stays hidden unless the format writes XMP, unchanged.

- [ ] Step 3: Contributor guidance and documentation
  - Done when:
    - `crates/core/i18n/README.md` explains: one file per language named by its code, picked up automatically by the build; the namespaced shape with `en.json` as the reference; that `meta.name` is the language's own name for itself; that `lightroom.colorLabels` values must be copied from a real Lightroom's default color label set in that UI language and `verified` must name the Lightroom version / OS / UI language they were checked against; and that `cargo test -p riffle-core` validates it.
    - `CONTRIBUTING.md` gains a short "Adding a Lightroom label preset language" pointer to that README.
    - `README.md` and `README.ja.md` no longer say a preset is provided only for the Japanese default set, but that presets for Lightroom's localized default sets are provided (English and Japanese so far) and that adding a language is a JSON file (link to the i18n README). Both change in the same PR.
    - `CLAUDE.md`'s Layout paragraph mentions `crates/core/src/i18n.rs`, `crates/core/build.rs` and `crates/core/i18n/` (the per-language JSON with the Lightroom color label presets).
    - `docs/agents/tauri-app.md` is checked for statements the change makes wrong, and fixed only if so.
    - `mise run ci` passes (including `lychee --offline` for the new links).
  - Implementation approach:
    - Assumes Steps 1 and 2 are merged so the docs describe what exists.
    - Keep the README wording minimal; the README is user-facing, the i18n README is contributor-facing.

## Trade-offs and risks

- `riffle-core` gains `serde` / `serde_json` (user decision, keeps the data next to `LabelNames` and reachable from the CLI).
- build.rs adds a small build script in exchange for JSON-only contributions (user decision).
- Preselecting the preset that matches the stored names is not planned; the dropdown starts at English.
- The fallback rule is mostly latent while only `lightroom.colorLabels` exists; the key-path validation is what carries it forward.

## Progress

- (none yet)
