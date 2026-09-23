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

# American spelling

## Purpose

The repository mixes British and American spellings (`colour` next to
`ColorLabel`, `labelled` next to `LabelColor`, `judgement`, `behaviour`,
`centre`, `grey`, `serialise`, ...). Converting everything to American
spelling makes prose, identifiers and searches consistent with the external
vocabulary the app already uses (XMP `photoshop:LabelColor`, Lightroom's
"Color Label Set", GitHub's API) and removes a recurring source of review
noise. After this work, a grep for the British forms returns only the
external-spec literals listed below.

## Decisions

- `judgement` / `judgements` become `judgment` / `judgments`, including the
  Rust and TS `Judgement` domain types (decided by the user).
- Two steps, two PRs: code and top-level docs first, then `docs/plans/**`.
- `docs/plans/_archived/20260919-filter-colour-label/` and
  `docs/plans/review-history/filter-colour-label-step-{1,2,3}/` are renamed
  with `git mv` (nothing in the repository links to them).
- `docs/plans/review-history/` and `CHANGELOG.md` are converted too.

## Word list

Convert every occurrence, in prose, comments, string literals (including UI
text and log lines), and identifiers (types, functions, variables, constants,
test names, CSS classes), of:

| British | American |
| --- | --- |
| colour / colours / coloured | color / colors / colored |
| behaviour / behaviours / behavioural | behavior / behaviors / behavioral |
| centre / centres / centred | center / centers / centered |
| labelled / labelling / relabelled / relabelling | labeled / labeling / relabeled / relabeling |
| grey | gray |
| cancelled / cancelling | canceled / canceling |
| serialise / serialises / serialised / serialising / serialisation (and deserialis*) | serialize ... serialization |
| optimise / optimised / optimisation(s) | optimize / optimized / optimization(s) |
| normalise / normalises / normalised / normalising / normalisation | normalize ... normalization |
| initialise / initialised | initialize / initialized |
| licence | license |
| artefact | artifact |
| modelled | modeled |
| recognise / unrecognised / unrecognisable | recognize / unrecognized / unrecognizable |
| favour / honour / honoured / honours | favor / honor / honored / honors |
| organise / customised / finalised / signalled | organize / customized / finalized / signaled |
| defence | defense |
| neighbour / neighbours / neighbouring | neighbor / neighbors / neighboring |
| judgement / judgements | judgment / judgments |
| catalogue, travelled, analyse, and any other British form found while sweeping | the American form |

Not British, leave alone (they match naive regexes): `optimistic`,
`optimistically`, `realistic`, `unrealistic`, `checkerboard`, `dialogue`,
`afterwards`, `towards`, `fulfilled`, `unfulfilled`, `LightingPriorityRegions`.

## Allowed exceptions (must stay British)

- GitHub Actions run conclusion literals compared against the API:
  `"cancelled"` in `.claude/skills/merge/scripts/wait-post-merge-runs.sh`
  (the jq filters), `"CANCELLED"` in `.claude/skills/pr/scripts/pr-status.sh`,
  the `run Release cancelled ...` fixture in
  `.claude/skills/merge/scripts/wait-post-merge-runs.test.sh` (the value is
  the conclusion the fake API returns), and the backticked `cancelled` row
  verdict in `.claude/agents/merger.md`, which names the value printed in the
  results table. Prose and comments around them (e.g. "treating cancelled
  runs as failures", the header comment of `wait-post-merge-runs.sh`, and the
  `expect "cancelled superseded"` test description) are free to change; the
  shell variable `cancelled_tsv` may be renamed but the jq string literals
  must not.
- `aria-labelledby` in `crates/app/ui/settings.html`: WAI-ARIA attribute name.
- `crates/core/models/LICENSE`: third-party license text, untouched.
- `Cargo.lock`, `pnpm-lock.yaml`, `node_modules`, `target`, binary assets.
- XMP / `.dop` field names and settings-store keys: none carry a British
  spelling (`labelNames`, `sidecarFormat`, `shortcuts`, `autoAdvance`,
  `lastFolder`, `sortOrder`; SQLite columns `label`, `label_known`), so
  nothing to preserve, but re-verify with the acceptance grep.

Acceptance grep (run from the repo root; must print only the lines listed
above):

```sh
git ls-files -z | grep -zvE 'Cargo\.lock|pnpm-lock\.yaml|\.(png|jpg|ico|icns|onnx|ARW|DNG)$' \
  | xargs -0 grep -niE 'colour|behaviour|centre|centred|labell(ed|ing)|\bgrey|cancell(ed|ing)|serialis|optimis(e|ed|ation|es|ing)\b|normalis|initialis|licence|artefact|modelled|recognis|favour|honour|organis|customis|finalis|signalled|defence|neighbour|judgement|catalogue|travelled|analys(e|ed|es|ing)\b'
```

## Steps

- [x] Step 1: Convert code, config, `.claude/`, `.github/`, `tools/`, and the top-level and `docs/agents` / `docs/*.md` documents
  - Done when:
    - The acceptance grep, restricted to everything outside `docs/plans/`, prints only the allowed exceptions.
    - `mise run ci` passes (vp check/test, cargo fmt/clippy/test, shellcheck, actionlint, lychee, the two bash test scripts).
    - Renamed identifiers are renamed everywhere they are referenced, including test names and the `labelled` CSS class in both `crates/app/ui/style.css` and `crates/app/ui/src/strip.ts`.
  - Implementation approach:
    - Files: `crates/core/src/*.rs` (not `crates/core/models`); `crates/app/src/*.rs`; `crates/app/ui/src/*.ts`, `crates/app/ui/settings.html`, `crates/app/ui/style.css`; `crates/cli`; `.claude/agents/*.md`, `.claude/skills/**/*.md`, `.claude/skills/**/scripts/*.sh`; `.github/ISSUE_TEMPLATE/*.yml`; `README.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `todo.md`, `Cargo.toml` (comment), `mise.toml` (task description), `docs/agents/tauri-app.md`, `docs/performance.md`, `docs/usage.md`, `tools/git/delete_merged_branches.sh`, `tools/macos/export-menu-icons.swift`.
    - Identifiers to rename (all internal, none persisted or serialized): Rust `Judgement` struct and `judgement` field/locals in `crates/app/src/sidecar.rs`; TS `Judgement` interface/type in `filter.ts` and `main.ts` and `judgements()` in `selection.ts`; `LIGHTROOM_LABELLED` and the `labelled()` test helpers in `crates/core/src/xmp.rs` and `crates/core/src/dop.rs`; `labelled_fresh`, `centre` locals in `sharpness.rs` and `strip.ts`; the `labelled` local in `exif.ts`; test function names containing any word in the list.
    - Use a case-preserving, whole-word sed/perl pass driven by the word list, then hand-check the exception sites listed above and revert them. Apply `mise run fmt` afterwards so the Rust/TS reflow does not fail `cargo fmt --check`.
    - The rename must not touch the string values written to sidecars or read from the settings store; confirm by running `cargo test` and by diffing `grep -rn 'store\.\(get\|set\)' crates/app/src` before and after.
    - `CHANGELOG.md` is generated by release-please but only prepended; editing past entries is safe.
    - Commit as `refactor: use American spelling across code and docs`.
- [ ] Step 2: Convert `docs/plans/**` (the active plans, `_archived/`, and `review-history/`)
  - Done when:
    - The acceptance grep over the whole repository prints only the allowed exceptions.
    - `lychee --offline --include-fragments` (part of `mise run lint`) still passes, so no intra-doc link or fragment was broken (headings containing a converted word change their anchors; update links that point at them).
    - `docs/plans/_archived/20260919-filter-colour-label/` and `docs/plans/review-history/filter-colour-label-step-{1,2,3}/` are renamed with `git mv` to their `color` forms.
    - `docs/plans/_archived/20260919-undo-judgements/` is renamed with `git mv` to `20260919-undo-judgments/`, and the citation in `docs/agents/tauri-app.md` (the Source line of the undo-anchoring learnings entry) is updated to the new path.
  - Implementation approach:
    - Assumes Step 1 is merged; reuse the same sed/perl pass on `docs/plans/` alone.
    - In-flight plans (including this one) get their prose converted but keep their structure. This plan's own word list and exception list must keep the British forms, since they document the conversion; list this file as an allowed exception.
    - Heading anchors: search the converted files for `](#...colour...)`-style fragments and `docs/plans/...#...` links and update them; lychee's `--include-fragments` will flag any that were missed.
    - Commit as `docs(plans): use American spelling`.

## Trade-offs and risks

- **Regex false positives.** A blind `s/ise/ize/` pass would corrupt words
  like `optimistic`, `realistic`, `dialogue`, `towards`. The pass must be
  whole-word and driven by the explicit list; review the diff for them.
- **Formatting churn.** Word-length changes reflow rustfmt/prettier output;
  run `mise run fmt` before pushing or `cargo fmt --check` fails.
- **External references.** Old PR bodies and the user's `MEMORY.md` mention
  old plan paths; nothing in CI depends on them.

## Progress

- (2026-09-23) Step 1 complete
