# Learnings

## Step 1

- `IconMenuItem::with_id_and_native_icon` takes the icon as
  `Option<NativeIcon>` between `enabled` and the accelerator, so only the
  construction changed; `file.prepend_items` / `app.insert_items` took the new
  items unchanged because `IconMenuItem` implements `IsMenuItem`.
- `#[cfg]` attributes work on the `let` bindings directly, so the non-macOS
  build keeps the existing `MenuItem`. `IconMenuItem` / `NativeIcon` are
  imported in their own `#[cfg(target_os = "macos")]` `use`, keeping clippy's
  unused-import warning quiet elsewhere.
- Only `aarch64-apple-darwin` is installed (`rustup target list --installed`),
  so `cargo check -p riffle-app --target x86_64-unknown-linux-gnu` could not be
  run. The non-macOS path is unchanged from the previously compiling code and
  every new item is behind `cfg(target_os = "macos")`, so it still compiles.

## Deferred issues (todo candidates)

- muda never calls `setTemplate` on a custom menu `NSImage` and Tauri exposes
  no template flag for menu items (only for the tray icon), so a bundled PNG
  cannot tint with the menu appearance. Upstreaming template support in muda /
  Tauri is the proper fix; out of scope here. Basis: the plan's "Trade-offs and
  risks" section, relevant to `crates/app/src/main.rs` (`app_menu`) and Step 2's
  bundled icons.
