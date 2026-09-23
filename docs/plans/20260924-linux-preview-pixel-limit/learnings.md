# Learnings

## Step 1

- `previewPixelLimit` is declared right after `worker` in `main.ts`, not next
  to the `timing_logs` / `auto_advance` fetches near the end of the file as the
  plan suggested: a `const` read by `requestPreview()` must be declared above
  any top-level path that can reach that function (see "A `let` used only
  inside a function still needs to be declared above..." in
  `docs/agents/tauri-app.md`), and declaring it next to the worker avoids the
  TDZ question entirely.
- The await sits after the `preview` invoke's `current !== seq` check and
  repeats that check once it resolves, so a page turned during the first
  (unresolved) await still never posts a stale page.
- Importing `./decode.js` turns `worker.ts` into an ES module; the worker is
  already created with `type: "module"`, so nothing else changed.
- The manual Linux GUI check (`mise run dev` under WSLg with a SIGMA fp L DNG)
  was not performed by the implementation agent.
