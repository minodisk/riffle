<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Record pending manual checks in todo.md at wrap-up

## Purpose

Some acceptance criteria can only be met by a hands-on check on a real
machine ("Manually verified on Windows ...", the macOS `cfg` branch of a menu
build, a native dialog). The implementer cannot do them, so it ticks the step
on the automated criteria and notes the check as pending. Twice in a row
(`strip-keep-scroll-on-rescan`, #528, and `view-menu`, #545) that note lived
only in the PR body and in a free-form "Pending: ..." bullet of
`learnings.md`; `learnings-extractor` and `todo-curator` both read it as a
one-off task and dropped it, and the user had to add the View menu item to
`todo.md` by hand (`### App: real-device checks for the View menu`, commit
`f626157` on `feature/view-menu`).

After this work the `develop` skill's wrap-up (3.3 in normal mode, S.5 in
single-PR mode) records every such check in `todo.md` as its own item — what
to verify, on which platform, the steps, and the archived plan path — through
`todo-curator`'s addition proposals, applied like any other addition and
enumerated in the 3.6 / S.6 PR body. Nothing pending is left only in a PR
body or `learnings.md`. The implementer contract gains the matching source:
a check it could not do is recorded under the fixed
`## Deferred issues (todo candidates)` heading, where the curator picks it up
mechanically instead of on a best-effort basis.

The source of the miss: `todo-curator.md` §4 only searches the fixed
`## Deferred issues (todo candidates)` heading mechanically and treats
anything outside it as best-effort; both features wrote the pending check
under `## Step 1` (last bullet of
`../_archived/20260928-strip-keep-scroll-on-rescan/learnings.md`, and the
same shape in view-menu's `learnings.md`), and `implementer.md` step 6 only
tells the implementer to put out-of-scope issues and unaddressed review
feedback under that heading, not an acceptance criterion it could not verify.

## Steps

- [x] Step 1: Make the wrap-up record every pending manual check as a `todo.md` item
  - Done when:
    - `.claude/agents/todo-curator.md`:
      - Its description / preamble says addition proposals also cover the
        manual checks a feature left pending, alongside the deferred issues.
      - §1 ("Read the sources") says that while reading `plan.md` it lists
        every Done-when line that requires a hands-on check on a real
        machine or platform (wording such as "Manually verified on
        Windows", "verify by hand", "on macOS confirm", a native dialog, a
        `cfg` branch not built here) and, from `learnings.md`, every check
        recorded as pending or "left to the user" — whether under
        `## Deferred issues (todo candidates)` or anywhere else in the
        file. A check is unmet unless `plan.md` / `learnings.md` /
        `Progress` states it was run and passed.
      - §4 ("Write the addition proposals") has an explicit subsection for
        **pending manual checks**: **every** unmet manual-check criterion
        from `plan.md` and every pending check from `learnings.md` becomes
        an addition proposal. **Never classify one as a one-off task and
        drop it, and never return it as "deferred" instead of an addition**
        (the reason to state: the check cannot be run by an agent, so the
        only place it survives is `todo.md`). Each such proposal's pasteable
        Markdown must contain: a platform-neutral `### {area}: ...` heading
        (the platform goes in the TODO lines, so an item can grow a second
        platform later); a Background that names the feature, what CI /
        tests cover and what was never exercised, the archived plan path
        written as a bare path in the post-3.5 form
        `docs/plans/_archived/YYYYMMDD-{feature-name}/plan.md` (the folder
        is archived in 3.5 / S.5 before the PR is created, so that path is
        the one that will exist; the unarchived path would go stale), and a
        `Files:` line; and `#### TODO` with one `- [ ]` per platform,
        starting "On {platform}, ...", giving the concrete steps and the
        expected result, copied from the Done-when text rather than
        paraphrased into a vaguer form. Point at the existing
        `### App: real-device checks for the View menu` and
        `### App: the wait-for-scan manual checks for Move Rejected to Trash
        and Rename are still open` items as the shape to match.
      - The same subsection says: if `todo.md` already has an item covering
        the same check (search the headings and the `#### TODO` lines for
        the feature name, the archived plan path, and the concrete handles —
        the way `strip-keep-scroll-on-rescan`'s Windows check is already a
        line of `### App: Windows real-device check of the merged
        folder-tree, scan-wait and strip-scroll work`), do not file a
        second one: propose merging only the missing platform / step lines
        into it, as a partial-edit proposal with the edited Markdown.
      - The Output section lists pending manual checks as a named part of
        the addition proposals (so the caller can enumerate them in the PR
        body), and says "0 manual checks" when there are none.
    - `.claude/skills/develop/SKILL.md`:
      - 3.3: after the `PROPOSED` branch, one short paragraph: the
        additions include one item per manual check the feature left
        pending (an unmet "verified by hand on ..." Done-when line, or a
        check `learnings.md` records as pending on the user); the main
        agent applies them like any other addition, and a check that only
        appears in a PR body or `learnings.md` is not filed.
      - 3.6: the "always include" list gains a bullet: the todo headings
        added for **pending manual checks** (or the existing heading they
        were merged into), so a human can see after the fact that no
        hands-on check was lost.
      - S.5 item 2 (`todo-curator`): add the same sentence as 3.3 in
        parenthetical form (pending manual checks become todo items here
        too).
      - S.6: the "as in 3.6, always include" enumeration gains "the todo
        headings added or extended for pending manual checks".
      - Nothing else in SKILL.md changes (no new phase, no new script).
    - `.claude/agents/implementer.md`, step 6: a new sub-bullet next to
      the existing "Record issues you decided were out of scope" one: when
      a Done-when criterion needs a hands-on check on a real machine or
      platform that the implementer cannot perform (it says so explicitly:
      "this check is the user's", a platform not built here, a native
      dialog), record it under the same
      `## Deferred issues (todo candidates)` heading as a **pending manual
      check**, with what to verify, on which platform, the steps and the
      expected result, and that the step's checkbox was ticked on the
      automated criteria. Do not write it only as a free-form "Pending:"
      bullet elsewhere in `learnings.md` (that is what the two features did
      and what got dropped). The Output section's "how many items you
      recorded under `## Deferred issues (todo candidates)`" line counts
      pending manual checks separately.
    - `.claude/skills/develop/README.md`: the 3.3 and S.5 rows of the
      mapping table (and the §"todo-curator / learnings-extractor have
      neither Write nor Edit" paragraph if it enumerates what lands in
      `todo.md`) mention that pending manual checks land in `todo.md`
      through the same route. Keep it to a phrase per place; do not
      restructure the README.
    - `learnings-extractor.md` is not edited (the curator reads
      `learnings.md` directly; see Trade-offs).
    - `mise run ci` passes (Prettier formatting of the Markdown).
  - Implementation approach:
    - Surgical edits in the existing voice of each file; match the
      surrounding bullet style and line width (the files are
      Prettier-wrapped at 80 columns).
    - In `todo-curator.md`, keep the "Do not read `todo.md` in full" rule:
      the duplicate search for an existing manual-check item is
      `rg -n` on the headings and `- \[ \]` lines for the feature name /
      archived path / concrete handles, then a ranged `Read`.
    - Read `todo.md` lines 376-419 and 990-1044 and commit `f626157` on
      `origin/feature/view-menu` (`git show f626157 -- todo.md`) for the
      item shape before writing the curator's field list; do not quote
      those items in full in the agent definition, just name them.
    - The two archived / in-flight examples to cite in the agent text as the
      failure this closes: the last bullet of
      `../_archived/20260928-strip-keep-scroll-on-rescan/learnings.md` and
      view-menu's `learnings.md` (once #545 merges its plan will be under
      `docs/plans/_archived/20260928-view-menu/`; cite it by feature name and
      PR number rather than a path so the reference is not stale either way).
    - The wrap-up of this very plan is a live test: this plan has no manual
      check, so the curator should return "0 manual checks" for it.

