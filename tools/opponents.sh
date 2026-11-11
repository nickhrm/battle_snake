#!/usr/bin/env bash
# Foreign open-source snakes as local sparring partners (see CLAUDE.md, "Fremde Gegner").
#
#   tools/opponents.sh fetch            clone the pinned commits into data/opponents/
#   tools/opponents.sh build            build them (snork: cargo, Sandworm: gcc)
#   tools/opponents.sh snork [agent]    run snork on 127.0.0.1:5001 in the foreground
#                                       (agent: Flood, Tree or Mobility; default Flood)
#   tools/opponents.sh spar <mode> <games> <variant-json>...
#                                       play every variant against snork, one line each
#
# The code is foreign: it stays in data/ (not in the repo) and its servers only bind to
# localhost. Nothing from it goes into our sources without checking the licence first
# (snork: MIT, Sandworm: no licence file).
#
# Time limits. snork only searches when the game timeout minus its --latency is above 150 ms
# (below that it plays a one-ply move). We give it 160 ms on SNORK_THREADS worker threads and
# ourselves MS (40) ms on one thread; that is about even in duels (EXPERIMENTS.md, round 9).
# Sandworm is a CGI program with compiled-in limits: sandworm400 is the original (400 ms),
# sandworm50 searches 50 ms. Use it with `sim --bcmd data/opponents/build/sandworm50 --timeout 70`.
set -euo pipefail
cd "$(dirname "$0")/.."
D=data/opponents
SIM=${SIM:-target/release/sim}
MS=${MS:-40}
THREADS=${THREADS:-4}
SNORK_THREADS=${SNORK_THREADS:-4}
SEED=${SEED:-1}

fetch_one() { # <dir> <url> <commit>
    if [ ! -d "$D/$1" ]; then
        git clone -q "$2" "$D/$1"
    fi
    git -C "$D/$1" checkout -q "$3"
    echo "$1 at $(git -C "$D/$1" log -1 --format='%h %ad' --date=short)"
}

start_snork() { # <agent>
    TOKIO_WORKER_THREADS=$SNORK_THREADS "$D/snork/target/release/server" --host 127.0.0.1:5001 --latency 100 \
        --config "{\"$1\":{}}" >/dev/null 2>&1 &
    SNORK_PID=$!
    trap 'kill $SNORK_PID 2>/dev/null || true' EXIT
    until curl -s -o /dev/null 127.0.0.1:5001/; do sleep 0.1; done
}

case "${1:-}" in
fetch)
    mkdir -p "$D"
    fetch_one snork https://github.com/wrenger/snork 76bec0c9c76b31b209fbeb8e409bb3e4c9133eb2
    fetch_one Sandworm https://github.com/Bricktech2000/Sandworm 64a79ed33f24b2667db08135c6a1d40ab4c8a41f
    ;;
build)
    (cd "$D/snork" && cargo build --release --bin server)
    mkdir -p "$D/build"
    # make Sandworm's two time limits overridable from the command line
    sed -e 's|^#define TOTAL_TIME 500 / 1000.*|#ifndef TOTAL_TIME\n#define TOTAL_TIME 500 / 1000\n#endif|' \
        -e 's|^#define SEARCH_TIME 400 / 1000.*|#ifndef SEARCH_TIME\n#define SEARCH_TIME 400 / 1000\n#endif|' \
        "$D/Sandworm/move.c" >"$D/build/sandworm_move.c"
    F="-O3 -std=c99 -march=native -flto -w -I$D/Sandworm"
    gcc $F -o "$D/build/sandworm400" "$D/Sandworm/vendor/jsonw.c" "$D/build/sandworm_move.c"
    gcc $F '-DTOTAL_TIME=62 / 1000' '-DSEARCH_TIME=50 / 1000' -o "$D/build/sandworm50" \
        "$D/Sandworm/vendor/jsonw.c" "$D/build/sandworm_move.c"
    ls -l "$D/build"
    ;;
snork)
    TOKIO_WORKER_THREADS=$SNORK_THREADS exec "$D/snork/target/release/server" --host 127.0.0.1:5001 --latency 100 \
        --config "{\"${2:-Flood}\":{}}"
    ;;
spar)
    mode=$2
    games=$3
    shift 3
    start_snork "${SNORK_AGENT:-Flood}"
    for v in "$@"; do
        echo "== $v vs snork"
        "$SIM" --mode "$mode" --games "$games" --ms "$MS" --threads "$THREADS" --seed "$SEED" \
            --av "$v" --bhttp 127.0.0.1:5001 --timeout 260 | tail -2
    done
    ;;
*)
    sed -n '2,10p' "$0"
    exit 1
    ;;
esac
