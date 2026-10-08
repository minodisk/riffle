"""Scratch fit for mesh-eye-focus Step 2; not part of the workspace.

Reads the per-file JSON lines `riffle-cli meshdump` (the scratch subcommand in
`meshdump.patch`, applied to the Step 1 commit) wrote for the training and
held-out folders, checks them against the Step 1 `candidates` lines, fits the
variants with plain maximum-likelihood logistic regression and prints the
tables of `fit.md`. Pure Python 3, no dependencies.

    python fit.py <dir with training-dump.jsonl, heldout-dump.jsonl,
                   training.txt, heldout.txt> [--frozen out.json]
"""

import json
import math
import sys
from pathlib import Path

MARGINS = [0.0, 0.25, 0.5, 1.0]
BASE = (-4.725633355883976, 0.6826458385175557, -1.1949836425055467)
BASE_T = 1.2194
TARGET_COVERAGE = 310  # of 339 training picks: the baseline's 91.4%


def load(path):
    frames = []
    for line in Path(path).read_text().splitlines():
        r = json.loads(line)
        if r["flag"] is None or r["base"]["edge"] is None:
            continue
        r["pick"] = r["flag"] == "Pick"
        b = r["base"]
        r["base_logit"] = (
            BASE[0]
            + BASE[1] * math.log(b["lap"] + 1)
            + BASE[2] * math.log(b["edge"] / b["window"])
        )
        frames.append(r)
    return frames


def check_step1(frames, path):
    """The 0.25-margin contour values must match the Step 1 lines, file by
    file in order (names repeat across folders)."""
    lines = []
    for line in Path(path).read_text().splitlines():
        f = line.split()
        if len(f) > 30 and f[1] != "Unknown":
            lines.append(f)
    lines = iter(lines)
    bad = 0
    for r in frames:
        f = next(f for f in lines if f[0] == r["name"])
        i = f.index("L")
        for eye, j in (("left", i + 1), ("right", i + 7)):
            c = r[eye]["contour"][1] if r[eye] else None
            want = "-" if c is None else f"{c['w']}x{c['h']}"
            got_lap = "-" if c is None else f"{c['lap']:.1f}"
            bad += f[j] != want or f[j + 1] != got_lap
    return bad


# --- logistic regression ----------------------------------------------------


def solve(a, b):
    n = len(b)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(m[r][c]))
        m[c], m[p] = m[p], m[c]
        for r in range(n):
            if r != c:
                k = m[r][c] / m[c][c]
                m[r] = [x - k * y for x, y in zip(m[r], m[c])]
    return [m[i][n] / m[i][i] for i in range(n)]


def fit(rows):
    """Maximum-likelihood logistic regression by Newton; rows (x, y)."""
    k = len(rows[0][0]) + 1
    w = [0.0] * k
    for _ in range(100):
        g = [0.0] * k
        h = [[0.0] * k for _ in range(k)]
        for x, y in rows:
            v = [1.0] + list(x)
            z = sum(a * b for a, b in zip(w, v))
            p = 1 / (1 + math.exp(-z))
            for i in range(k):
                g[i] += (y - p) * v[i]
                for j in range(k):
                    h[i][j] += p * (1 - p) * v[i] * v[j]
        step = solve(h, g)
        w = [a + b for a, b in zip(w, step)]
        if max(abs(s) for s in step) < 1e-12:
            break
    return w


def logit(w, x):
    return w[0] + sum(a * b for a, b in zip(w[1:], x))


def auc(scored):
    pos = [s for p, s in scored if p]
    neg = [s for p, s in scored if not p]
    t = sum(1.0 if a > b else 0.5 if a == b else 0.0 for a in pos for b in neg)
    return t / (len(pos) * len(neg))


# --- per-eye features -------------------------------------------------------


def region(r, eye, margin):
    e = r[eye]
    if e is None:
        return None
    return e["contour"][MARGINS.index(margin)]


def feats(r, eye, margin, iris):
    c = region(r, eye, margin)
    if c is None or c["edge"] is None:
        return None
    x = [math.log(c["lap"] + 1), math.log(c["edge"] / max(c["w"], c["h"]))]
    if iris:
        i = r[eye]["iris"][1]
        if i is None:
            return None
        x.append(math.log(i["lap"] + 1))
    return x


def side(r, eye, margin):
    c = region(r, eye, margin)
    return 0 if c is None else max(c["w"], c["h"])


def near_by_af(r):
    af = r["af"]
    d = {}
    for eye in ("left", "right"):
        if r[eye] is None or r[eye]["center"] is None:
            continue
        cx, cy = r[eye]["center"]
        d[eye] = math.hypot(cx - af[0], cy - af[1])
    return min(d, key=d.get) if d else None


