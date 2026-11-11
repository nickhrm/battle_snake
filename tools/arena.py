#!/usr/bin/env python3
"""Arena data pipeline: download games once, keep them in data/, report from the local copy.

  tools/arena.py sync [--pages N] [--top N] [--top-pages N]   download our games and the top snakes' games
  tools/arena.py board                                        print the current leaderboards
  tools/arena.py reindex                                      rebuild the index from the stored games
  tools/arena.py report [--since 'YYYY-MM-DD HH:MM'] [--until ...] [--mode duel]
                                                              our results from the local copy

Games are stored slimmed as data/games/<id>.json in the format the Rust tools read
({"meta": ..., "frames": [...]}), one summary line per game in data/index.jsonl.
Run with `python3 -I`: the script reads downloaded data.
"""
import argparse
import collections
import datetime
import json
import os
import re
import sys
import time
import urllib.request
from concurrent.futures import ThreadPoolExecutor

BASE = "https://arena.battlesnake.com"
ME = "honey_badger"
PROFILE = "837dbb1c-2781-4e13-8b33-7d1b9c75f7b1"
BOARDS = {
    "duel": "77212f22-20f5-4e0c-8db6-78844318a9bc",
    "standard": "d692fd01-5376-4060-8c4c-91dc2ecfe3ba",
}
HEADERS = {"User-Agent": "battlesnake-analysis"}
ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "data")
KEEP = ("ID", "Name", "Author", "Body", "Health", "Latency", "Death", "Error")


def get(url, as_json=True):
    for attempt in range(5):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=HEADERS), timeout=30) as r:
                data = r.read()
            return json.loads(data) if as_json else data.decode()
        except Exception:
            time.sleep(1 + 2 * attempt)
    return None


def text(html):
    t = re.sub(r"<script.*?</script>|<style.*?</style>", "", html, flags=re.S)
    t = re.sub(r"<[^>]+>", "|", t)
    t = re.sub(r"\s+", " ", t)
    return re.sub(r"(\| ?)+", "|", t)


def leaderboard(board):
    """[(rank, name, author, rating, win%, games, entry_id)]"""
    html = get(f"{BASE}/leaderboards/{board}", as_json=False) or ""
    entries = []
    for m in re.finditer(rf'/leaderboards/{board}/entries/([a-f0-9-]+)', html):
        if m[1] not in entries:
            entries.append(m[1])
    rows = []
    for m in re.finditer(r"\|(\d\d)\|([^|]+)\|by \|([^|]+)\|(?:[^|\d][^|]*\|)*?(-?\d+\.\d)\|(\d+\.\d)\|[\d.]+\|(\d+)(?=\|)", text(html)):
        rows.append([int(m[1]), m[2].strip(), m[3].strip(), float(m[4]), float(m[5]), int(m[6])])
    for i, r in enumerate(rows):
        r.append(entries[i] if i < len(entries) else None)
    return rows


def page_ids(url):
    html = get(url, as_json=False) or ""
    return list(dict.fromkeys(re.findall(r"/games/([a-f0-9-]{36})", html)))


def mode_of(game, n):
    name = game.get("RulesetName") or ""
    if name == "constrictor" or game.get("Map") == "constrictor":
        return "constrictor"
    if name == "royale" or game.get("Map") == "royale":
        return "royale"
    return "duel" if n == 2 else "standard"


