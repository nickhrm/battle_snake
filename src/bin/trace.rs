//! Score trace of one snake through a replay, to find where a game turned.
//!
//! trace <ms> <snake name> <game.json> [from_turn] [to_turn]
//!
//! One line per turn: lengths, health, Voronoi cells (ours / strongest opponent), the search
//! score and depth, our move and the move played. `*` marks turns where the two differ.
//! Variants via the AGREE_* environment variables.

use honey_badger::config_for;
use honey_badger::eval::voronoi_p;
use honey_badger::replay::{is_dead, Replay};
use honey_badger::search::search;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ms: u64 = args[0].parse().expect("ms");
    let rp = Replay::load(&args[2]).expect("replay");
    let from: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    let to: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let id = rp.find(0, &args[1]).expect("snake not in game")["ID"].clone();
    println!("turn  len opp  hp ohp  area opp  depth   score  ours  played");
    for i in from.max(1)..rp.frames.len().min(to.saturating_add(1)) - 1 {
        if rp.snakes(i).iter().any(|s| s["ID"] == id && is_dead(s)) {
            break;
        }
        let (Some(gs), Some(played)) = (rp.request(i, &id), rp.played(i, &id)) else { continue };
        let st = gs.to_state();
        let mut cfg = config_for(&st, false);
        honey_badger::apply_env(&mut cfg, &st);
        let r = search(&st, &cfg, Instant::now() + Duration::from_millis(ms));
        let v = voronoi_p(&st, cfg.weights.hazard_cell, 0.0);
        let opp = (1..st.snakes.len()).max_by_key(|&j| v.cells[j]).unwrap_or(0);
        let o = &st.snakes[opp];
        println!(
            "{:4} {:4} {:3} {:3} {:3} {:5} {:3} {:6} {:7.0}  {:5} {:5} {}",
            gs.turn, st.snakes[0].len(), o.len(), st.snakes[0].health, o.health, v.cells[0], v.cells[opp],
            r.depth, r.score, r.best.name(), played.name(), if r.best != played && r.depth > 0 { "*" } else { "" }
        );
    }
}
