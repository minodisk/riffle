# Learnings: ORF old `OLYMP\0` MakerNote preview

## Step 1

- One code path covers all three headers: `header()` also returns whether the
  note's offsets are absolute, and `maker_note_preview` builds the walker with
  `base` 0 (old note, file byte order) or the note start (new notes, their own
  byte order), then passes `at - base + ifd` and adds `base` to
  `PreviewImageStart`. The walker's end stays `min(note end, buf.len())`, so
  the `cut` rules apply unchanged; the old notes' CameraSettings IFD lies
  inside the note on every sample.
- `camera_settings_preview` accepts type 7 with `count > 4` as the inline
  sub-IFD at the entry's value offset, besides type 13 / 4 count 1.
- The synthetic old-note fixture reuses `maker_note`'s CameraSettings IFD bytes
  (its tail after the 30-byte header + main IFD) and builds the file twice
  (`old_orf`) so the absolute offsets match where the note lands.
- Tooling: in the Git Bash heredoc, `\\0` / `\\x02` in a Python script written
  to modify Rust source came out as real NUL / 0x02 bytes in the file. Use the
  Edit tool (or a script file built with `chr(92)`) for escapes.

### Real samples (`D:\Photos\samples\ORF\`, `riffle-cli info` / `bench` / `scan`)

`riffle-cli info` on all 143 files before and after the change: only these
five changed, from `preview: None` to (dimensions from the SOF at the offset,
each ending with EOI; `bench` decodes preview and 1:1 on all five):

| Body  | File                          | Offset | Length | Size      |
| ----- | ----------------------------- | ------ | ------ | --------- |
| E-1   | `E-1_E_1__C106743_gredos.ORF` | 34156  | 291265 | 1280x960  |
| E-300 | `E-300_P1252148.ORF`          | 24576  | 414528 | 1600x1200 |
| E-330 | `E-330_P3307182.ORF`          | 24576  | 417920 | 1600x1200 |
| E-400 | `E-400__A270187.ORF`          | 24576  | 308009 | 1600x1200 |
| E-500 | `E-500__1010010.ORF`          | 24576  | 437066 | 1600x1200 |

- `riffle-cli scan D:\Photos\samples\ORF`: 17 errors before, 12 after. The 12
  are the `OLYMP\0` compacts without 0x2020 (C5050Z, C5060WZ, C7070WZ, C8080WZ,
  E-10, E-20, SP350, SP500UZ, SP510UZ, SP550UZ, SP565UZ, SP570UZ), as planned.
- The `OLYMPUS\0` / `OM SYSTEM` samples give the same `Embedded` as before,
  e.g. E-30 (32768, 1117755), E-M1 Mark III `__3160530` (52224, 933606),
  OM-1 `__OM17123` (15882, 1096710), XZ-10 (196608, 1074882).
