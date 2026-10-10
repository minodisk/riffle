"""Before / after comparison of the eye-line roll correction (mesh-roll Step 2).

Reads the `riffle-cli meshfit` TSVs of Step 1 (re-dumped in Step 2 with the
three YuNet eye-geometry columns), the misfit truth set (`sample.tsv`,
`labels.tsv` next to its sheets), the closed-eyes and head-pose truth sets of
the archived plans and the 60 starred good-mark frames, and prints the tables
of `compare.md`. Pure Python 3, no dependencies; nothing is fitted here unless a
frozen number drops (see `compare.md`).

    python compare.py [tests root, default D:/Photos/tests]

A variant decides per face whether the `after` (rotated) fit replaces the
`before` one; every value of the chosen fit is read from the same TSV row, so
a variant is a choice between two columns, not a re-run.
"""

import csv
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else "D:/Photos/tests")
DUMP = ROOT / "2026-10-09-mesh-roll" / "dump"
SHEETS = ROOT / "2026-10-09-mesh-roll" / "sheets"
STARS = ROOT / "2026-10-09-good-mark" / "samples-scored.tsv"
ARCHIVED = Path(__file__).resolve().parent.parent / "_archived"
EYES_TRUTH = ARCHIVED / "20261007-closed-eyes-detection" / "eyes-truth.md"
POSE_TRUTH = ARCHIVED / "20261007-head-pose" / "pose-truth.md"

SIX = ["2026-07-11", "2026-07-24", "2026-08-08", "2026-09-19", "2026-09-27-a", "2026-10-03"]
TRAIN = ["2026-06-05", "2026-07-31", "2026-09-13-a", "2026-09-19", "2026-09-19-focus-sample-2"]
HELD = ["2026-06-14", "2026-07-18", "2026-08-01", "2026-08-22"]
CANDIDATE_LOGIT = 1.2194
EYES_CLOSED_EAR = 0.137
MAX_EYE_OFFSET = 0.10
EYES_MIN_FACE = 60.0


def num(s):
    try:
        return float(s)
    except (TypeError, ValueError):
        return None


def load(name):
    rows = []
    for r in csv.DictReader(open(DUMP / f"{name}.tsv", encoding="utf-8"), delimiter="\t"):
        if r["side"] in ("-", "err"):
            continue
        for k in list(r):
            if k not in ("folder", "file", "xmp", "af") and not k.endswith(("_state", "_scored")):
                r[k] = num(r[k])
        rows.append(r)
    return rows


def focus_set(name):
    return load(name if name.endswith("-2") else f"{name}-focus-sample")


# --- variants -----------------------------------------------------------------


def two_pass_yaw(r, band, limit):
    return abs(r["yunet_roll"]) >= band and r["before_yaw"] is not None and abs(r["before_yaw"]) < limit


VARIANTS = {
    "before": lambda r: False,
    "always": lambda r: True,
    "band 10": lambda r: abs(r["yunet_roll"]) >= 10,
    "band 20": lambda r: abs(r["yunet_roll"]) >= 20,
    "band 10-25": lambda r: 10 <= abs(r["yunet_roll"]) <= 25,
    "wide 0.15": lambda r: r["yunet_eye_dist"] >= 0.15,
    "wide 0.20": lambda r: r["yunet_eye_dist"] >= 0.20,
    "wide 0.25": lambda r: r["yunet_eye_dist"] >= 0.25,
    "band 10 + wide 0.20": lambda r: abs(r["yunet_roll"]) >= 10 and r["yunet_eye_dist"] >= 0.20,
    "band 10 + wide 0.25": lambda r: abs(r["yunet_roll"]) >= 10 and r["yunet_eye_dist"] >= 0.25,
    "band 10 + yaw 45 (2 pass)": lambda r: two_pass_yaw(r, 10, 45),
}
GUARD_GRID = [0.10, 0.15, 0.20, 0.25, 0.30]


def pick(r, variant):
    """The fit of `r` the variant keeps: a dict of the unprefixed columns."""
    p = "after_" if VARIANTS[variant](r) else "before_"
    return {k[len(p):]: v for k, v in r.items() if k.startswith(p)}


