# Learnings

## Step 1

- Confirmed on jq 1.6 that the new test fails before the rename: the three
  cases that reach `print_result` exit 3 with `syntax error, unexpected label,
  expecting IDENT`; only `no_runs` passed. After renaming `$label` to
  `$verdict`, all 4 cases pass and the real run against
  e63b25ae653bac6738c4eb0ae8bcbc51e4152c57 prints `STATUS=success`.
- The stub `gh` dispatches on `"$1 $2 $3"`, so a new gh call in the script
  fails the test loudly with "stub gh: unexpected arguments".

## Deferred issues (todo candidates)

- **jq reserved words as variable names in skill scripts.** jq 1.6 rejects `$label` (`label` is a keyword) with `syntax error, unexpected label, expecting IDENT`; jq 1.7+ accepts it, so CI never notices. Other keywords (`def`, `as`, `if`, `reduce`, `foreach`, `try`, `import`, `include`, `and`, `or`, `not`, ...) would fail the same way. No guide covers skill-script conventions, so there is nowhere to consolidate this yet. Done when either a guide for skill-script (shell/jq) conventions is created and carries an "avoid jq keywords as variable names" note, or the item is judged not worth a guide and closed with no action.
