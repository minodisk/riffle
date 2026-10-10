"""Step 3: does the sharpness score separate picks on face-free frames?

Usage: python -I facefree.py <dump-dir> [<handcheck-dir> <riffle-cli> <cut>]

Reads the `riffle-cli features` dumps (`<folder>.tsv`, `<folder>.output.txt`)
and prints the tables of sharpness-fallback.md (Markdown) on stdout. With the
three extra arguments, also picks the hand-check frames (20 face-free frames
at absolute sharpness >= `<cut>`, 20 below it, seed 0), writes their list to `<handcheck-dir>/list.tsv` and a
crop around the AF point of each through `riffle-cli crop`.

Frames and labels follow the `20261008-burst-keep-score` plan's metrics.py: a
frame is one base stem (the `-DxO_...` suffix of a DeepPRIME DNG removed, the
camera file's row preferred); a pick is the `.dop` Pick (first item), a
virtual copy of the frame in `Output/` (`<stem>_<n>`), or XMP Pick where the
frame has no `.dop`; in a folder without sidecars, a frame in `Output/`. A
reject is the `.dop` Reject, or XMP Reject where the frame has no `.dop`. A
non-pick is every frame that is not a pick (rejects included).

A frame is face-free when its cue `state` is `Unknown`: with an AF point, no
face near it; without one, no confident face on the whole preview (after
Step 2 of this plan, `FACES_VERSION` 9). The grouping is cross-checked against
`af` / `cue_side` and `noaf` / `judged_side`.

Bursts follow `crates/app/ui/src/burst.ts` `groupBursts` at 1000 ms over every
frame of the folder; a face-free burst is one of two or more frames that are
all face-free. The relative form is measured on those, and again on the
face-free frames of every burst of two or more, ranked and scaled against the
whole burst (as `relativeSharpness` would see them). A single is a face-free
frame alone in its burst.

The score has no fitted parameter and its direction (higher is sharper, so a
pick) is fixed in advance, so its AUC needs no held-out fit; the AUC reported
as held out is the folder-stratified one (pairs within a folder only, every
folder weighted by its pairs). The cut is held out by folder: for each folder,
the cut is chosen on the other folders (the highest precision among the cuts
that mark at least 20% of their face-free frames) and applied to it.
"""

import csv
import math
import os
import random
import re
import subprocess
import sys
from datetime import datetime, timezone

GAP = 1000
CUTS = (100, 200, 300, 400, 600, 800, 1000)
RELS = (0.5, 0.7, 0.8, 0.9)
MIN_COVERAGE = 0.20
MIN_REJECTS = 30
HAND = 20


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
            flag = r["dop"] if r["dop"] != "-" else r["xmp"]
            frames.append(
                {
                    "folder": folder,
                    "file": r["file"],
                    "time": r["capture_time"],
                    "subsec": r["subsec"],
                    "ms": capture_ms(r["capture_time"], r["subsec"]),
                    "pick": pick,
                    "reject": flag == "Reject" and not pick,
                    "sharp": num(r["sharpness"]),
                    "free": r["state"] == "Unknown",
                    "af": r["af"] == "af",
                    "cue": num(r["cue_side"]) is not None,
                    "judged": num(r["judged_side"]) is not None,
                }
            )
        folders.append((folder, kind, frames))
    return folders


def order_key(f):
    missing = f["time"] in ("-", "err", "")
    sub = "" if f["subsec"] == "-" else f["subsec"]
    return (missing, "" if missing else f["time"], sub.ljust(16, "0"), f["file"])


def bursts_of(frames):
    groups, previous = [], None
    for f in sorted(frames, key=order_key):
        ms = f["ms"]
        if ms is None or previous is None or ms - previous > GAP:
            groups.append([f])
        else:
            groups[-1].append(f)
        previous = ms
    return groups


def auc_counts(pos, negs):
    """(wins, pairs): wins counts P(pos > neg) with ties as 0.5."""
    if not pos or not negs:
        return 0.0, 0
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
    return rank_sum - n1 * (n1 + 1) / 2, n1 * n0


def auc(pos, negs):
    w, n = auc_counts(pos, negs)
    return None if n == 0 else w / n


def stratified(groups, positive, negative):
    """AUC over pairs within each group only, and the number of pairs."""
    wins = pairs = 0
    for g in groups:
        w, n = auc_counts(
            [f["sharp"] for f in g if positive(f)], [f["sharp"] for f in g if negative(f)]
        )
        wins += w
        pairs += n
    return (None if pairs == 0 else wins / pairs), pairs


def pct(n, d):
    return "-" if d == 0 else f"{100.0 * n / d:.1f}%"


def fmt(x, digits=3):
    return "-" if x is None else f"{x:.{digits}f}"


