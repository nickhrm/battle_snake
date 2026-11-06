#!/usr/bin/env python3
"""Summarise `agree` output for moves of the top snakes in games they won.

usage: tools/top_report.py <table.csv> <names.json>

agree  = share of positions where our search plays the top snake's move
miss   = share where our search rates their move at least 30 points below its own choice
"""
import collections
import csv
import json
import sys

BANDS = [(0, 25), (25, 100), (100, 250), (250, 100000)]


def main():
    names = json.load(open(sys.argv[2]))
    top = {n for v in names.values() for n in v}
    stats = collections.defaultdict(lambda: [0, 0, 0])
    for r in csv.DictReader(open(sys.argv[1])):
        if r["snake"] not in top or r["won"] != "1" or r["v_theirs"] == "illegal":
            continue
        ours, theirs = float(r["v_ours"]), float(r["v_theirs"])
        if abs(ours) > 9e5 or abs(theirs) > 9e5:
            continue
        mode = "duel" if r["n"] == "2" else "standard"
        turn = int(r["turn"])
        band = next(b for b in BANDS if b[0] <= turn < b[1])
        for key in ((mode, "all"), (mode, band)):
            s = stats[key]
            s[0] += 1
            s[1] += r["theirs"] == r["ours"]
            s[2] += ours - theirs >= 30
    for mode in ("duel", "standard"):
        for band in ["all"] + BANDS:
            n, same, miss = stats[(mode, band)]
            if n:
                print(f"{mode:9s} {str(band):14s} n={n:5d} agree={same / n:.3f} miss={miss / n:.3f}")


main()
