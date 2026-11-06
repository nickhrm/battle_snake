#!/usr/bin/env bash
# usage: tools/gauntlet.sh <mode> <games> <ms> <variant-json> [sim-binary]
# Plays the variant against a pool of different playing styles instead of only against itself.
# Prints one line per style and the mean score.
set -e
mode=$1; games=$2; ms=$3; variant=$4
bin=${5:-"$(dirname "$0")/../target/release/sim"}
styles=(
  'self:{}'
  'hungry:{"food_dist":4,"hunger_health":70,"want_length_lead":8}'
  'territory:{"opp_area":1.2,"length":10}'
  'cautious:{"trap":20,"edge":4}'
  'long:{"length":40,"length_cap":16}'
)
sum=0
for s in "${styles[@]}"; do
  name=${s%%:*}; json=${s#*:}
  line=$("$bin" --mode "$mode" --games "$games" --ms "$ms" --seed 2026 --a "$variant" --b "$json" | tail -1)
  score=$(sed -E 's/.*scoreA=([0-9.]+).*/\1/' <<<"$line")
  printf '%-10s %s\n' "$name" "$line"
  sum=$(python3 -c "print($sum+$score)")
done
python3 -c "print('mean score', round($sum/${#styles[@]}, 3))"