# --- buckets ------------------------------------------------------------------


def roll_bucket(r):
    a = abs(r["yunet_roll"])
    return "roll <10" if a < 10 else "roll 10-25" if a <= 25 else "roll >25"


def yaw_bucket(r):
    y = r["before_yaw"]
    if y is None:
        return "yaw none"
    a = abs(y)
    return "yaw <30" if a < 30 else "yaw 30-60" if a < 60 else "yaw >=60"


def side_bucket(r):
    s = r["side"]
    return "side <60" if s < 60 else "side 60-99" if s < 100 else "side 100-199" if s < 200 else "side >=200"


def offset_bucket(o):
    if o is None:
        return "offset -"
    return "<0.06" if o < 0.06 else "0.06-0.10" if o <= 0.10 else "0.10-0.20" if o <= 0.20 else ">0.20"


SPLITS = [("roll", roll_bucket), ("yaw", yaw_bucket), ("side", side_bucket)]


def auc(scored):
    pos = [s for p, s in scored if p]
    neg = [s for p, s in scored if not p]
    if not pos or not neg:
        return None
    t = sum(1.0 if a > b else 0.5 if a == b else 0.0 for a in pos for b in neg)
    return t / (len(pos) * len(neg))


def fmt(v, pct=False):
    if v is None:
        return "-"
    return f"{v:.1%}" if pct else f"{v:.3f}"


def table(head, rows):
    print("| " + " | ".join(head) + " |")
    print("|" + "|".join("---" for _ in head) + "|")
    for r in rows:
        print("| " + " | ".join(str(c) for c in r) + " |")
    print()


# --- the six folders ----------------------------------------------------------


def six_rows():
    out = []
    for name in SIX:
        for r in load(name):
            if r["af"] == "af" and r["side"] >= EYES_MIN_FACE and r["before_eye_offset"] is not None:
                out.append(r)
    return out


def over(rows, variant):
    return sum(pick(r, variant)["eye_offset"] > MAX_EYE_OFFSET for r in rows)


def report_six(rows):
    print("## Six folders: share over MAX_EYE_OFFSET (AF, side >= 60, meshed)\n")
    n = len(rows)
    base = over(rows, "before")
    out = []
    for v in VARIANTS:
        o = over(rows, v)
        fixed = sum(
            r["before_eye_offset"] > MAX_EYE_OFFSET and pick(r, v)["eye_offset"] <= MAX_EYE_OFFSET for r in rows
        )
        broke = sum(
            r["before_eye_offset"] <= MAX_EYE_OFFSET and pick(r, v)["eye_offset"] > MAX_EYE_OFFSET for r in rows
        )
        rotated = sum(VARIANTS[v](r) for r in rows)
        out.append([v, f"{o} ({o / n:.1%})", fixed, broke, f"{(base - o) / base:+.1%}", rotated])
    table(["variant", f"over 0.10 of {n}", "fell under", "rose over", "net recovered of the before share", "rotated"], out)
    print("By `yunet_roll` and by YuNet eye distance (before -> always / band 10 + wide 0.20):\n")
    out = []
    for name, f in [("roll", roll_bucket), ("eye dist", eye_dist_bucket)]:
        for b in sorted({f(r) for r in rows}):
            rs = [r for r in rows if f(r) == b]
            out.append([b, len(rs)] + [f"{over(rs, v) / len(rs):.1%}" for v in ("before", "always", "band 10 + wide 0.20")])
    table(["bucket", "n", "before", "always", "band 10 + wide 0.20"], out)


def eye_dist_bucket(r):
    d = r["yunet_eye_dist"]
    return "eye dist <0.10" if d < 0.10 else "eye dist 0.10-0.20" if d < 0.20 else "eye dist >=0.20"


# --- the misfit truth set -----------------------------------------------------


