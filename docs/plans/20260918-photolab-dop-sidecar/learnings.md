# Learnings

## Step 1: Core `.dop` read, patch and template

- `write_rating` takes a fourth parameter, the file `name`, besides the
  existing bytes, the rating and the timestamp: the fresh template carries
  `Name = "<file name>"`, and the core cannot derive it from the bytes. It is
  ignored when patching. Signature:
  `write_rating(existing: Option<&[u8]>, rating: Option<i8>, name: &str, now: &str)`.
- Timestamp: hand-rolled `civil_from_days` (Howard Hinnant's algorithm) in
  `dop::timestamp(SystemTime)`, tested against the sample's instant, a leap
  day and the epoch. No `time` crate.
- UUIDs: generated from two `RandomState::hash_one` calls (version and variant
  bits set), no `uuid` crate. Good enough for PhotoLab's opaque identifiers;
  not cryptographic.
- The samples end with `}\n\r\n` (an empty last line with CRLF), not `}\r\n`.
- The fixtures under `crates/core/src/fixtures/dop/` are marked `-text` in a
  new root `.gitattributes`, so a Windows checkout with `core.autocrlf` cannot
  rewrite their line endings and break the byte-identity tests.
- Trimmed fixtures (0001, 0002, 0003, 0005) are the sample's lines 1-28, the
  first `HSLHueSlices` entry (with the `Label = "Red"` decoy) and lines 544 to
  the end; 0004 is the full file.
- A missing key is inserted at the start of the line holding the table's
  closing `}`; if that `}` shares its line with other content the write is an
  `Err` rather than guessing a layout.
