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

# Add `.github/FUNDING.yml` for the GitHub Sponsors button

## Purpose

The repository has no funding metadata, so GitHub shows no Sponsor button on
the repository page. Adding `.github/FUNDING.yml` with a single GitHub
Sponsors entry makes the button appear and point at the maintainer's
sponsorship profile. Only GitHub Sponsors is used; the other platform keys from
GitHub's template are deliberately left out.

## Steps

- [x] Step 1: Add `.github/FUNDING.yml` containing only `github: [minodisk]`
  - Done when:
    - `.github/FUNDING.yml` exists and its content is exactly the one line
      `github: [minodisk]` (plus a trailing newline); no other keys, no
      comments copied from GitHub's template
    - The file is valid YAML
    - `mise run ci` passes
    - The commit is a Conventional Commit, e.g.
      `chore: add FUNDING.yml for the GitHub Sponsors button`
  - Implementation approach:
    - `.github/` currently holds only `workflows/ci.yml` and
      `workflows/release.yml`; `FUNDING.yml` goes next to `workflows/`, at
      `.github/FUNDING.yml` (the path GitHub requires)
    - No existing tooling targets this file: `actionlint` only reads
      `.github/workflows/`, `lychee` only reads `*.md`, and the `vp fmt`
      ignore list in `vite.config.ts` does not exclude `.github/`. If
      `pnpm exec vp check` (run by `mise run lint`) reports a formatting
      change for the YAML file, accept the formatter's output as long as the
      semantics stay `github: [minodisk]`
    - No README change: the Sponsor button is rendered by GitHub itself and
      needs no documentation

## Trade-offs and risks

- Flow (`github: [minodisk]`) vs block (`github:\n  - minodisk`) sequence
  style: both are valid for GitHub. The requirement asks for the flow form; if
  the repository formatter rewrites it into block form, keep whichever the
  formatter produces so `mise run ci` stays green rather than adding a
  formatter ignore for a one-line file.
- The button is only visible once the `minodisk` GitHub Sponsors profile is
  active; the file itself cannot be verified beyond YAML validity and CI.

## Progress

- 2026-09-21: Step 1 done (added .github/FUNDING.yml)
