# Learnings

## Step 1: keep the EXIF of a file whose extraction failed

- `scan::extract` now fails with `scan::Failure { message, orientation, shot }`
  (`shot` boxed for clippy's `result_large_err`; `Display` prints the message; `From<String>` / `From<&str>` build one with
  no metadata, which kept the many `Err("x".into())` test rows short).
  `for_each_path` became generic over the error type, so the analysis pass
  keeps `String`; the tests that call it directly needed `Ok::<_, String>`.
- `write_batch` is now one `INSERT` for both outcomes: an error row with no
  shot writes `Shot::default()`'s NULLs and a NULL orientation. `indexed_file`
  tells "no metadata" apart by that NULL orientation (an error row from before
  `EXTRACTOR_VERSION` 12 has it NULL too), so `exif` stays `None` there.
- The fixture in `a_file_whose_preview_is_not_a_jpeg_is_one_error_not_a_crash`
  fails at the thumbnail encode, after the parse, so the failure carries the
  parsed orientation and shot without the extra `read_metadata` call; a
  missing file covers the `shot: None` path.
- Frontend: `main.ts` reads `capture_time` / `exif` / `focus` from the entry
  regardless of `error`, and `filter.ts` / `burst.ts` never look at `error`,
  so nothing changed there. A failed file now also gets the meta pane's EXIF
  rows and, when its AF point was parsed, a focus mark.
- `EXTRACTOR_VERSION` went 11 -> 12 (origin/main had 11 when the branch was
  cut). If `fix/big-endian-dng` lands a bump first, take the next number.
