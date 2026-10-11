# Results: hand check of the performance graph overlay

Checked by the user on 2026-10-11 / 12 on their Windows machine, with a debug
build of `main` after #780 (Step 3). Folders: `D:/Photos/2026/2026-10-03`
(1803 RAWs) and `D:/Photos/2026/2026-10-11` (3190 RAWs).

## (a) Overlay on, paging through a folder

- `d` showed the overlay; every page turn added one green latency bar, aligned
  in time with the CPU lines.
- During the scan's second pass (the status line at "analyzing N / 1803"), the
  system CPU line was pegged at 100 % and Riffle's line ran at about 80–90 %,
  jagged. Switching still felt instant, and the bars were about 20–100 ms
  against the latency axis maximum of 500 ms (the floor).
- Reading: Riffle is the CPU consumer, but the preview is not starved, because
  the analysis workers run below normal OS priority.

## (b) External hog running

- Hog: `D:/Photos/tests/2026-10-10-facefree/riffle-cli.exe features D:/photos/2026/2026-07-18 24`.
- The system line pegged at 100 % while Riffle's line stayed near 0 %, and the
  bars grew to about 100–200 ms, roughly 2–4× the idle-analysis case of (a).
- The graph reads "an external process" as intended.

## (c) Overlay off

- After the hog finished, `d` again hid the overlay, and Task Manager showed
  `riffle-app.exe` at about 0 % CPU: no sampler activity while off.

## Measured latencies

| Situation | Latency bars |
|-----------|--------------|
| Riffle's scan analysis running (Riffle CPU ~80–90 %, system 100 %) | ~20–100 ms |
| External 24-thread hog (Riffle CPU ~0 %, system 100 %) | ~100–200 ms |

## Decisions from the check

- **Latency bar color stays single (no threshold).** Even under the hog the
  bars stayed at or under about 200 ms, below the 500 ms axis floor, so a
  `--destructive` color above a threshold has no measured case to mark yet.
  Revisit with numbers if a slower case shows up.
- **Axis floor stays 500 ms.** All measured bars fit under it, so a quiet
  graph keeps its bars short, as intended.
- **Legend readability.** The user told the two CPU lines apart only by their
  apparent thickness ("the thin line pegs, the thick line moves jaggedly");
  the legend differed only by text color and did not say which line was which.
  Per the plan's fallback (line weight / dashes, no new colors), Riffle CPU is
  now a 2 px solid line and system CPU a 1 px dashed line, and the legend
  draws a short swatch of each line before its label. This change itself was
  not hand-checked (see `learnings.md`).