def lift(p, b):
    return "-" if not b or p is None else f"{p / b:.2f}"


def is_pick(f):
    return f["pick"]


def non_pick(f):
    return not f["pick"]


def is_reject(f):
    return f["reject"]


def mark_stats(frames, marked):
    m = [f for f in frames if marked(f)]
    picks = sum(f["pick"] for f in m)
    rejects = sum(f["reject"] for f in m)
    return len(m), picks, rejects


def counts_table(folders):
    print("| Folder | Kind | Frames | Face-free | AF, no face near it | No AF, no face | "
          "Grouping mismatches | Picks | Rejects | Non-picks (unlabeled) | Pick base |")
    print("| --- | --- | " + " | ".join(["---:"] * 9) + " |")
    tot = [0] * 8
    for folder, kind, frames in folders:
        free = [f for f in frames if f["free"]]
        a = sum(f["af"] for f in free)
        n = len(free) - a
        mism = sum(
            1
            for f in frames
            if f["free"] != ((f["af"] and not f["cue"]) or (not f["af"] and not f["judged"]))
        )
        p = sum(f["pick"] for f in free)
        r = sum(f["reject"] for f in free)
        u = len(free) - p - r
        row = [len(frames), len(free), a, n, mism, p, r, u]
        tot = [x + y for x, y in zip(tot, row)]
        print(f"| `{folder}` | {kind} | " + " | ".join(str(x) for x in row) + f" | {pct(p, len(free))} |")
    print("| **Total** | | " + " | ".join(str(x) for x in tot) + f" | {pct(tot[5], tot[1])} |")


def free_by_folder(folders, singles_only=False):
    out = []
    for folder, kind, frames in folders:
        if singles_only:
            free = [b[0] for b in bursts_of(frames) if len(b) == 1 and b[0]["free"]]
        else:
            free = [f for f in frames if f["free"]]
        free = [f for f in free if f["sharp"] is not None]
        if free:
            out.append((folder, kind, free))
    return out