def truth_rows():
    index = {}
    for name in SIX:
        for r in load(name):
            index[(r["folder"], r["file"])] = r
    labels = {}
    for line in open(SHEETS / "labels.tsv", encoding="utf-8"):
        i, before, after, yunet = line.rstrip("\n").split("\t")
        labels[int(i)] = (before, after, yunet)
    out = []
    for line in open(SHEETS / "sample.tsv", encoding="utf-8"):
        i, folder, file, stratum = line.rstrip("\n").split("\t")
        r = dict(index[(folder, file)])
        r["label_before"], r["label_after"], r["yunet"] = labels[int(i)]
        r["stratum"] = stratum
        out.append(r)
    return out


def kind(label):
    return label.split(":")[1] if label.startswith("off:") else label


def report_truth(rows, six):
    print(f"## Misfit truth set ({len(rows)} frames)\n")
    strata = sorted(Counter(r["stratum"] for r in rows).items())
    print("Strata: " + ", ".join(f"`{k}` {v}" for k, v in strata) + "\n")
    out = []
    for k in ["on", "rotated", "side", "small", "occluded", "cut", "notface", "other", "x"]:
        out.append([k, sum(kind(r["label_before"]) == k for r in rows), sum(kind(r["label_after"]) == k for r in rows)])
    table(["label", "before", "after (always)"], out)
    yunet = sorted(Counter(r["yunet"] for r in rows).items())
    print("YuNet's eye points: " + ", ".join(f"`{k}` {v}" for k, v in yunet) + "\n")
    readable = [r for r in rows if r["label_before"] != "x"]
    print(f"Share of `off` among the {len(readable)} readable frames, `before` mesh:\n")
    out = []
    groups = [("offset " + offset_bucket(r["before_eye_offset"]), r) for r in readable]
    groups += [(side_bucket(r), r) for r in readable]
    groups += [(roll_bucket(r), r) for r in readable]
    groups += [(eye_dist_bucket(r), r) for r in readable]
    for b in dict.fromkeys(g for g, _ in groups):
        rs = [r for g, r in groups if g == b]
        off = [r for r in rs if r["label_before"] != "on"]
        artifacts = sum(r["label_before"] == "on" and r["yunet"] == "off" for r in rs)
        out.append([b, len(rs), f"{len(off)} ({len(off) / len(rs):.0%})", artifacts])
    table(["bucket", "n", "off", "on with `yunet off`"], out)
    hi = [r for r in readable if r["before_eye_offset"] > MAX_EYE_OFFSET]
    print(
        f"Over 0.10: {len(hi)} readable, {sum(r['label_before'] == 'on' for r in hi)} `on` "
        f"({sum(r['label_before'] == 'on' and r['yunet'] == 'off' for r in hi)} of them with `yunet off`); "
        f"at or under 0.10: {len(readable) - len(hi)}, "
        f"{sum(r['label_before'] != 'on' for r in readable if r not in hi)} `off`.\n"
    )
    print("Per variant over the readable frames (label of the fit the variant keeps):\n")
    out = []
    for v in VARIANTS:
        keep = [r["label_after"] if VARIANTS[v](r) else r["label_before"] for r in readable]
        on = sum(k == "on" for k in keep)
        fixed = sum(r["label_before"] != "on" and k == "on" for r, k in zip(readable, keep))
        broke = sum(r["label_before"] == "on" and k != "on" for r, k in zip(readable, keep))
        out.append([v, f"{on} ({on / len(readable):.0%})", fixed, broke])
    table(["variant", "on", "off -> on", "on -> off"], out)
    report_weighted(rows, six)
    print("Frames whose label changed under `always`:\n")
    out = []
    for r in readable:
        if r["label_before"] != r["label_after"]:
            out.append(
                [
                    f"`{r['folder']}_{r['file'][:-4]}`",
                    r["label_before"],
                    r["label_after"],
                    r["yunet"],
                    f"{r['yunet_roll']:+.0f}",
                    f"{r['yunet_eye_dist']:.2f}",
                    "-" if r["before_yaw"] is None else f"{r['before_yaw']:+.0f}",
                    f"{r['before_eye_offset']:.3f} -> {r['after_eye_offset']:.3f}",
                ]
            )
    table(["frame", "before", "after", "YuNet", "yunet_roll", "eye dist", "before yaw", "eye_offset"], out)


