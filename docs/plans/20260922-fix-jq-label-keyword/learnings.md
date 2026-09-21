# Learnings

## Step 1

- Confirmed on jq 1.6 that the new test fails before the rename: the three
  cases that reach `print_result` exit 3 with `syntax error, unexpected label,
  expecting IDENT`; only `no_runs` passed. After renaming `$label` to
  `$verdict`, all 4 cases pass and the real run against
  e63b25ae653bac6738c4eb0ae8bcbc51e4152c57 prints `STATUS=success`.
- The stub `gh` dispatches on `"$1 $2 $3"`, so a new gh call in the script
  fails the test loudly with "stub gh: unexpected arguments".
