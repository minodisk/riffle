"""Scratch fit for mesh-eye-mask Step 2; not part of the workspace.

Copied from the archived mesh-eye-focus `fit.py` and extended with a loader for
the per-file JSON lines `riffle-cli maskdump` (the scratch subcommand in
`maskdump.patch`) wrote. The fitting method is the archived one unchanged:
plain maximum-likelihood logistic regression by Newton's method, the sharper
eye (higher per-eye logit) scoring the frame, both eyes carrying the frame's
label, the window fallback only when no eye counts, the mesh threshold keeping
310 of 339 training picks, the held-out set at that frozen threshold, the
fallback logits shifted by `threshold - 1.2194` for the pooled AUC, ties 0.5.
Pure Python 3, no dependencies.

    python fit.py <dir with training-mask.jsonl, heldout-mask.jsonl>
                  <dir with the archived training-dump.jsonl, heldout-dump.jsonl>
                  [--frozen out.json]
"""

import json
import math
import sys
from pathlib import Path

DILATIONS = [0.0, 0.1, 0.25, 0.5, 1.0]
SIDE_FLOORS = [0, 8, 12, 16, 24]
PIXEL_FLOORS = [0, 25, 50, 100, 200, 400]
BASE = (-4.725633355883976, 0.6826458385175557, -1.1949836425055467)
BASE_T = 1.2194
TARGET_COVERAGE = 310  # of 339 training picks: the baseline's 91.4%
# The shipped rectangle model (archived frozen.json), before the intercept
# shift onto CANDIDATE_LOGIT.
FROZEN_W = [-8.158737592019197, 1.9417143647766388, -1.3161251379398846]
FROZEN_T = 0.8343419969086643
# The archived held-out numbers the plan's adoption rule names.
ARCHIVED_HELD = {"auc": 0.8000100826779593, "hits": 328, "candidates": 370, "picks": 342}


def base_logit(r):
    b = r["base"]
    return (
        BASE[0]
        + BASE[1] * math.log(b["lap"] + 1)
        + BASE[2] * math.log(b["edge"] / b["window"])
    )


def load(path):
    frames = []
    for line in Path(path).read_text().splitlines():
        r = json.loads(line)
        if r["flag"] is None or r["base"]["edge"] is None:
            continue
        r["pick"] = r["flag"] == "Pick"
        r["base_logit"] = base_logit(r)
        frames.append(r)
    return frames


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


# --- per-eye regions ----------------------------------------------------------


def region(r, eye, kind, d):
    """(lap, edge width, longer side, pixels) of one eye, or None.

    kind "rect": the current 0.5-margin rectangle of the mask dump; "mask": the
    masked lap and masked edge width at dilation d; "maplap": the masked lap
    and the rectangle edge width of the mask's bounding window; "archived":
    the archived mesh dump's contour rectangle at margin d.
    """
    e = r[eye]
    if e is None:
        return None
    if kind == "archived":
        c = e["contour"][[0.0, 0.25, 0.5, 1.0].index(d)]
        if c is None:
            return None
        return c["lap"], c["edge"], max(c["w"], c["h"]), c["w"] * c["h"]
    if kind == "rect":
        c = e["rect"]
        if c is None:
            return None
        _, _, w, h = c["window"]
        return c["lap"], c["edge"], max(w, h), w * h
    m = e["mask"][DILATIONS.index(d)]
    if m["window"] is None:
        return None
    _, _, w, h = m["window"]
    edge = m["edge"] if kind == "mask" else m["rect_edge"]
    return m["lap"], edge, max(w, h), m["pixels"]


class Variant:
    """The sharper eye over one region kind at one dilation and floor."""

    def __init__(self, kind, d=0.5, floor=0, floor_rule="side"):
        self.kind, self.d, self.floor, self.floor_rule = kind, d, floor, floor_rule

    def label(self):
        rule = "px" if self.floor_rule == "pixels" else "side"
        return f"{self.kind} d{self.d} {rule} {self.floor}"

    def eyes(self, r):
        """The eyes that count for `r`, with their feature rows."""
        out = {}
        for eye in ("left", "right"):
            c = region(r, eye, self.kind, self.d)
            # A zero width (a seed on the window's edge with nowhere to walk,
            # only on masks a few pixels wide) has no log; it counts as no
            # edge width.
            if c is None or c[1] is None or c[1] <= 0:
                continue
            lap, edge, side, pixels = c
            size = pixels if self.floor_rule == "pixels" else side
            if size >= self.floor:
                out[eye] = [math.log(lap + 1), math.log(edge / side)]
        return out

    def train_rows(self, r):
        return list(self.eyes(r).values())

    def chosen(self, w, r):
        """The frame's mesh logit, or None for the window fallback."""
        e = self.eyes(r)
        return max(logit(w, x) for x in e.values()) if e else None

    def fit(self, train):
        rows = [(x, r["pick"]) for r in train for x in self.train_rows(r)]
        self.w = fit(rows)
        return self

    def scores(self, frames):
        """(pick, mesh logit or None, baseline logit) of every frame."""
        return [(r["pick"], self.chosen(self.w, r), r["base_logit"]) for r in frames]


