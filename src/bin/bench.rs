//! Search benchmark on fixture positions: cargo run --release --bin bench -- tests/fixtures/duel.json 300
use honey_badger::{api::GameState, config_for, search::search};
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("tests/fixtures/duel.json");
    let ms: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(300);
    let parallel = args.get(3).map(|s| s != "seq").unwrap_or(true);
    let gs: GameState = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let st = gs.to_state();
    let mut cfg = config_for(&st, parallel);
    honey_badger::apply_env(&mut cfg, &st);
    // BENCH_TIMEOUT=early_ms,late_ms: timeout-aware deadline as on the server
    if let Ok(v) = std::env::var("BENCH_TIMEOUT") {
        let (e, l) = v.split_once(',').expect("early,late");
        cfg.fallback = Some(gs.you.last_move());
        cfg.early = Duration::from_millis(e.parse().unwrap());
        cfg.late = Duration::from_millis(l.parse().unwrap());
    }
    if args.get(3).map(|s| s == "micro").unwrap_or(false) {
        micro(&st);
        return;
    }
    for _ in 0..3 {
        let t = Instant::now();
        let r = search(&st, &cfg, t + Duration::from_millis(ms));
        let el = t.elapsed().as_secs_f64();
        println!(
            "best={} ms={:.0} depth={} score={:.1} nodes={} knps={:.0} scores={:?}",
            r.best.name(),
            el * 1000.0,
            r.depth,
            r.score,
            r.nodes,
            r.nodes as f64 / el / 1000.0,
            r.scores
        );
    }
}

#[allow(dead_code)]
pub fn micro(st: &honey_badger::game::State) {
    use honey_badger::eval::{evaluate, Weights};
    use honey_badger::game::Dir;
    let w = Weights::default();
    let t = Instant::now();
    let mut acc = 0.0;
    for _ in 0..200_000 {
        acc += evaluate(std::hint::black_box(st), &w);
    }
    println!(
        "eval: {:.0} ns ({acc})",
        t.elapsed().as_nanos() as f64 / 200_000.0
    );
    let t = Instant::now();
    let mut z = 0u32;
    for _ in 0..200_000 {
        z += honey_badger::eval::voronoi(std::hint::black_box(st), 0.25).cells[0] as u32;
    }
    println!(
        "voronoi: {:.0} ns ({z}) bitboard={}",
        t.elapsed().as_nanos() as f64 / 200_000.0,
        st.rules.bitboard
    );
    let t = Instant::now();
    let mut k = 0;
    for _ in 0..200_000 {
        let mut c = std::hint::black_box(st).clone();
        c.advance(&[Dir::Up, Dir::Left, Dir::Left, Dir::Left]);
        k += c.snakes.len();
    }
    println!(
        "clone+advance: {:.0} ns ({k})",
        t.elapsed().as_nanos() as f64 / 200_000.0
    );
    let t = Instant::now();
    for _ in 0..200_000 {
        let c = std::hint::black_box(st).clone();
        k += c.snakes.len();
    }
    println!(
        "clone: {:.0} ns ({k})",
        t.elapsed().as_nanos() as f64 / 200_000.0
    );
}
