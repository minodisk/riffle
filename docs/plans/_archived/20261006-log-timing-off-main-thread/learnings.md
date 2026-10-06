# Learnings

## Step 1: Make `log_timing` an `async fn`

- The change is the signature and one doc comment sentence. An `async fn`
  Tauri command with no return type and owned (`String`) arguments compiles
  as is; only borrowed arguments (`&str`, `State`) would force a `Result`
  return in an `async` command.
- The first `mise run fmt` exited with a Node.js error (an uncaught
  `require` failure in the frontend formatter); re-running it passed
  unchanged, so it was a transient tool failure, not the change.
