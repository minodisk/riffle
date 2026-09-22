# Learnings

## Step 1

- The GUI manual check was not run from the subagent. Instead a throwaway
  integration test called `riffle_core::reader::read_metadata` on
  `/mnt/d/Photos/2026/2026-09-19`: `_DSC1796.ARW` gave
  `electronic_front_curtain == Some(1)` (Mechanical) and `_DSC2450.ARW` gave
  `Some(0)` (Electronic). The Leica DNG case is covered by the existing
  `a_non_sony_maker_note_is_skipped` test, now also asserting the field is `None`.
