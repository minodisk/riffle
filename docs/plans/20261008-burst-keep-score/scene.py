"""The Step 3 burst-level keep measurement: does a strict rule mark the
bursts (scenes) the user kept, and the single frames the user picked?

Usage: python -I scene.py <dump-dir>

Prints the tables of fit.md's "Burst level" part (Markdown) on stdout.
Reads the dumps through metrics.py and the rule grid of keep.py (same
folder). Every burst of the sidecar-labeled ARW folders counts, at the
app's 1000 ms gap, including the bursts without a pick and the single
frames that Steps 2 and 3 left out so far.

- A burst of two or more frames is **kept** when at least one of its frames
  is a pick, and **marked** by a rule when the rule marks at least one of
  its faced frames (keep.py's frame rule; `rel` is over the burst's
  maximum, so `rel >= 1` reads "the burst's sharpest frame passes").
  Precision = the kept share of the marked bursts; base = the kept share of
  all bursts of two or more frames.
- A **single frame** (a burst of one) is marked when it is faced and passes
  the rule (its `rel` is 1); precision = the pick share of the marked
  singles; base = the pick share of all singles.
- Frames without a full face (a cue face, an eyes judgment and a pose) are
  scored apart with sharpness-only rules: the bursts with no faced frame,
  and the face-free singles.

A last section repeats the held-out selections with the grid limited to
the rules without a sharpness cut (`eye_focus`, eyes open and the pose
only), and scores the rules chosen with sharpness again without their
sharpness cuts.

Held out by folder as in keep.py: for each folder, the grid rule with the
most marked units on the other folders (at least 30) among those whose
training precision reaches a target, scored on the held-out folder,
pooled.
"""

import os
import statistics
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import keep  # noqa: E402
import metrics  # noqa: E402

GAP = 1000
TARGETS = (0.5, 0.6, 0.7, 0.75, 0.8, 0.85, 0.9, 0.95)
MIN_TRAIN = 30
MIN_FOLDER = 10
SIZE_CUTS = (2, 3, 4, 5, 6, 8, 10, 15, 20)


def dominated(t):
    """The rule indices a frame of levels t passes."""
    out = [()]
    for v in t:
        out = [u + (w,) for u in out for w in range(v + 1)]
    return [keep.index(u) for u in out]


DOM = {t: dominated(t) for t in keep.ALL}


def derive(burst):
    top = max(f["sharp"] for f in burst)
    out = []
    for i, f in enumerate(burst):
        g = dict(f)
        g["pos"] = i
        g["rel"] = f["sharp"] / top if top > 0 else 1.0
        out.append(g)
    return out


def units(folders):
    """Per ARW folder: bursts of 2+ as (kept, size, set of rule indices marking it,
    faced?) and singles as (pick, set of rule indices, faced?, frame)."""
    by = {}
    for name, kind, frames in folders:
        if kind != "arw":
            continue
        bursts, singles = [], []
        for g in metrics.bursts_of(frames, GAP):
            d = derive(g)
            marks = set()
            for f in d:
                if keep.faced(f):
                    marks.update(DOM[keep.levels(f)])
            anyface = any(keep.faced(f) for f in d)
            if len(d) == 1:
                singles.append((d[0]["pick"], marks, anyface, d[0]))
            else:
                bursts.append((any(f["pick"] for f in d), len(d), marks, anyface, d))
        by[name] = (bursts, singles)
    return by


def counts(items, label):
    """Per rule index: (marked, positive) over items of (positive, marks)."""
    m = [0] * len(keep.ALL)
    p = [0] * len(keep.ALL)
    for pos, marks in items:
        for i in marks:
            m[i] += 1
            p[i] += pos
    return m, p


def pc(x, d=1):
    return keep.pc(x, d)


def spread(per):
    good = [x for x in per if x is not None]
    if not good:
        return "-"
    return f"{pc(statistics.median(good))} ({pc(min(good))}-{pc(max(good))}), {len(good)}"