def absolute_tables(groups, title):
    pooled = [f for _, _, fr in groups for f in fr]
    picks = sum(f["pick"] for f in pooled)
    rejects = sum(f["reject"] for f in pooled)
    base_rate = picks / len(pooled) if pooled else 0
    print(f"#### {title}: pooled\n")
    print(f"{len(pooled)} face-free frames, {picks} picks (base {pct(picks, len(pooled))}), "
          f"{rejects} rejects.\n")
    print("| Cut | Marked (coverage) | Picks marked (recall) | Precision | Lift | Rejects marked | "
          "Below the cut: pick share |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for t in CUTS:
        n, p, r = mark_stats(pooled, lambda f: f["sharp"] >= t)
        below = len(pooled) - n
        print(f"| abs >= {t} | {n} ({pct(n, len(pooled))}) | {p} ({pct(p, picks)}) | {pct(p, n)} | "
              f"{lift(p / n if n else None, base_rate)} | {r} of {rejects} | {pct(picks - p, below)} |")
    print()
    by_folder = [fr for _, _, fr in groups]
    a_pool = auc([f["sharp"] for f in pooled if f["pick"]], [f["sharp"] for f in pooled if not f["pick"]])
    a_strat, pairs = stratified(by_folder, is_pick, non_pick)
    r_pool = auc([f["sharp"] for f in pooled if f["pick"]], [f["sharp"] for f in pooled if f["reject"]])
    r_strat, r_pairs = stratified(by_folder, is_pick, is_reject)
    print("| AUC | Pooled | Within folder (held out) | Pairs within folder |")
    print("| --- | ---: | ---: | ---: |")
    print(f"| Pick vs non-pick | {fmt(a_pool)} | {fmt(a_strat)} | {pairs} |")
    print(f"| Pick vs reject | {fmt(r_pool)} | {fmt(r_strat)} | {r_pairs} |")
    print()
    print(f"#### {title}: per folder\n")
    print("| Folder | Face-free | Picks (base) | Rejects | AUC pick vs non-pick | AUC pick vs reject | "
          + " | ".join(f"Precision / coverage at {t}" for t in (200, 400, 800)) + " |")
    print("| --- | " + " | ".join(["---:"] * 8) + " |")
    for folder, _, fr in groups:
        p = sum(f["pick"] for f in fr)
        r = sum(f["reject"] for f in fr)
        cells = []
        for t in (200, 400, 800):
            n, pp, _ = mark_stats(fr, lambda f: f["sharp"] >= t)
            cells.append(f"{pct(pp, n)} / {pct(n, len(fr))}")
        a = auc([f["sharp"] for f in fr if f["pick"]], [f["sharp"] for f in fr if not f["pick"]])
        ra = auc([f["sharp"] for f in fr if f["pick"]], [f["sharp"] for f in fr if f["reject"]])
        print(f"| `{folder}` | {len(fr)} | {p} ({pct(p, len(fr))}) | {r} | {fmt(a)} | {fmt(ra)} | "
              + " | ".join(cells) + " |")
    print()
    return held_out(groups, CUTS, lambda f, t: f["sharp"] >= t, "abs", title)


def held_out(groups, cuts, marks, label, title):
    """Leave one folder out: the cut with the best precision at >= 20% coverage on the rest."""
    total = picks = marked = marked_picks = 0
    chosen = {}
    for i, (folder, _, fr) in enumerate(groups):
        rest = [f for j, (_, _, g) in enumerate(groups) if j != i for f in g]
        best = None
        for t in cuts:
            n, p, _ = mark_stats(rest, lambda f: marks(f, t))
            if rest and n >= MIN_COVERAGE * len(rest) and n:
                if best is None or p / n > best[0]:
                    best = (p / n, t)
        total += len(fr)
        picks += sum(f["pick"] for f in fr)
        if best is None:
            continue
        chosen[best[1]] = chosen.get(best[1], 0) + 1
        n, p, _ = mark_stats(fr, lambda f: marks(f, best[1]))
        marked += n
        marked_picks += p
    base_rate = picks / total if total else 0
    prec = marked_picks / marked if marked else None
    print(f"#### {title}: the cut held out by folder\n")
    print("| Cuts chosen (folders) | Marked (coverage) | Precision | Base | Lift | Passes (>= 2x base, >= 20%) |")
    print("| --- | ---: | ---: | ---: | ---: | --- |")
    ok = prec is not None and prec >= 2 * base_rate and marked >= MIN_COVERAGE * total
    chosen_s = ", ".join(f"{label} >= {t} ({n})" for t, n in sorted(chosen.items()))
    print(f"| {chosen_s} | {marked} ({pct(marked, total)}) | {pct(marked_picks, marked)} | "
          f"{pct(picks, total)} | {lift(prec, base_rate)} | {'yes' if ok else 'no'} |")
    print()
    return ok


def relative_tables(folders, mixed_too):
    """Face-free bursts only, or with `mixed_too` the face-free frames of every
    burst of two or more, ranked and scaled against the whole burst."""
    groups = []
    singles = 0
    mixed = 0
    for folder, kind, frames in folders:
        bursts = []
        for b in bursts_of(frames):
            if len(b) == 1:
                singles += b[0]["free"]
                continue
            if not any(f["free"] for f in b):
                continue
            if not all(f["free"] for f in b):
                mixed += sum(f["free"] for f in b)
                if not mixed_too:
                    continue
            if any(f["sharp"] is None for f in b):
                continue
            top = max(f["sharp"] for f in b)
            ranked = sorted(b, key=lambda f: -f["sharp"])
            bursts.append(
                [
                    dict(f, rel=(f["sharp"] / top if top > 0 else 1.0), rank=k, size=len(b))
                    for k, f in enumerate(ranked)
                    if f["free"]
                ]
            )
        if bursts:
            groups.append((folder, kind, bursts))
    all_bursts = [b for _, _, bs in groups for b in bs]
    frames = [f for b in all_bursts for f in b]
    picks = sum(f["pick"] for f in frames)
    rejects = sum(f["reject"] for f in frames)
    with_pick = sum(any(f["pick"] for f in b) for b in all_bursts)
    what = (
        "bursts of two or more frames with a face-free frame"
        if mixed_too
        else "face-free bursts of two or more frames"
    )
    print(f"{len(all_bursts)} {what} ({len(frames)} face-free frames, "
          f"{picks} picks, base {pct(picks, len(frames))}, {rejects} rejects; {with_pick} bursts "
          f"hold a face-free pick) in {len(groups)} folders. Face-free frames in bursts that also "
          f"hold a faced frame: {mixed}. Face-free singles: {singles}.\n")
    rules = [
        ("Sharpest frame", lambda f, b: f["rank"] == 0),
        ("Top half", lambda f, b: f["rank"] < f["size"] / 2),
    ] + [(f"rel >= {r}", (lambda r: lambda f, b: f["rel"] >= r)(r)) for r in RELS]
    print("| Rule | Marked (coverage) | Picks marked (share of picks) | Precision | Lift | Rejects marked |")
    print("| --- | ---: | ---: | ---: | ---: | ---: |")
    base_rate = picks / len(frames) if frames else 0
    for name, rule in rules:
        m = [f for b in all_bursts for f in b if rule(f, b)]
        p = sum(f["pick"] for f in m)
        r = sum(f["reject"] for f in m)
        print(f"| {name} | {len(m)} ({pct(len(m), len(frames))}) | {p} ({pct(p, picks)}) | "
              f"{pct(p, len(m))} | {lift(p / len(m) if m else None, base_rate)} | {r} of {rejects} |")
    print()
    a, pairs = stratified(all_bursts, is_pick, non_pick)
    ra, rpairs = stratified(all_bursts, is_pick, is_reject)
    print("| Pairwise AUC within a burst | AUC | Pairs |")
    print("| --- | ---: | ---: |")
    print(f"| Pick vs non-pick | {fmt(a)} | {pairs} |")
    print(f"| Pick vs reject | {fmt(ra)} | {rpairs} |")
    print()
    print("| Folder | Bursts | Frames | Picks | Rejects | Within-burst AUC pick vs non-pick | Pick share at the sharpest frame |")
    print("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for folder, _, bs in groups:
        fr = [f for b in bs for f in b]
        p = sum(f["pick"] for f in fr)
        r = sum(f["reject"] for f in fr)
        fa, _ = stratified(bs, is_pick, non_pick)
        top = sum(f["pick"] for f in fr if f["rank"] == 0)
        print(f"| `{folder}` | {len(bs)} | {len(fr)} | {p} | {r} | {fmt(fa)} | {pct(top, p)} |")
    print()
    flat = [(folder, kind, [f for b in bs for f in b]) for folder, kind, bs in groups]
    title = "Every burst" if mixed_too else "Face-free bursts"
    ok = held_out(flat, RELS, lambda f, r: f["rel"] >= r, "rel", title)
    return ok, a


def handcheck(folders, out, cli, cut):
    free = [f for _, _, fr in folders for f in fr if f["free"] and f["sharp"] is not None]
    rng = random.Random(0)
    marked = [f for f in free if f["sharp"] >= cut]
    failed = [f for f in free if f["sharp"] < cut]
    take = rng.sample(marked, min(HAND, len(marked))) + rng.sample(failed, min(HAND, len(failed)))
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, "list.tsv"), "w", encoding="utf-8", newline="") as f:
        f.write("crop\tfolder\tfile\tsharpness\tmarked\tpick\treject\n")
        for i, fr in enumerate(take):
            is_marked = fr["sharp"] >= cut
            name = f"{'marked' if is_marked else 'failed'}-{i:02d}-{fr['folder']}-{os.path.splitext(fr['file'])[0]}.png"
            root = "D:/photos/samples/ARW" if fr["folder"].startswith("good-mark") else "D:/photos/2026"
            subprocess.run(
                [cli, "crop", f"{root}/{fr['folder']}/{fr['file']}", os.path.join(out, name)],
                check=True,
                stdout=subprocess.DEVNULL,
            )
            f.write(f"{name}\t{fr['folder']}\t{fr['file']}\t{fr['sharp']:.1f}\t{is_marked}\t"
                    f"{fr['pick']}\t{fr['reject']}\n")


