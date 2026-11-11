#!/usr/bin/env python3
"""Paired comparison of two `agree` tables (same games, same sampling): does variant B play the
top snakes' moves more often than variant A?

usage: tools/agree_cmp.py <a.csv> <b.csv> <names.json> [min_alive]

Only rows of the top snakes in games they won, with at least `min_alive` living snakes
(default 2; use 3 for the multi-player search). z is the McNemar statistic on the positions where
exactly one of the two variants agrees: |z| > 2 is a real difference.
"""
import csv
import json
import math
import sys


def load(path, top, min_alive):
    out = {}
    for r in csv.DictReader(open(path)):
        if r["snake"] not in top or r["won"] != "1" or r["v_theirs"] == "illegal" or int(r["alive"]) < min_alive:
            continue
        ours, theirs = float(r["v_ours"]), float(r["v_theirs"])
        key = (r["game"], r["turn"], r["snake"])
        out[key] = (r["ours"] == r["theirs"], abs(ours) < 9e5 and abs(theirs) < 9e5 and ours - theirs >= 30,
                    int(r["turn"]), int(r["alive"]))
    return out


def main():
    top = {n for v in json.load(open(sys.argv[3])).values() for n in v}
    min_alive = int(sys.argv[4]) if len(sys.argv) > 4 else 2
    a, b = load(sys.argv[1], top, min_alive), load(sys.argv[2], top, min_alive)
    keys = [k for k in a if k in b]
    bands = [("all", lambda t, n: True), ("turn < 30", lambda t, n: t < 30), ("turn 30-100", lambda t, n: 30 <= t < 100),
             ("turn >= 100", lambda t, n: t >= 100), ("turn >= 250", lambda t, n: t >= 250), ("4 alive", lambda t, n: n == 4), ("3 alive", lambda t, n: n == 3),
             ("2 alive", lambda t, n: n == 2)]
    for name, sel in bands:
        ks = [k for k in keys if sel(a[k][2], a[k][3])]
        if not ks:
            continue
        n = len(ks)
        only_a = sum(a[k][0] and not b[k][0] for k in ks)
        only_b = sum(b[k][0] and not a[k][0] for k in ks)
        z = (only_b - only_a) / math.sqrt(max(1, only_a + only_b))
        print(f"{name:12s} n={n:6d}  A {100 * sum(a[k][0] for k in ks) / n:5.1f} %  B {100 * sum(b[k][0] for k in ks) / n:5.1f} %"
              f"  only A {only_a:4d}  only B {only_b:4d}  z {z:+.1f}   miss A {100 * sum(a[k][1] for k in ks) / n:.1f} %  B {100 * sum(b[k][1] for k in ks) / n:.1f} %")


main()
