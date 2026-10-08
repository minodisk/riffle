"""The Step 3 keep-mark measurement: strict AND rules that mark the frames
clearly meeting the minimum conditions for a pick, scored by precision.

Usage: python -I keep.py <dump-dir> [<frozen.json>]

Prints the tables of fit.md's "Keep mark" part (Markdown) on stdout; with a
second argument, also writes the rule of the Decision's alternative (KEEP
below) and its numbers to that file. Reads
the dumps through metrics.py and fit.py (same folder), so frames, labels,
bursts and scorable bursts are Step 2's (sidecar-labeled ARW folders, gap
1000 ms).

A rule marks a frame when every condition holds: relative sharpness (over
the burst maximum) at or above a cut, absolute sharpness at or above a
cut, `eye_focus` at or above a cut, eyes open at or above a cut, and
|yaw| / |pitch| within a frontal range. The full rule applies to faced
frames (a cue face, an eyes judgment and a pose); frames lacking any of
them are reported apart with sharpness-only rules.

Precision is the pick share of the marked frames, a lower bound, since a
non-pick may be an OK frame the user did not choose. Lift is precision over
the base pick rate of the same frames (pooled), and over the mean pick
share of the marked frames' bursts (per burst). Recall (picks marked over
picks) is for reference only.

Fixed grid rules need no fitting and are evaluated on every folder; picking
the best grid rule is a selection, so the held-out rows choose the rule on
the other folders (the rule with the most marked frames whose training
precision reaches a target) and score it on the held-out one, pooled.
"""

import json
import os
import statistics
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fit  # noqa: E402
import metrics  # noqa: E402

GAP = 1000
REL = (0.0, 0.5, 0.7, 0.9, 1.0)
ABS = (0.0, 200.0, 400.0, 600.0, 800.0)
EF = (0.0, 0.9, 0.95, 0.97)
EO = (0.0, 0.98, 0.995, 0.999)
POSE = (None, (30.0, 20.0), (15.0, 10.0), (8.0, 5.0))
DIMS = (REL, ABS, EF, EO, POSE)
TARGETS = (0.35, 0.4, 0.45, 0.5, 0.6, 0.7, 0.8, 0.9)
MIN_TRAIN_MARKED = 30
MIN_FOLDER_MARKED = 10
TOPS = (0.3, 0.2, 0.1, 0.05, 0.02, 0.01, 0.005)
KEEP = (4, 1, 1, 1, 0)


def faced(f):
    return f["ef"] is not None and f["eo"] is not None and f["yaw"] is not None


def level(cuts, v):
    return max(i for i, c in enumerate(cuts) if v >= c)


def pose_level(f):
    best = 0
    for i, c in enumerate(POSE):
        if c is None or (abs(f["yaw"]) <= c[0] and abs(f["pitch"]) <= c[1]):
            best = i
    return best


def levels(f):
    return (level(REL, f["rel"]), level(ABS, f["sharp"]), level(EF, f["ef"]), level(EO, f["eo"]), pose_level(f))


SIZES = [len(d) for d in DIMS]


def index(t):
    i = 0
    for v, n in zip(t, SIZES):
        i = i * n + v
    return i


def tuples():
    out = [()]
    for n in SIZES:
        out = [t + (v,) for t in out for v in range(n)]
    return out


ALL = tuples()


def suffix(counts):
    """counts[rule] = sum of the cells dominating the rule's levels."""
    a = list(counts)
    stride = 1
    for d in reversed(range(len(SIZES))):
        n = SIZES[d]
        for t in ALL:
            if t[d] < n - 1:
                continue
            for v in range(n - 2, -1, -1):
                tt = t[:d] + (v,) + t[d + 1 :]
                a[index(tt)] += a[index(tt) + stride]
        stride *= n
    return a


def describe(t):
    parts = []
    if t[0]:
        parts.append(f"rel >= {REL[t[0]]:g}")
    if t[1]:
        parts.append(f"abs >= {ABS[t[1]]:g}")
    if t[2]:
        parts.append(f"ef >= {EF[t[2]]:g}")
    if t[3]:
        parts.append(f"eo >= {EO[t[3]]:g}")
    if t[4]:
        y, pch = POSE[t[4]]
        parts.append(f"\\|yaw\\| <= {y:g}, \\|pitch\\| <= {pch:g}")
    return ", ".join(parts) or "(faced, no cut)"


