# Learnings

## Step 1

- **main's post-merge CI lint went red on this plan's link.** The plan PR
  (#827) linked the similar-scene-groups plan at
  `../20261015-similar-scene-groups/plan.md`. Another session's PR #826
  (stop and archive similar-scene-groups) moved that folder to
  `docs/plans/_archived/20261015-similar-scene-groups/` and merged first, so
  after both merged lychee (`mise run lint`) failed on main with a missing
  file. Each PR passed on its own; the break only exists in their
  combination. Fixed in the Step 1 PR by pointing the link at
  `../_archived/20261015-similar-scene-groups/plan.md` and saying the plan
  was stopped and archived in #826. Lesson: a plan that links a sibling plan
  in flight can break when that plan is archived; prefer linking archived or
  stable targets, or re-run lint after rebasing onto a main that moved plans.
- `folderwide.md`'s pre-registration was written from file lists, the
  archived harnesses' function names and the extras TSVs' header lines only;
  no label-dependent number was computed in this step.