def cell(r):
    return f"off{offset_bucket(r['before_eye_offset'])}|{roll_bucket(r).replace(' ', '')}"


def report_weighted(rows, six):
    """The twelve offset x roll strata weighted back to the six folders (the
    random and tilted-wide slices left out), unreadable frames dropped."""
    population = Counter(cell(r) for r in six)
    drawn = [r for r in rows if r["stratum"] not in ("random", "tilted-wide") and r["label_before"] != "x"]
    est = Counter()
    for c, n in population.items():
        rs = [r for r in drawn if cell(r) == c]
        if not rs:
            continue
        w = n / len(rs)
        hi = c.startswith(("off0.10-0.20", "off>0.20"))
        for r in rs:
            est["all"] += w
            est["hi"] += w * hi
            if r["label_before"] != "on":
                est["off"] += w
                est["offhi"] += w * hi
                est["kind " + kind(r["label_before"])] += w
            if r["label_before"] == "on" and r["yunet"] == "off":
                est["artifacthi"] += w * hi
            if r["label_before"] != "on" and r["label_after"] == "on":
                est["fixed"] += w
            if r["label_before"] == "on" and r["label_after"] != "on":
                est["broken"] += w
    kinds = sorted((k for k in est if k.startswith("kind ")), key=lambda k: -est[k])
    print("Weighted to the six folders (the twelve strata only, readable frames):\n")
    print(f"- `before` off the face: {est['off'] / est['all']:.1%} of the meshed AF frames")
    print(
        f"- of the frames over 0.10: {est['offhi'] / est['hi']:.0%} off, "
        f"{est['artifacthi'] / est['hi']:.0%} on with `yunet off`"
    )
    print("- off frames by kind: " + ", ".join(f"{k[5:]} {est[k] / est['off']:.0%}" for k in kinds))
    print(
        f"- `always`: {est['fixed'] / est['all']:.1%} of the frames off -> on, "
        f"{est['broken'] / est['all']:.1%} on -> off\n"
    )


# --- AF eye on the focus-sample folders --------------------------------------


def focus_rows(names):
    out = []
    for name in names:
        for r in focus_set(name):
            if r["xmp"] in ("Pick", "Reject") and r["before_logit"] is not None and r["after_logit"] is not None:
                out.append(r)
    return out


def af_metrics(rows, variant):
    fits = [(r["xmp"] == "Pick", pick(r, variant)) for r in rows]
    a = auc([(p, f["logit"]) for p, f in fits])
    cand = [(p, f["state"] == "Candidate") for p, f in fits]
    n_c = sum(c for _, c in cand)
    hits = sum(c and p for p, c in cand)
    picks = sum(p for p, _ in cand)
    return a, hits / max(n_c, 1), hits / max(picks, 1), n_c, hits, picks


