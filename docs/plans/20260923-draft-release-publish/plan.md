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

# Publish releases only after the installers are attached

## Purpose

release-please publishes the GitHub release the moment the release PR merges, so
it becomes "Latest" while the four build jobs are still compiling. During that
window (30+ minutes, observed with v0.2.2)
`https://github.com/minodisk/riffle/releases/latest/download/latest.json`
returns 404 and the Tauri updater fails with "Could not fetch a valid release
JSON from the remote". Creating the release as a draft, uploading the bundles
and `latest.json` to the draft, and publishing only after every build job
succeeded keeps "Latest" pointing at a complete release at all times.

## Steps

- [x] Step 1: Create the release as a draft and publish it after the build matrix succeeds
  - Done when:
    - `release-please-config.json` sets `"draft": true` and
      `"force-tag-creation": true` (root level, next to
      `include-component-in-tag`).
    - `release.yml` passes the draft release to `tauri-action` by id rather than
      by tag, and a new job that runs after all four `build` matrix jobs
      succeeded publishes the release and marks it latest.
    - On `workflow_dispatch` the build job still runs and only uploads workflow
      artifacts (no release interaction), as today.
    - `mise run lint` (actionlint) and CI pass.
    - `learnings.md` records how to verify on the next release
      (see "Verification on the next release" below).
  - Implementation approach:
    - `release-please-config.json`: add `"draft": true` and
      `"force-tag-creation": true`. Both are in the upstream config schema;
      `force-tag-creation` exists precisely because GitHub does not create the
      tag of a draft until it is published, and release-please would otherwise
      fail to find the previous release on its next run.
    - `release.yml`, `release-please` job: add an output
      `release_id: ${{ steps.release.outputs.id }}`. release-please-action
      emits every field of its `CreatedRelease` object as an output, and `id`
      is one of them (undocumented in the README table but set by
      `outputReleases` in the action's `src/index.ts`). Keep `tag_name` as an
      output for the publish job.
    - `release.yml`, `build` job: replace
      `tagName: ${{ needs.release-please.outputs.tag_name }}` with
      `releaseId: ${{ needs.release-please.outputs.release_id }}`. This matters:
      tauri-action with `tagName` alone calls `getReleaseByTag`, which 404s for
      drafts, and then *creates a second release* (or fails for lack of
      `releaseName`). With `releaseId` it skips lookup/creation and uploads
      directly. `latest.json` is already merged per release id across the
      matrix jobs (tauri-action downloads the existing asset, merges
      `platforms`, deletes and re-uploads), so no change there. On
      `workflow_dispatch` the output is empty, which tauri-action treats as
      "no release", same as the empty `tagName` today. Update the comment
      above the job accordingly.
    - `release.yml`, new `publish` job:
      `needs: [release-please, build]`,
      `if: needs.release-please.outputs.release_created == 'true'` (no
      `always()`, so any failed build leaves the release a draft),
      `runs-on: ubuntu-latest`, single step
      `gh release edit "$TAG" --draft=false --latest` with
      `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}` and `GH_REPO: ${{ github.repository }}`
      (the workflow already has `contents: write`). Pass the tag through an
      `env:` variable, not inline `${{ }}` in `run:`, to satisfy actionlint.
      Add a short comment explaining why the release is published last.
    - Do not touch `crates/app/tauri.conf.json`; the `/releases/latest/download/latest.json`
      endpoint stays.
    - Verification on the next release (record in learnings): after
      merging the next `chore(main): release x.y.z` PR, check that
      `gh release view vX.Y.Z --json isDraft` is `true` while the build jobs
      run, that the tag `vX.Y.Z` exists immediately
      (`git ls-remote --tags origin vX.Y.Z`), that no duplicate `vX.Y.Z`
      release appears, that after the `publish` job the release is
      `isDraft: false`, marked Latest, and has all bundles plus a single
      `latest.json` whose `platforms` covers linux-x86_64, darwin-aarch64,
      darwin-x86_64 and windows-x86_64, and that
      `curl -sI https://github.com/minodisk/riffle/releases/latest/download/latest.json`
      redirects (302) to that asset. Also confirm the following release-please
      run on `main` opens the next release PR with a correct changelog base.

## Trade-offs and risks

- `releaseId` was chosen over `tagName` + `releaseDraft: true` (user decision):
  it never falls into tauri-action's create-a-release path, at the cost of
  relying on an `id` output present in release-please-action's code but absent
  from its README table.
- If any build job fails, the release stays a draft and "Latest" keeps
  pointing at the previous complete release. That is intended; fixing the
  build and re-running the failed jobs (then the `publish` job) completes the
  release, or `gh release edit <tag> --draft=false --latest` by hand.
- The `publish` job uses `GITHUB_TOKEN`, so publishing does not trigger
  `release:`-event workflows — none exist today.
- The release's `published_at` becomes the publish time rather than the merge
  time; no consumer depends on it.

## Progress

- (none yet)