## Trade-offs and risks

- **Where the curator's source lives.** Option A (chosen): the implementer
  writes the pending check under the fixed `## Deferred issues (todo
  candidates)` heading _and_ the curator also scans the rest of `plan.md` /
  `learnings.md` for unmet manual checks. Both are needed: the heading gives
  a mechanical source for future features, and the plan scan covers plans
  written before this change and implementers that forget. Option B (curator
  scan only, no implementer change) leaves the curator judging free text,
  which is exactly what failed twice. Option C (implementer only) would miss
  a Done-when check that no agent noted anywhere.
- **`learnings-extractor.md` untouched.** Its "record only" output is not the
  path a manual check should take (a check is not knowledge for
  `docs/agents/`), and the curator reads `learnings.md` itself, so the
  extractor does not need to know.
- **Archived path as a bare path vs. a Markdown link.** The existing items use
  a bare path in parentheses (`docs/plans/_archived/.../plan.md`); the plan
  tells the curator to use the bare path to match the file's convention and
  never depend on the link check.
- **Merging into an existing item.** A missed match yields a second item for
  the same check, which the PR body enumeration makes visible; that is the
  accepted failure mode (a lost check is invisible, a duplicate is not).
- **Ticking a step whose manual criterion is unmet.** This plan does not
  change the practice of ticking the checkbox on the automated criteria; it
  only makes the pending part survive. Changing the practice would block
  2.4 / S.5, which refuse to proceed on an unticked step, so it is out of
  scope here.

## Progress

- (2026-09-29) Step 1 complete
