# Learnings

## Step 1: Core `.dop` read, patch and template

- `write_rating` takes a fourth parameter, the file `name`, besides the
  existing bytes, the rating and the timestamp: the fresh template carries
  `Name = "<file name>"`, and the core cannot derive it from the bytes. It is
  ignored when patching. Signature:
  `write_rating(existing: Option<&[u8]>, rating: Option<i8>, name: &str, now: &str)`.
- Timestamp: hand-rolled `civil_from_days` (Howard Hinnant's algorithm) in
  `dop::timestamp(SystemTime)`, tested against the sample's instant, a leap
  day and the epoch. No `time` crate.
- UUIDs: generated from two `RandomState::hash_one` calls (version and variant
  bits set), no `uuid` crate. Good enough for PhotoLab's opaque identifiers;
  not cryptographic.
- The samples end with `}\n\r\n` (an empty last line with CRLF), not `}\r\n`.
- The fixtures under `crates/core/src/fixtures/dop/` are marked `-text` in a
  new root `.gitattributes`, so a Windows checkout with `core.autocrlf` cannot
  rewrite their line endings and break the byte-identity tests.
- Trimmed fixtures (0001, 0002, 0003, 0005) are the sample's lines 1-28, the
  first `HSLHueSlices` entry (with the `Label = "Red"` decoy) and lines 544 to
  the end; 0004 is the full file.
- A missing key is inserted at the start of the line holding the table's
  closing `}`; if that `}` shares its line with other content the write is an
  `Err` rather than guessing a layout.

## Step 2: App writes and reconciles the selected format

- `SidecarFormat` (in `crates/app/src/sidecar.rs`) is the only place that
  knows both formats; `write_rating` takes the ARW path so the `.dop` branch
  can pass the file name and `dop::timestamp(SystemTime::now())`.
- `tauri-plugin-store` resolves a relative store path against the app
  **data** dir (`resolve_store_path` uses `BaseDirectory::AppData`), so the
  settings file is opened with an absolute `app_config_dir()/settings.json`
  path to keep it where `last_folder` lived.
- There were no `last_folder` tests to port; the migration's file half
  (`take_legacy_last_folder`) got its own test, the store half is not
  unit-testable without a Tauri app.
- No capability was added: the store is only touched from Rust.
- On a case-insensitive file system (macOS APFS) `sidecar_path(arw).exists()`
  is true for `H.ARW.DOP`, so the rename may land on the other spelling; the
  case test asserts "still exactly one `.dop`" rather than a name.
- Step 2's checkbox stays unchecked: only the manual PhotoLab check remains.

## Step 3: The Sidecar menu and the format switch

- `Builder::menu` runs before the plugins are initialised, so a menu that
  needs the settings store cannot be built there. The menu (default items,
  `Sidecar`, and the dev-only `Debug`) is now built in `setup` with
  `app.set_menu`, after `load_settings`; `debug_menu::build` became
  `debug_menu::append`, and one `on_menu_event` dispatches to it.
- The switch lives in `commands::switch_sidecar_format` (drain, persist,
  update state, `Index::reset_sidecars`), run from the menu handler through
  `spawn_blocking`; `sidecar-format` is emitted after it returns. A failed
  store save is logged, not fatal: the switch still applies for the session.
- DNG: `dop::sidecar_path` gives `<name>.DNG.dop`, but Step 2's
  `SidecarFormat::matches` only accepted `.arw.dop`, so a DNG's `.dop` was
  never listed (never read, and a dirty row would be re-written blindly).
  `matches` now strips `.dop` and applies `scan::is_raw_file` to the rest;
  unit-tested with `L1000001.DNG.dop` and a `.jpg.dop` negative.
- No meta-pane change (the optional sidecar name) was made.

## Deferred issues (todo candidates)

- Review feedback (photolab-dop-sidecar-step-3, Round 1, item 1) asked for a
  test that sets a rating between the format swap and `reset_sidecars` in
  `switch_sidecar_format`. This was dismissed for this round: the function
  takes a real `tauri::AppHandle` backed by `tauri_plugin_store`, and the
  codebase has no `tauri::test` mock-app harness. Building one (mock runtime,
  store plugin wiring) would let this and other `AppHandle`-taking commands
  in `crates/app/src/commands.rs` (e.g. `switch_sidecar_format`, `set_rating`,
  `scan_folder`) be unit-tested. Related files: `crates/app/src/commands.rs`,
  `crates/app/src/main.rs`.
