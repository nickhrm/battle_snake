#!/usr/bin/env python3
"""Download recent arena games of the top snakes for tools/top_report.py.

usage: tools/fetch_top.py <out_dir> [top_n] [history_pages]
"""
import json
import os
import re
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

BASE = "https://arena.battlesnake.com"
BOARDS = {
    "duel": "77212f22-20f5-4e0c-8db6-78844318a9bc",
    "standard": "d692fd01-5376-4060-8c4c-91dc2ecfe3ba",
}
HEADERS = {"User-Agent": "battlesnake-analysis"}


def get(url, as_json=True):
    for attempt in range(4):
        try:
            with urllib.request.urlopen(
                urllib.request.Request(url, headers=HEADERS), timeout=30
            ) as r:
                data = r.read()
            return json.loads(data) if as_json else data.decode()
        except Exception:
            time.sleep(1 + 2 * attempt)
    return None


def top_entries(board, n):
    html = get(f"{BASE}/leaderboards/{board}", as_json=False) or ""
    seen = []
    for m in re.finditer(rf'/leaderboards/{board}/entries/([a-f0-9-]+)"[^>]*>(?:<[^>]+>)*([^<]+)<', html):
        if m[1] not in [e for e, _ in seen]:
            seen.append((m[1], m[2].strip()))
    return seen[:n]


def game_ids(board, entry, pages):
    ids = set()
    for p in range(pages):
        html = get(f"{BASE}/leaderboards/{board}/entries/{entry}?page={p}", as_json=False) or ""
        ids |= set(re.findall(r"/games/([a-f0-9-]{36})", html))
    return ids


def fetch(job):
    mode, gid, out = job
    path = f"{out}/{mode}_{gid}.json"
    if os.path.exists(path):
        return
    meta = get(f"{BASE}/api/games/{gid}")
    if not meta or meta["Game"].get("Status") != "complete":
        return
    frames, offset = [], 0
    while True:
        page = (get(f"{BASE}/api/games/{gid}/frames?offset={offset}&limit=100") or {}).get("Frames") or []
        frames += page
        offset += len(page)
        if len(page) < 100:
            break
    with open(path, "w") as f:
        json.dump({"meta": meta, "frames": frames}, f)


def main():
    out = sys.argv[1]
    top_n = int(sys.argv[2]) if len(sys.argv) > 2 else 5
    pages = int(sys.argv[3]) if len(sys.argv) > 3 else 2
    os.makedirs(out, exist_ok=True)
    names, jobs = {}, []
    for mode, board in BOARDS.items():
        names[mode] = []
        for entry, name in top_entries(board, top_n):
            names[mode].append(name)
            jobs += [(mode, g, out) for g in game_ids(board, entry, pages)]
    with open(f"{out}/names.json", "w") as f:
        json.dump(names, f, ensure_ascii=False)
    jobs = list(set(jobs))
    print(names, len(jobs), "games")
    with ThreadPoolExecutor(4) as ex:
        list(ex.map(fetch, jobs))
    print("FETCHDONE")


main()
