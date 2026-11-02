#!/usr/bin/env bash
# usage: tools/tune.sh <mode> <games> <ms> <baseline-json> <variant-json>...
# Plays each variant (A) against the baseline (B) and prints one line per variant.
set -e
mode=$1; games=$2; ms=$3; base=$4; shift 4
bin="$(dirname "$0")/../target/release/sim"
for v in "$@"; do
  merged=$(python3 -c 'import json,sys; b=json.loads(sys.argv[1]); b.update(json.loads(sys.argv[2])); print(json.dumps(b))' "$base" "$v")
  printf '%-60s ' "$v"
  "$bin" --mode "$mode" --games "$games" --ms "$ms" --a "$merged" --b "$base"
done