HEAD = "| {} | Precision | Lift | Marked (of all) | Positives marked (recall) | Per-folder precision median (min-max), folders |"


def stats_row(m, p, n, npos, per):
    prec = keep.rate(p, m)
    base = keep.rate(npos, n)
    lift = None if prec is None or not base else prec / base
    return [pc(prec), "-" if lift is None else f"{lift:.2f}", f"{m} ({pc(keep.rate(m, n))})", f"{p} ({pc(keep.rate(p, npos))})", spread(per)]


def block(by, pick_items, title, unit, allowed=None):
    """pick_items(name) -> list of (positive, marks) for that folder. allowed
    limits the grid rules the selections may choose (all when None); with it,
    the fixed-rule table is left out. Returns the most chosen held-out rule
    per target."""
    names = list(by)
    grid = [t for t in keep.ALL if allowed is None or allowed(t)]
    per_counts = {n: counts(pick_items(n), unit) for n in names}
    tot_n = {n: len(pick_items(n)) for n in names}
    tot_pos = {n: sum(pos for pos, _ in pick_items(n)) for n in names}
    n_all = sum(tot_n.values())
    pos_all = sum(tot_pos.values())
    print(f"{n_all} {unit}, {pos_all} positive (base {pc(pos_all / n_all)}).\n")

    def fixed(t):
        i = keep.index(t)
        m = sum(per_counts[n][0][i] for n in names)
        p = sum(per_counts[n][1][i] for n in names)
        per = [keep.rate(per_counts[n][1][i], per_counts[n][0][i]) if per_counts[n][0][i] >= MIN_FOLDER else None for n in names]
        return stats_row(m, p, n_all, pos_all, per)

    if allowed is None:
        print(f"#### {title}: fixed rules\n")
    rules = [
        ("any faced frame", (0, 0, 0, 0, 0)),
        ("sharpest frame faced (`rel >= 1`)", (4, 0, 0, 0, 0)),
        ("sharpest + abs 200, ef 0.9, eo 0.98", (4, 1, 1, 1, 0)),
        ("sharpest + abs 400, ef 0.95, eo 0.995", (4, 2, 2, 2, 0)),
        ("sharpest + abs 600, ef 0.97, eo 0.999", (4, 3, 3, 3, 0)),
        ("sharpest + abs 800, ef 0.97, eo 0.999", (4, 4, 3, 3, 0)),
        ("abs 400, ef 0.95, eo 0.995", (0, 2, 2, 2, 0)),
        ("abs 800, ef 0.97, eo 0.999", (0, 4, 3, 3, 0)),
        ("ef 0.97, eo 0.999", (0, 0, 3, 3, 0)),
        ("ef 0.97, eo 0.999, pose 15-10", (0, 0, 3, 3, 2)),
        ("level 1 everywhere", (1, 1, 1, 1, 1)),
        ("level 2 everywhere", (2, 2, 2, 2, 2)),
    ]
    if allowed is None:
        keep_metrics_table(HEAD.format("Rule"), [[label] + fixed(t) for label, t in rules])

    print(f"#### {title}: best grid rule at a coverage floor (in-sample)\n")
    rows = []
    for floor in (0.3, 0.2, 0.1, 0.05, 0.02, 0.01):
        best = None
        for t in grid:
            i = keep.index(t)
            m = sum(per_counts[n][0][i] for n in names)
            p = sum(per_counts[n][1][i] for n in names)
            if m >= floor * n_all and m and (best is None or p / m > best[0]):
                best = (p / m, t)
        rows.append([f">= {floor:.0%}", keep.describe(best[1])] + fixed(best[1]))
    keep_metrics_table(HEAD.format("Coverage floor").replace("| Precision", "| Rule | Precision", 1), rows)

    print(f"#### {title}: held out, the rule chosen on the other folders by a target precision\n")
    rows, picked = [], []
    for target in TARGETS:
        m = p = n = npos = 0
        per, missing, chosen = [], 0, {}
        for held in names:
            train = [x for x in names if x != held]
            best = None
            for t in grid:
                i = keep.index(t)
                tm = sum(per_counts[x][0][i] for x in train)
                tp = sum(per_counts[x][1][i] for x in train)
                if tm >= MIN_TRAIN and tp / tm >= target and (best is None or tm > best[0]):
                    best = (tm, t)
            n += tot_n[held]
            npos += tot_pos[held]
            if best is None:
                missing += 1
                per.append(None)
                continue
            i = keep.index(best[1])
            chosen[best[1]] = chosen.get(best[1], 0) + 1
            hm, hp = per_counts[held][0][i], per_counts[held][1][i]
            m += hm
            p += hp
            per.append(keep.rate(hp, hm) if hm >= MIN_FOLDER else None)
        common = max(chosen.items(), key=lambda kv: kv[1]) if chosen else None
        if common:
            picked.append((target, common[0]))
        rows.append([f"target {target:.0%}", missing, f"{keep.describe(common[0])} ({common[1]} folds)" if common else "-"] + stats_row(m, p, n, npos, per))
    keep_metrics_table(HEAD.format("Training target").replace("| Precision", "| Folds without a rule | Most chosen rule | Precision", 1), rows)
    return picked


