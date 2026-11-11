#!/usr/bin/env python3
"""Draw the board of a stored arena game at one or more turns.

usage: tools/show.py <game id or prefix> <turn> [turn...]

Heads are capital letters (A = first snake in the game), bodies lower case, tails `'`, food `*`,
hazards `~`. Run with `python3 -I`.
"""
import glob
import json
import os
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data", "games")


def main():
    path = glob.glob(f"{ROOT}/{sys.argv[1]}*.json")[0]
    g = json.load(open(path))
    w, h = g["meta"]["Game"]["Width"], g["meta"]["Game"]["Height"]
    for t in map(int, sys.argv[2:]):
        f = g["frames"][t]
        grid = [["." for _ in range(w)] for _ in range(h)]
        for c in f.get("Hazards") or []:
            grid[c["Y"]][c["X"]] = "~"
        for c in f["Food"]:
            grid[c["Y"]][c["X"]] = "*"
        legend = []
        for i, s in enumerate(f["Snakes"]):
            dead = s["Death"] and s["Death"]["Turn"] <= f["Turn"]
            legend.append(f"{chr(65 + i)}={s['Name']} len {len(s['Body'])} hp {s['Health']}" + (" (dead)" if dead else ""))
            if dead:
                continue
            for k, c in reversed(list(enumerate(s["Body"]))):
                if 0 <= c["X"] < w and 0 <= c["Y"] < h:
                    grid[c["Y"]][c["X"]] = chr(65 + i) if k == 0 else "'" if k == len(s["Body"]) - 1 else chr(97 + i)
        print(f"turn {f['Turn']}: " + ", ".join(legend))
        for y in range(h - 1, -1, -1):
            print(f"{y:2d} " + " ".join(grid[y]))
        print("   " + " ".join(str(x % 10) for x in range(w)))


main()
