# Learnings

## Step 1

- The planning-time line numbers (548-564 and 945-959) still held; each
  section was deleted together with its trailing blank line (548-565 and
  945-960), leaving exactly one blank line between the neighbours. Deleting
  the lower block first kept the upper block's line numbers valid in one
  `sed` call.

- The first `mise run fmt` failed inside a Node.js step (stack trace only, no
  file named); an immediate re-run passed and `mise run ci` exited 0, so it
  was transient and unrelated to the docs-only change.
