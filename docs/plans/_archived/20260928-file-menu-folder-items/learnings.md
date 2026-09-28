# Learnings

## Step 1

- `main` had moved since planning: the File menu's trash item had become a thin
  wrapper (`trashRejected`) around `trashRejectedIn([openDir], false)`, which
  the tree also calls, so only the wrapper went. `runTrash` refreshes the strip
  through `resync`, so the `docs/agents/tauri-app.md` note listing
  `trashRejected` among the refilter paths now lists `resync` and
  `strip.setFiles` only.
- Beyond the files the plan named, two more places described the File menu
  path and were updated: the `style.css` comment above `#sequence-dialog` and
  the `CLAUDE.md` line describing `src/sequence.ts` ("its flow from the folder
  picker").
- `pick_folder` stays: `openFolder()` still invokes it for `File > Open Folder…`.
- The tree context menu's action names (`trashRejected`, `sequenceTimestamps`
  in `context.ts`) are unrelated to the removed functions and were left as is.

## Deferred issues (todo candidates)

- The comment in `crates/app/src/sequence.rs` still says the output goes to
  "a sibling of the picked folder"; with the picker gone the folder is the one
  right-clicked in the tree. Harmless wording, left to keep the change to the
  header line the plan named. Basis: Step 1 implementation.