class Control(Variant):
    """The window features, refitted on the frames `Variant` would mesh."""

    def train_rows(self, r):
        return [window_feats(r)] if self.eyes(r) else []

    def chosen(self, w, r):
        return logit(w, window_feats(r)) if self.eyes(r) else None


def window_feats(r):
    b = r["base"]
    return [math.log(b["lap"] + 1), math.log(b["edge"] / b["window"])]


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
        s_tr = v.scores(train)
        t = threshold(s_tr)
    except (ZeroDivisionError, OverflowError, IndexError):
        # Separable (or empty) training rows: no maximum-likelihood fit.
        return None
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
        print(f"| {label} | no fit (separable or no eye counts) |" + " |" * 9)
        return
    _, _, tr, he = result
    print(f"| {label} | {fmt_eval(tr)} | {fmt_eval(he)} |")


def header():
    cols = ["AUC", "prec", "cov", "fallback", "meshed AUC mesh / window"]
    names = [f"train {c}" for c in cols] + [f"held {c}" for c in cols]
    print("| variant | " + " | ".join(names) + " |")
    print("| --- " * (len(names) + 1) + "|")


def beats(e, base):
    """The plan's adoption rule against one baseline's held-out numbers: AUC
    above it, precision not below it, coverage not below it (exact ratios)."""
    return {
        "auc": e["auc"] > base["auc"],
        "precision": e["hits"] * base["candidates"] >= base["hits"] * e["candidates"],
        "coverage": e["hits"] * base["picks"] >= base["hits"] * e["picks"],
    }


def frozen_baseline(frames, w, t):
    """The shipped rectangle model on `frames` with no refit."""
    v = Variant("rect", 0.5, 24)
    v.w = w
    return evaluate(v.scores(frames), t)


