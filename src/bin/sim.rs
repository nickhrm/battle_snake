//! Self-play arena: pits two configurations against each other with the
//! official rules (food spawning, royale shrinking, constrictor growth).
//!
//! cargo run --release --bin sim -- --mode duel --games 200 --ms 20 --a '{"length":8}' --b '{}'

use honey_badger::eval::Weights;
use honey_badger::game::{Dir, Rules, Snake, State, NONE};
use honey_badger::search::{search, SearchConfig};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Duel,
    Standard,
    Royale,
    Constrictor,
}

#[derive(Clone)]
struct Player {
    cfg: SearchConfig,
    ms: u64,
    /// simple heuristic snake instead of the search (sparring partner)
    greedy: bool,
    /// external snake server (e.g. "127.0.0.1:5001"), queried like the real engine does
    http: Option<String>,
}

fn mode_names(mode: Mode) -> (&'static str, &'static str) {
    match mode {
        Mode::Duel | Mode::Standard => ("standard", "standard"),
        Mode::Royale => ("royale", "royale"),
        Mode::Constrictor => ("constrictor", "standard"),
    }
}

/// Engine-compatible JSON for snake `you` (index in `st.snakes`).
fn engine_json(st: &State, you: usize, mode: Mode, game_id: &str, timeout: u64) -> String {
    use serde_json::json;
    let r = &*st.rules;
    let c = |cell: u16| {
        let (x, y) = r.xy(cell);
        json!({"x": x, "y": y})
    };
    let snake = |i: usize| {
        let s = &st.snakes[i];
        json!({
            "id": format!("s{i}"), "name": format!("s{i}"), "health": s.health,
            "body": s.body.iter().map(|&b| c(b)).collect::<Vec<_>>(),
            "head": c(s.head()), "length": s.len(), "latency": "0", "shout": "", "squad": "",
            "customizations": {"color": "#888888", "head": "default", "tail": "default"}
        })
    };
    let (name, map) = mode_names(mode);
    let hazards: Vec<_> = (0..r.cells() as u16).filter(|&h| r.hazards[h as usize] > 0).map(c).collect();
    let v = json!({
        "game": {
            "id": game_id,
            "ruleset": {"name": name, "version": "v1.2.3", "settings": {
                "foodSpawnChance": 15,
                "minimumFood": 1, "hazardDamagePerTurn": if mode == Mode::Royale { 14 } else { 0 },
                "hazardMap": "", "hazardMapAuthor": "",
                "royale": {"shrinkEveryNTurns": 25},
                "squad": {"allowBodyCollisions": false, "sharedElimination": false, "sharedHealth": false, "sharedLength": false}
            }},
            "map": map, "timeout": timeout, "source": "custom"
        },
        "turn": st.turn,
        "board": {
            "height": r.height, "width": r.width,
            "food": st.food.iter().map(|&f| c(f)).collect::<Vec<_>>(),
            "hazards": hazards,
            "snakes": (0..st.snakes.len()).filter(|&i| st.snakes[i].alive).map(snake).collect::<Vec<_>>()
        },
        "you": snake(you)
    });
    v.to_string()
}

fn http_post(addr: &str, path: &str, body: &str, timeout: Duration) -> Option<String> {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(addr).ok()?;
    s.set_read_timeout(Some(timeout)).ok()?;
    s.set_nodelay(true).ok()?;
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = String::new();
    s.read_to_string(&mut buf).ok()?;
    let i = buf.find("\r\n\r\n")?;
    Some(buf[i + 4..].to_string())
}

/// Ask an external snake for its move; on timeout/error keep going straight (engine behaviour).
fn http_move(addr: &str, st: &State, you: usize, mode: Mode, game_id: &str, timeout: u64) -> (Dir, bool) {
    let s = &st.snakes[you];
    let straight = {
        let r = &*st.rules;
        let (hx, hy) = r.xy(s.head());
        let (nx, ny) = r.xy(s.body[1]);
        match (hx - nx, hy - ny) {
            (0, -1) => Dir::Down,
            (-1, 0) => Dir::Left,
            (1, 0) => Dir::Right,
            _ => Dir::Up,
        }
    };
    let body = engine_json(st, you, mode, game_id, timeout);
    let t0 = Instant::now();
    let resp = http_post(addr, "/move", &body, Duration::from_millis(timeout + 200));
    let late = t0.elapsed() > Duration::from_millis(timeout);
    let mv = resp.and_then(|b| {
        let v: serde_json::Value = serde_json::from_str(&b).ok()?;
        match v.get("move")?.as_str()? {
            "up" => Some(Dir::Up),
            "down" => Some(Dir::Down),
            "left" => Some(Dir::Left),
            "right" => Some(Dir::Right),
            _ => None,
        }
    });
    match mv {
        Some(m) if !late => (m, false),
        _ => (straight, true),
    }
}

