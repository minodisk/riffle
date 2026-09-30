# Learnings

## Step 1

### Delisted bodies (docs only; the files still open)

Preview size is the JPEG `reader::read_preview` hands back, measured on the
local samples on 2026-09-30. Release years are from memory of the makers'
announcements and are all well before 2010.

| Body | Preview | Released |
|---|---|---|
| Fujifilm FinePix S2 Pro | 1440x960 | 2002 |
| Fujifilm FinePix S3 Pro | 1440x960 | 2004 |
| Fujifilm FinePix S5 Pro | 1440x960 | 2006 |
| Fujifilm DBP for GX680 | 1344x960 | 2003 |
| Fujifilm FinePix S5000 | 1280x960 | 2003 |
| Fujifilm FinePix S5500 | 1280x960 | 2004 |
| Fujifilm FinePix S7000 | 1280x960 | 2003 |
| Fujifilm FinePix S100FS | 1600x1200 | 2008 |
| Fujifilm FinePix S9500 | 1600x1200 | 2005 |
| Fujifilm FinePix S9600 | 1600x1200 | 2006 |
| Fujifilm FinePix S5200 | 1600x1200 | 2005 |
| Fujifilm FinePix S6000fd | 1600x1200 | 2006 |
| Fujifilm FinePix S6500fd | 1600x1200 | 2006 |
| Fujifilm FinePix S200EXR | 2048x1536 | 2009 |

The year check of every other listed body (README Compatibility and
`docs/cameras.md`) found none before 2010: the oldest are the FinePix X100
(announced 2010), X10, X-S1 and F550EXR (2011), then the X-Pro1 / X-E1
(2012); the listed ARW, CR3, DNG, NEF and ORF bodies are all from 2016 on.

### Support rule added mid-step

The user added a rule to `docs/cameras.md`: a body before 2010, or with an
embedded JPEG under 1280 px on the long edge, is not supported (files may
still open). Checked against the sweep: no listed body falls under 1280 px
(the smallest listed previews are 1616x1080 on some ARW bodies and
1920x1280 on the older X-series). The S5000 / S5500 / S7000 sit at exactly
1280 px, so they are delisted by year, not by size. Many sub-1280 previews
in the sweep are Adobe DNG conversions (1024x683) or phones / drones, none
of which is listed.

### README paragraph left as is

The RAF paragraph of `README.md` / `README.ja.md` ("1920x1280 or smaller on
most of them ... at most 2176x1448", "the FinePix S and HS lines") stays
true after the delisting: 25 of the 40 older listed bodies are at 1920x1280,
and the S1 (2014) keeps the S line listed.
