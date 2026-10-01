# Learnings

## Step 1

### Uuid-less `.dop` measurement (PhotoLab 10.0.1, Windows, 2026-10-01)

- Test folder `D:\Photos\tests\dop-uuid-test` with three copies of one
  DSC-RX100M4 ARW: `A_omit`, `B_empty`, `C_random`.
- The folder was opened once in PhotoLab so all three were registered (the
  database's `Items` rows dated 00:11:04Z) with no `.dop` written, then
  PhotoLab was closed.
- A fresh `.dop` from `dop.rs`'s template was placed next to each, with
  `Date` / `CreationDate` / `ModificationDate` stamped 00:12:05Z (newer than
  the database items), ratings 5 / 4 / 3 and `ShouldProcess` 0:
  - A: the `Items[0].Uuid` and `Source.Uuid` lines omitted.
  - B: both Uuids `""`.
  - C: random Uuids (control).
- PhotoLab was reopened on the folder:
  - A: a virtual copy; a new `Items` row with Uuid
    `00000000-0000-0000-0000-000000000000` carrying rating 5; the master
    stayed unrated.
  - B: no virtual copy, the master stayed unrated, the database unchanged.
  - C: a virtual copy whose `Items` row Uuid is the `.dop`'s item Uuid,
    reproducing the documented behaviour.
- PhotoLab rewrote none of the three `.dop` files.
- Conclusion: a Uuid-less `.dop` does not sidestep `photolab.rs`'s lookup;
  the database's Uuids remain the only fix, so `photolab.rs` stays.

### Move and references

- The four sections moved verbatim, `###` raised to `##` since the new guide
  has no "Rust side" / "Frontend" split; `tauri-app.md` lost 112 lines and
  gained a 4-line `###` pointer section in their place.
- `CLAUDE.md`'s Layout paragraph got the parenthetical
  "see `docs/agents/photolab.md`" after the `src/photolab.rs` description
  (kept rather than relying on the `tauri-app.md` pointer alone).
- Reference grep before the PR: `tauri-app.md#` finds only
  `raw-metadata-parsing.md`'s link to the Sony MakerNote section (not moved);
  the four heading texts appear only in `photolab.md` and this plan; no
  `.claude/**` file or code comment names `tauri-app.md` together with
  PhotoLab / `.dop`. No other reference needed fixing.
- Lightroom / Lightroom Classic: the user asked mid-planning to split their
  knowledge out of `docs/agents` too; none exists there, so nothing was
  created.
- The first `mise run ci` failed on lychee: `plan.md` quoted the guide's
  `./tauri-app.md` / `./photolab.md` links, which resolve from the plan
  folder. They now point at `../../agents/`. Also, this worktree had no
  `node_modules`, so `mise run fmt` failed until
  `mise exec -- pnpm install --frozen-lockfile` (bare `pnpm` is not on PATH).
