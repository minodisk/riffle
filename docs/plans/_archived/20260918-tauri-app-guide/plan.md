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

# Tauri app guide (`docs/agents/tauri-app.md`)

## Purpose

`docs/agents/` had nowhere to consolidate the `crates/app` gotchas from the
tauri-skeleton plan, so each new task touching Tauri commands,
`crates/app/tauri.conf.json` or the `crates/app/ui` frontend would rediscover
them. The archived learnings also carry one claim that #11 proved wrong (that
sync commands run off the main thread). This plan lands a single pitfall-list
guide, with every claim checked against the archived learnings, commit
`e5d4fff` (#11) and the current code, so agents read one correct source.

## Steps

- [x] Step 1: Review and refine the draft `docs/agents/tauri-app.md` at `a377d0f` and open the PR
  - Done when:
    - `docs/agents/tauri-app.md` exists and covers all eight points: (1)
      `frontendDist` resolves relative to the directory holding
      `tauri.conf.json`, so `"ui"` not `"../ui"`, wrong value fails
      `tauri::generate_context!()` at build; (2) `tsc` rejects `outDir ==
      rootDir` (TS18003), omit both to emit in place; (3) a `.ts` file with no
      `import`/`export` is a global script whose top-level names collide with
      `lib.dom` globals such as `status`; (4) `DedicatedWorkerGlobalScope` is
      absent from the `dom` lib and `webworker` clashes with `dom`, so declare a
      local interface and cast `self`; (5) relative frontend imports need an
      explicit `.js` because `moduleResolution: "bundler"` accepts extensionless
      imports but there is no bundler; (6) sync `#[tauri::command]` runs inline
      on the main thread (`body_blocking`, no spawn), the `blocking_pick_folder`
      hang, the async-command + `tauri::async_runtime::channel(1)` fix, and
      `spawn_blocking` for blocking IO in async commands; (7) CI caches the
      output of `pnpm store path --silent` instead of a hard-coded path; (8)
      `osascript` assistive access is denied on this Mac, so GUI checks are
      planned as manual user confirmation and reported honestly as "not
      verified".
    - Every point has a one-line "why" and is tagged Hit or Inferred, and the
      tag matches the sources (see approach below for the two corrections).
    - Every concrete claim matches the code and sources: `crates/app/tauri.conf.json`
      (`"frontendDist": "ui"`), `crates/app/ui/tsconfig.json` (no `outDir`/`rootDir`,
      `lib` = `dom`, `dom.iterable`, `es2022`, `moduleResolution: "bundler"`),
      `crates/app/ui/src/main.ts` (`export {};` at the end), `crates/app/ui/src/worker.ts`
      (`WorkerScope` interface, `self as unknown as WorkerScope`),
      `crates/app/src/commands.rs` (`pick_folder` is `async` awaiting
      `tauri::async_runtime::channel(1)`; `preview` uses `spawn_blocking`),
      `.github/workflows/ci.yml` (`pnpm store path --silent` step).
    - `mise run ci` passes.
    - The diff touches only `docs/agents/tauri-app.md` (plus this plan's
      progress update); no changes under `crates/`.
    - The PR uses a Conventional Commit title in English (the existing commit
      `docs(agents): add the crates/app pitfalls guide` is suitable).
  - Implementation approach:
    - Work on branch `docs/tauri-app-guide`; the draft at `a377d0f` already
      passes `mise run ci` and covers all eight points in the required tone.
      Refine it rather than rewrite.
    - Sources to check against: `docs/plans/_archived/20260917-tauri-skeleton/learnings.md`
      (Step 2 for points 1, 2, 3, 7, 8; Step 3 for the wrong sync-command
      claim; Step 4 for points 4, 5; "Verification status" for 8), commit
      `e5d4fff` message, and
      `docs/plans/review-history/fix-folder-picker-deadlock/review-20260917-2241.md`
      (confirms `body_blocking`, `ExecutionContext::Blocking`, `sync_channel(0)`
      against `tauri-macros` 2.6.3 / `tauri-plugin-dialog` 2.7.3).
    - Two corrections found at planning time:
      - Point 7 is tagged **Hit** in the draft, but the learnings (Step 2) and
        the `ci.yml` comment describe a deliberate choice, not a failure that
        occurred. Retag it **Inferred** so it matches the guide's own tag
        definitions.
      - Point 6 says the bug "passed ... three rounds of local review".
        `docs/plans/review-history/tauri-skeleton-step-3/` shows two rounds
        (NEEDS_FIX, APPROVED). Count the rounds across the step-3 and step-4
        review records; if three cannot be substantiated, write "local review"
        without a number.
    - Keep the structure (Rust side / Frontend / CI / Verification, "Why" and
      "What broke" bullets) and the concise tone of `README.md` and `CLAUDE.md`.
      Do not expand into a tutorial.
    - Do not touch `crates/core` or `crates/app`; another session edits them in
      parallel. If a code fact and the draft disagree, fix the guide's wording,
      not the code.
    - Run `mise run ci` before pushing.

## Trade-offs and risks

- Point 7 tag: the sources support Inferred only. If an actual cache miss
  motivated it, Hit is fine, but nothing in the repository records one.
- "Three review rounds" wording: keeping the number is a stronger cautionary
  line but is not backed by the step-3 record alone; dropping the number is
  safer. The step decides after counting the records.
- Point 5 stays Inferred and the guide says nothing enforces it. Switching
  `moduleResolution` to `node16` would enforce `.js`, but the learnings explain
  why that was rejected, and it would touch `crates/app`, out of scope here.

## Progress

- 2026-09-18: Step 1 done. Reviewed the draft from `a377d0f` and corrected two
  points: retagged point 7 (`pnpm store path --silent` caching) from Hit to
  Inferred, since the sources describe a deliberate choice rather than an
  observed failure; removed the "three review rounds" count for point 6 since
  the step-3 review records show only two rounds (NEEDS_FIX, APPROVED), and
  the wording now says "local review" without a number. `mise run ci` passed.
