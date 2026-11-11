//! Feature table from duel replays, for fitting the evaluation against game outcomes.
//!
//! duelfeat <every_nth_turn> <game.json>... > table.csv
//!
//! One row per sampled position and snake (seen from that snake). Decided games only.

use honey_badger::eval::voronoi_p;
use honey_badger::game::NONE;
use honey_badger::replay::{is_dead, Replay};
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let nth: usize = args[0].parse().expect("nth");
    let out = std::io::stdout();
    let mut out = std::io::BufWriter::new(out.lock());
    writeln!(out, "game,snake,opp,turn,end,won,len,olen,hp,ohp,area,oarea,cells,ocells,fd,ofd,food,edge,oedge,hdist").unwrap();
    for path in &args[1..] {
        let Some(rp) = Replay::load(path) else { continue };
        if rp.players() != 2 || rp.game["RulesetName"] != "standard" || rp.game["Map"] != "standard" {
            continue;
        }
        let Some(winner) = rp.winner().map(|s| s.to_string()) else { continue };
        let end = rp.frames.len() - 1;
        for i in (2 + rp.id.len() % nth..end).step_by(nth) {
            let sn = rp.snakes(i);
            if sn.iter().any(is_dead) {
                break;
            }
            for k in 0..2 {
                let Some(gs) = rp.request(i, &sn[k]["ID"]) else { continue };
                let st = gs.to_state();
                if st.snakes.len() != 2 {
                    continue;
                }
                let v = voronoi_p(&st, 1.0, 0.5);
                let r = &*st.rules;
                let edge = |c: u16| {
                    let (x, y) = r.xy(c);
                    (x == 0 || y == 0 || x == r.width - 1 || y == r.height - 1) as i32
                };
                let fd = |d: u16| if d == NONE { -1 } else { d as i32 };
                let (me, op) = (&st.snakes[0], &st.snakes[1]);
                let name = |j: usize| sn[j]["Name"].as_str().unwrap_or("?").replace(',', " ");
                writeln!(
                    out,
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                    &rp.id[..8], name(k), name(1 - k), gs.turn, end, (name(k) == winner) as i32,
                    me.len(), op.len(), me.health, op.health, v.area[0], v.area[1], v.cells[0], v.cells[1],
                    fd(v.food_dist[0]), fd(v.food_dist[1]), st.food.len(), edge(me.head()), edge(op.head()),
                    r.dist(me.head(), op.head())
                )
                .unwrap();
            }
        }
    }
}