def report_af(train, held):
    print(f"## AF eye ({len(train)} training / {len(held)} held-out labeled frames)\n")
    out = []
    for v in VARIANTS:
        t, h = af_metrics(train, v), af_metrics(held, v)
        out.append(
            [v, fmt(t[0]), fmt(t[1], 1), fmt(t[2], 1), fmt(h[0]), f"{h[1]:.1%} ({h[4]}/{h[3]})", f"{h[2]:.1%} ({h[4]}/{h[5]})"]
        )
    table(["variant", "train AUC", "train prec", "train cov", "held AUC", "held prec", "held cov"], out)
    print("Guard grid on the training rows (`band 10 + wide T`): train AUC, frames rotated\n")
    out = []
    for t in GUARD_GRID:
        VARIANTS["_grid"] = lambda r, t=t: abs(r["yunet_roll"]) >= 10 and r["yunet_eye_dist"] >= t
        m = af_metrics(train, "_grid")
        out.append([f"{t:.2f}", fmt(m[0]), fmt(m[1], 1), fmt(m[2], 1), sum(VARIANTS["_grid"](r) for r in train)])
    del VARIANTS["_grid"]
    table(["T", "train AUC", "train prec", "train cov", "rotated"], out)
    print("Held-out split (AUC / precision / coverage; before -> always -> band 10 + wide 0.20):\n")
    out = []
    for name, f in SPLITS:
        for b in sorted({f(r) for r in held}):
            rs = [r for r in held if f(r) == b]
            cells = []
            for v in ("before", "always", "band 10 + wide 0.20"):
                m = af_metrics(rs, v)
                cells.append(f"{fmt(m[0])} / {m[1]:.0%} / {m[2]:.0%}")
            out.append([b, len(rs), sum(r["xmp"] == "Pick" for r in rs)] + cells)
    table(["bucket", "n", "picks", "before", "always", "band 10 + wide 0.20"], out)
    print("Training split (same columns):\n")
    out = []
    for name, f in SPLITS:
        for b in sorted({f(r) for r in train}):
            rs = [r for r in train if f(r) == b]
            cells = []
            for v in ("before", "always", "band 10 + wide 0.20"):
                m = af_metrics(rs, v)
                cells.append(f"{fmt(m[0])} / {m[1]:.0%} / {m[2]:.0%}")
            out.append([b, len(rs), sum(r["xmp"] == "Pick" for r in rs)] + cells)
    table(["bucket", "n", "picks", "before", "always", "band 10 + wide 0.20"], out)


# --- closed eyes --------------------------------------------------------------


def eyes_truth():
    text = EYES_TRUTH.read_text(encoding="utf-8")
    block = text.split("## All labeled faces", 1)[1]
    out = {}
    for m in re.finditer(r"^(\S+\.(?:ARW|DNG))\s+(\d+)px\s+\S+\s+([ocux]) ([ocux])", block, re.M):
        eyes = m.group(3) + m.group(4)
        closed = "c" in eyes and "o" not in eyes
        opened = "o" in eyes and "c" not in eyes
        if closed or opened:
            out[m.group(1)] = (closed, float(m.group(2)))
    return out


def labeled_faces(truth):
    rows = {}
    for name in ("2026-09-19", "2026-02-01"):
        for r in load(name):
            if r["file"] in truth:
                rows[r["file"]] = r
    return rows


def report_eyes():
    truth = eyes_truth()
    rows = labeled_faces(truth)
    missing = sorted(set(truth) - set(rows))
    side_off = sum(abs(rows[f]["side"] - truth[f][1]) > 2 for f in rows)
    print(
        f"## Closed eyes ({len(truth)} labeled faces, {sum(c for c, _ in truth.values())} closed; "
        f"{len(rows)} found in the dump, {side_off} with a face side more than 2 px off the label's)\n"
    )
    if missing:
        print("Not in the dump:", missing, "\n")
    data = [(truth[f][0], r) for f, r in rows.items()]

    def metrics(data, v):
        fits = [(c, pick(r, v)["ear"]) for c, r in data]
        fits = [(c, e) for c, e in fits if e is not None]
        a = auc([(c, -e) for c, e in fits])
        acc = sum(c == (e <= EYES_CLOSED_EAR) for c, e in fits) / max(len(fits), 1)
        tp = sum(c and e <= EYES_CLOSED_EAR for c, e in fits)
        pp = sum(e <= EYES_CLOSED_EAR for c, e in fits)
        pos = sum(c for c, _ in fits)
        return a, acc, tp / max(pp, 1), tp / max(pos, 1), len(fits), sum(c != (e <= EYES_CLOSED_EAR) for c, e in fits)

    out = []
    for v in VARIANTS:
        a, acc, p, rc, n, wrong = metrics(data, v)
        out.append([v, fmt(a), fmt(acc), fmt(p), fmt(rc), n, wrong])
    table(["variant", "AUC", "accuracy", "precision", "recall", "n", "wrong"], out)
    print("Split (AUC / accuracy; before -> always -> band 10 + wide 0.20):\n")
    out = []
    for name, f in SPLITS:
        for b in sorted({f(r) for _, r in data}):
            d = [(c, r) for c, r in data if f(r) == b]
            cells = []
            for v in ("before", "always", "band 10 + wide 0.20"):
                m = metrics(d, v)
                cells.append(f"{fmt(m[0])} / {m[1]:.3f}")
            out.append([b, len(d), sum(c for c, _ in d)] + cells)
    table(["bucket", "n", "closed", "before", "always", "band 10 + wide 0.20"], out)
    return data


