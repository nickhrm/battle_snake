//! Runs the search on the test positions from `blunders` and counts how often it avoids the
//! proven loss.
//!
//! suite <ms> <suite.jsonl> [par]
//!
//! Variants via the AGREE_* environment variables (see `apply_env`). Sequential search unless
//! "par" is given.

use honey_badger::api::GameState;
use honey_badger::search::search;
use honey_badger::{apply_env, config_for};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ms: u64 = args[0].parse().expect("ms");
    let par = args.get(2).map(|s| s == "par").unwrap_or(false);
    let rows: Vec<Value> = std::fs::read_to_string(&args[1])
        .expect("suite file")
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .filter(|v| v["kind"] == "blunder")
        .collect();
    let next = AtomicUsize::new(0);
    // per bucket (duel, multi): n, solved, sum depth
    let res = Mutex::new([[0u64; 3]; 2]);
    let failed = Mutex::new(Vec::new());
    let threads = if par { 1 } else { std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4) };
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                if k >= rows.len() {
                    break;
                }
                let v = &rows[k];
                let Ok(gs) = serde_json::from_value::<GameState>(v["request"].clone()) else { continue };
                let st = gs.to_state();
                let mut cfg = config_for(&st, par);
                apply_env(&mut cfg, &st);
                let r = search(&st, &cfg, Instant::now() + Duration::from_millis(ms));
                let solved = v["ok"].as_array().unwrap().iter().any(|m| m == r.best.name());
                let b = if st.snakes.len() == 2 { 0 } else { 1 };
                let mut g = res.lock().unwrap();
                g[b][0] += 1;
                g[b][1] += solved as u64;
                g[b][2] += r.depth as u64;
                if !solved {
                    failed.lock().unwrap().push(format!("{}:{}", &v["game"].as_str().unwrap_or("?")[..8], v["turn"]));
                }
            });
        }
    });
    let g = res.lock().unwrap();
    for (b, name) in [(0, "duel"), (1, "multi")] {
        let n = g[b][0].max(1) as f64;
        println!("{name}: n={} solved={} ({:.1} %) mean depth {:.1}", g[b][0], g[b][1], 100.0 * g[b][1] as f64 / n, g[b][2] as f64 / n);
    }
    if std::env::var("SUITE_FAILED").is_ok() {
        let mut f = failed.lock().unwrap();
        f.sort();
        println!("failed: {}", f.join(" "));
    }
}
