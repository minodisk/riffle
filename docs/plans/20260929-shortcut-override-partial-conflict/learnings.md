# Learnings

## Step 1

- The plan assumed applying a partial override never frees a key, so no
  retry was needed after it. That is wrong: applying it replaces the action's
  current (default) keys, so e.g. `previous` taking `["w"]` from
  `["arrowup", "w"]` frees its default `arrowleft`, which a pending
  `burstFramePrevious: ["arrowleft"]` wants. `from_overrides` therefore
  applies one partial override at a time (the first in action order with a
  free key) and reruns the full-apply retry loop after each; the test
  `a_partly_applied_override_frees_its_default_for_another` covers it.
- The remaining edge, corrected in review round 1: a partial application
  can itself free a key that an *earlier* partial application dropped, and
  the UI can produce this (round 1's example: from the defaults, add `w`
  to `previous`, remove `arrowleft` from `previous`, add `arrowleft` to
  `burstPrevious`, remove `arrowup` from `burstPrevious`, add `arrowup` to
  `previous` — every step passes `check_bindable` / `remove`). Fixed by
  keeping each partly applied action's requested keys in a `partial` list
  and, once the outer loop settles, recomputing each of their keys as the
  requested keys filtered by `holder(i, key).is_none()` at that point. Test
  `a_key_freed_by_a_later_partial_application_is_recovered` covers it.
- The worktree-isolated Bash refuses compound commands it cannot verify
  (a heredoc plus a Python edit script); plain Edit calls work.

## Deferred issues (todo candidates)

- (none)
