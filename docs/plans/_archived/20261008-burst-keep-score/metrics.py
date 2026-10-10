"""The Step 2 burst statistics, metrics and baselines from the `riffle-cli
features` dumps.

Usage: python -I metrics.py <dump-dir> [<sample.tsv>]

Prints the tables of results.md (Markdown) on stdout. With a second
argument, also writes the hand-check sample (30 flagged non-picks of the
"all checks" rule at 1000 ms, seed 0) to that file.

Frames and labels follow inventory.py: a frame is one base stem (the
`-DxO_...` suffix of a DeepPRIME DNG removed, the camera file's row
preferred); a pick is the `.dop` Pick (first item), or a virtual copy of the
frame in `Output/` (`<stem>_<n>`), or XMP Pick where the frame has no
`.dop`; in a folder without sidecars, a frame in `Output/`. Everything else
is a non-pick (unlabeled).

Bursts follow `crates/app/ui/src/burst.ts` `groupBursts`: frames ordered by
capture time, then subsec (right-padded with zeros), then file name; a new
burst starts where the gap to the previous frame is over the gap (so a gap
equal to it stays in the burst); a missing subsec counts as 0 ms. A
scorable burst has two or more frames, at least one pick and at least one
non-pick; every metric is over the scorable bursts.

Scores are oriented so that higher is better. Sharpness is taken over the
burst maximum (`rel`) and as is (`abs`); `eye_focus`, the eyes-open
probability (1 - closed) and the pose axes (as -|angle|, and as -|angle -
burst median|) are absolute. A frame lacking a feature is treated two ways:
"missing passes" leaves it out of that feature's position and AUC metrics
and never flags it; "missing last" ranks it below every frame with the
feature and always flags it. Ties share their average rank.
"""

import csv
import math
import os
import random
import re
import statistics
import sys
from datetime import datetime, timezone

GAPS = (1000, 2000, 5000)
CANDIDATE_P = 1.0 / (1.0 + math.exp(-1.2194))
KINDS = ("arw", "dng-sidecar", "dng-output")
KIND_TITLE = {
    "arw": "Sidecar-labeled ARW folders",
    "dng-sidecar": "Sidecar-labeled Leica DNG folders",
    "dng-output": "`Output/`-labeled DNG folders",
}
KEEPS = (0.99, 0.95)


def base(stem):
    return re.sub(r"-DxO_.*$", "", stem)


def num(v):
    return None if v in ("-", "err") else float(v)


def capture_ms(time, subsec):
    try:
        t = datetime.strptime(time, "%Y:%m:%d %H:%M:%S").replace(tzinfo=timezone.utc)
    except ValueError:
        return None
    s = subsec[:3].ljust(3, "0") if subsec != "-" else ""
    return int(t.timestamp()) * 1000 + (int(s) if re.fullmatch(r"\d{3}", s) else 0)


def load(dump):
    folders = []
    for name in sorted(os.listdir(dump)):
        if not name.endswith(".tsv"):
            continue
        folder = name[:-4]
        with open(os.path.join(dump, name), newline="", encoding="utf-8") as f:
            rows = list(csv.DictReader(f, delimiter="\t"))
        by = {}
        for r in rows:
            stem = os.path.splitext(r["file"])[0]
            b = base(stem)
            if b not in by or stem == b:
                by[b] = r
        listing = os.path.join(dump, folder + ".output.txt")
        outputs, copies = None, set()
        if os.path.exists(listing):
            outputs = set()
            with open(listing, encoding="utf-8") as f:
                for line in f:
                    s = base(line.strip())
                    m = re.match(r"^(.*)_\d+$", s)
                    if s and s not in by and m and m.group(1) in by:
                        s = m.group(1)
                        copies.add(s)
                    if s:
                        outputs.add(s)
        dng = all(r["file"].lower().endswith(".dng") for r in rows)
        sidecars = any(r["dop"] != "-" or r["xmp"] != "-" for r in rows)
        kind = "arw" if not dng else ("dng-sidecar" if sidecars else "dng-output")
        frames = []
        for b, r in by.items():
            if kind == "dng-output":
                pick = outputs is not None and b in outputs
            else:
                pick = (
                    r["dop"] == "Pick"
                    or b in copies
                    or (r["dop"] == "-" and r["xmp"] == "Pick")
                )
            frames.append(
                {
                    "folder": folder,
                    "file": r["file"],
                    "time": r["capture_time"],
                    "subsec": r["subsec"],
                    "ms": capture_ms(r["capture_time"], r["subsec"]),
                    "pick": pick,
                    "sharp": num(r["sharpness"]),
                    "ef": num(r["eye_focus"]),
                    "notcand": r["state"] == "NotCandidate",
                    "eo": num(r["eyes_open"]),
                    "yaw": num(r["yaw"]),
                    "pitch": num(r["pitch"]),
                    "roll": num(r["roll"]),
                }
            )
        folders.append((folder, kind, frames))
    return folders


