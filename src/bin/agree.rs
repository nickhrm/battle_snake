//! Compares the moves in arena replays with what our search would play.
//!
//! agree <ms> <every_nth_turn> <snake,snake,...|all> <game.json>... > table.csv
//!
//! One row per sampled move of a named snake. Scores are full-window values of each root move.

use honey_badger::api::GameState;
use honey_badger::config_for;
use honey_badger::game::Dir;
use honey_badger::search::search;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn dir_of(a: &Value, b: &Value) -> Option<Dir> {
    let (dx, dy) = (
        b["X"].as_i64()? - a["X"].as_i64()?,
        b["Y"].as_i64()? - a["Y"].as_i64()?,
    );
    match (dx, dy) {
        (0, 1) => Some(Dir::Up),
        (0, -1) => Some(Dir::Down),
        (-1, 0) => Some(Dir::Left),
        (1, 0) => Some(Dir::Right),
        _ => None,
    }
}

fn pt(p: &Value) -> Value {
    json!({"x": p["X"], "y": p["Y"]})
}

fn api_snake(s: &Value) -> Value {
    let body: Vec<Value> = s["Body"].as_array().unwrap().iter().map(pt).collect();
    json!({"id": s["ID"], "name": s["Name"], "health": s["Health"], "head": body[0], "length": body.len(),
           "body": body, "latency": "0", "shout": ""})
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ms: u64 = args[0].parse().unwrap();
    let nth: usize = args[1].parse().unwrap();
    let names: Vec<&str> = args[2].split(',').collect();
    let files = &args[3..];
    let next = AtomicUsize::new(0);
    let out = Mutex::new(());
    println!("game,turn,n,alive,snake,theirs,ours,v_theirs,v_ours,won,len,maxopp");
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::SeqCst);
                if k >= files.len() {
                    break;
                }
                let Ok(txt) = std::fs::read_to_string(&files[k]) else { continue };
                let Ok(v) = serde_json::from_str::<Value>(&txt) else { continue };
                let g = &v["meta"]["Game"];
                let Some(fr) = v["frames"].as_array() else { continue };
                if fr.len() < 3 {
                    continue;
                }
                let dead = |s: &Value| !s["Death"].is_null();
                let last = fr[fr.len() - 1]["Snakes"].as_array().unwrap();
                let surv: Vec<&Value> = last.iter().filter(|s| !dead(s)).collect();
                let winner = if surv.len() == 1 { surv[0]["Name"].as_str() } else { None };
                let settings = json!({"foodSpawnChance": 15, "minimumFood": 1,
                    "hazardDamagePerTurn": g["Ruleset"]["damagePerTurn"].as_str().and_then(|x| x.parse::<i64>().ok()).unwrap_or(0)});
                for i in (2..fr.len() - 1).step_by(nth) {
                    let (f, nf) = (&fr[i], &fr[i + 1]);
                    let cur: Vec<&Value> =
                        f["Snakes"].as_array().unwrap().iter().filter(|s| !dead(s)).collect();
                    if cur.len() < 2 {
                        continue;
                    }
                    for s in &cur {
                        let name = s["Name"].as_str().unwrap_or("");
                        if names[0] != "all" && !names.contains(&name) {
                            continue;
                        }
                        let Some(ns) = nf["Snakes"].as_array().unwrap().iter().find(|x| x["ID"] == s["ID"]) else { continue };
                        if dead(ns) {
                            continue;
                        }
                        let Some(theirs) = dir_of(&s["Body"][0], &ns["Body"][0]) else { continue };
                        let snakes: Vec<Value> = cur.iter().map(|x| api_snake(x)).collect();
                        let gj = json!({"game": {"id": "x", "ruleset": {"name": g["RulesetName"], "version": "1", "settings": settings},
                                 "map": g["Map"], "timeout": 500, "source": "arena", "players": last.len()},
                            "turn": f["Turn"],
                            "board": {"height": 11, "width": 11, "food": f["Food"].as_array().unwrap().iter().map(pt).collect::<Vec<_>>(),
                                "hazards": f["Hazards"].as_array().map(|h| h.iter().map(pt).collect::<Vec<_>>()).unwrap_or_default(),
                                "snakes": snakes},
                            "you": api_snake(s)});
                        let Ok(gs) = serde_json::from_value::<GameState>(gj) else { continue };
                        let st = gs.to_state();
                        let mut cfg = config_for(&st, false);
                        cfg.exact_root = true;
                        honey_badger::apply_env(&mut cfg, &st);
                        let r = search(&st, &cfg, Instant::now() + Duration::from_millis(ms));
                        if r.depth == 0 {
                            continue; // forced move
                        }
                        let vt = r.scores.iter().find(|x| x.0 == theirs).map(|x| x.1);
                        let ml = cur.iter().filter(|x| x["ID"] != s["ID"]).map(|x| x["Body"].as_array().unwrap().len()).max().unwrap_or(0);
                        let _g = out.lock().unwrap();
                        println!(
                            "{},{},{},{},{},{},{},{},{:.1},{},{},{}",
                            g["ID"].as_str().unwrap_or("?"), f["Turn"], last.len(), cur.len(), name.replace(',', " "),
                            theirs.name(), r.best.name(),
                            vt.map(|x| format!("{x:.1}")).unwrap_or_else(|| "illegal".into()),
                            r.score, (winner == Some(name)) as i32,
                            s["Body"].as_array().unwrap().len(), ml
                        );
                    }
                }
            });
        }
    });
}
