# Learnings: todo-trash-manual-checks

## Step 1

- The plan's verification grep (`native confirmation`) also matches check
  (8) of `### App: the Clear Cache button's manual GUI verification is still open`,
  which legitimately describes Clear Cache's native confirm and is outside
  this step's scope ("no other `todo.md` item is touched"). It was left as
  is; the grep is clean for the four touched items. Item A's note on the
  earlier Windows run says "the OS's own confirm dialog" instead, so the
  touched items do not match the pattern.
- The dialog check in Item B says `Ctrl+,` rather than the plan's
  `Cmd/Ctrl+,`, since the item is Windows-only.
- The multi-folder label in `crates/app/ui/src/context.ts` is
  `Move Rejected in N Folders to Trash…`, used as is.