def order_key(f):
    missing = f["time"] in ("-", "err", "")
    sub = "" if f["subsec"] == "-" else f["subsec"]
    return (missing, "" if missing else f["time"], sub.ljust(16, "0"), f["file"])


def bursts_of(frames, gap):
    groups, previous = [], None
    for f in sorted(frames, key=order_key):
        ms = f["ms"]
        if ms is None or previous is None or ms - previous > gap:
            groups.append([f])
        else:
            groups[-1].append(f)
        previous = ms
    return groups


def derive(burst):
    """Per-frame features that depend on the burst, as new dicts."""
    top = max(f["sharp"] for f in burst)
    med = {}
    for axis in ("yaw", "pitch", "roll"):
        vals = [f[axis] for f in burst if f[axis] is not None]
        med[axis] = statistics.median(vals) if vals else None
    out = []
    for i, f in enumerate(burst):
        g = dict(f)
        g["pos"] = i
        g["rel"] = f["sharp"] / top if top > 0 else 1.0
        for axis in ("yaw", "pitch", "roll"):
            v = f[axis]
            g["d" + axis] = None if v is None else abs(v - med[axis])
        out.append(g)
    return out


def neg_abs(key):
    return lambda f: None if f[key] is None else -abs(f[key])


def neg(key):
    return lambda f: None if f[key] is None else -f[key]


# (name, score, may be missing, how to print a threshold)
SCORES = [
    ("Sharpness / burst max", lambda f: f["rel"], False, lambda t: f"rel >= {t:.3f}"),
    ("Sharpness (absolute)", lambda f: f["sharp"], False, lambda t: f"abs >= {t:.1f}"),
    ("`eye_focus`", lambda f: f["ef"], True, lambda t: f">= {t:.3f}"),
    ("Eyes open", lambda f: f["eo"], True, lambda t: f">= {t:.3f}"),
    ("\\|yaw\\|", neg_abs("yaw"), True, lambda t: f"<= {-t:.1f} deg"),
    ("\\|pitch\\|", neg_abs("pitch"), True, lambda t: f"<= {-t:.1f} deg"),
    ("\\|roll\\|", neg_abs("roll"), True, lambda t: f"<= {-t:.1f} deg"),
    ("\\|yaw - burst median\\|", neg("dyaw"), True, lambda t: f"<= {-t:.1f} deg"),
    ("\\|pitch - burst median\\|", neg("dpitch"), True, lambda t: f"<= {-t:.1f} deg"),
    ("\\|roll - burst median\\|", neg("droll"), True, lambda t: f"<= {-t:.1f} deg"),
]
BASELINES = [
    ("Random (seed 0)", lambda f: f["rand"], False, None),
    ("First frame", lambda f: -f["pos"], False, None),
]


def ab(f, key, cut):
    return f[key] is not None and abs(f[key]) > cut


def rule_a(cut):
    return lambda f: f["rel"] < cut


def rule_b(f):
    return f["notcand"]


def rule_c(f):
    return f["eo"] is not None and f["eo"] < 0.5


def rule_d(yaw, pitch):
    return lambda f: ab(f, "yaw", yaw) or ab(f, "pitch", pitch)


def any_of(*rules):
    return lambda f: any(r(f) for r in rules)