/// A typical "decent" heuristic snake: flood fill for space, avoids heads of
/// bigger snakes, goes for food when hungry or not the longest, hunts smaller heads.
fn greedy_move(st: &State) -> Dir {
    let mut moves = [Dir::Up; 4];
    let n = st.safe_moves(0, &mut moves);
    if n == 0 {
        return st.fallback_move(0);
    }
    let r = &*st.rules;
    let me = st.me();
    let my_len = me.len();
    let max_opp = st
        .snakes
        .iter()
        .skip(1)
        .filter(|s| s.alive)
        .map(|s| s.len())
        .max()
        .unwrap_or(0);
    let mut best = moves[0];
    let mut best_v = f32::MIN;
    for &m in &moves[..n] {
        let c = r.step(me.head(), m);
        let mut v = 0.0f32;
        // head danger / opportunity
        for o in st.snakes.iter().skip(1).filter(|s| s.alive) {
            if r.dist(o.head(), c) == 1 {
                if o.len() >= my_len {
                    v -= 500.0
                } else {
                    v += 30.0
                }
            }
        }
        // flood fill from c
        let mut seen = vec![false; r.cells()];
        for s in st.snakes.iter().filter(|s| s.alive) {
            let k = if s.tail_moves() { s.len() - 1 } else { s.len() };
            for &b in s.body.iter().take(k) {
                seen[b as usize] = true;
            }
        }
        let mut q = vec![c];
        seen[c as usize] = true;
        let mut i = 0;
        while i < q.len() && q.len() < 2 * my_len + 10 {
            let x = q[i];
            i += 1;
            for d in Dir::ALL {
                let y = r.step(x, d);
                if y != NONE && !seen[y as usize] {
                    seen[y as usize] = true;
                    q.push(y);
                }
            }
        }
        let space = q.len();
        if space < my_len {
            v -= 300.0 - space as f32;
        }
        v += space.min(2 * my_len + 10) as f32 * 0.5;
        // food
        if me.health < 50 || my_len <= max_opp + 1 {
            let fd = st.food.iter().map(|&f| r.dist(f, c)).min().unwrap_or(30);
            v -= fd as f32 * 3.0;
        }
        if r.hazards[c as usize] > 0 {
            v -= 40.0;
        }
        if v > best_v {
            best_v = v;
            best = m;
        }
    }
    best
}

struct Args {
    mode: Mode,
    games: usize,
    threads: usize,
    seed: u64,
    a: Player,
    b: Player,
    verbose: bool,
    timeout: u64,
    dump: Option<String>,
    /// optional field snake: in 4-player games seats 2 and 3 play this config, and A is
    /// scored against B by who survived longer
    x: Option<Player>,
}

