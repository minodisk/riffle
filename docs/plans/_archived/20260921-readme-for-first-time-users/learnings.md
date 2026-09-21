# Learnings

## Step 1

- The moved blocks were extracted from the pre-change README with `sed` line
  ranges, so `docs/usage.md` and `docs/performance.md` are byte-identical to
  the old text apart from headings, a lead-in line and the relative
  `CONTRIBUTING.md` link (now `../CONTRIBUTING.md`). The "To build from source"
  line was kept at the end of the Installing section in `docs/usage.md`.
- The README sharpness one-liner follows current main (AF point or, without
  one, the sharpest region), not the older draft wording.

## Step 2

- The forms set no `labels`: the repository has no dedicated labels for these
  reports, and a label that does not exist is silently dropped. The `title`
  prefixes (`[OS]`, `[Camera]`, `[Software]`) make them filterable instead.
- Validation is review-only before merge (actionlint only covers
  `workflows/`). After merge, open
  `https://github.com/minodisk/riffle/issues/new/choose` and each form once;
  result not yet recorded.

## Deferred issues (todo candidates)

- Discussions "works" report threads (OS / camera / software) do not exist yet.
  `README.md` links `https://github.com/minodisk/riffle/discussions` with
  `<!-- TODO: replace with the ... works-report thread -->` markers. Follow-up:
  enable Discussions if needed, create the three threads (needs the user's
  confirmation), and replace the links in `README.md` (and in
  `.github/ISSUE_TEMPLATE/config.yml` from Step 2). Basis: plan.md, Step 1
  Compatibility rules.

- **Verify the issue report forms on GitHub.** The three forms (`os.yml`,
  `camera.yml`, `software.yml`) and `config.yml` were only checked by YAML
  parsing and review before merge; nothing validates `ISSUE_TEMPLATE/` locally.
  Open `https://github.com/minodisk/riffle/issues/new/choose`, walk through each
  form once and confirm the fields render. Done when all three are confirmed
  working (or fixed).
