# Learnings

## Step 1

- The `riffle-cli bench` crop-scaling todo item was stale: both `bench()` and
  `crop()` in `crates/cli/src/main.rs` call `partial::decode_focus_crop(.., a.shot.focus, ..)`,
  which scales sensor coordinates via `focus_point`. `partial.rs` already had a
  test for a JPEG smaller than the sensor (3504x2336 vs 7008x4672), so no new
  test was added.
- mozjpeg panics on both non-JPEG bytes and a JPEG truncated to half its
  length; `decode_rgb` now catches the panic and returns `Err`.

## Step 2

- `git fetch origin main:main` refuses (`fatal: refusing to fetch into branch ... checked out at ...`) when any worktree, including the current one, has `main` checked out. `git:main` therefore detaches first when the current worktree is on `main`, then treats a failed `main:main` fetch as a notice. Measured in a scratch clone with a second worktree holding `main`.
- `git remote set-head origin -a` needs network access to the remote; when it fails the script exits with a message naming that command (verified with an unreachable remote URL).
- `.claude/skills/pr/SKILL.md` and `.claude/skills/develop/README.md` only say `git:main` "syncs main", which stays accurate, so they were left untouched.