class Tally:
    """Per folder and per rule: marked, marked picks, sum of burst pick shares, bursts with a mark."""

    def __init__(self, by):
        self.by = by
        self.folders = list(by)
        self.cells = {}
        self.base = {}
        for n, bs in by.items():
            m = [0] * len(ALL)
            p = [0] * len(ALL)
            s = [0.0] * len(ALL)
            nf = npk = 0
            for b in bs:
                share = sum(f["pick"] for f in b) / len(b)
                for f in b:
                    if not faced(f):
                        continue
                    i = index(levels(f))
                    m[i] += 1
                    p[i] += f["pick"]
                    s[i] += share
                    nf += 1
                    npk += f["pick"]
            self.cells[n] = (suffix(m), suffix(p), suffix(s))
            self.base[n] = (nf, npk)

    def bursts_marked(self, names, t):
        out = 0
        for n in names:
            for b in self.by[n]:
                if any(faced(f) and all(a >= c for a, c in zip(levels(f), t)) for f in b):
                    out += 1
        return out

    def sums(self, names, i):
        m = sum(self.cells[n][0][i] for n in names)
        p = sum(self.cells[n][1][i] for n in names)
        s = sum(self.cells[n][2][i] for n in names)
        return m, p, s


def pc(x, d=1):
    return "-" if x is None else f"{100 * x:.{d}f}%"


def rate(a, b):
    return a / b if b else None


def row_stats(m, p, s, nf, npk, bursts_marked, nbursts, per):
    prec = rate(p, m)
    base = rate(npk, nf)
    lift = None if prec is None else prec / base
    blift = None if prec is None or not s else prec / (s / m)
    good = [x for x in per if x is not None]
    spread = f"{pc(statistics.median(good))} ({pc(min(good))}-{pc(max(good))}), {len(good)}" if good else "-"
    return [
        pc(prec),
        "-" if lift is None else f"{lift:.2f}",
        "-" if blift is None else f"{blift:.2f}",
        f"{m} ({pc(rate(m, nf))})",
        f"{p} ({pc(rate(p, npk))})",
        f"{bursts_marked} ({pc(rate(bursts_marked, nbursts))})",
        spread,
    ]


HEAD = (
    "| {} | Precision | Lift (pooled) | Lift (per burst) | Frames marked (of faced) | Picks marked (recall) "
    "| Bursts with a mark | Per-folder precision median (min-max), folders |"
)


def per_folder(tally, t):
    i = index(t)
    out = []
    for n in tally.folders:
        m, p, _ = tally.sums([n], i)
        out.append(rate(p, m) if m >= MIN_FOLDER_MARKED else None)
    return out


def fixed_rows(tally, rules):
    names = tally.folders
    nf = sum(tally.base[n][0] for n in names)
    npk = sum(tally.base[n][1] for n in names)
    nb = sum(len(tally.by[n]) for n in names)
    rows = []
    for label, t in rules:
        m, p, s = tally.sums(names, index(t))
        rows.append([label] + row_stats(m, p, s, nf, npk, tally.bursts_marked(names, t), nb, per_folder(tally, t)))
    return rows


def heldout_select(tally, target):
    """Per fold, the rule with the most training marked frames whose training precision reaches the target."""
    folders = tally.folders
    tot = [0, 0, 0.0]
    nf = npk = nb = bm = 0
    per, missing, chosen = [], 0, {}
    for held in folders:
        train = [n for n in folders if n != held]
        best = None
        for t in ALL:
            i = index(t)
            m, p, _ = tally.sums(train, i)
            if m >= MIN_TRAIN_MARKED and p / m >= target and (best is None or m > best[0]):
                best = (m, t)
        nf += tally.base[held][0]
        npk += tally.base[held][1]
        nb += len(tally.by[held])
        if best is None:
            missing += 1
            per.append(None)
            continue
        t = best[1]
        chosen[t] = chosen.get(t, 0) + 1
        m, p, s = tally.sums([held], index(t))
        tot = [tot[0] + m, tot[1] + p, tot[2] + s]
        bm += tally.bursts_marked([held], t)
        per.append(rate(p, m) if m >= MIN_FOLDER_MARKED else None)
    common = max(chosen.items(), key=lambda kv: kv[1]) if chosen else None
    return row_stats(tot[0], tot[1], tot[2], nf, npk, bm, nb, per), missing, common