def summary(gid, meta, frames):
    g = meta["Game"]
    first, last = frames[0]["Snakes"], frames[-1]["Snakes"]
    snakes = []
    for s in last:
        d = s.get("Death") or {}
        lat = sorted(int(f_s["Latency"] or 0) for f in frames[1:] for f_s in f["Snakes"]
                     if f_s["ID"] == s["ID"] and not f_s.get("Death") and (f_s["Latency"] or "0") != "0")
        snakes.append({
            "name": s["Name"], "author": s.get("Author"), "len": len(s["Body"]),
            "death": d.get("Cause"), "death_turn": d.get("Turn"), "by": d.get("EliminatedBy") or None,
            "lat50": lat[len(lat) // 2] if lat else None,
            # a timeout in the turn of death still counts, later frames repeat the dead snake
            "timeouts": sum(1 for f in frames for f_s in f["Snakes"]
                            if f_s["ID"] == s["ID"] and f_s.get("Error")
                            and not (f_s.get("Death") and f["Turn"] > f_s["Death"]["Turn"])),
            "timeout_death": any(f_s["ID"] == s["ID"] and f_s.get("Error") and f_s.get("Death")
                                 and f["Turn"] == f_s["Death"]["Turn"] for f in frames for f_s in f["Snakes"]),
        })
    id2name = {s["ID"]: s["Name"] for s in last}
    for s in snakes:
        s["by"] = id2name.get(s["by"], s["by"])
    # placement: survivors first, then by death turn (later is better); same turn shares the place
    key = lambda s: 10 ** 9 if s["death"] is None else s["death_turn"]
    order = sorted({key(s) for s in snakes}, reverse=True)
    for s in snakes:
        s["place"] = 1 + sum(1 for o in snakes if key(o) > key(s))
    alive = [s for s in snakes if s["death"] is None]
    return {
        "id": gid, "created": int(g["Created"]) // 1_000_000, "mode": mode_of(g, len(first)),
        "turns": frames[-1]["Turn"], "winner": alive[0]["name"] if len(alive) == 1 else None,
        "snakes": snakes, "_": len(order),
    }


def fetch(gid):
    path = f"{ROOT}/games/{gid}.json"
    if os.path.exists(path):
        return None
    meta = get(f"{BASE}/api/games/{gid}")
    if not meta or meta["Game"].get("Status") != "complete":
        return None
    frames, offset = [], 0
    while True:
        page = (get(f"{BASE}/api/games/{gid}/frames?offset={offset}&limit=100") or {}).get("Frames") or []
        for f in page:
            frames.append({"Turn": f["Turn"], "Food": f["Food"], "Hazards": f.get("Hazards") or [],
                           "Snakes": [{k: s.get(k) for k in KEEP} for s in f["Snakes"]]})
        offset += len(page)
        if len(page) < 100:
            break
    if len(frames) < 2:
        return None
    meta.pop("LastFrame", None)
    tmp = path + ".tmp"
    with open(tmp, "w") as f:
        json.dump({"meta": meta, "frames": frames}, f, separators=(",", ":"))
    os.replace(tmp, path)
    return summary(gid, meta, frames)


def load_index():
    out = {}
    try:
        for line in open(f"{ROOT}/index.jsonl"):
            r = json.loads(line)
            out[r["id"]] = r
    except FileNotFoundError:
        pass
    return out


def cmd_sync(a):
    os.makedirs(f"{ROOT}/games", exist_ok=True)
    have = load_index()
    ids = []
    for p in range(1, a.pages + 1):
        new = page_ids(f"{BASE}/battlesnakes/{PROFILE}/profile?page={p}")
        ids += new
        if new and all(i in have for i in new):
            break  # everything older is already here
    print(f"own: {len(set(ids))} ids", file=sys.stderr)
    names = {}
    if a.top > 0:
        for mode, board in BOARDS.items():
            rows = leaderboard(board)
            names[mode] = [r[1] for r in rows[:a.top]]
            for r in rows[:a.top]:
                if r[6] is None:
                    continue
                for p in range(1, a.top_pages + 1):
                    ids += page_ids(f"{BASE}/leaderboards/{board}/entries/{r[6]}?page={p}")
        with open(f"{ROOT}/names.json", "w") as f:
            json.dump(names, f, ensure_ascii=False)
    todo = [i for i in dict.fromkeys(ids) if i not in have]
    print(f"{len(todo)} games to download", file=sys.stderr)
    n = 0
    with ThreadPoolExecutor(6) as ex, open(f"{ROOT}/index.jsonl", "a") as idx:
        for s in ex.map(fetch, todo):
            if s:
                idx.write(json.dumps(s, ensure_ascii=False) + "\n")
                idx.flush()
                n += 1
    print(f"downloaded {n}, index has {len(load_index())} games")


def cmd_reindex(a):
    """Rebuild data/index.jsonl from the stored games (after a change to `summary`)."""
    import glob
    rows = []
    for path in glob.glob(f"{ROOT}/games/*.json"):
        d = json.load(open(path))
        rows.append(summary(os.path.basename(path)[:-5], d["meta"], d["frames"]))
    rows.sort(key=lambda r: r["created"])
    with open(f"{ROOT}/index.jsonl.tmp", "w") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
    os.replace(f"{ROOT}/index.jsonl.tmp", f"{ROOT}/index.jsonl")
    print(len(rows), "games")


def cmd_board(a):
    for mode, board in BOARDS.items():
        rows = leaderboard(board)
        print(f"== {mode} ({time.strftime('%Y-%m-%d %H:%M UTC', time.gmtime())})")
        for r in rows:
            if r[0] <= a.n or r[1] == ME:
                print(f"{r[0]:3d} {r[1][:28]:28s} {r[3]:5.1f}  win {r[4]:4.1f}%  games {r[5]}")


def ts(s):
    return datetime.datetime.strptime(s, "%Y-%m-%d %H:%M").replace(tzinfo=datetime.timezone.utc).timestamp()


def cmd_report(a):
    since = ts(a.since) if a.since else 0
    until = ts(a.until) if a.until else 1e18
    games = [g for g in load_index().values()
             if since <= g["created"] < until and any(s["name"] == ME for s in g["snakes"])]
    for mode in ("duel", "standard", "royale", "constrictor"):
        if a.mode and a.mode != mode:
            continue
        gs = [g for g in games if g["mode"] == mode]
        if not gs:
            continue
        me = [next(s for s in g["snakes"] if s["name"] == ME) for g in gs]
        wins = sum(g["winner"] == ME for g in gs)
        print(f"\n== {mode}: {len(gs)} games, {wins} wins ({100 * wins / len(gs):.1f} %), "
              f"mean place {sum(s['place'] for s in me) / len(me):.2f}, "
              f"timeouts {sum(s['timeouts'] for s in me)} in {sum(1 for s in me if s['timeouts'])} games, "
              f"died in a timeout turn {sum(1 for s in me if s.get('timeout_death'))}")
        causes = collections.Counter(s["death"] for s in me if s["death"])
        print("   deaths:", ", ".join(f"{k} {v}" for k, v in causes.most_common()))
        dts = sorted(s["death_turn"] for s in me if s["death"])
        if dts:
            print(f"   death turn: p25 {dts[len(dts) // 4]}, median {dts[len(dts) // 2]}, p75 {dts[3 * len(dts) // 4]}")
        if mode in ("duel", "constrictor"):
            ahead = sum(1 for g, s in zip(gs, me) if s["death"] and g["winner"]
                        and s["len"] >= 5 + max(o["len"] for o in g["snakes"] if o["name"] != ME))
            print(f"   losses with a length lead >= 5: {ahead}")
        # head to head: duel = win/loss, multi = we outlived them / they outlived us
        h2h = collections.defaultdict(lambda: [0, 0, 0])
        for g, s in zip(gs, me):
            for o in g["snakes"]:
                if o["name"] == ME:
                    continue
                r = h2h[o["name"]]
                r[0 if s["place"] < o["place"] else 2 if s["place"] > o["place"] else 1] += 1
        print("   opponent                      better-equal-worse")
        for name, r in sorted(h2h.items(), key=lambda kv: -sum(kv[1]))[:a.n]:
            print(f"   {name[:28]:28s} {r[0]:4d} {r[1]:3d} {r[2]:4d}   {100 * r[0] / max(1, r[0] + r[2]):5.1f} %")


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("sync")
    s.add_argument("--pages", type=int, default=70, help="max profile pages of our own history")
    s.add_argument("--top", type=int, default=8, help="top N snakes per board")
    s.add_argument("--top-pages", type=int, default=3)
    s.set_defaults(f=cmd_sync)
    sub.add_parser("reindex").set_defaults(f=cmd_reindex)
    s = sub.add_parser("board")
    s.add_argument("-n", type=int, default=20)
    s.set_defaults(f=cmd_board)
    s = sub.add_parser("report")
    s.add_argument("--since")
    s.add_argument("--until")
    s.add_argument("--mode")
    s.add_argument("-n", type=int, default=25)
    s.set_defaults(f=cmd_report)
    a = p.parse_args()
    a.f(a)


main()