def near_by_yaw(r):
    # Yaw positive: the face turns toward the image's right, so the
    # image-right eye faces the camera.
    return "right" if r["pose"]["yaw"] > 0 else "left"


class Variant:
    """One eye rule at one margin, floor, yaw cut and feature set."""

    def __init__(self, rule, margin=0.25, floor=0, cut=None, iris=False):
        self.rule, self.margin, self.floor = rule, margin, floor
        self.cut, self.iris = cut, iris

    def eyes(self, r):
        """The eyes measured for `r`, above the floor with a feature row."""
        out = {}
        for eye in ("left", "right"):
            x = feats(r, eye, self.margin, self.iris)
            if x is not None and side(r, eye, self.margin) >= self.floor:
                out[eye] = x
        return out

    def turned(self, r):
        return (
            self.cut is not None
            and r["pose"] is not None
            and abs(r["pose"]["yaw"]) > self.cut
        )

    def train_rows(self, r):
        e = self.eyes(r)
        if self.rule == "af":
            eye = near_by_af(r)
            return [e[eye]] if eye in e else []
        if self.rule == "g" and self.turned(r):
            eye = near_by_yaw(r)
            return [e[eye]] if eye in e else list(e.values())
        return list(e.values())

    def chosen(self, w, r):
        """(eye, logit) of the frame, or None for the window fallback."""
        e = self.eyes(r)
        if not e:
            return None
        if self.rule == "af":
            eye = near_by_af(r)
            return (eye, logit(w, e[eye])) if eye in e else None
        if self.rule in ("f", "g") and self.turned(r):
            eye = near_by_yaw(r)
            if eye in e:
                return eye, logit(w, e[eye])
        return max(((k, logit(w, x)) for k, x in e.items()), key=lambda t: t[1])

    def fit(self, train):
        rows = [(x, r["pick"]) for r in train for x in self.train_rows(r)]
        self.w = fit(rows)
        self.n_rows = len(rows)
        return self

    def scores(self, frames):
        """(pick, mesh logit or None, baseline logit) of every frame."""
        out = []
        for r in frames:
            c = self.chosen(self.w, r)
            out.append((r["pick"], None if c is None else c[1], r["base_logit"]))
        return out


def threshold(scored):
    """The mesh threshold that keeps the training coverage at 310 picks.

    Fallback frames keep the baseline's state (logit >= BASE_T); the mesh
    threshold lies midway between the logit of the pick that brings the
    picks at or above it, both paths counted, to TARGET_COVERAGE and the next
    lower mesh logit of any frame, so no frame sits on the boundary.
    """
    fixed = sum(1 for p, m, b in scored if p and m is None and b >= BASE_T)
    need = TARGET_COVERAGE - fixed
    picks = sorted((m for p, m, _ in scored if p and m is not None), reverse=True)
    edge = picks[need - 1]
    below = [m for _, m, _ in scored if m is not None and m < edge]
    return (edge + max(below)) / 2 if below else edge - 1.0


class Control(Variant):
    """The window features, refitted on the frames `Variant` would mesh."""

    def train_rows(self, r):
        return [window_feats(r)] if self.eyes(r) else []

    def chosen(self, w, r):
        return ("window", logit(w, window_feats(r))) if self.eyes(r) else None


def window_feats(r):
    b = r["base"]
    return [math.log(b["lap"] + 1), math.log(b["edge"] / b["window"])]


def evaluate(scored, t):
    """AUC (fallback logits shifted onto the mesh threshold), precision and
    coverage, how many frames fell back, and on the frames that did not, the
    AUC of the mesh logit and of the window logit (the same frames)."""
    shift = t - BASE_T
    pooled = [(p, b + shift if m is None else m) for p, m, b in scored]
    cand = [(p, (b >= BASE_T) if m is None else (m >= t)) for p, m, b in scored]
    n_c = sum(c for _, c in cand)
    hits = sum(c and p for p, c in cand)
    picks = sum(p for p, _ in cand)
    meshed = [(p, m, b) for p, m, b in scored if m is not None]
    off = sum(not p for p, _, _ in meshed)
    both = 0 < off < len(meshed)
    return {
        "auc": auc(pooled),
        "precision": hits / max(n_c, 1),
        "coverage": hits / picks,
        "candidates": n_c,
        "hits": hits,
        "picks": picks,
        "fallback": len(scored) - len(meshed),
        "n": len(scored),
        "meshed_off": off,
        "meshed_auc_mesh": auc([(p, m) for p, m, _ in meshed]) if both else None,
        "meshed_auc_window": auc([(p, b) for p, _, b in meshed]) if both else None,
    }


