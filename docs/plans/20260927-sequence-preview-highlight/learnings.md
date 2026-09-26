# Learnings

## Step 1

- `rowParts` finds the first differing character of the new time, then backs up
  to just after the preceding `:` or space, so a seconds bump highlights `59`
  and a rollover highlights `05:00` / `04:00:00`. No diff means `diff === ""`,
  and `main.ts` then adds no span.
- Writing the test file through a Python heredoc ate the escaped backslashes of
  the Windows path (`"C:\\x\\..."` became `"C:\x\..."`, which is a parse error
  in TS). Check string escapes after scripted edits.
