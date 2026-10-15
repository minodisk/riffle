"""Candidate 2's six criterion columns per frame from `detections.tsv`
(detect_all.py), so the feature rules can change without re-running the
detector.

Usage: python -I -B detect_features.py <detections.tsv> <extras_c2.tsv> --ball <c> --person <c>

--ball is the confidence the hand labels pick for the best ball F1
(detect_eval.py), --person the detector's default confidence. Columns, in
ballgate.py's extras format (`folder\\tfile\\t<column>...`):

- `ball present`: the top `sports ball` confidence, 0 when none;
- `ball indicator`: 1 when that confidence is at or above --ball;
- `ln persons`: ln(1 + persons at or above --person);
- `contested`: 1 when a second person box overlaps the carrier's or sits
  within one carrier-box width of it (the gap between the boxes), the
  carrier being the person box containing the ball's center, else the one
  nearest it (the distance from the center to the box, then to the box's
  center); 0 otherwise, also when no ball reaches --ball or no person
  --person; the ball is the top-confidence `sports ball` box;
- `ball size`: that ball box's long side over the frame's long side when
  the ball reaches --ball, else 0;
- `ln persons x ball present`: the product of the two.
"""

import csv
import math
import sys

COLUMNS = ("ball present", "ball indicator", "ln persons", "contested", "ball size", "ln persons x ball present")


def gap(a, b):
    dx = max(0.0, a[0] - b[2], b[0] - a[2])
    dy = max(0.0, a[1] - b[3], b[1] - a[3])
    return math.hypot(dx, dy)


def features(width, height, dets, ball_conf, person_conf):
    balls = sorted((d for d in dets if d[0] == "sports ball"), key=lambda d: -d[1])
    persons = [d[2:] for d in dets if d[0] == "person" and d[1] >= person_conf]
    present = balls[0][1] if balls else 0.0
    has_ball = present >= ball_conf and present > 0
    ln_persons = math.log1p(len(persons))
    contested = 0.0
    size = 0.0
    if has_ball:
        bx0, by0, bx1, by1 = balls[0][2:]
        size = max(bx1 - bx0, by1 - by0) / max(width, height)
        cx, cy = (bx0 + bx1) / 2, (by0 + by1) / 2
        if persons:
            point = (cx, cy, cx, cy)
            carrier = min(
                range(len(persons)),
                key=lambda i: (gap(point, persons[i]), math.hypot((persons[i][0] + persons[i][2]) / 2 - cx, (persons[i][1] + persons[i][3]) / 2 - cy)),
            )
            c = persons[carrier]
            reach = c[2] - c[0]
            contested = float(any(gap(c, p) <= reach for i, p in enumerate(persons) if i != carrier))
    return [present, float(has_ball), ln_persons, contested, size, ln_persons * present]


def main(argv):
    opts = {"--ball": None, "--person": None}
    pos, it = [], iter(argv)
    for a in it:
        if a in opts:
            opts[a] = next(it)
        else:
            pos.append(a)
    if len(pos) != 2 or None in opts.values():
        raise SystemExit(__doc__)
    src, out = pos
    ball_conf, person_conf = float(opts["--ball"]), float(opts["--person"])
    frames = {}
    with open(src, newline="", encoding="utf-8") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            key = (r["folder"], r["file"])
            entry = frames.setdefault(key, (int(r["width"]), int(r["height"]), []))
            if r["class"] != "-":
                entry[2].append((r["class"], float(r["confidence"]), float(r["x0"]), float(r["y0"]), float(r["x1"]), float(r["y1"])))
    sums = [0.0] * len(COLUMNS)
    with open(out, "w", encoding="utf-8", newline="") as f:
        f.write("folder\tfile\t" + "\t".join(COLUMNS) + "\n")
        for (folder, file), (w, h, dets) in sorted(frames.items()):
            v = features(w, h, dets, ball_conf, person_conf)
            sums = [s + x for s, x in zip(sums, v)]
            f.write(f"{folder}\t{file}\t" + "\t".join(f"{x:.6g}" for x in v) + "\n")
    print(f"{len(frames)} frames; column means: " + ", ".join(f"{n} {s / len(frames):.4f}" for n, s in zip(COLUMNS, sums)))


if __name__ == "__main__":
    main(sys.argv[1:])
