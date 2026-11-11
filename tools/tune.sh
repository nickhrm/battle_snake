#!/usr/bin/env bash
# usage: tools/tune.sh <mode> <games> <ms> <variant-json>...
# Plays each variant (A) against the production config (B) and prints one line per variant.
# A variant is a variant object ({"weights": {...}, "focus": true, "duel": {...}}, see CLAUDE.md).
# Both sides use the production config of every position, like the server.
set -e
mode=$1; games=$2; ms=$3; shift 3
bin=${SIM:-"$(dirname "$0")/../target/release/sim"}
for v in "$@"; do
  printf '%-70s ' "$v"
  "$bin" --mode "$mode" --games "$games" --ms "$ms" --seed "${SEED:-1}" --av "$v" --bv '{}' | grep scoreA
done
