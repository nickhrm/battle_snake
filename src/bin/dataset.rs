//! Turns arena replays into a feature table for fitting the evaluation weights.
//!
//! dataset <game.json>... > table.csv
//!
//! Every sampled position yields one row per living snake, seen from that snake.

use honey_badger::eval::voronoi;
use honey_badger::game::{Rules, Snake, State, NONE};
use serde_json::Value;
use std::collections::VecDeque;
use std::io::Write;
use std::sync::Arc;

fn snake(r: &Rules, s: &Value) -> Snake {
    Snake {
        body: s["Body"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| r.cell(p["X"].as_i64().unwrap() as i32, p["Y"].as_i64().unwrap() as i32))
            .collect::<VecDeque<u16>>(),
        health: s["Health"].as_i64().unwrap() as i32,
        alive: true,
    }
}

fn is_dead(s: &Value) -> bool {
    !s["Death"].is_null()
}

fn main() {
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    writeln!(
        out,
        "game,turn,n,alive,len,won,place,my_area,max_opp_area,sum_opp_area,len_diff,sum_len_diff,longest,food_d,nearest,food_owned,room_lack,edge,health,opp_dist,big_adj"
    )
    .unwrap();
    for path in std::env::args().skip(1) {
        let txt = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let v: Value = match serde_json::from_str(&txt) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let frames = v["frames"].as_array().unwrap();
        let g = &v["meta"]["Game"];
        if frames.is_empty() || g["Status"] != "complete" || g["Map"] != "standard" {
            continue;
        }
        let last = &frames[frames.len() - 1]["Snakes"];
        let last = last.as_array().unwrap();
        let nsn = last.len();
        // outcome per snake: death turn decides the order
        let dt = |s: &Value| {
            if is_dead(s) {
                s["Death"]["Turn"].as_i64().unwrap_or(0)
            } else {
                i64::MAX
            }
        };
        let survivors = last.iter().filter(|s| !is_dead(s)).count();
        let id = g["ID"].as_str().unwrap_or("?");
        let mut rules = Rules::new(11, 11, false, false, 0);
        rules.finalize();
        let rules = Arc::new(rules);
        for fr in frames.iter().step_by(4) {
            let turn = fr["Turn"].as_i64().unwrap() as u32;
            if turn < 3 {
                continue;
            }
            let food: Vec<u16> = fr["Food"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| {
                    rules.cell(p["X"].as_i64().unwrap() as i32, p["Y"].as_i64().unwrap() as i32)
                })
                .collect();
            let sn = fr["Snakes"].as_array().unwrap();
            let alive: Vec<usize> = (0..sn.len()).filter(|&i| !is_dead(&sn[i])).collect();
            if alive.len() < 2 {
                continue;
            }
            for &i in &alive {
                let mut order = vec![i];
                order.extend(alive.iter().copied().filter(|&j| j != i));
                let st = State {
                    rules: rules.clone(),
                    snakes: order.iter().map(|&j| snake(&rules, &sn[j])).collect(),
                    food: food.clone(),
                    turn,
                };
                let vr = voronoi(&st, 1.0);
                let me = st.me();
                let ml = me.len() as i32;
                let (mut mxa, mut sma, mut mxl, mut sld, mut od, mut big) =
                    (0f32, 0f32, 0i32, 0f32, 99, 0);
                for (k, s) in st.snakes.iter().enumerate().skip(1) {
                    mxa = mxa.max(vr.area[k]);
                    sma += vr.area[k];
                    mxl = mxl.max(s.len() as i32);
                    sld += ((ml - s.len() as i32) as f32).clamp(-5.0, 5.0);
                    let dist = rules.dist(me.head(), s.head());
                    od = od.min(dist);
                    if dist <= 2 && s.len() as i32 >= ml {
                        big += 1;
                    }
                }
                let fd = vr.food_dist[0];
                let near = st.food.iter().map(|&f| rules.dist(me.head(), f)).min();
                let food_d = if fd == NONE { -1 } else { fd as i32 };
                let near = near.unwrap_or(-1);
                let owned = food_owned(&st);
                let room = vr.cells[0] as i32;
                let (x, y) = rules.xy(me.head());
                let edge = (x == 0 || y == 0 || x == 10 || y == 10) as i32;
                let place = 1 + last.iter().filter(|s| dt(s) > dt(&last[i])).count();
                let won = survivors == 1 && !is_dead(&last[i]);
                writeln!(
                    out,
                    "{id},{turn},{nsn},{},{ml},{},{place},{},{mxa},{sma},{},{sld},{},{food_d},{near},{owned},{},{edge},{},{od},{big}",
                    alive.len(),
                    won as i32,
                    vr.area[0],
                    ml - mxl,
                    (ml > mxl) as i32,
                    (ml - room).max(0),
                    me.health,
                )
                .unwrap();
            }
        }
    }
}

/// Number of food items strictly closer to us (Manhattan) than to every opponent.
fn food_owned(st: &State) -> i32 {
    let r = &*st.rules;
    let h = st.me().head();
    st.food
        .iter()
        .filter(|&&f| {
            let d = r.dist(h, f);
            st.snakes.iter().skip(1).all(|s| r.dist(s.head(), f) > d)
        })
        .count() as i32
}
