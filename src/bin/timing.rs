//! How does the timeout-aware deadline split our arena moves?
//!
//! timing <budget_ms> <early_ms> <late_ms> <every_nth_turn> <snake name> <game.json>...
//!
//! Prints the share of moves answered early (a timeout would be costly), on time, and late
//! (the timeout move is our own choice), with the mean answer time and depth of each class.

use honey_badger::config_for;
use honey_badger::replay::{is_dead, Replay};
use honey_badger::search::search;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let num = |i: usize| args[i].parse::<u64>().expect("number");
    let (budget, early, late, nth) = (num(0), num(1), num(2), num(3) as usize);
    let name = args[4].as_str();
    let files = &args[5..];
    let next = AtomicUsize::new(0);
    // per mode (duel, multi) and class (early, normal, late): n, ms, depth
    let res = Mutex::new([[[0f64; 3]; 3]; 2]);
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                if k >= files.len() {
                    break;
                }
                let Some(rp) = Replay::load(&files[k]) else { continue };
                let Some(me) = rp.find(0, name) else { continue };
                let id = me["ID"].clone();
                for i in (1 + k % nth..rp.frames.len() - 1).step_by(nth) {
                    if rp.snakes(i).iter().any(|s| s["ID"] == id && is_dead(s)) {
                        break;
                    }
                    let Some(gs) = rp.request(i, &id) else { continue };
                    let st = gs.to_state();
                    let mut cfg = config_for(&st, false);
                    honey_badger::apply_env(&mut cfg, &st);
                    cfg.fallback = Some(gs.you.last_move());
                    cfg.early = Duration::from_millis(early);
                    cfg.late = Duration::from_millis(late);
                    let t = Instant::now();
                    let r = search(&st, &cfg, t + Duration::from_millis(budget));
                    if r.depth == 0 {
                        continue; // forced move
                    }
                    let ms = t.elapsed().as_millis() as u64;
                    if ms + 5 < budget - early {
                        continue; // proven win or loss
                    }
                    let class = if ms + 5 < budget { 0 } else if ms > budget + 5 { 2 } else { 1 };
                    let b = if st.snakes.len() == 2 { 0 } else { 1 };
                    let mut g = res.lock().unwrap();
                    g[b][class][0] += 1.0;
                    g[b][class][1] += ms as f64;
                    g[b][class][2] += r.depth as f64;
                }
            });
        }
    });
    let g = res.lock().unwrap();
    for (b, mode) in [(0, "duel"), (1, "multi")] {
        let n: f64 = g[b].iter().map(|c| c[0]).sum::<f64>().max(1.0);
        let ms: f64 = g[b].iter().map(|c| c[1]).sum::<f64>() / n;
        println!("{mode}: n={n} mean ms {ms:.0}");
        for (c, class) in ["early", "normal", "late"].iter().enumerate() {
            let k = g[b][c][0].max(1.0);
            println!("  {class:6} {:5.1} %  ms {:.0}  depth {:.1}", 100.0 * g[b][c][0] / n, g[b][c][1] / k, g[b][c][2] / k);
        }
    }
}
