//! How well do the heuristic opponent moves predict real arena moves?
//!
//! policy <game.json>...   (files from tools/fetch_top.py: {"meta":..., "frames":[...]})
use honey_badger::api::GameState;
use honey_badger::game::Dir;
use honey_badger::search::{simple_move, smart_move_p, SmartParams};
use serde_json::{json, Value};

fn pt(p: &Value) -> Value {
    json!({"x": p["X"], "y": p["Y"]})
}

fn main() {
    // optional first arg: JSON-ish overrides "key=value,key=value"
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let mut p = SmartParams::default();
    if let Some(a) = args.first().filter(|a| a.contains('=')).cloned() {
        args.remove(0);
        for kv in a.split(',') {
            let (k, v) = kv.split_once('=').unwrap();
            let x: f32 = v.parse().unwrap();
            match k {
                "danger" => p.danger = x,
                "trap" => p.trap = x,
                "trap_cell" => p.trap_cell = x,
                "room" => p.room = x,
                "cap_extra" => p.cap_extra = x as u32,
                "hunger_low" => p.hunger_low = x,
                "hunger_mid" => p.hunger_mid = x,
                "hunger_full" => p.hunger_full = x,
                "eat" => p.eat = x,
                "center" => p.center = x,
                _ => panic!("unknown {k}"),
            }
        }
    }
    let files = args;
    // [n, simple hits, smart hits] per (players bucket)
    let mut st_ = [[0u64; 3]; 2];
    for f in &files {
        let Ok(txt) = std::fs::read_to_string(f) else { continue };
        let Ok(v) = serde_json::from_str::<Value>(&txt) else { continue };
        let Some(fr) = v["frames"].as_array() else { continue };
        if fr.len() < 3 || v["meta"]["Game"]["Width"] != 11 {
            continue;
        }
        let dead = |s: &Value| !s["Death"].is_null();
        for i in 1..fr.len() - 1 {
            let (f0, f1) = (&fr[i], &fr[i + 1]);
            let cur: Vec<&Value> = f0["Snakes"].as_array().unwrap().iter().filter(|s| !dead(s)).collect();
            if cur.len() < 2 {
                continue;
            }
            let bucket = if cur.len() == 2 { 0 } else { 1 };
            let snakes: Vec<Value> = cur
                .iter()
                .map(|s| {
                    let body: Vec<Value> = s["Body"].as_array().unwrap().iter().map(pt).collect();
                    json!({"id": s["ID"], "name": s["Name"], "health": s["Health"], "head": body[0],
                           "length": body.len(), "body": body, "latency": "0", "shout": ""})
                })
                .collect();
            for (k, s) in cur.iter().enumerate() {
                let Some(ns) = f1["Snakes"].as_array().unwrap().iter().find(|x| x["ID"] == s["ID"]) else { continue };
                if dead(ns) {
                    continue;
                }
                let (h, nh) = (&s["Body"][0], &ns["Body"][0]);
                let actual = match (nh["X"].as_i64().unwrap() - h["X"].as_i64().unwrap(), nh["Y"].as_i64().unwrap() - h["Y"].as_i64().unwrap()) {
                    (0, 1) => Dir::Up,
                    (0, -1) => Dir::Down,
                    (-1, 0) => Dir::Left,
                    (1, 0) => Dir::Right,
                    _ => continue,
                };
                let gj = json!({"game": {"id": "x", "ruleset": {"name": "standard", "version": "1", "settings": {"foodSpawnChance": 15, "minimumFood": 1, "hazardDamagePerTurn": 0}},
                    "map": "standard", "timeout": 500, "source": "arena"}, "turn": f0["Turn"],
                    "board": {"height": 11, "width": 11, "food": f0["Food"].as_array().unwrap().iter().map(pt).collect::<Vec<_>>(), "hazards": [], "snakes": snakes},
                    "you": snakes[k]});
                let Ok(gs) = serde_json::from_value::<GameState>(gj) else { continue };
                let st = gs.to_state();
                let mut mv = [Dir::Up; 4];
                let n = st.safe_moves(0, &mut mv);
                if n < 2 || !mv[..n].contains(&actual) {
                    continue;
                }
                let a = simple_move(&st, 0, &mv[..n]);
                let b = smart_move_p(&st, 0, &mv[..n], &p);
                let c = &mut st_[bucket];
                c[0] += 1;
                c[1] += (a == actual) as u64;
                c[2] += (b == actual) as u64;
            }
        }
    }
    for (b, name) in [(0, "2 snakes"), (1, "3-4 snakes")] {
        let c = st_[b];
        let n = c[0].max(1) as f64;
        println!("{name}: n={} simple={:.3} smart={:.3}", c[0], c[1] as f64 / n, c[2] as f64 / n);
    }
}
