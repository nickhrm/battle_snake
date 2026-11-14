#!/usr/bin/env python3
"""Who takes food that lies next to the head? Arena duels, by phase and length lead.

  tools/appetite.py snakes [--since 'YYYY-MM-DD HH:MM'] [--me-since ...] [--names a,b]
      one row per snake: share of the positions with food next to its head in which it eats,
      by phase (turn < 100, 100-250, >= 250) and by its length lead over the opponent
  tools/appetite.py agree <names.json> <a.csv> [b.csv ...]
      the same question for `agree` tables: on the positions of the top snakes (in games they
      won) with food next to the head, how often they eat and how often each of our variants does

Only positions where the snake has at least 50 health and the food is not next to the opponent's
head as well (that is a head-to-head question, not appetite). A snake that does not care about
food takes it in 35-50 % of the cases (one of two or three safe moves).
Run with `python3 -I` from the repo root: it reads downloaded data.
"""
import argparse
import collections
import csv
import datetime
import functools
import json
import os

ME = "honey_badger"
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data")
DIRS = {"up": (0, 1), "down": (0, -1), "left": (-1, 0), "right": (1, 0)}
PHASES = ("turn < 100", "100-250", ">= 250")
LEADS = ("<= 0", "1-2", "3-5", "> 5")


def head(s):
    return s["Body"][0]["X"], s["Body"][0]["Y"]


def dist(a, b):
    return abs(a[0] - b[0]) + abs(a[1] - b[1])


def phase(turn):
    return PHASES[0] if turn < 100 else PHASES[1] if turn < 250 else PHASES[2]


def near_food(frame, a, b):
    """Food next to the head of `a` and not next to the head of `b`."""
    return {c for c in ((f["X"], f["Y"]) for f in frame["Food"]) if dist(c, head(a)) == 1 and dist(c, head(b)) != 1}


def ts(s):
    return datetime.datetime.strptime(s, "%Y-%m-%d %H:%M").replace(tzinfo=datetime.timezone.utc).timestamp()


@functools.lru_cache(maxsize=4)
def frames(gid):
    return json.load(open(f"{ROOT}/games/{gid}.json"))["frames"]


def cmd_snakes(a):
    since, me_since = ts(a.since), ts(a.me_since or a.since)
    names = a.names.split(",") if a.names else [ME] + json.load(open(f"{ROOT}/names.json"))["duel"]
    cnt = collections.defaultdict(lambda: [0, 0])
    for line in open(f"{ROOT}/index.jsonl"):
        g = json.loads(line)
        ours = any(s["name"] == ME for s in g["snakes"])
        if g["mode"] != "duel" or g["created"] < (me_since if ours else since):
            continue
        try:
            fr = frames(g["id"])
        except FileNotFoundError:
            continue
        for f, nf in zip(fr, fr[1:]):
            alive = [s for s in f["Snakes"] if not s.get("Death")]
            if len(alive) != 2:
                break
            for x, y in (alive, alive[::-1]):
                if x["Name"] not in names or x["Health"] < 50:
                    continue
                near = near_food(f, x, y)
                after = next((s for s in nf["Snakes"] if s["ID"] == x["ID"]), None)
                if not near or not after or after.get("Death"):
                    continue
                lead = len(x["Body"]) - len(y["Body"])
                band = LEADS[0] if lead <= 0 else LEADS[1] if lead <= 2 else LEADS[2] if lead <= 5 else LEADS[3]
                for key in ((x["Name"], phase(f["Turn"]), band), (x["Name"], phase(f["Turn"]), "all")):
                    cnt[key][0] += head(after) in near
                    cnt[key][1] += 1
    for ph in PHASES:
        print(f"== {ph}: food next to the head taken, by length lead (n)")
        print(f"{'':18s}" + "".join(f"{b:>13s}" for b in ("all",) + LEADS))
        for n in names:
            cells = [cnt[(n, ph, b)] for b in ("all",) + LEADS]
            print(f"{n[:18]:18s}" + "".join(f"{100 * e / t:5.0f} % ({t:4d})" if t else f"{'-':>13s}" for e, t in cells))


def cmd_agree(a):
    top = set(json.load(open(a.names))["duel"])
    tables = []
    for path in a.tables:
        rows = {}
        for r in csv.DictReader(open(path)):
            if r["snake"] in top and r["won"] == "1" and r["alive"] == "2":
                rows[(r["game"], int(r["turn"]), r["snake"])] = r
        tables.append(rows)
    res = collections.defaultdict(lambda: [0] * (len(tables) + 2))
    for key in sorted(set.intersection(*[set(t) for t in tables])):
        gid, turn, name = key
        f = frames(gid)[turn]
        alive = [s for s in f["Snakes"] if not s.get("Death")]
        x = next(s for s in alive if s["Name"] == name)
        y = next(s for s in alive if s["Name"] != name)
        near = near_food(f, x, y)
        if not near or x["Health"] < 50:
            continue
        step = lambda m: (head(x)[0] + DIRS[m][0], head(x)[1] + DIRS[m][1])
        r = res[phase(turn)]
        r[0] += 1
        r[1] += step(tables[0][key]["theirs"]) in near
        for i, t in enumerate(tables):
            r[2 + i] += step(t[key]["ours"]) in near
    print(f"{'phase':12s}{'n':>6s}  {'top':>7s}  " + "  ".join(f"{os.path.basename(p)[:12]:>12s}" for p in a.tables))
    for ph in PHASES:
        r = res[ph]
        if r[0]:
            print(f"{ph:12s}{r[0]:6d}  {100 * r[1] / r[0]:5.1f} %  " + "  ".join(f"{100 * v / r[0]:10.1f} %" for v in r[2:]))


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("snakes")
    s.add_argument("--since", default="2026-11-09 00:00")
    s.add_argument("--me-since", help="our own games only from this time on (after a deploy)")
    s.add_argument("--names", help="snakes to list (default: us and the top duel snakes in data/names.json)")
    s.set_defaults(f=cmd_snakes)
    s = sub.add_parser("agree")
    s.add_argument("names")
    s.add_argument("tables", nargs="+")
    s.set_defaults(f=cmd_agree)
    a = p.parse_args()
    a.f(a)


main()
