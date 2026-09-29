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
- The remaining edge: a partial override applied before a later full retry
  frees one of the keys it dropped keeps the smaller set. Partial
  applications only happen once full ones have settled, so this needs two
  overrides whose conflicts chain through each other's defaults, which the
  UI cannot produce.
- The worktree-isolated Bash refuses compound commands it cannot verify
  (a heredoc plus a Python edit script); plain Edit calls work.

## Deferred issues (todo candidates)

- (none)