/// `s` overrides the production weights of the mode, so a test sees what the server plays.
fn parse_weights(s: &str, mode: Mode) -> Weights {
    let players = if mode == Mode::Duel { 2 } else { 4 };
    let base = honey_badger::profile(&World::new(mode, players, 1).st);
    let mut v = serde_json::to_value(base).unwrap();
    let over: serde_json::Value =
        serde_json::from_str(s).unwrap_or_else(|e| panic!("bad weights json {s}: {e}"));
    for (k, x) in over.as_object().expect("weights must be a JSON object") {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

fn parse_args() -> Args {
    let mut it = std::env::args().skip(1);
    let mut mode = Mode::Duel;
    let mut games = 100;
    let mut threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let mut seed = 1;
    let mut a = Player {
        cfg: SearchConfig {
            parallel: false,
            ..Default::default()
        },
        ms: 20,
        greedy: false,
        http: None,
    };
    let mut b = a.clone();
    let mut verbose = false;
    let mut timeout = 500;
    let mut dump: Option<String> = None;
    let mut a_json = "{}".to_string();
    let mut b_json = "{}".to_string();
    let mut x_json: Option<String> = None;
    let mut x_reach = 99;
    let mut x_greedy = false;
    while let Some(k) = it.next() {
        let mut v = || it.next().expect("missing value");
        match k.as_str() {
            "--mode" => {
                mode = match v().as_str() {
                    "duel" => Mode::Duel,
                    "standard" => Mode::Standard,
                    "royale" => Mode::Royale,
                    "constrictor" => Mode::Constrictor,
                    m => panic!("unknown mode {m}"),
                }
            }
            "--games" => games = v().parse().unwrap(),
            "--threads" => threads = v().parse().unwrap(),
            "--seed" => seed = v().parse().unwrap(),
            "--ms" => {
                let ms = v().parse().unwrap();
                a.ms = ms;
                b.ms = ms;
            }
            "--ams" => a.ms = v().parse().unwrap(),
            "--bms" => b.ms = v().parse().unwrap(),
            "--a" => a_json = v(),
            "--b" => b_json = v(),
            "--x" => x_json = Some(v()),
            "--xreach" => x_reach = v().parse().unwrap(),
            "--xgreedy" => x_greedy = true,
            "--bgreedy" => b.greedy = true,
            "--bhttp" => b.http = Some(v()),
            "--timeout" => timeout = v().parse().unwrap(),
            "--apar" => a.cfg.parallel = true,
            "--dump" => dump = Some(v()),
            "--arel" => a.cfg.max_relevant = v().parse().unwrap(),
            "--brel" => b.cfg.max_relevant = v().parse().unwrap(),
            "--areach" => a.cfg.max_reach = v().parse().unwrap(),
            "--breach" => b.cfg.max_reach = v().parse().unwrap(),
            "--adepth" => a.cfg.max_depth = v().parse().unwrap(),
            "--bdepth" => b.cfg.max_depth = v().parse().unwrap(),
            "-v" => verbose = true,
            _ => panic!("unknown arg {k}"),
        }
    }
    a.cfg.weights = parse_weights(&a_json, mode);
    b.cfg.weights = parse_weights(&b_json, mode);
    let x = x_json.map(|js| {
        let mut p = b.clone();
        p.cfg.weights = parse_weights(&js, mode);
        p.cfg.max_reach = x_reach;
        p.greedy = x_greedy;
        p
    });
    Args {
        x,
        mode,
        games,
        threads,
        seed,
        a,
        b,
        verbose,
        timeout,
        dump,
    }
}

struct World {
    st: State,
    rng: SmallRng,
    mode: Mode,
    // royale safe zone
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl World {
    fn new(mode: Mode, n: usize, seed: u64) -> World {
        let mut rng = SmallRng::seed_from_u64(seed);
        let (w, h) = (11, 11);
        let dmg = if mode == Mode::Royale { 14 } else { 0 };
        let mut rules = Rules::new(w, h, false, mode == Mode::Constrictor, dmg);
        rules.royale = mode == Mode::Royale;
        let corners = [(1, 1), (1, 9), (9, 1), (9, 9)];
        let cardinals = [(1, 5), (5, 1), (5, 9), (9, 5)];
        let mut pts: Vec<(i32, i32)> = if n <= 4 {
            if rng.gen_bool(0.5) {
                corners.to_vec()
            } else {
                cardinals.to_vec()
            }
        } else {
            corners.iter().chain(cardinals.iter()).cloned().collect()
        };
        pts.shuffle(&mut rng);
        let snakes: Vec<Snake> = (0..n)
            .map(|i| {
                let c = rules.cell(pts[i].0, pts[i].1);
                Snake {
                    body: VecDeque::from(vec![c, c, c]),
                    health: 100,
                    alive: true,
                }
            })
            .collect();
        let mut food = Vec::new();
        if mode != Mode::Constrictor {
            for p in pts.iter().take(n) {
                let (x, y) = *p;
                let d0 = (x - 5).abs() + (y - 5).abs();
                let mut opts: Vec<(i32, i32)> = [
                    (x - 1, y - 1),
                    (x - 1, y + 1),
                    (x + 1, y - 1),
                    (x + 1, y + 1),
                ]
                .into_iter()
                .filter(|&(fx, fy)| {
                    (fx - 5).abs() + (fy - 5).abs() >= d0 && fx >= 0 && fy >= 0 && fx < w && fy < h
                })
                .filter(|&(fx, fy)| !(fx == 5 && fy == 5))
                .collect();
                if opts.is_empty() {
                    opts.push((x, y + if y < 5 { -1 } else { 1 }));
                }
                let f = *opts.choose(&mut rng).unwrap();
                food.push(rules.cell(f.0, f.1));
            }
            food.push(rules.cell(5, 5));
        }
        World {
            st: State {
                rules: Arc::new(rules),
                snakes,
                food,
                turn: 0,
            },
            rng,
            mode,
            min_x: 0,
            max_x: w - 1,
            min_y: 0,
            max_y: h - 1,
        }
    }

    fn spawn_food(&mut self) {
        if self.mode == Mode::Constrictor {
            return;
        }
        let min_food = 1;
        let chance = 15;
        let need = if (self.st.food.len() as i32) < min_food {
            min_food - self.st.food.len() as i32
        } else if self.rng.gen_range(0..100) < chance {
            1
        } else {
            0
        };
        for _ in 0..need {
            let cells = self.st.rules.cells() as u16;
            let free: Vec<u16> = (0..cells)
                .filter(|c| {
                    !self.st.food.contains(c)
                        && !self.st.snakes.iter().any(|s| s.alive && s.body.contains(c))
                })
                .collect();
            if let Some(&c) = free.choose(&mut self.rng) {
                self.st.food.push(c);
            }
        }
    }

    fn shrink(&mut self) {
        if self.mode != Mode::Royale || self.st.turn == 0 || self.st.turn % 25 != 0 {
            return;
        }
        if self.min_x > self.max_x || self.min_y > self.max_y {
            return;
        }
        match self.rng.gen_range(0..4) {
            0 => self.min_x += 1,
            1 => self.max_x -= 1,
            2 => self.min_y += 1,
            _ => self.max_y -= 1,
        }
        let mut r = (*self.st.rules).clone();
        for y in 0..r.height {
            for x in 0..r.width {
                let c = r.cell(x, y) as usize;
                let inside =
                    x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y;
                r.hazards[c] = if inside { 0 } else { 1 };
            }
        }
        r.finalize();
        self.st.rules = Arc::new(r);
    }
}

/// view of the state with snake `i` as index 0
fn view(st: &State, i: usize) -> (State, Vec<usize>) {
    let mut order: Vec<usize> = vec![i];
    order.extend((0..st.snakes.len()).filter(|&j| j != i && st.snakes[j].alive));
    let snakes = order.iter().map(|&j| st.snakes[j].clone()).collect();
    (
        State {
            rules: st.rules.clone(),
            snakes,
            food: st.food.clone(),
            turn: st.turn,
        },
        order,
    )
}

/// returns index of the winner (None = draw), number of turns
fn play(
    players: &[Player],
    mode: Mode,
    seed: u64,
    verbose: bool,
    timeout: u64,
    timeouts: &AtomicUsize,
    hist: &mut VecDeque<State>,
    died: &mut Vec<u32>,
) -> (Option<usize>, u32) {
    let mut w = World::new(mode, players.len(), seed);
    let game_id = format!("sim-{seed}-{}", rand::random::<u32>());
    let n = players.len();
    *died = vec![u32::MAX; n];
    loop {
        for i in 0..n {
            if !w.st.snakes[i].alive && died[i] == u32::MAX {
                died[i] = w.st.turn;
            }
        }
        let alive = w.st.alive_count();
        if alive == 0 {
            return (None, w.st.turn);
        }
        if alive == 1 {
            return ((0..n).find(|&i| w.st.snakes[i].alive), w.st.turn);
        }
        if w.st.turn > 1500 {
            return (None, w.st.turn);
        }
        hist.push_back(w.st.clone());
        if hist.len() > 6 {
            hist.pop_front();
        }
        let mut moves = vec![Dir::Up; n];
        for i in 0..n {
            if !w.st.snakes[i].alive {
                continue;
            }
            let (v, _) = view(&w.st, i);
            let dl = Instant::now() + Duration::from_millis(players[i].ms);
            if players[i].greedy {
                moves[i] = greedy_move(&v);
                continue;
            }
            if let Some(addr) = &players[i].http {
                let (m, to) = http_move(addr, &w.st, i, mode, &game_id, timeout);
                if to {
                    timeouts.fetch_add(1, Ordering::Relaxed);
                }
                moves[i] = m;
                continue;
            }
            let r = search(&v, &players[i].cfg, dl);
            moves[i] = r.best;
            if verbose && i == 0 {
                eprintln!(
                    "t{} p{} {:?} d{} s{:.1}",
                    w.st.turn, i, r.best, r.depth, r.score
                );
            }
        }
        w.st.advance(&moves);
        w.shrink();
        w.spawn_food();
        let _ = NONE;
    }
}

fn main() {
    let args = parse_args();
    let n = match args.mode {
        Mode::Duel => 2,
        _ => 4,
    };
    let next = AtomicUsize::new(0);
    let timeouts = AtomicUsize::new(0);
    // results: wins A, wins B, draws, total turns
    let res = Mutex::new((0usize, 0usize, 0usize, 0u64));
    let t0 = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..args.threads {
            s.spawn(|| loop {
                let g = next.fetch_add(1, Ordering::SeqCst);
                if g >= args.games {
                    break;
                }
                // alternate seats so both configs get every start position
                let a_first = g % 2 == 0;
                // with a field snake: A and B take seats 0/1 or 2/3 (swapped every game), X the rest;
                // the seat pairs alternate over four games so both see every start position
                let field = args.x.is_some() && n == 4;
                let seat_of = |i: usize| -> u8 {
                    if field {
                        let pair = (g / 2) % 2; // which pair of seats A/B use
                        if i / 2 != pair {
                            2
                        } else if (i % 2 == 0) == a_first {
                            0
                        } else {
                            1
                        }
                    } else if (i % 2 == 0) == a_first {
                        0
                    } else {
                        1
                    }
                };
                let players: Vec<Player> = (0..n)
                    .map(|i| match seat_of(i) {
                        0 => args.a.clone(),
                        1 => args.b.clone(),
                        _ => args.x.clone().unwrap(),
                    })
                    .collect();
                let is_a: Vec<bool> = (0..n).map(|i| seat_of(i) == 0).collect();
                let is_b: Vec<bool> = (0..n).map(|i| seat_of(i) == 1).collect();
                let seed = args.seed.wrapping_mul(1_000_003).wrapping_add(g as u64 / 2);
                let mut hist = VecDeque::new();
                let mut died = Vec::new();
                let (winner, turns) = play(&players, args.mode, seed, args.verbose, args.timeout, &timeouts, &mut hist, &mut died);
                if let Some(dir) = &args.dump {
                    // dump the last positions of every A snake that died
                    let a_won = matches!(winner, Some(wi) if is_a[wi]);
                    if !a_won {
                        for (i, &ia) in is_a.iter().enumerate() {
                            let alive_states: Vec<&State> = hist.iter().filter(|s| s.snakes[i].alive).collect();
                            if !ia || alive_states.len() == hist.len() {
                                continue;
                            }
                            for (k, st) in alive_states.iter().rev().take(4).enumerate() {
                                let js = engine_json(st, i, args.mode, &format!("loss-{g}"), args.timeout);
                                let _ = std::fs::write(format!("{dir}/g{g}_s{i}_m{k}.json"), js);
                            }
                        }
                    }
                }
                let mut r = res.lock().unwrap();
                if field {
                    let da = (0..n).filter(|&i| is_a[i]).map(|i| died[i]).max().unwrap();
                    let db = (0..n).filter(|&i| is_b[i]).map(|i| died[i]).max().unwrap();
                    let da = if matches!(winner, Some(wi) if is_a[wi]) { u32::MAX } else { da };
                    let db = if matches!(winner, Some(wi) if is_b[wi]) { u32::MAX } else { db };
                    match da.cmp(&db) {
                        std::cmp::Ordering::Greater => r.0 += 1,
                        std::cmp::Ordering::Less => r.1 += 1,
                        _ => r.2 += 1,
                    }
                } else {
                    match winner {
                        Some(wi) if is_a[wi] => r.0 += 1,
                        Some(_) => r.1 += 1,
                        None => r.2 += 1,
                    }
                }
                r.3 += turns as u64;
            });
        }
    });
    let (a, b, d, turns) = *res.lock().unwrap();
    let total = (a + b + d).max(1);
    let score = (a as f64 + 0.5 * d as f64) / total as f64;
    // Elo difference estimate
    let elo = if score > 0.0 && score < 1.0 {
        -400.0 * (1.0 / score - 1.0).log10()
    } else {
        f64::NAN
    };
    println!(
        "mode={:?} games={} A={} B={} draws={} scoreA={:.3} elo={:+.0} avg_turns={:.0} time={:.1}s",
        args.mode,
        total,
        a,
        b,
        d,
        score,
        elo,
        turns as f64 / total as f64,
        t0.elapsed().as_secs_f64()
    );
    let to = timeouts.load(Ordering::Relaxed);
    if to > 0 {
        println!("external snake timeouts/errors: {to}");
    }
}