def keep_metrics_table(head, rows):
    metrics.table(head, rows)


def size_baseline(by):
    rows = []
    allb = [b for bursts, _ in by.values() for b in bursts]
    n, npos = len(allb), sum(b[0] for b in allb)
    for k in SIZE_CUTS:
        sel = [b for b in allb if b[1] >= k]
        p = sum(b[0] for b in sel)
        per = []
        for bursts, _ in by.values():
            s = [b for b in bursts if b[1] >= k]
            per.append(keep.rate(sum(b[0] for b in s), len(s)) if len(s) >= MIN_FOLDER else None)
        rows.append([f"size >= {k}"] + stats_row(len(sel), p, n, npos, per))
    return rows


def size_and_rule(by):
    rows = []
    allb = [b for bursts, _ in by.values() for b in bursts]
    n, npos = len(allb), sum(b[0] for b in allb)
    rules = (
        ("no rule", None),
        ("any faced frame", (0, 0, 0, 0, 0)),
        ("ef 0.9, eo 0.98", (0, 0, 1, 1, 0)),
        ("abs 200, ef 0.95, eo 0.995", (0, 1, 2, 2, 0)),
        ("abs 400, ef 0.95, eo 0.995", (0, 2, 2, 2, 0)),
    )
    for k in (4, 8, 15):
        for label, t in rules:
            i = None if t is None else keep.index(t)
            sel = [b for b in allb if b[1] >= k and (i is None or i in b[2])]
            per = []
            for bursts, _ in by.values():
                s = [b for b in bursts if b[1] >= k and (i is None or i in b[2])]
                per.append(keep.rate(sum(b[0] for b in s), len(s)) if len(s) >= MIN_FOLDER else None)
            rows.append([f"size >= {k}, {label}"] + stats_row(len(sel), sum(b[0] for b in sel), n, npos, per))
    return rows


