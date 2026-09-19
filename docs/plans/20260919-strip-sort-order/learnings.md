# Learnings

## Step 1

- `sort.ts` compares file names with JavaScript `<` (UTF-16 code units). This
  matches `Path::file_name` byte order for every name without characters
  outside the BMP, which covers camera file names.
- `subsec` is compared by right-padding the shorter string with `"0"`, so
  `"5"` (.5) sorts after `"122"` (.122); a missing `subsec` counts as `.0`.
- A rating of `0` is treated as unrated, like an absent entry.
