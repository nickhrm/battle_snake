# honey_badger 🐍

A competitive [Battlesnake](https://play.battlesnake.com) written in Rust.

## How it plays

* **Exact rules engine** (`src/game.rs`): movement, starvation, hazard damage (stacked hazards), feeding,
  simultaneous elimination with head-to-head resolution. It supports standard, royale, constrictor and
  wrapped games and is covered by unit tests.
* **Search** (`src/search.rs`): iterative-deepening **paranoid alpha-beta** over simultaneous moves.
  We pick a move, then every nearby opponent replies *knowing* our move. A move only looks good if
  no combination of opponent moves can punish it. Snakes too far away to interact in the remaining
  depth get one cheap heuristic move each, which keeps 4-player games deep enough. The last remaining
  opponent is always searched in full, so a proven win never rests on a guessed move. Each root move is
  searched on its own thread. Killer moves order both max and min nodes.
* **Evaluation** (`src/eval.rs`): a **tail-aware Voronoi** territory count. Cells become available
  once the body segment on them has moved away. It runs as a `u128` bitboard on 11x11 (~0.5 µs).
  Terms: territory vs. the strongest opponent, length advantage, food distance when hungry or not
  clearly longest, trap and starvation penalties, a bonus per eliminated opponent, and discounted
  hazard cells.
* **Time management** (`src/main.rs`): the engine reports our latency every turn. The server
  subtracts its own compute time from that to estimate the network overhead per game, then sizes
  the search budget as `timeout - network - safety`.

## Experiment log

Tried approaches, their self-play results and next ideas: [EXPERIMENTS.md](EXPERIMENTS.md).

## Run locally

```sh
cargo run --release            # listens on $PORT (default 8000)
cargo test --release           # rules + evaluation tests
```

### Self-play arena / tuning

```sh
# A vs B, weights given as (partial) JSON, see eval::Weights
cargo run --release --bin sim -- --mode duel --games 200 --ms 15 --a '{"length":10}' --b '{}'
# modes: duel | standard | royale | constrictor
tools/tune.sh duel 200 10 '{}' '{"food_dist":3}' '{"trap":15}'
cargo run --release --bin bench -- tests/fixtures/standard.json 300   # search depth / speed
```

## Deploy on Coolify (Docker)

1. Push this repo to GitHub/GitLab (or any git remote Coolify can reach).
2. In Coolify: **+ New Resource → Public/Private Repository**, pick the repo and branch.
3. **Build Pack: `Dockerfile`** (the Dockerfile is in the repo root).
4. **Ports Exposes: `8000`**. Set a domain, e.g. `https://snake.example.com`. Coolify's proxy
   handles TLS.
5. Health check: the image has a `HEALTHCHECK` built in (`honey_badger --healthcheck`). If you enable
   the Coolify health check, use path `/` on port `8000`. `curl` is included in the image.
6. Deploy, open `https://snake.example.com/`, and you should see the snake info JSON.
7. On [play.battlesnake.com](https://play.battlesnake.com) → *Create Battlesnake*, enter that URL, then
   join the **Standard**, **Duels**, **Royale** and **Constrictor** leaderboards.

### Environment variables (all optional)

| Variable          | Default   | Meaning                                                         |
|-------------------|-----------|-----------------------------------------------------------------|
| `PORT`            | `8000`    | HTTP port                                                       |
| `NET_ESTIMATE_MS` | `150`     | assumed network overhead before the first latency measurement   |
| `SAFETY_MS`       | `60`      | extra safety margin subtracted from the timeout                 |
| `MAX_BUDGET_MS`   | `360`     | upper limit for thinking time per move                          |
| `PARALLEL`        | auto      | `0` disables the multi-threaded root search                     |
| `SNAKE_COLOR`     | `#E8A317` | customization                                                   |
| `SNAKE_HEAD`      | `evil`    |                                                                 |
| `SNAKE_TAIL`      | `sharp`   |                                                                 |
| `SNAKE_AUTHOR`    | `nickhrm` |                                                                 |
| `SNAKE_WEIGHTS`   | –         | JSON that overrides all evaluation weights (for experiments)    |

### Latency matters

Battlesnake runs its game engine on Google Cloud in three regions: Oregon (US-WEST), Virginia
(US-EAST) and the Netherlands (EUROPE). For your own games you can choose the region. Every
millisecond of network round-trip is thinking time lost, so host the server close to the engine.
The server adapts automatically. Check the `net=` value in the logs: it shows the measured
overhead per game.

Log line per move:
`MOVE game=… turn=… move=up depth=9 score=… nodes=… ms=… budget=… net=…`