def main():
    d = Path(sys.argv[1])
    old = Path(sys.argv[2])
    train = load(d / "training-mask.jsonl")
    held = load(d / "heldout-mask.jsonl")
    a_train = load(old / "training-dump.jsonl")
    a_held = load(old / "heldout-dump.jsonl")
    sets = (("train", train, a_train), ("held", held, a_held))
    print("frames:", len(train), len(held), "archived:", len(a_train), len(a_held))

    print("\n## Dump check: mask dump vs archived mesh dump, file by file")
    for name, fr, ar in sets:
        same = sum(x["name"] == y["name"] for x, y in zip(fr, ar))
        gated = [
            (x, y) for x, y in zip(fr, ar) if y["mesh"] and not x["mesh"]
        ]
        bad = 0
        for x, y in zip(fr, ar):
            if not x["mesh"]:
                continue
            for eye in ("left", "right"):
                a = region(x, eye, "rect", 0.5)
                b = region(y, eye, "archived", 0.5)
                bad += (a is None) != (b is None) or (
                    a is not None and (a[0], a[1], a[2]) != (b[0], b[1], b[2])
                )
        arch24 = Variant("archived", 0.5, 24)
        counted = [(x, y) for x, y in gated if arch24.eyes(y)]
        print(
            f"{name}: names aligned {same}/{len(fr)}, meshed {sum(x['mesh'] for x in fr)}"
            f" (archived {sum(y['mesh'] for y in ar)}), rectangle lap / edge / side"
            f" mismatches on meshed eyes {bad}, gated frames that counted at"
            f" margin 0.5 floor 24 before the gate {len(counted)}"
        )
        for x, y in counted:
            sides = [region(y, e, "archived", 0.5) for e in ("left", "right")]
            print(
                f"  {x['name']} {x['flag']} face {x['side']:.1f} px, archived mesh"
                f" logit {Variant('archived', 0.5, 24).chosen(FROZEN_W, y):.3f}"
                f" (threshold {FROZEN_T:.3f}), window logit {x['base_logit']:.3f}"
                f" (threshold {BASE_T}), eye sides {[s and s[2] for s in sides]}"
            )

    print("\n## (a) baseline: the rectangle, margin 0.5, floor 24")
    header()
    pre = run(Variant("archived", 0.5, 24), a_train, a_held)
    row("archived dump (pre-gate), refit", pre)
    post = run(Variant("rect", 0.5, 24), train, held)
    row("mask dump (post-gate), refit", post)
    ftr = frozen_baseline(train, FROZEN_W, FROZEN_T)
    fhe = frozen_baseline(held, FROZEN_W, FROZEN_T)
    print(f"| mask dump (post-gate), shipped coefficients | {fmt_eval(ftr)} | {fmt_eval(fhe)} |")
    row("control: window refit (post-gate)", run(Control("rect", 0.5, 24), train, held))
    for label, res in (("pre-gate", pre), ("post-gate", post)):
        v, t, _, _ = res
        print(f"{label} refit: w = {v.w!r}, threshold = {t!r}")

    grid = {}
    for kind, title in (
        ("mask", "(b) mask for both measures"),
        ("maplap", "(c) mask for the Laplacian only (rectangle edge width of the mask window)"),
    ):
        for rule, floors in (("side", SIDE_FLOORS), ("pixels", PIXEL_FLOORS)):
            print(f"\n## {title}, floor on the mask window's {'pixel count' if rule == 'pixels' else 'longer side'}")
            header()
            for dd in DILATIONS:
                for fl in floors:
                    v = Variant(kind, dd, fl, rule)
                    res = run(v, train, held)
                    grid[(kind, rule, dd, fl)] = res
                    row(v.label(), res)

    def best(keys):
        keys = [k for k in keys if grid[k] is not None]
        # The highest pooled training AUC; ties: the lower floor, then the
        # smaller dilation. Training data only.
        return max(keys, key=lambda k: (grid[k][2]["auc"], -k[3], -k[2]))

    print("\n## Best cell per measure rule and floor rule (training AUC)")
    header()
    for kind in ("mask", "maplap"):
        for rule in ("side", "pixels"):
            k = best([k for k in grid if k[:2] == (kind, rule)])
            row(grid[k][0].label(), grid[k])

    side_best = best([k for k in grid if k[1] == "side"])
    pix_best = best([k for k in grid if k[1] == "pixels"])
    rule = "pixels" if grid[pix_best][2]["auc"] > grid[side_best][2]["auc"] else "side"
    key = pix_best if rule == "pixels" else side_best
    v, t, tr, he = grid[key]
    print(f"\nfloor rule: {rule} (best side cell {grid[side_best][2]['auc']:.4f}, best pixel cell {grid[pix_best][2]['auc']:.4f})")
    print(f"selected: {v.label()}")
    print(f"w = {v.w!r}, threshold = {t!r}")

    print("\n## Selected cell with its control")
    header()
    row(v.label(), grid[key])
    row("control: window refit", run(Control(v.kind, v.d, v.floor, v.floor_rule), train, held))

    post_he = post[3]
    against = {
        "post-gate refit": beats(he, post_he),
        "post-gate shipped": beats(he, fhe),
        "archived": beats(he, ARCHIVED_HELD),
    }
    print("\n## Adoption rule (held-out)")
    for name, res in against.items():
        print(f"against {name}: {res} -> {all(res.values())}")
    adopt = all(all(res.values()) for res in against.values())
    print(f"adopt: {adopt}")
    e = he
    print(
        f"selected held-out: AUC {e['auc']!r}, precision {e['hits']}/{e['candidates']},"
        f" coverage {e['hits']}/{e['picks']}; post-gate baseline: AUC"
        f" {post_he['auc']!r}, precision {post_he['hits']}/{post_he['candidates']},"
        f" coverage {post_he['hits']}/{post_he['picks']}"
    )
    k = side_best
    e = grid[k][3]
    print(
        f"best longer-side cell {grid[k][0].label()} held-out: AUC {e['auc']!r},"
        f" precision {e['hits']}/{e['candidates']}, coverage {e['hits']}/{e['picks']}"
    )

    # Post hoc, for the Decision only (selection stays on training data):
    # the cells whose held-out numbers would pass the rule.
    print("\n## Cells passing the rule on held-out (post hoc, not a selection)")
    header()
    for k, res in grid.items():
        if res is None:
            continue
        if all(all(beats(res[3], b).values()) for b in (post_he, fhe, ARCHIVED_HELD)):
            row(res[0].label(), res)

    if "--frozen" in sys.argv:
        out = Path(sys.argv[sys.argv.index("--frozen") + 1])
        frozen = {
            "variant": "b sharper",
            "formula": "c + k1*ln(lap+1) + k2*ln(edge_w/longer side)",
            "c": v.w[0],
            "k1": v.w[1],
            "k2": v.w[2],
            "threshold": t,
            "region": "eyelid contour mask",
            "dilation": v.d,
            "floor": v.floor,
            "floor_rule": "mask pixel count" if v.floor_rule == "pixels" else "mask window longer side",
            "measure_rule": (
                "masked lap + masked edge width"
                if v.kind == "mask"
                else "masked lap + rectangle edge width of the mask window"
            ),
            "eye_rule": "sharper",
            "fallback_threshold": BASE_T,
            "train": tr,
            "heldout": he,
            "baseline_post_gate": {"train": post[2], "heldout": post_he},
            "adopt": adopt,
        }
        out.write_text(json.dumps(frozen, indent=2) + "\n")


main()