RULES = [
    ("(a) rel sharpness < 0.7", rule_a(0.7)),
    ("(a) rel sharpness < 0.8", rule_a(0.8)),
    ("(b) `eye_focus` < candidate", rule_b),
    ("(c) eyes open < 0.5", rule_c),
    ("(d) \\|yaw\\| > 45", rule_d(45, 1e9)),
    ("(d) \\|pitch\\| > 30", rule_d(1e9, 30)),
    ("(d) \\|yaw\\| > 45 or \\|pitch\\| > 30", rule_d(45, 30)),
    ("(d) \\|yaw\\| > 60 or \\|pitch\\| > 45", rule_d(60, 45)),
    ("(e) all checks (a 0.7, d 45/30)", any_of(rule_a(0.7), rule_b, rule_c, rule_d(45, 30))),
    ("(e) all checks (a 0.8, d 45/30)", any_of(rule_a(0.8), rule_b, rule_c, rule_d(45, 30))),
    ("(e) all checks (a 0.7, d 60/45)", any_of(rule_a(0.7), rule_b, rule_c, rule_d(60, 45))),
    ("(f) a-c (a 0.7)", any_of(rule_a(0.7), rule_b, rule_c)),
    ("(f) a-c (a 0.8)", any_of(rule_a(0.8), rule_b, rule_c)),
]
SHORT_RULES = (1, 2, 3, 6, 8, 11, 12)
HAND_CHECK_RULE = 8


def pct(n, d):
    return "-" if d == 0 else f"{100.0 * n / d:.1f}%"


def fmt(x, digits=3):
    return "-" if x is None else f"{x:.{digits}f}"


def ranks(values):
    """Average 1-based ranks, highest value first."""
    order = sorted(range(len(values)), key=lambda i: -values[i])
    r = [0.0] * len(values)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and values[order[j + 1]] == values[order[i]]:
            j += 1
        for k in range(i, j + 1):
            r[order[k]] = (i + j) / 2 + 1
        i = j + 1
    return r


def auc(pos, negs):
    """P(pick > non-pick), ties 0.5, by sorting."""
    if not pos or not negs:
        return None
    allv = sorted([(v, 1) for v in pos] + [(v, 0) for v in negs])
    rank_sum, i = 0.0, 0
    while i < len(allv):
        j = i
        while j + 1 < len(allv) and allv[j + 1][0] == allv[i][0]:
            j += 1
        avg = (i + j) / 2 + 1
        rank_sum += avg * sum(1 for k in range(i, j + 1) if allv[k][1] == 1)
        i = j + 1
    n1, n0 = len(pos), len(negs)
    return (rank_sum - n1 * (n1 + 1) / 2) / (n1 * n0)


def quantile(vals, q):
    if not vals:
        return None
    s = sorted(vals)
    return s[min(len(s) - 1, int(q * len(s)))]


def position_metrics(bursts, score, missing_last):
    picks_q, worst, top1 = [], [], []
    top_half = top_third = 0
    pair_num = pair_den = 0.0
    abs_pos, abs_neg = [], []
    for b in bursts:
        vals = [score(f) for f in b]
        if missing_last:
            vals = [-math.inf if v is None else v for v in vals]
            members = list(zip(b, vals))
        else:
            members = [(f, v) for f, v in zip(b, vals) if v is not None]
        for f, v in members:
            (abs_pos if f["pick"] else abs_neg).append(v)
        pv = [v for f, v in members if f["pick"]]
        nv = [v for f, v in members if not f["pick"]]
        for p in pv:
            for n in nv:
                pair_num += 1.0 if p > n else 0.5 if p == n else 0.0
        pair_den += len(pv) * len(nv)
        m = len(members)
        if m < 2 or not pv or not nv:
            continue
        r = ranks([v for _, v in members])
        qs = []
        for (f, _), rk in zip(members, r):
            if f["pick"]:
                q = (rk - 1) / (m - 1)
                qs.append(q)
                top_half += (rk - 1) * 2 < (m - 1)
                top_third += (rk - 1) * 3 < (m - 1)
        picks_q.extend(qs)
        worst.append(max(qs))
        best = min(r)
        tied = [f for (f, _), rk in zip(members, r) if rk == best]
        top1.append(sum(f["pick"] for f in tied) / len(tied))
    n = len(picks_q)
    return {
        "picks": n,
        "bursts": len(worst),
        "top_half": pct(top_half, n),
        "top_third": pct(top_third, n),
        "worst_median": fmt(quantile(worst, 0.5), 2),
        "worst_p90": fmt(quantile(worst, 0.9), 2),
        "worst_near1": pct(sum(w >= 0.9 for w in worst), len(worst)),
        "pair_auc": fmt(pair_num / pair_den if pair_den else None),
        "abs_auc": fmt(auc(abs_pos, abs_neg)),
        "top1": pct(sum(top1), len(top1)),
        "mean_rank": fmt(statistics.mean(picks_q) if picks_q else None, 3),
    }


