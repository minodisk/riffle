# Learnings

## Step 1

- `Image` in `app_menu` was macOS-only; the About icon needs it on every
  platform, so the import lost its `cfg`.
- The About item is swapped right after `Menu::default`, before any insert, so
  the macOS app-menu inserts at indices 1 and 3 and the Help `prepend` still
  land where they did. The target submenu is picked per platform (first
  submenu on macOS, `Help` elsewhere) and the swap reuses the Edit
  Undo/Redo guard (`MenuItemKind::Predefined` at index 0).
- GUI behavior (icon visible in macOS About panel / GTK dialog) can only be
  checked manually; CI proves compilation on all three OSes.
