# Learnings

## Step 1: Sony FocusMode (0x201b)

- Chose `Shot::focus_mode: Option<u8>` (raw value) over `manual_focus: bool`:
  it keeps AF-S / AF-C / DMF distinguishable for later routing and costs
  nothing extra; manual focus is `Some(0)`.
- A BYTE (type 1, count 1) value rides inline in the entry's value field; it
  is read via a small `byte()` sibling of `integer()`.
- `riffle-cli info ~/Downloads/_DSC6978.ARW` (a real ARW, AF-C shot) printed
  `focus mode: 3` next to `focus: 7008 4672 3613 1732`, confirming the parse.
- 0xb04e / 0xb042 (older bodies) are not read; those files keep `None`.