def run(v, train, held):
    try:
        v.fit(train)
    except (ZeroDivisionError, OverflowError):
        return None
    s_tr = v.scores(train)
    t = threshold(s_tr)
    return v, t, evaluate(s_tr, t), evaluate(v.scores(held), t)


def pct(x):
    return f"{100 * x:.1f}%"


def fmt_eval(e):
    a, b = e["meshed_auc_mesh"], e["meshed_auc_window"]
    return (
        f"{e['auc']:.3f} | {pct(e['precision'])} | {pct(e['coverage'])} | "
        f"{e['fallback']}/{e['n']} | "
        + ("-" if a is None else f"{a:.3f} / {b:.3f}")
    )


def row(label, result):
    if result is None:
        print(f"| {label} | no fit (separable) |" + " |" * 9)
        return
    _, _, tr, he = result
    print(f"| {label} | {fmt_eval(tr)} | {fmt_eval(he)} |")


def header():
    cols = ["AUC", "prec", "cov", "fallback", "meshed AUC mesh / window"]
    names = [f"train {c}" for c in cols] + [f"held {c}" for c in cols]
    print("| variant | " + " | ".join(names) + " |")
    print("| --- " * (len(names) + 1) + "|")


def adopted(e):
    """The plan's adopt rule on held-out results: AUC above 0.754, and
    precision and coverage not both below 89.1% / 95.3%."""
    low_precision = e["hits"] * 1000 < 891 * e["candidates"]
    low_coverage = e["hits"] * 1000 < 953 * e["picks"]
    return e["auc"] > 0.754 and not (low_precision and low_coverage)


def rules(m, fl):
    out = [
        ("b sharper", Variant("b", m, fl)),
        ("c AF-nearest", Variant("af", m, fl)),
        ("e iris", Variant("b", m, fl, iris=True)),
    ]
    for cut in (15, 30, 45, 60):
        out.append((f"f yaw {cut}", Variant("f", m, fl, cut)))
        out.append((f"g yaw {cut}", Variant("g", m, fl, cut)))
    out.append(("control: window refit", Control("b", m, fl)))
    return out


def yaw_report(v, train, held):
    print(
        "| cut | set | above (off) | at or below (off) | no pose "
        "| pose eye != sharper (pick / reject) |"
    )
    print("| --- | --- | --- | --- | --- | --- |")
    for cut in (15, 30, 45, 60):
        for name, fr in (("train", train), ("held", held)):
            above = [r for r in fr if r["pose"] and abs(r["pose"]["yaw"]) > cut]
            below = [r for r in fr if r["pose"] and abs(r["pose"]["yaw"]) <= cut]
            nopose = sum(not r["pose"] for r in fr)
            differ = []
            for r in above:
                e = v.eyes(r)
                if len(e) == 2:
                    sharp = max(e, key=lambda k: logit(v.w, e[k]))
                    if sharp != near_by_yaw(r):
                        differ.append(r)
            pk = sum(r["pick"] for r in differ)
            off_a = sum(not r["pick"] for r in above)
            off_b = sum(not r["pick"] for r in below)
            print(
                f"| {cut} | {name} | {len(above)} ({off_a}) | {len(below)} ({off_b}) "
                f"| {nopose} | {len(differ)} ({pk} / {len(differ) - pk}) |"
            )


def quantiles(s):
    s = sorted(s)
    return [s[min(int(f * len(s)), len(s) - 1)] for f in (0.1, 0.5, 0.9)]