def main(dump, hand=None, cli=None, cut=None):
    folders = load(dump)
    print("### The face-free set\n")
    counts_table(folders)
    print()
    print("### Absolute threshold\n")
    abs_ok = absolute_tables(free_by_folder(folders), "All face-free frames")
    absolute_tables(free_by_folder(folders, singles_only=True), "Face-free singles")
    print("### Within-burst relative\n")
    print("#### Face-free bursts\n")
    rel_ok, rel_auc = relative_tables(folders, False)
    print("#### Face-free frames of every burst, against the whole burst\n")
    mix_ok, mix_auc = relative_tables(folders, True)
    pooled = [f for _, _, fr in free_by_folder(folders) for f in fr]
    a, _ = stratified([fr for _, _, fr in free_by_folder(folders)], is_pick, non_pick)
    print("### Against the criterion\n")
    print("| Form | Held-out AUC pick vs non-pick (>= 0.70) | Cut at >= 2x base, >= 20% marked, held out |")
    print("| --- | ---: | --- |")
    print(f"| Absolute | {fmt(a)} | {'yes' if abs_ok else 'no'} |")
    print(f"| Within-burst relative, face-free bursts | {fmt(rel_auc)} | {'yes' if rel_ok else 'no'} |")
    print(f"| Within-burst relative, every burst | {fmt(mix_auc)} | {'yes' if mix_ok else 'no'} |")
    rejects = {}
    for f in pooled:
        rejects[f["folder"]] = rejects.get(f["folder"], 0) + f["reject"]
    big = [k for k, v in rejects.items() if v >= MIN_REJECTS]
    print()
    print(f"Folders with {MIN_REJECTS} or more face-free rejects: {', '.join(big) if big else 'none'} "
          f"(most: {max(rejects.values()) if rejects else 0}).")
    if hand:
        handcheck(folders, hand, cli, float(cut))


if __name__ == "__main__":
    main(*sys.argv[1:])