def threshold(bursts, score, keep):
    vals = sorted(v for b in bursts for f in b if f["pick"] for v in [score(f)] if v is not None)
    if not vals:
        return None
    return vals[int(math.floor((1 - keep) * len(vals)))]


def flag_metrics(bursts, fails):
    """Pick false-fail, non-pick flag and mean remaining share, pooled and per folder."""
    per = {}
    remain = []
    for b in bursts:
        flagged = [fails(f) for f in b]
        remain.append(1 - sum(flagged) / len(b))
        c = per.setdefault(b[0]["folder"], [0, 0, 0, 0])
        for f, x in zip(b, flagged):
            if f["pick"]:
                c[0] += x
                c[1] += 1
            else:
                c[2] += x
                c[3] += 1
    tot = [sum(c[i] for c in per.values()) for i in range(4)]
    return {
        "ff": pct(tot[0], tot[1]),
        "ff_n": tot[0],
        "flag": pct(tot[2], tot[3]),
        "flag_n": tot[2],
        "remain": pct(sum(remain), len(remain)),
        "per": {k: (pct(c[0], c[1]), pct(c[2], c[3])) for k, c in per.items()},
    }


def score_rows(with_baselines=True):
    rows = []
    entries = SCORES + (BASELINES if with_baselines else [])
    for name, score, may_miss, show in entries:
        for missing_last in (False, True) if may_miss else (False,):
            label = name + (" (missing last)" if missing_last else "")
            rows.append((label, score, missing_last, show))
    return rows


def thresholded(score, t, missing_last):
    def fails(f):
        v = score(f)
        if v is None:
            return missing_last
        return v < t

    return fails


def burst_stats(groups):
    multi = [g for g in groups if len(g) > 1]
    sizes = [len(g) for g in multi]
    picks = [sum(f["pick"] for f in g) for g in multi]
    scorable = [g for g, p in zip(multi, picks) if 0 < p < len(g)]
    return [
        len(groups),
        sum(len(g) == 1 for g in groups),
        len(multi),
        sum(len(g) for g in multi),
        fmt(statistics.median(sizes), 1) if sizes else "-",
        max(sizes) if sizes else "-",
        sum(s == 2 for s in sizes),
        sum(3 <= s <= 5 for s in sizes),
        sum(6 <= s <= 10 for s in sizes),
        sum(s > 10 for s in sizes),
        sum(p == 0 for p in picks),
        sum(p == 1 for p in picks),
        sum(p > 1 for p in picks),
        sum(p == len(g) for g, p in zip(multi, picks)),
        len(scorable),
    ]


BURST_HEAD = (
    "| {} | Bursts | Single frames | Bursts of 2+ | Frames in 2+ | Median size (2+) | Max size "
    "| Size 2 | 3-5 | 6-10 | 11+ | No pick | One pick | Several picks | All picked | Scorable |"
)


def table(head, rows):
    print(head)
    cols = len(re.findall(r"(?<!\\)\|", head)) - 2
    print("| " + " | ".join(["---"] + ["---:"] * cols) + " |")
    for r in rows:
        print("| " + " | ".join(str(x) for x in r) + " |")
    print()


def scorable_of(groups, rng):
    out = []
    for g in groups:
        if len(g) < 2:
            continue
        p = sum(f["pick"] for f in g)
        if 0 < p < len(g):
            d = derive(g)
            for f in d:
                f["rand"] = rng.random()
            out.append(d)
    return out


