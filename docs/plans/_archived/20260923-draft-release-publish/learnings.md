# Learnings

## Step 1

- `gh release edit <tag>` resolves drafts by tag once `force-tag-creation`
  makes release-please push the tag up front; tauri-action still needs
  `releaseId` because its `getReleaseByTag` lookup skips drafts.

## Verification on the next release

After merging the next `chore(main): release x.y.z` PR:

- [ ] `gh release view vX.Y.Z --json isDraft` is `true` while the build jobs run.
- [ ] The tag exists immediately: `git ls-remote --tags origin vX.Y.Z`.
- [ ] No duplicate `vX.Y.Z` release appears.
- [ ] After the `publish` job the release is `isDraft: false`, marked Latest,
      and has all bundles plus a single `latest.json` whose `platforms` covers
      linux-x86_64, darwin-aarch64, darwin-x86_64 and windows-x86_64.
- [ ] `curl -sI https://github.com/minodisk/riffle/releases/latest/download/latest.json`
      redirects (302) to that asset.
- [ ] The following release-please run on `main` opens the next release PR
      with a correct changelog base.