def logistic_rows(by):
    """Held-out pointwise logistic (fit.py (iv), all features) over faced frames:
    the top share of training faced frames by score, and the target-precision cut."""
    out_top = {q: [0, 0, 0.0, 0] for q in TOPS}
    out_tp = {t: [0, 0, 0.0, 0, 0] for t in TARGETS}
    per_top = {q: [] for q in TOPS}
    per_tp = {t: [] for t in TARGETS}
    nf = npk = nb = 0
    fitter = fit.fit_pointwise(fit.GROUPS)
    for held in by:
        train = [b for n, bs in by.items() if n != held for b in bs]
        model = fitter(train)
        s = fit.linear(model)
        tr = sorted(((s(f), f["pick"]) for b in train for f in b if faced(f)), reverse=True)
        cum, cuts = 0, []
        for k, (v, pk) in enumerate(tr, 1):
            cum += pk
            cuts.append((v, k, cum / k))
        hb = by[held]
        nb += len(hb)
        for b in hb:
            share = sum(f["pick"] for f in b) / len(b)
            for f in b:
                f["s"], f["share"] = s(f), share
        hf = [f for b in hb for f in b if faced(f)]
        nf += len(hf)
        npk += sum(f["pick"] for f in hf)

        def tally(acc, per, thr):
            marked = [f for f in hf if f["s"] >= thr]
            acc[0] += len(marked)
            acc[1] += sum(f["pick"] for f in marked)
            acc[2] += sum(f["share"] for f in marked)
            acc[3] += sum(1 for b in hb if any(faced(f) and f["s"] >= thr for f in b))
            per.append(rate(sum(f["pick"] for f in marked), len(marked)) if len(marked) >= MIN_FOLDER_MARKED else None)

        for q in TOPS:
            thr = tr[max(0, int(q * len(tr)) - 1)][0]
            tally(out_top[q], per_top[q], thr)
        for t in TARGETS:
            ok = [c for c in cuts if c[1] >= MIN_TRAIN_MARKED and c[2] >= t]
            if not ok:
                out_tp[t][4] += 1
                per_tp[t].append(None)
                continue
            thr = max(ok, key=lambda c: c[1])[0]
            tally(out_tp[t], per_tp[t], thr)
    rows_top = [[f"top {q:.1%} of training faced frames"] + row_stats(a[0], a[1], a[2], nf, npk, a[3], nb, per_top[q]) for q, a in out_top.items()]
    rows_tp = [[f"target {t:.0%}", a[4]] + row_stats(a[0], a[1], a[2], nf, npk, a[3], nb, per_tp[t]) for t, a in out_tp.items()]
    return rows_top, rows_tp


def best_per_burst(by, tally_rules):
    """One mark per burst: among the burst's frames passing a rule, the one with the
    highest held-out logistic score ("s", set by logistic_rows), or the first passing."""
    rows = []
    allb = [b for bs in by.values() for b in bs]
    for label, t in tally_rules:
        for how in ("highest score", "first passing"):
            marked = picks = 0
            per = {}
            for b in allb:
                ok = [f for f in b if faced(f) and all(a >= c for a, c in zip(levels(f), t))]
                if not ok:
                    continue
                f = max(ok, key=lambda g: g["s"]) if how == "highest score" else min(ok, key=lambda g: g["pos"])
                marked += 1
                picks += f["pick"]
                c = per.setdefault(f["folder"], [0, 0])
                c[0] += 1
                c[1] += f["pick"]
            good = [rate(c[1], c[0]) for c in per.values() if c[0] >= MIN_FOLDER_MARKED]
            spread = f"{pc(statistics.median(good))} ({pc(min(good))}-{pc(max(good))}), {len(good)}" if good else "-"
            rows.append([label, how, f"{marked} ({pc(rate(marked, len(allb)))})", pc(rate(picks, marked)), spread])
    for how, pick in (
        ("first frame", lambda b: min(b, key=lambda g: g["pos"])),
        ("highest score", lambda b: max(b, key=lambda g: g["s"])),
        ("sharpest (`rel` = 1)", lambda b: max(b, key=lambda g: g["rel"])),
    ):
        fs = [pick(b) for b in allb]
        rows.append(["every burst", how, f"{len(fs)} (100.0%)", pc(sum(f["pick"] for f in fs) / len(fs)), "-"])
    rows.append(["every burst", "random frame (expected)", f"{len(allb)} (100.0%)", pc(statistics.fmean(sum(f["pick"] for f in b) / len(b) for b in allb)), "-"])
    return rows


