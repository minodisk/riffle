# Learnings

## Step 1

- The rule went in as one paragraph right under the step 6 heading, covering
  both commands, so the existing per-command paragraphs (relative path, the
  sandbox-bypass reasons) stay as they were.
- No CI failures or surprises; the change is prose only and cannot be
  unit-tested. The real check is the next unattended merger run.
- The first `mise run fmt` exited with a Node.js stack trace (`requireStack`)
  and `[fmt] ERROR task failed`; an immediate re-run passed with no changes,
  so it looked transient rather than caused by the edit.
