# Learnings: camera-support-policy

## Step 1: release-year verification (2026-10-02)

Years are announcement years (the plan's rule for bodies whose announcement
and shipping straddle a year end). All 25 Fujifilm bodies of the plan were
confirmed released before 2016 and were delisted; no other listed body turned
out to be older than 2016.

| Body | Year | Source |
|---|---|---|
| X-T1 | 2014 (Jan 27) | Wikipedia, "Fujifilm X series" |
| X-T10 | 2015 (May) | Wikipedia, "Fujifilm X series" |
| X-Pro1 | 2012 (Jan 10) | Wikipedia, "Fujifilm X series" |
| X-E2 | 2013 (Oct 18) | Wikipedia, "Fujifilm X series" |
| X-E1 | 2012 (Sep 6) | Wikipedia, "Fujifilm X series" |
| X-A2 | 2015 (Jan 15) | Wikipedia, "Fujifilm X series" |
| X-A1 | 2013 (Sep 17) | Wikipedia, "Fujifilm X series" |
| X-M1 | 2013 (Jun 25) | Wikipedia, "Fujifilm X series" |
| X100T | 2014 (Sep 10) | Wikipedia, "Fujifilm X series" |
| X100S | 2013 (Jan 7) | Wikipedia, "Fujifilm X series" |
| X30 | 2014 (Aug 26) | Wikipedia, "Fujifilm X series" |
| X20 | 2013 (Jan 7) | Wikipedia, "Fujifilm X series" |
| X10 | 2011 (Sep 1) | Wikipedia, "Fujifilm X series" |
| XF1 | 2012 (Sep 17) | Wikipedia, "Fujifilm X series" |
| XQ2 | 2015 (Jan) | Wikipedia, "Fujifilm X series" |
| XQ1 | 2013 (Oct 18) | Wikipedia, "Fujifilm X series" |
| X-S1 | 2011 (Nov, release) | Wikipedia, "Fujifilm X-S1" |
| FinePix X100 | 2010 (Photokina announcement; shipped Feb/Mar 2011) | Wikipedia, "Fujifilm FinePix" and "Fujifilm X series" |
| FinePix F770EXR | 2012 (Jan 5) | dpreview product page (fujifilm_f770exr) |
| FinePix F550EXR | 2011 (Jan 5) | dpreview product page (fujifilm_f550exr) |
| FinePix HS50EXR | 2013 (Jan 7) | dpreview product page (fujifilm_hs50exr) |
| FinePix HS33EXR | 2012 | No page of its own found; it is the regional variant of the HS30EXR, announced with it |
| FinePix HS30EXR | 2012 | digicamdb.com spec page (fujifilm_finepix-hs30exr) |
| FinePix SL1000 | 2013 (Jan 7) | dpreview product page (fujifilm_sl1000); Wikipedia's image caption says 2014, the announcement was 2013 |
| FinePix S1 | 2014 | digicamdb.com spec page (fujifilm_finepix-s1); Wikipedia "Fujifilm FinePix S1" (a 2014 bridge camera) |

2016 edge cases confirmed and kept: Nikon D500 (announced 2016-01-06,
Wikipedia), Olympus PEN-F (released January 2016, Wikipedia), Fujifilm
X-Pro2, X-E2S, X70 (2016-01-15), X-T2 (2016-07-07), X-A3 (August 2016),
X-A10 (announced December 2016; announcement year used) (all Wikipedia,
"Fujifilm X series").

## Notes

- No Japanese copy of `cameras.md` / `raw-formats.md` existed on
  `origin/main` at implementation time, so only the English files changed.
- `todo.md` is LF-only; the two sections were cut with a byte-preserving
  rewrite, so the diff is only the deleted lines.
- The phrase "older X bodies" now means the 1920x1280 X bodies (X-H1, X-T2,
  ...), which are still listed; the FinePix and 2048x1536 compact sizes are
  gone from the human docs but stay in `docs/agents/raw-metadata-parsing.md`
  as engineering knowledge, per the plan.