def main():
    d = Path(sys.argv[1])
    train = load(d / "training-dump.jsonl")
    held = load(d / "heldout-dump.jsonl")
    sets = (("train", train), ("held", held))
    print("frames:", len(train), len(held))
    print(
        "step 1 mismatches:",
        check_step1(train, d / "training.txt"),
        check_step1(held, d / "heldout.txt"),
    )

    print("\n## (a) baseline window, frozen coefficients")
    for name, fr in sets:
        sc = [(r["pick"], r["base_logit"]) for r in fr]
        n_c = sum(s >= BASE_T for _, s in sc)
        hits = sum(p and s >= BASE_T for p, s in sc)
        picks = sum(p for p, _ in sc)
        print(
            f"{name}: AUC {auc(sc):.3f}  precision {pct(hits / n_c)}"
            f"  coverage {pct(hits / picks)}"
            f"  ({n_c} candidates, {hits} hits, {picks} picks)"
        )

    print("\n## Region sizes (larger side of the frame's two contour regions)")
    for m in MARGINS:
        for name, fr in sets:
            q = quantiles(max(side(r, "left", m), side(r, "right", m)) for r in fr)
            print(f"margin {m} {name}: p10 / median / p90 {q}")
    for name, fr in sets:
        sizes = [
            max(r[e]["iris"][1]["w"], r[e]["iris"][1]["h"])
            for r in fr
            for e in ("left", "right")
            if r[e] and r[e]["iris"][1]
        ]
        ge = sum(s >= 12 for s in sizes)
        print(
            f"iris (margin 0.25) {name}: {len(sizes)} regions, "
            f"p10 / median / p90 {quantiles(sizes)}, >= 12 px {ge}"
        )

    print("\n## AUC by bucket: (b) fitted with no floor, mesh vs window, same frames")
    buckets = [(0, 8), (8, 12), (12, 16), (16, 24), (24, 10**6)]
    for m in MARGINS:
        v = Variant("b", m).fit(train)
        print(f"\nmargin {m}")
        print(
            "| larger side | train n (off) | mesh | window "
            "| held n (off) | mesh | window |"
        )
        print("| --- | --- | --- | --- | --- | --- | --- |")
        for lo, hi in buckets:
            cells = []
            for _, fr in sets:
                sel = [
                    r
                    for r in fr
                    if lo <= max(side(r, "left", m), side(r, "right", m)) < hi
                ]
                sc = [(p, ml, b) for p, ml, b in v.scores(sel) if ml is not None]
                off = sum(not p for p, _, _ in sc)
                ok = 0 < off < len(sc)
                cells += [
                    f"{len(sc)} ({off})",
                    f"{auc([(p, ml) for p, ml, _ in sc]):.3f}" if ok else "-",
                    f"{auc([(p, b) for p, _, b in sc]):.3f}" if ok else "-",
                ]
            hi_s = "+" if hi > 10**5 else f"-{hi - 1}"
            print(f"| {lo}{hi_s} | " + " | ".join(cells) + " |")

    floors = [0, 6, 8, 10, 12, 16, 24]
    print("\n## (b) sharper eye, (d) margin x floor")
    header()
    grid = {}
    for m in MARGINS:
        for fl in floors:
            res = run(Variant("b", m, fl), train, held)
            grid[(m, fl)] = res
            row(f"b m{m} floor {fl}", res)

    # Margin and floor: the highest training AUC of (b); on a tie the lower
    # floor, then the smaller margin. Training data only.
    m, fl = max(grid, key=lambda k: (grid[k][2]["auc"], -k[1], -k[0]))
    print(f"\nchosen on training: margin {m}, floor {fl}")

    results = {}
    for mm, ff in sorted({(m, fl), (0.25, 0), (m, 0)}):
        print(f"\n## eye rules at margin {mm}, floor {ff}")
        header()
        for label, v in rules(mm, ff):
            res = run(v, train, held)
            results[(label, mm, ff)] = res
            row(label, res)

    for mm, ff in sorted({(m, fl), (0.25, 0)}):
        print(f"\n## yaw buckets, pose eye vs sharper eye (margin {mm}, floor {ff})")
        yaw_report(Variant("b", mm, ff).fit(train), train, held)

    # Eye rule (the plan): (f) or (g) if one beats (b) on held-out AUC, else (b).
    b = results[("b sharper", m, fl)]
    pose_rules = [
        k for k in results if k[1:] == (m, fl) and k[0][0] in "fg" and results[k]
    ]
    best = max(pose_rules, key=lambda k: results[k][3]["auc"])
    key = best if results[best][3]["auc"] > b[3]["auc"] else ("b sharper", m, fl)
    v, t, tr, he = results[key]
    print(f"\neye rule: {key[0]}; adopt: {adopted(he)}")
    print(f"w = {v.w!r}, threshold = {t!r}")

    if "--frozen" in sys.argv:
        out = Path(sys.argv[sys.argv.index("--frozen") + 1])
        frozen = {
            "variant": key[0],
            "formula": "c + k1*ln(lap+1) + k2*ln(edge_w/longer side)",
            "c": v.w[0],
            "k1": v.w[1],
            "k2": v.w[2],
            "threshold": t,
            "margin": v.margin,
            "floor": v.floor,
            "eye_rule": "sharper" if v.rule == "b" else v.rule,
            "yaw_cut": v.cut,
            "fallback_threshold": BASE_T,
            "train": tr,
            "heldout": he,
            "adopt": adopted(he),
        }
        out.write_text(json.dumps(frozen, indent=2) + "\n")


main()
