# Learnings: reject-filter-eval

## Step 1: pre-registration

- Before writing `reject.md`, only headers were read: the `scores.tsv` header
  (`folder`, `file`, then the 14 variant columns `W0` … `W0 +c1+c2`) and the
  `features/<folder>.tsv` header (which holds the raw `ear` column the rule
  filters may need). No bottom-oriented number was computed.
- The plan's "ten cue variants" count `W0+off` among them (the genre offsets
  on `W0`); `scores.tsv` has nine further cue columns. `reject.md` lists
  `W0+off` with the cue variants and names it explicitly in the genre
  criterion against `W0`, next to the other three genre columns.
- The `rule count` score is the number of the four rules of `any of` that
  fire (`eyes closed`, `eye focus low`, `face away`, `sharpness bottom 10%`);
  `not good` is not one of them, as the plan's list implies.
- Every link in `reject.md` targets `../_archived/...` (the five archived
  plans' scripts and result docs), so no in-flight plan's archive move can
  break lint here.
