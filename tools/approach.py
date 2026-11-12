#!/usr/bin/env python3
"""Does a variant walk towards food that is ours? Position test for the food race in duels.

usage: tools/approach.py <ms> <variant json>... [--since 'YYYY-MM-DD HH:MM'] [--opponents a,b]

Takes our arena duels against the given opponents (default: the top duel snakes in data/names.json)
and every food of turn 1-60 that lay closer to us than to the opponent when it appeared. On the
positions of the six turns after it appeared (while it is still there and we are still closer) it
counts how often the move of our search shortens the distance to it, separately for food on the
edge. One line per variant, `{}` is the production profile. Needs target/release/trace.
Run with `python3 -I` from the repo root: it reads downloaded data.
"""
import argparse
import collections
import datetime
import json
import os
import subprocess

ME = "honey_badger"
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
DIRS = {"up": (0, 1), "down": (0, -1), "left": (-1, 0), "right": (1, 0)}


def head(s):
    return s["Body"][0]["X"], s["Body"][0]["Y"]


def dist(a, b):
    return abs(a[0] - b[0]) + abs(a[1] - b[1])


def cases_of(path):
    """[(turn, our head, food, on the edge)] of one game"""
    game = json.load(open(path))
    frames, w, h = game["frames"], game["meta"]["Game"]["Width"], game["meta"]["Game"]["Height"]
    food_at = [{(c["X"], c["Y"]) for c in f["Food"]} for f in frames]
    out = []
    for i in range(1, min(len(frames) - 1, 61)):
        for c in food_at[i] - food_at[i - 1]:
            for k in range(i, min(i + 6, len(frames) - 1)):
                alive = [s for s in frames[k]["Snakes"] if not s.get("Death")]
                me = next((s for s in alive if s["Name"] == ME), None)
                if len(alive) != 2 or me is None or c not in food_at[k]:
                    break
                op = next(s for s in alive if s["Name"] != ME)
                if not 1 < dist(head(me), c) < dist(head(op), c):
                    break
                out.append((frames[k]["Turn"], head(me), c, c[0] in (0, w - 1) or c[1] in (0, h - 1)))
    return out


def main():
    p = argparse.ArgumentParser()
    p.add_argument("ms")
    p.add_argument("variants", nargs="+")
    p.add_argument("--since", default="2000-01-01 00:00")
    p.add_argument("--opponents")
    a = p.parse_args()
    since = datetime.datetime.strptime(a.since, "%Y-%m-%d %H:%M").replace(tzinfo=datetime.timezone.utc).timestamp()
    if a.opponents:
        opponents = set(a.opponents.split(","))
    else:
        opponents = set(json.load(open(f"{ROOT}/data/names.json"))["duel"]) - {ME}
    games = {}
    for line in open(f"{ROOT}/data/index.jsonl"):
        g = json.loads(line)
        names = {s["name"] for s in g["snakes"]}
        if g["mode"] == "duel" and g["created"] >= since and ME in names and names & opponents:
            path = f"{ROOT}/data/games/{g['id']}.json"
            cases = cases_of(path)
            if cases:
                games[path] = cases
    print(f"{sum(len(c) for c in games.values())} positions in {len(games)} duels")
    for v in a.variants:
        res = collections.Counter()
        for path, cases in games.items():
            turns = [c[0] for c in cases]
            out = subprocess.run([f"{ROOT}/target/release/trace", a.ms, ME, path, str(min(turns)), str(max(turns))],
                                 capture_output=True, text=True, env={**os.environ, "VARIANT": v}).stdout
            moves = {int(p[0]): p[9] for p in (line.split() for line in out.splitlines()[1:]) if len(p) >= 10}
            for turn, hd, food, edge in cases:
                d = DIRS.get(moves.get(turn))
                if d:
                    res[edge, "n"] += 1
                    res[edge, "closer"] += dist((hd[0] + d[0], hd[1] + d[1]), food) < dist(hd, food)
        rate = lambda e: f"{100 * res[e, 'closer'] / max(1, res[e, 'n']):.1f} % of {res[e, 'n']}"
        print(f"{v}\n   towards our food: on the edge {rate(True)}, elsewhere {rate(False)}")


main()
