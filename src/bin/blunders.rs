//! Finds the decisive mistake in lost arena games: the last position that a long search does not
//! prove lost, and whether the move played there walks into a proven loss.
//!
//! blunders <ms> <snake name> <game.json>... > suite.jsonl
//!
//! One line per lost game. kind = "blunder": the played move is proven lost, another move is
//! not (a test position for `suite`). kind = "quiet": the played move was not provably lost,
//! so the mistake lies beyond the horizon of the long search. kind = "lost": lost throughout
//! the scanned window.

use honey_badger::game::Dir;
use honey_badger::replay::{is_dead, Replay};
use honey_badger::search::{is_loss, search};
use honey_badger::{apply_env, config_for};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const WINDOW: usize = 80;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ms: u64 = args[0].parse().expect("ms");
    let name = args[1].as_str();
    let files = &args[2..];
    let next = AtomicUsize::new(0);
    let out = Mutex::new(());
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                if k >= files.len() {
                    break;
                }
                let Some(rp) = Replay::load(&files[k]) else { continue };
                let last = rp.frames.len() - 1;
                let Some(me) = rp.find(last, name) else { continue };
                if !is_dead(me) {
                    continue;
                }
                let id = me["ID"].clone();
                let death = me["Death"]["Turn"].as_u64().unwrap_or(0) as usize;
                // frame index = turn in complete replays; be defensive anyway
                let Some(end) = rp.frames.iter().position(|f| f["Turn"].as_u64() == Some(death as u64)) else { continue };
                let mut kind = "lost";
                let mut rec = json!(null);
                let mut i = end;
                while i > 0 && end - i < WINDOW {
                    i -= 1;
                    let (Some(gs), Some(played)) = (rp.request(i, &id), rp.played(i, &id)) else { continue };
                    let st = gs.to_state();
                    let mut cfg = config_for(&st, false);
                    cfg.exact_root = true;
                    apply_env(&mut cfg, &st);
                    let r = search(&st, &cfg, Instant::now() + Duration::from_millis(ms));
                    if r.depth == 0 {
                        continue; // forced move, or nothing to choose
                    }
                    let ok: Vec<Dir> = r.scores.iter().filter(|x| !is_loss(x.1)).map(|x| x.0).collect();
                    if ok.is_empty() {
                        continue; // already lost here, look further back
                    }
                    let names = |v: &[Dir]| v.iter().map(|d| d.name()).collect::<Vec<_>>();
                    let bad: Vec<Dir> = r.scores.iter().filter(|x| is_loss(x.1)).map(|x| x.0).collect();
                    kind = if bad.contains(&played) { "blunder" } else { "quiet" };
                    let lens: Vec<usize> = st.snakes.iter().map(|s| s.len()).collect();
                    rec = json!({"turn": gs.turn, "before_death": death as i64 - gs.turn as i64, "played": played.name(),
                        "ok": names(&ok), "bad": names(&bad), "depth": r.depth, "score": r.score, "lens": lens,
                        "request": if kind == "blunder" { serde_json::to_value(&gs).unwrap() } else { json!(null) }});
                    break;
                }
                let mut line = json!({"game": rp.id, "players": rp.players(), "ruleset": rp.game["RulesetName"],
                    "kind": kind, "death": death, "cause": me["Death"]["Cause"]});
                if let Some(o) = rec.as_object() {
                    for (k, v) in o {
                        line[k] = v.clone();
                    }
                }
                let _g = out.lock().unwrap();
                println!("{line}");
            });
        }
    });
}