# --- head pose ----------------------------------------------------------------


def pose_truth():
    out = {}
    for m in re.finditer(
        r"^\| \d+ \| `(\S+)` \| \S+ \| \d+ \| (\w+) \| (\w*) \| (\w+) \| (\w+) \|$",
        POSE_TRUTH.read_text(encoding="utf-8"),
        re.M,
    ):
        if m.group(2) != "x":
            out[m.group(1)] = (m.group(2), m.group(3), m.group(4), m.group(5))
    return out


def yaw_class(y):
    a = abs(y)
    return "frontal" if a < 15 else "oblique" if a < 50 else "profile"


def pose_metrics(data, v):
    fits = [(lab, pick(r, v)) for lab, r in data]
    fits = [(lab, f) for lab, f in fits if f["yaw"] is not None]
    ys = [(lab[1] == "right") == (f["yaw"] > 0) for lab, f in fits if lab[1] in ("left", "right")]
    ps = [(lab[2] == "up") == (f["pitch"] > 0) for lab, f in fits if lab[2] in ("up", "down")]
    rs = [(lab[3] == "right") == (f["roll"] > 0) for lab, f in fits if lab[3] in ("left", "right")]
    yc = [yaw_class(f["yaw"]) == lab[0] for lab, f in fits]
    return ys, ps, rs, yc, len(data) - len(fits)


def report_pose():
    truth = pose_truth()
    rows = labeled_faces(truth)
    print(f"## Head pose ({len(truth)} labeled faces, {len(rows)} found in the dump)\n")
    data = [(truth[f], r) for f, r in rows.items()]
    out = []
    for v in VARIANTS:
        ys, ps, rs, yc, nopose = pose_metrics(data, v)
        out.append(
            [
                v,
                f"{sum(ys)}/{len(ys)} ({sum(ys) / len(ys):.0%})",
                f"{sum(ps)}/{len(ps)} ({sum(ps) / len(ps):.0%})",
                f"{sum(rs)}/{len(rs)}",
                f"{sum(yc)}/{len(yc)} ({sum(yc) / len(yc):.0%})",
                nopose,
            ]
        )
    table(["variant", "yaw sign", "pitch sign", "roll sign", "yaw class", "no pose"], out)
    print("Split (yaw sign / yaw class agreeing; before -> always -> band 10 + wide 0.20):\n")
    out = []
    for name, f in SPLITS:
        for b in sorted({f(r) for _, r in data}):
            d = [(lab, r) for lab, r in data if f(r) == b]
            cells = []
            for v in ("before", "always", "band 10 + wide 0.20"):
                ys, ps, rs, yc, _ = pose_metrics(d, v)
                cells.append(f"{sum(ys)}/{len(ys)} / {sum(yc)}/{len(yc)}")
            out.append([b, len(d)] + cells)
    table(["bucket", "n", "before", "always", "band 10 + wide 0.20"], out)
    return data


# --- misjudgments -------------------------------------------------------------