def face_free(by):
    allb = [b for bs in by.values() for b in bs]
    frames = [f for b in allb for f in b if not faced(f)]
    npk = sum(f["pick"] for f in frames)
    rows = []
    for r in REL:
        for a in ABS:
            m = [f for f in frames if f["rel"] >= r and f["sharp"] >= a]
            p = sum(f["pick"] for f in m)
            rows.append([f"rel >= {r:g}, abs >= {a:g}", f"{len(m)} ({pc(rate(len(m), len(frames)))})", f"{p} ({pc(rate(p, npk))})", pc(rate(p, len(m)))])
    return len(frames), npk, rows


def report(folders, frozen_path):
    by = fit.arw_bursts(folders, GAP)
    tally = Tally(by)
    nf = sum(v[0] for v in tally.base.values())
    npk = sum(v[1] for v in tally.base.values())
    allf = [f for bs in by.values() for b in bs for f in b]
    print(
        f"Sidecar-labeled ARW block, gap {GAP} ms: {len(allf)} frames in {sum(len(v) for v in by.values())} scorable bursts, "
        f"{sum(f['pick'] for f in allf)} picks (base {pc(sum(f['pick'] for f in allf) / len(allf))}); faced frames "
        f"(cue face, eyes judgment and pose) {nf}, of them picks {npk} (base {pc(npk / nf)}). Per-folder precision "
        f"is over the folders with at least {MIN_FOLDER_MARKED} marked frames.\n"
    )

    print("## One condition at a time, toward the strictest cut\n")
    rules = [("faced, no cut", (0, 0, 0, 0, 0))]
    for d, (name, cuts) in enumerate((("rel", REL), ("abs", ABS), ("ef", EF), ("eo", EO), ("pose", POSE))):
        for v in range(1, len(cuts)):
            t = tuple(v if i == d else 0 for i in range(5))
            rules.append((describe(t), t))
    metrics.table(HEAD.format("Rule"), fixed_rows(tally, rules))

    print("## All four conditions at once, by strictness\n")
    print("Levels: rel 0 / 0.5 / 0.7 / 0.9 / 1.0 (the burst's sharpest), abs 0-800, ef 0 / 0.9 / 0.95 / 0.97, eo 0 / 0.98 / 0.995 / 0.999, pose none / 30-20 / 15-10 / 8-5 deg.\n")
    rules = []
    for k in range(1, 4):
        rules.append((f"level {k}, rel", (k, 0, k, k, k)))
        rules.append((f"level {k}, rel + abs", (k, k, k, k, k)))
        rules.append((f"level {k}, abs", (0, k, k, k, k)))
    rules.append(("level 3, rel = burst max + abs 800", (4, 4, 3, 3, 3)))
    rules.append(("burst's sharpest + level 1 (abs 200, ef 0.9, eo 0.98, pose 30-20)", (4, 1, 1, 1, 1)))
    rules.append(("burst's sharpest + abs 200, ef 0.9, eo 0.98, no pose", (4, 1, 1, 1, 0)))
    rules.append(("burst's sharpest + abs 400, ef 0.95, eo 0.995, no pose", (4, 2, 2, 2, 0)))
    rules.append(("ef + eo strictest, no sharpness, no pose", (0, 0, 3, 3, 0)))
    rules.append(("ef + eo + pose strictest", (0, 0, 3, 3, 3)))
    metrics.table(HEAD.format("Rule"), fixed_rows(tally, rules))

    print("## Best precision of the grid at a coverage floor (in-sample selection)\n")
    print(f"Over the {len(ALL)} grid rules on every folder; the selection makes these optimistic, the held-out table below is the honest one.\n")
    rows = []
    names = tally.folders
    for floor in (0.2, 0.1, 0.05, 0.02, 0.01, 0.005, 0.002):
        best = None
        for t in ALL:
            m, p, _ = tally.sums(names, index(t))
            if m >= floor * nf and (best is None or p / m > best[0]):
                best = (p / m, t)
        rows.append([f">= {floor:.1%}", describe(best[1])] + fixed_rows(tally, [("", best[1])])[0][1:])
    metrics.table(HEAD.format("Coverage floor") .replace("| Precision", "| Rule | Precision", 1), rows)

    print("## Held out: the rule chosen on the other folders by a target precision\n")
    print(
        f"Per fold, among the {len(ALL)} grid rules, the one marking the most training faced frames whose training "
        f"precision reaches the target (at least {MIN_TRAIN_MARKED} marked); scored on the held-out folder, pooled. "
        "\"Folds without a rule\" counts the folds where no rule reaches the target (their frames count as unmarked).\n"
    )
    rows = []
    for target in TARGETS:
        stats, missing, common = heldout_select(tally, target)
        rows.append([f"target {target:.0%}", missing, f"{describe(common[0])} ({common[1]} folds)" if common else "-"] + stats)
    metrics.table(HEAD.format("Training target").replace("| Precision", "| Folds without a rule | Most chosen rule | Precision", 1), rows)

    print("## Held out: the pointwise logistic's top scores (learned alternative)\n")
    print("fit.py's (iv) on every feature, fitted on the other folders' scorable frames; the cut is set on the training faced frames.\n")
    top, tp = logistic_rows(by)
    metrics.table(HEAD.format("Cut"), top)
    metrics.table(HEAD.format("Training target").replace("| Precision", "| Folds without a cut | Precision", 1), tp)

    print("## One mark per burst\n")
    print(
        "Among a burst's faced frames passing the rule, the one with the highest held-out logistic score, or the "
        "earliest; bursts with no passing frame get no mark. Baselines over every scorable burst: its first frame, "
        "its highest-scoring frame, its sharpest frame, and a random frame (the mean pick share of a burst).\n"
    )
    rules = [
        ("faced, no cut", (0, 0, 0, 0, 0)),
        ("level 1, rel", (1, 0, 1, 1, 1)),
        ("level 2, rel", (2, 0, 2, 2, 2)),
        ("level 2, rel + abs", (2, 2, 2, 2, 2)),
        ("level 3, rel", (3, 0, 3, 3, 3)),
        ("ef + eo strictest", (0, 0, 3, 3, 0)),
    ]
    metrics.table("| Rule | Frame | Bursts marked (of scorable) | Precision | Per-folder precision median (min-max), folders |", best_per_burst(by, rules))

    n, p, rows = face_free(by)
    print("## Frames without a full face (sharpness only)\n")
    print(f"{n} frames of the scorable bursts lack a cue face, an eyes judgment or a pose; {p} of them are picks (base {pc(p / n)}).\n")
    metrics.table("| Rule | Frames marked | Picks marked (recall) | Precision |", rows)

    if frozen_path:
        names = tally.folders
        m, p, s = tally.sums(names, index(KEEP))
        frozen = {
            "rule": "mark a frame of a burst of two or more frames when every condition holds",
            "gap_ms": GAP,
            "conditions": {
                "rel_at_least": REL[KEEP[0]],
                "rel_definition": "sharpness / max sharpness of the burst, so 1.0 is the burst's sharpest frame",
                "sharpness_at_least": ABS[KEEP[1]],
                "eye_focus_at_least": EF[KEEP[2]],
                "eyes_open_at_least": EO[KEEP[3]],
                "eyes_open_definition": "1 - Eyes.probability of the judged face (>= EYES_MIN_FACE)",
                "pose": "not used",
            },
            "frames_lacking_a_feature": "not marked (a frame needs a cue face, an eyes judgment and a pose)",
            "measured_on": "every scorable burst of the sidecar-labeled ARW folders (a fixed rule, chosen from the grid in-sample)",
            "precision": p / m,
            "base_pick_rate_faced": sum(v[1] for v in tally.base.values()) / nf,
            "frames_marked": m,
            "share_of_faced_frames_marked": m / nf,
            "picks_marked": p,
            "bursts_with_a_mark": tally.bursts_marked(names, KEEP),
            "scorable_bursts": sum(len(tally.by[n]) for n in names),
        }
        with open(frozen_path, "w", encoding="utf-8", newline="\n") as out:
            json.dump(frozen, out, indent=2)
            out.write("\n")


if __name__ == "__main__":
    report(metrics.load(sys.argv[1]), sys.argv[2] if len(sys.argv) > 2 else None)