def counts_rows(frames_by_folder, groups, scorable):
    frames = sum(len(v) for v in frames_by_folder)
    picks = sum(f["pick"] for v in frames_by_folder for f in v)
    multi = [g for g in groups if len(g) > 1]
    sf = [f for b in scorable for f in b]
    faced = [sum(f["ef"] is not None or f["eo"] is not None for f in b) for b in scorable]
    return [
        ("Folders", len(frames_by_folder)),
        ("Frames / picks", f"{frames} / {picks}"),
        ("Bursts (all) / single-frame", f"{len(groups)} / {sum(len(g) == 1 for g in groups)}"),
        ("Bursts of 2+ dropped: no pick", sum(not any(f["pick"] for f in g) for g in multi)),
        ("Bursts of 2+ dropped: every frame picked", sum(all(f["pick"] for f in g) for g in multi)),
        ("Scorable bursts", len(scorable)),
        ("Frames / picks / non-picks in scorable bursts", f"{len(sf)} / {sum(f['pick'] for f in sf)} / {sum(not f['pick'] for f in sf)}"),
        ("Picks outside scorable bursts (single frames, all-picked bursts)", picks - sum(f["pick"] for f in sf)),
        ("Scorable frames without a cue face (`eye_focus`)", sum(f["ef"] is None for f in sf)),
        ("Scorable frames without an eyes judgment (< 60 px or none)", sum(f["eo"] is None for f in sf)),
        ("Scorable frames without a pose", sum(f["yaw"] is None for f in sf)),
        ("Scorable bursts face-free / mixed / all-faced", f"{sum(n == 0 for n in faced)} / {sum(0 < n < len(b) for n, b in zip(faced, scorable))} / {sum(n == len(b) for n, b in zip(faced, scorable))}"),
    ]


def block(kind, folders, gap):
    rng = random.Random(0)
    sel = [(n, fr) for n, k, fr in folders if k == kind]
    groups, scorable = [], []
    per_folder_groups = {}
    for n, fr in sel:
        g = bursts_of(fr, gap)
        per_folder_groups[n] = g
        groups.extend(g)
        scorable.extend(scorable_of(g, rng))
    return sel, groups, scorable, per_folder_groups