def report_misjudgments(train, held, eyes, pose, truth):
    print("## Misjudgments on the frames whose `before` mesh is off\n")
    off_truth = {(r["folder"], r["file"]) for r in truth if r["label_before"].startswith("off")}
    print(
        f"The truth set's `before off` frames ({len(off_truth)}) come from the six folders; "
        "only `2026-09-19` overlaps a labeled set (the closed-eyes / head-pose faces), so the "
        "restriction below is `before eye_offset` > 0.10 or a `before off` label.\n"
    )

    def restricted(r):
        return (r["before_eye_offset"] is not None and r["before_eye_offset"] > MAX_EYE_OFFSET) or (
            (r["folder"], r["file"]) in off_truth
        )

    out = []
    for v in ("before", "always", "band 10", "band 20", "band 10 + wide 0.20", "band 10 + wide 0.25"):
        af = [r for r in train + held if restricted(r)]
        af_wrong = sum((pick(r, v)["state"] == "Candidate") != (r["xmp"] == "Pick") for r in af)
        ey = [(c, r) for c, r in eyes if restricted(r)]
        ey_wrong = sum(
            pick(r, v)["ear"] is not None and c != (pick(r, v)["ear"] <= EYES_CLOSED_EAR) for c, r in ey
        )
        po = [(lab, r) for lab, r in pose if restricted(r)]
        ys, ps, rs, yc, nopose = pose_metrics(po, v)
        out.append(
            [
                v,
                f"{af_wrong}/{len(af)}",
                f"{ey_wrong}/{len(ey)}",
                f"{len(ys) - sum(ys)}/{len(ys)}",
                f"{len(yc) - sum(yc)}/{len(yc)}",
                f"{len(rs) - sum(rs)}/{len(rs)}",
                nopose,
            ]
        )
    table(
        ["variant", "AF eye state wrong", "eyes class wrong", "yaw sign wrong", "yaw class wrong", "roll sign wrong", "no pose"],
        out,
    )


# --- starred frames -----------------------------------------------------------


def report_stars():
    stars = {}
    for r in csv.DictReader(open(STARS, encoding="utf-8"), delimiter="\t"):
        stars[r["name"]] = r["stars"]
    rows = [r for r in load("good-mark-2026-10-09") if r["file"] in stars]
    print(f"## The {len(rows)} starred frames of `samples-scored.tsv` (information only)\n")
    out = []
    for r in sorted(rows, key=lambda r: (-int(stars[r["file"]]), r["file"])):
        changed = abs(r["after_eye_offset"] - r["before_eye_offset"]) > 0.01 or (
            r["before_ear"] is not None and abs(r["after_ear"] - r["before_ear"]) > 0.01
        )
        g = "yes" if VARIANTS["band 10 + wide 0.20"](r) else ""
        out.append(
            [
                f"`{r['file'][:-4]}`",
                stars[r["file"]],
                f"{r['yunet_roll']:+.0f}",
                f"{r['yunet_eye_dist']:.2f}",
                g,
                f"{r['before_eye_focus']:.2f} -> {r['after_eye_focus']:.2f}",
                f"{r['before_ear']:.3f} -> {r['after_ear']:.3f}",
                f"{fmt_angle(r['before_yaw'])} -> {fmt_angle(r['after_yaw'])}",
                f"{fmt_angle(r['before_roll'])} -> {fmt_angle(r['after_roll'])}",
                f"{r['before_eye_offset']:.3f} -> {r['after_eye_offset']:.3f}",
                "*" if changed else "",
            ]
        )
    table(
        ["frame", "stars", "yunet_roll", "eye dist", "guard rotates", "eye_focus", "EAR", "yaw", "roll", "eye_offset", "moved"],
        out,
    )
    for s in sorted({stars[r["file"]] for r in rows}, reverse=True):
        rs = [r for r in rows if stars[r["file"]] == s]
        print(
            f"- {s} stars: {len(rs)} frames, over 0.10 before {sum(r['before_eye_offset'] > 0.1 for r in rs)}, "
            f"always {sum(r['after_eye_offset'] > 0.1 for r in rs)}, "
            f"guard {sum(pick(r, 'band 10 + wide 0.20')['eye_offset'] > 0.1 for r in rs)}"
        )
    print()


def fmt_angle(a):
    return "-" if a is None else f"{a:+.0f}"


def main():
    truth, six = truth_rows(), six_rows()
    report_truth(truth, six)
    report_six(six)
    train, held = focus_rows(TRAIN), focus_rows(HELD)
    report_af(train, held)
    eyes = report_eyes()
    pose = report_pose()
    report_misjudgments(train, held, eyes, pose, truth)
    report_stars()


if __name__ == "__main__":
    main()
