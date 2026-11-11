#!/usr/bin/env python3
"""Fit P(win) in duels against position features, per game phase (logistic regression).

usage: tools/fit_duel.py <duelfeat.csv> <names.json> [top|all]

Input from `target/release/duelfeat`. Population "top" (default): both snakes are in the duel top
list of names.json, without us. Rows closer than 25 turns to the end are dropped, there the
features only describe the squeeze that is already won. All features are antisymmetric (the
table has every position from both sides), so there is no intercept.

Output per phase: coefficient (logit), z, and the same in evaluation points, scaled so that one
cell of area difference in the last phase is one point.
"""
import csv
import json
import math
import sys

PHASES = [(0, 60), (60, 150), (150, 300), (300, 100000)]
NAMES = ["area diff /10", "lead >= 1", "lead >= 2", "lead >= 4", "lead >= 7", "health diff /50", "food dist diff"]


def features(r):
    lead = int(r["len"]) - int(r["olen"])
    sg = (lead > 0) - (lead < 0)
    fd, ofd = int(r["fd"]), int(r["ofd"])
    # distance to the nearest food we reach first, 20 if there is none
    fd, ofd = (fd if fd >= 0 else 20), (ofd if ofd >= 0 else 20)
    return [
        (float(r["area"]) - float(r["oarea"])) / 10.0,
        sg * (abs(lead) >= 1), sg * (abs(lead) >= 2), sg * (abs(lead) >= 4), sg * (abs(lead) >= 7),
        (int(r["hp"]) - int(r["ohp"])) / 50.0,
        (ofd - fd) / 10.0,
    ]


def solve(a, b):
    n = len(b)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for i in range(n):
        p = max(range(i, n), key=lambda k: abs(m[k][i]))
        m[i], m[p] = m[p], m[i]
        for k in range(i + 1, n):
            f = m[k][i] / m[i][i]
            for j in range(i, n + 1):
                m[k][j] -= f * m[i][j]
    x = [0.0] * n
    for i in range(n - 1, -1, -1):
        x[i] = (m[i][n] - sum(m[i][j] * x[j] for j in range(i + 1, n))) / m[i][i]
    return x


def fit(xs, ys, ridge=1e-3):
    k = len(xs[0])
    w = [0.0] * k
    for _ in range(12):
        g = [0.0] * k
        h = [[ridge if i == j else 0.0 for j in range(k)] for i in range(k)]
        for x, y in zip(xs, ys):
            z = sum(wi * xi for wi, xi in zip(w, x))
            p = 1.0 / (1.0 + math.exp(-max(-30, min(30, z))))
            for i in range(k):
                if x[i] == 0:
                    continue
                g[i] += (y - p) * x[i]
                for j in range(k):
                    h[i][j] += p * (1 - p) * x[i] * x[j]
        step = solve(h, g)
        w = [wi + si for wi, si in zip(w, step)]
        if max(abs(s) for s in step) < 1e-6:
            break
    # standard errors from the inverse Hessian diagonal
    se = []
    for i in range(k):
        e = [1.0 if j == i else 0.0 for j in range(k)]
        se.append(math.sqrt(max(1e-12, solve(h, e)[i])))
    return w, se


def main():
    top = set(json.load(open(sys.argv[2]))["duel"]) - {"honey_badger"}
    pop = sys.argv[3] if len(sys.argv) > 3 else "top"
    rows = [r for r in csv.DictReader(open(sys.argv[1]))
            if int(r["end"]) - int(r["turn"]) >= 25 and (pop == "all" or (r["snake"] in top and r["opp"] in top))]
    fits = []
    for lo, hi in PHASES:
        sub = [r for r in rows if lo <= int(r["turn"]) < hi]
        xs, ys = [features(r) for r in sub], [int(r["won"]) for r in sub]
        fits.append((lo, hi, len(sub), *fit(xs, ys)))
    scale = 10.0 / fits[-1][3][0]  # points per logit: late area diff of 10 cells = 10 points
    print(f"population {pop}, {len(rows)} rows, {scale:.0f} points per logit")
    for lo, hi, n, w, se in fits:
        print(f"turn {lo}-{hi if hi < 9999 else ''} (n={n})")
        for name, wi, si in zip(NAMES, w, se):
            print(f"   {name:16s} {wi:+.3f}  z {wi / si:+5.1f}   {wi * scale:+7.1f} points")


main()