def heldout_size_rule(by, with_rule, allowed=None):
    """Held out: the (size cut, grid rule or none) with the most training bursts
    marked among those reaching the target training precision."""
    names = list(by)
    nr = len(keep.ALL) + 1
    rules = [r for r in range(nr - 1) if allowed is None or allowed(keep.ALL[r])] + [nr - 1]
    combos = [(k, r) for k in SIZE_CUTS for r in (rules if with_rule else (nr - 1,))]
    cnt = {}
    for n in names:
        m = [0] * (len(SIZE_CUTS) * nr)
        p = [0] * (len(SIZE_CUTS) * nr)
        for kept, size, marks, _, _ in by[n][0]:
            for ki, k in enumerate(SIZE_CUTS):
                if size < k:
                    break
                base = ki * nr
                for r in marks if with_rule else ():
                    m[base + r] += 1
                    p[base + r] += kept
                m[base + nr - 1] += 1
                p[base + nr - 1] += kept
        cnt[n] = (m, p)
    tm = [sum(cnt[n][0][i] for n in names) for i in range(len(SIZE_CUTS) * nr)]
    tp = [sum(cnt[n][1][i] for n in names) for i in range(len(SIZE_CUTS) * nr)]
    allb = [b for n in names for b in by[n][0]]
    nb, npos = len(allb), sum(b[0] for b in allb)
    rows, picked = [], []
    for target in TARGETS:
        m = p = 0
        per, missing, chosen = [], 0, {}
        for held in names:
            hm_, hp_ = cnt[held]
            best = None
            for k, r in combos:
                i = SIZE_CUTS.index(k) * nr + r
                a, b = tm[i] - hm_[i], tp[i] - hp_[i]
                if a >= MIN_TRAIN and b / a >= target and (best is None or a > best[0]):
                    best = (a, i, (k, r))
            if best is None:
                missing += 1
                per.append(None)
                continue
            i = best[1]
            chosen[best[2]] = chosen.get(best[2], 0) + 1
            m += hm_[i]
            p += hp_[i]
            per.append(keep.rate(hp_[i], hm_[i]) if hm_[i] >= MIN_FOLDER else None)
        if chosen:
            (k, r), c = max(chosen.items(), key=lambda kv: kv[1])
            picked.append((target, k, None if r == nr - 1 else keep.ALL[r]))
            desc = f"size >= {k}" + ("" if r == nr - 1 else ", " + keep.describe(keep.ALL[r])) + f" ({c} folds)"
        else:
            desc = "-"
        rows.append([f"target {target:.0%}", missing, desc] + stats_row(m, p, nb, npos, per))
    return rows, picked


def no_sharp(t):
    return t[0] == 0 and t[1] == 0


def fixed_burst(by, k, t):
    """Pooled stats of marking the bursts of at least k frames that a rule t (or none) marks."""
    allb = [b for bursts, _ in by.values() for b in bursts]
    n, npos = len(allb), sum(b[0] for b in allb)
    i = None if t is None else keep.index(t)
    sel = [b for b in allb if b[1] >= k and (i is None or i in b[2])]
    per = []
    for bursts, _ in by.values():
        s = [b for b in bursts if b[1] >= k and (i is None or i in b[2])]
        per.append(keep.rate(sum(b[0] for b in s), len(s)) if len(s) >= MIN_FOLDER else None)
    return stats_row(len(sel), sum(b[0] for b in sel), n, npos, per)


def no_sharpness(by, burst_picked, size_picked):
    print("### Without sharpness cuts (`eye_focus`, eyes open and pose only)\n")
    print(
        "The same held-out selections with the grid limited to the rules without a relative or absolute "
        "sharpness cut (`eye_focus` already measures sharpness at the eyes): 64 of the 1600 rules.\n"
    )
    print("#### Bursts, no sharpness\n")
    block(by, lambda n: [(b[0], b[2]) for b in by[n][0]], "Bursts, no sharpness", "bursts", no_sharp)
    print("#### Bursts, no sharpness: held out, a size cut and a grid rule chosen together\n")
    head = HEAD.format("Training target").replace("| Precision", "| Folds without a rule | Most chosen rule | Precision", 1)
    keep_metrics_table(head, heldout_size_rule(by, True, no_sharp)[0])
    print("#### Single frames, no sharpness\n")
    block(by, lambda n: [(s[0], s[1]) for s in by[n][1]], "Single frames, no sharpness", "single frames", no_sharp)

    print("#### Dropping the sharpness cuts from the rules chosen with sharpness\n")
    print(
        "The most chosen held-out rule per target of the with-sharpness runs, scored as a fixed rule on every "
        "folder, then the same rule with its `rel` and absolute cuts removed.\n"
    )
    rows, seen = [], set()
    cases = [(f"bursts, target {tg:.0%}", 2, t) for tg, t in burst_picked]
    cases += [(f"size + rule, target {tg:.0%}", k, t) for tg, k, t in size_picked if t is not None]
    for label, k, t in cases:
        if no_sharp(t) or (k, t) in seen:
            continue
        seen.add((k, t))
        d = (0, 0) + t[2:]
        size = "" if k == 2 else f"size >= {k}, "
        rows.append([label, size + keep.describe(t)] + fixed_burst(by, k, t))
        rows.append(["", size + "without sharpness: " + keep.describe(d)] + fixed_burst(by, k, d))
    keep_metrics_table(HEAD.format("Chosen for").replace("| Precision", "| Rule | Precision", 1), rows)


