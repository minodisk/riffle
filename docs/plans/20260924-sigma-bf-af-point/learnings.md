# Learnings

## Step 1

- The real `Make` differs by body: the BF writes `Sigma` / `Sigma BF`, the fp L
  writes `SIGMA` / `SIGMA fp L`. That is why the make gate is a case-insensitive
  `SIGMA` prefix while the model gate is exactly `Sigma BF`.
- `README.md` already listed `SIGMA BF` under DNG as verified (the camera's
  previews already worked), so the README needed no change. `CLAUDE.md` was left
  alone too, since the change is a helper inside `arw.rs`.
- The inline `SHORT[2]` is read out of the entry's value field with
  `value.to_le_bytes()`, so it never touches `buf`. A wrong type or count gives
  `None`. `sigma_af_point` checks `count <= SIGMA_HEADER_LEN` before the range
  check, since a MakerNote entry with `count <= 4` stores its value inline
  (not as an offset into `buf`), matching the order `leica_focus_distance`
  uses. Only a MakerNote with `count > SIGMA_HEADER_LEN` whose range runs past
  the buffer is a real error.
- Manual verification (only the 11 camera originals `BF_[0-9]*.DNG`, release
  build of `riffle-cli`):
  - `riffle-cli info` prints `focus: 1000 667 x y` on all 11, and every value
    matches `exiftool -u -Sigma_0x0147`: 06156 386 323, 06180 317 263,
    06201 483 330, 06204 479 296, 06221 572 338, 06227 477 261, 06230 530 406,
    06235 532 384, 06240 552 406, 06244 527 341, 06253 497 425.
  - `riffle-cli info` on the 22 Sigma fp L `SDIM*.DNG` still prints `focus: -`.
  - `riffle-cli scan` over the 11 files: `0 errors`. The scan subcommand does
    not print per-file sharpness, so a throwaway binary in the scratchpad ran
    `scan::extract` on each file. All 11 got `Some` sharpness (552 to 2826),
    scored on the AF point window.
  - `riffle-cli focusbox` overlays: on 06180, 06221 and 06253 the window
    centers on or right next to a child's face, which supports `x/1000`,
    `y/667`. With `y/1000` the same points would land on the curtain or the
    wall above the children. On 06156 the window centers on the paper flowers
    of the float, next to the girl's face (the same inconclusive case as in the
    plan). On 06201 it sits in the gap between the cheering children. Neither is
    clearly wrong, so the constants stay at 1000x667.
- In a fresh worktree, `mise run fmt` fails with `Command "vp" not found`
  because it runs no `pnpm install` of its own. `mise run ci` does install, so
  after one `ci` run `fmt` works.