def report(folders, sample_path):
    print("## Burst statistics\n")
    print("Bursts per gap; \"No pick\" to \"All picked\" count the bursts of two or more frames.\n")
    for gap in GAPS:
        print(f"### Gap {gap} ms\n")
        rows = []
        for kind in KINDS:
            sel, groups, _, pfg = block(kind, folders, gap)
            for n, _ in sel:
                rows.append([f"`{n}`"] + burst_stats(pfg[n]))
            rows.append([f"**{kind}**"] + burst_stats(groups))
        table(BURST_HEAD.format("Folder"), rows)

    gap = 1000
    sample_pool = []
    for kind in KINDS:
        sel, groups, scorable, _ = block(kind, folders, gap)
        print(f"## {KIND_TITLE[kind]}, gap {gap} ms\n")
        print("### Counts\n")
        table("| Count | Value |", counts_rows([fr for _, fr in sel], groups, scorable))

        print("### Hand threshold rules\n")
        print("A frame lacking the feature a rule reads is not flagged by it.\n")
        rows, per = [], {}
        for i, (name, fails) in enumerate(RULES):
            m = flag_metrics(scorable, fails)
            rows.append([name, f"{m['ff']} ({m['ff_n']})", f"{m['flag']} ({m['flag_n']})", m["remain"]])
            if i in SHORT_RULES:
                per[name] = m["per"]
        table("| Rule | Pick false-fail | Non-pick flagged | Mean share of a burst kept |", rows)

        print("Per folder (pick false-fail / non-pick flagged):\n")
        names = list(per)
        prow = []
        for n, _ in sel:
            prow.append([f"`{n}`"] + [" / ".join(per[r].get(n, ("-", "-"))) for r in names])
        table("| Folder | " + " | ".join(names) + " |", prow)

        print("### Thresholds at a pick-keeping point\n")
        print(
            "The threshold keeps 99% (95%) of the block's picks that have the feature; "
            "the false-fail column counts every pick, so under \"missing last\" it includes "
            "the picks lacking the feature.\n"
        )
        rows, per99 = [], {}
        for label, score, missing_last, show in score_rows(with_baselines=False):
            for keep in KEEPS:
                t = threshold(scorable, score, keep)
                if t is None:
                    rows.append([label, f"{keep:.0%}", "-", "-", "-", "-"])
                    continue
                m = flag_metrics(scorable, thresholded(score, t, missing_last))
                rows.append([label, f"{keep:.0%}", show(t), f"{m['ff']} ({m['ff_n']})", f"{m['flag']} ({m['flag_n']})", m["remain"]])
                if keep == 0.99 and not missing_last:
                    per99[label] = m["per"]
        table("| Score | Picks kept | Threshold | Pick false-fail | Non-pick flagged | Mean share of a burst kept |", rows)

        print("Per folder at the 99% point, missing passes (pick false-fail / non-pick flagged):\n")
        names = list(per99)
        prow = []
        for n, _ in sel:
            prow.append([f"`{n}`"] + [" / ".join(per99[r].get(n, ("-", "-"))) for r in names])
        table("| Folder | " + " | ".join(names) + " |", prow)

        print("### Pick position and AUC\n")
        print(
            "Position q = (rank - 1) / (size - 1) over the frames ranked (0 best). "
            "Top half: q < 1/2; top third: q < 1/3. Worst pick: the largest q of a burst's picks. "
            "Pairwise AUC: pick / non-pick pairs of one burst. Absolute AUC: every frame of the "
            "scorable bursts. Top-1: the burst's best frame is a pick (ties split).\n"
        )
        rows = []
        for label, score, missing_last, _ in score_rows():
            m = position_metrics(scorable, score, missing_last)
            rows.append([label, m["bursts"], m["picks"], m["top_half"], m["top_third"], m["worst_median"], m["worst_p90"], m["worst_near1"], m["pair_auc"], m["abs_auc"], m["top1"], m["mean_rank"]])
        table(
            "| Score | Bursts | Picks | Picks in top half | Top third | Worst pick median | Worst pick p90 | Bursts with worst pick >= 0.9 | Pairwise AUC | Absolute AUC | Top-1 hit | Mean pick q |",
            rows,
        )
        fails = RULES[HAND_CHECK_RULE][1]
        for b in scorable:
            for f in b:
                if not f["pick"] and fails(f):
                    sample_pool.append((f, b))

    print("## Other gaps\n")
    print("Hand rules at 2000 and 5000 ms (pick false-fail / non-pick flagged / mean share kept), and the sharpness-over-max ranking.\n")
    rows = []
    for gap2 in GAPS:
        for kind in KINDS:
            _, _, scorable, _ = block(kind, folders, gap2)
            r = [f"{gap2}", kind, len(scorable)]
            for i in (1, 2, 3, 6, 8, 11):
                m = flag_metrics(scorable, RULES[i][1])
                r.append(f"{m['ff']} / {m['flag']} / {m['remain']}")
            pm = position_metrics(scorable, SCORES[0][1], False)
            r += [pm["pair_auc"], pm["top_half"]]
            rows.append(r)
    table(
        "| Gap ms | Block | Scorable bursts | "
        + " | ".join(RULES[i][0] for i in (1, 2, 3, 6, 8, 11))
        + " | Sharpness / max pairwise AUC | Picks in top half |",
        rows,
    )

    if sample_path:
        rng = random.Random(0)
        sample = rng.sample(sample_pool, 30)
        with open(sample_path, "w", encoding="utf-8", newline="") as out:
            out.write("folder\tfile\tbest\treasons\trel\teye_focus\teyes_open\tyaw\tpitch\n")
            for f, b in sample:
                reasons = [
                    tag
                    for tag, r in (("a", rule_a(0.7)), ("b", rule_b), ("c", rule_c), ("d", rule_d(45, 30)))
                    if r(f)
                ]
                best = max(b, key=lambda g: g["sharp"])["file"]
                out.write(
                    f"{f['folder']}\t{f['file']}\t{best}\t{','.join(reasons)}\t{f['rel']:.3f}\t"
                    f"{fmt(f['ef'])}\t{fmt(f['eo'])}\t{fmt(f['yaw'], 1)}\t{fmt(f['pitch'], 1)}\n"
                )


if __name__ == "__main__":
    report(load(sys.argv[1]), sys.argv[2] if len(sys.argv) > 2 else None)