def face_free(by):
    rows_b, rows_s = [], []
    fb = [b for bursts, _ in by.values() for b in bursts if not b[3]]
    fs = [s for _, singles in by.values() for s in singles if not s[2]]
    for a in keep.ABS:
        sel = [b for b in fb if max(f["sharp"] for f in b[4]) >= a]
        rows_b.append([f"sharpest frame abs >= {a:g}", f"{len(sel)} ({pc(keep.rate(len(sel), len(fb)))})", pc(keep.rate(sum(b[0] for b in sel), len(sel)))])
        sel = [s for s in fs if s[3]["sharp"] >= a]
        rows_s.append([f"abs >= {a:g}", f"{len(sel)} ({pc(keep.rate(len(sel), len(fs)))})", pc(keep.rate(sum(s[0] for s in sel), len(sel)))])
    return fb, fs, rows_b, rows_s


def report(folders):
    by = units(folders)
    allb = [b for bursts, _ in by.values() for b in bursts]
    alls = [s for _, singles in by.values() for s in singles]
    print(
        f"Sidecar-labeled ARW block, gap {GAP} ms: {len(allb)} bursts of two or more frames "
        f"({sum(b[0] for b in allb)} kept, {sum(b[3] for b in allb)} with a faced frame) and {len(alls)} single frames "
        f"({sum(s[0] for s in alls)} picks, {sum(s[2] for s in alls)} faced). Per-folder precision is over the folders "
        f"with at least {MIN_FOLDER} marked.\n"
    )
    print("### Bursts of two or more frames\n")
    burst_picked = block(by, lambda n: [(b[0], b[2]) for b in by[n][0]], "Bursts", "bursts")
    print("#### Bursts: by size alone (no technical feature)\n")
    keep_metrics_table(HEAD.format("Rule"), size_baseline(by))
    print("#### Bursts: size and a rule together\n")
    keep_metrics_table(HEAD.format("Rule"), size_and_rule(by))
    print("#### Bursts: held out, a size cut alone chosen by a target precision\n")
    head = HEAD.format("Training target").replace("| Precision", "| Folds without a rule | Most chosen rule | Precision", 1)
    keep_metrics_table(head, heldout_size_rule(by, False)[0])
    print("#### Bursts: held out, a size cut and a grid rule chosen together by a target precision\n")
    size_rows, size_picked = heldout_size_rule(by, True)
    keep_metrics_table(head, size_rows)

    print("### Single frames\n")
    block(by, lambda n: [(s[0], s[1]) for s in by[n][1]], "Single frames", "single frames")

    fb, fs, rows_b, rows_s = face_free(by)
    print("### Without a faced frame (sharpness only)\n")
    print(f"{len(fb)} bursts of two or more frames have no faced frame ({sum(b[0] for b in fb)} kept, base {pc(keep.rate(sum(b[0] for b in fb), len(fb)))}); {len(fs)} single frames are not faced ({sum(s[0] for s in fs)} picks, base {pc(keep.rate(sum(s[0] for s in fs), len(fs)))}).\n")
    keep_metrics_table("| Burst rule | Marked (of face-free bursts) | Precision (kept) |", rows_b)
    keep_metrics_table("| Single-frame rule | Marked (of face-free singles) | Precision (picked) |", rows_s)

    no_sharpness(by, burst_picked, size_picked)


if __name__ == "__main__":
    report(metrics.load(sys.argv[1]))
