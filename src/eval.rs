//! Static evaluation of a position from the perspective of snake 0.

use crate::game::{State, MAX_CELLS, NONE};

const NO_OWNER: u8 = 0xFF;
const CONTESTED: u8 = 0xFE;

/// Tunable evaluation weights.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Weights {
    /// value per controlled cell
    pub area: f32,
    /// penalty per cell controlled by the strongest opponent
    pub opp_area: f32,
    /// penalty per cell controlled by all opponents together (multi-player)
    pub opp_area_sum: f32,
    /// value per length unit of advantage over the longest opponent (capped)
    pub length: f32,
    pub length_cap: f32,
    /// bonus for being strictly the longest snake
    pub longest_bonus: f32,
    /// penalty per step distance to the closest owned food when we want food
    pub food_dist: f32,
    /// we want food while health is below this
    pub hunger_health: i32,
    /// ... or while we are not at least this much longer than everyone
    pub want_length_lead: i32,
    /// penalty when the reachable area is smaller than our length
    pub trap: f32,
    /// penalty per living opponent
    pub opp_alive: f32,
    /// value of a hazard cell relative to a normal cell
    pub hazard_cell: f32,
    /// penalty per point of missing health (small tie-breaker)
    pub health: f32,
    /// penalty for being adjacent to a wall/edge (head)
    pub edge: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Weights {
            area: 1.0,
            opp_area: 0.6,
            opp_area_sum: 0.15,
            length: 15.0,
            length_cap: 8.0,
            longest_bonus: 15.0,
            food_dist: 1.5,
            hunger_health: 35,
            want_length_lead: 2,
            trap: 6.0,
            opp_alive: 30.0,
            hazard_cell: 0.25,
            health: 0.02,
            edge: 1.0,
        }
    }
}

pub struct Voronoi {
    pub area: [f32; 16],
    pub cells: [u16; 16],
    /// distance to the nearest food owned by each snake (NONE if none)
    pub food_dist: [u16; 16],
}

struct Scratch {
    free_at: Vec<u16>,
    owner: Vec<u8>,
    dist: Vec<u16>,
    is_food: Vec<bool>,
    cur: Vec<(u16, u8)>,
    next: Vec<(u16, u8)>,
}

thread_local! {
    static SCRATCH: std::cell::RefCell<Scratch> = std::cell::RefCell::new(Scratch {
        free_at: vec![0; MAX_CELLS],
        owner: vec![NO_OWNER; MAX_CELLS],
        dist: vec![0; MAX_CELLS],
        is_food: vec![false; MAX_CELLS],
        cur: Vec::with_capacity(MAX_CELLS),
        next: Vec::with_capacity(MAX_CELLS),
    });
}

/// Simultaneous BFS from every head. A body cell can be entered once its
/// segment has left (tail-aware). Ties go to the longer snake, equal-length ties
/// are contested and block.
pub fn voronoi(st: &State, hazard_value: f32) -> Voronoi {
    if st.rules.bitboard {
        return voronoi_bb(st, hazard_value);
    }
    SCRATCH.with(|sc| voronoi_impl(st, hazard_value, &mut sc.borrow_mut()))
}

fn voronoi_impl(st: &State, hazard_value: f32, sc: &mut Scratch) -> Voronoi {
    let rules = &*st.rules;
    let n_cells = rules.cells();
    let n = st.snakes.len().min(16);
    let Scratch {
        free_at,
        owner,
        dist,
        is_food,
        cur,
        next,
    } = sc;
    let free_at = &mut free_at[..n_cells];
    let owner = &mut owner[..n_cells];
    let dist = &mut dist[..n_cells];
    let is_food = &mut is_food[..n_cells];
    free_at.fill(0);
    owner.fill(NO_OWNER);
    is_food.fill(false);
    cur.clear();
    for &f in &st.food {
        is_food[f as usize] = true;
    }

    for s in st.snakes.iter().filter(|s| s.alive) {
        let len = s.len();
        for (i, &c) in s.body.iter().enumerate() {
            let t = if rules.constrictor {
                u16::MAX
            } else {
                (len - i) as u16
            };
            if t > free_at[c as usize] {
                free_at[c as usize] = t;
            }
        }
    }

    let mut out = Voronoi {
        area: [0.0; 16],
        cells: [0; 16],
        food_dist: [NONE; 16],
    };
    let mut lens = [0usize; 16];
    for (i, s) in st.snakes.iter().enumerate().take(n) {
        if !s.alive {
            continue;
        }
        lens[i] = s.len();
        let h = s.head() as usize;
        if owner[h] == NO_OWNER {
            owner[h] = i as u8;
            dist[h] = 0;
            cur.push((h as u16, i as u8));
        }
    }

    let nbr = &rules.nbr[..];
    let mut t: u16 = 0;
    while !cur.is_empty() {
        t += 1;
        next.clear();
        for &(c, s) in cur.iter() {
            if owner[c as usize] != s {
                continue;
            }
            let base = c as usize * 4;
            for k in 0..4 {
                let nb = nbr[base + k];
                if nb == NONE {
                    continue;
                }
                let nbu = nb as usize;
                if free_at[nbu] > t {
                    continue;
                }
                let o = owner[nbu];
                if o == NO_OWNER {
                    owner[nbu] = s;
                    dist[nbu] = t;
                    next.push((nb, s));
                } else if o != s && o != CONTESTED && dist[nbu] == t {
                    let (ls, lo) = (lens[s as usize], lens[o as usize]);
                    if ls > lo {
                        owner[nbu] = s;
                        next.push((nb, s));
                    } else if ls == lo {
                        owner[nbu] = CONTESTED;
                    }
                }
            }
        }
        std::mem::swap(cur, next);
    }

    let hz = rules.hazard_damage != 0;
    for c in 0..n_cells {
        let o = owner[c];
        if o >= CONTESTED {
            continue;
        }
        let o = o as usize;
        out.cells[o] += 1;
        let v = if hz && rules.hazards[c] > 0 {
            hazard_value
        } else {
            1.0
        };
        out.area[o] += v;
        if is_food[c] && dist[c] < out.food_dist[o] {
            out.food_dist[o] = dist[c];
        }
    }
    out
}

/// Bitboard version of [`voronoi`] for boards with <= 128 cells (11x11).
/// Snakes expand from their whole owned region every step, so they can "wait"
/// for body cells that free up later.
pub fn voronoi_bb(st: &State, hazard_value: f32) -> Voronoi {
    match st.rules.width {
        11 => voronoi_bb_w::<11>(st, hazard_value),
        7 => voronoi_bb_w::<7>(st, hazard_value),
        _ => voronoi_bb_w::<0>(st, hazard_value),
    }
}

#[inline(always)]
fn voronoi_bb_w<const W: u32>(st: &State, hazard_value: f32) -> Voronoi {
    let rules = &*st.rules;
    let w = if W != 0 { W } else { rules.width as u32 };
    let n_cells = rules.cells() as u32;
    let full: u128 = if n_cells == 128 {
        u128::MAX
    } else {
        (1u128 << n_cells) - 1
    };
    // column masks
    let mut col0: u128 = 0;
    let mut col_last: u128 = 0;
    for y in 0..rules.height as u32 {
        col0 |= 1u128 << (y * w);
        col_last |= 1u128 << (y * w + w - 1);
    }
    let not_col0 = full & !col0;
    
    let not_col_last = full & !col_last;
    let expand = |b: u128| -> u128 {
        ((b << w) & full) | (b >> w) | ((b >> 1) & not_col_last) | ((b << 1) & not_col0)
    };

    let n = st.snakes.len().min(16);
    let mut out = Voronoi {
        area: [0.0; 16],
        cells: [0; 16],
        food_dist: [NONE; 16],
    };
    let mut food_bb: u128 = 0;
    for &f in &st.food {
        food_bb |= 1u128 << f;
    }

    let mut max_t = 0usize;
    let mut blocked: u128 = 0;
    let mut owned = [0u128; 16];
    let mut lens = [0usize; 16];
    let mut alive_idx = [0usize; 16];
    let mut na = 0;
    for (i, s) in st.snakes.iter().enumerate().take(n) {
        if !s.alive {
            continue;
        }
        let len = s.len();
        lens[i] = len;
        alive_idx[na] = i;
        na += 1;
        for &c in s.body.iter() {
            blocked |= 1u128 << c;
        }
        if !rules.constrictor {
            max_t = max_t.max(len);
        }
        owned[i] = 1u128 << s.head();
    }
    // sort alive snakes by length (desc) for tie resolution
    let order = &mut alive_idx[..na];
    order.sort_unstable_by(|&a, &b| lens[b].cmp(&lens[a]));

    let mut claimed: u128 = 0;
    for &i in order.iter() {
        claimed |= owned[i];
    }
    let mut t = 0usize;
    loop {
        t += 1;
        if t <= max_t {
            // segment k = len - t leaves at time t (stacked duplicates leave with the earlier one)
            for &i in order.iter() {
                let len = lens[i];
                if t <= len {
                    let k = len - t;
                    let body = &st.snakes[i].body;
                    if k == 0 || body[k] != body[k - 1] {
                        blocked &= !(1u128 << body[k]);
                    }
                }
            }
        }
        let avail = full & !blocked & !claimed;
        let mut any = false;
        let mut taken: u128 = 0;
        let mut g = 0;
        while g < na {
            // group of equal length
            let len = lens[order[g]];
            let mut e = g;
            while e < na && lens[order[e]] == len {
                e += 1;
            }
            let mut seen: u128 = 0;
            let mut dup: u128 = 0;
            let mut cand = [0u128; 16];
            for (k, &i) in order[g..e].iter().enumerate() {
                let c = expand(owned[i]) & avail & !taken;
                cand[k] = c;
                dup |= seen & c;
                seen |= c;
            }
            for (k, &i) in order[g..e].iter().enumerate() {
                let new = cand[k] & !dup;
                if new != 0 {
                    any = true;
                    owned[i] |= new;
                    if out.food_dist[i] == NONE && new & food_bb != 0 {
                        out.food_dist[i] = t as u16;
                    }
                }
            }
            taken |= seen;
            g = e;
        }
        claimed |= taken;
        if !any && t >= max_t {
            break;
        }
        if t > n_cells as usize + 130 {
            break;
        }
    }

    let hz = rules.hazard_damage != 0 && rules.hazard_bb != 0;
    for &i in order.iter() {
        let o = owned[i];
        let c = o.count_ones();
        out.cells[i] = c as u16;
        out.area[i] = if hz {
            let h = (o & rules.hazard_bb).count_ones();
            (c - h) as f32 + h as f32 * hazard_value
        } else {
            c as f32
        };
    }
    out
}

pub fn evaluate(st: &State, w: &Weights) -> f32 {
    let vor = voronoi(st, w.hazard_cell);
    let rules = &*st.rules;
    let me = st.me();
    let my_len = me.len() as i32;

    let mut max_opp_area: f32 = 0.0;
    let mut sum_opp_area: f32 = 0.0;
    let mut max_opp_len: i32 = 0;
    let mut opps = 0;
    for (i, s) in st.snakes.iter().enumerate().skip(1) {
        if !s.alive {
            continue;
        }
        opps += 1;
        let a = vor.area[i.min(15)];
        max_opp_area = max_opp_area.max(a);
        sum_opp_area += a;
        max_opp_len = max_opp_len.max(s.len() as i32);
    }

    let my_area = vor.area[0];
    let mut score = w.area * my_area - w.opp_area * max_opp_area - w.opp_area_sum * sum_opp_area;

    // trapped: not enough room for our body
    let room = vor.cells[0] as i32;
    if room < my_len {
        score -= w.trap * (my_len - room) as f32;
    }

    if !rules.constrictor {
        if opps > 0 {
            let diff = (my_len - max_opp_len) as f32;
            score += w.length * diff.clamp(-w.length_cap, w.length_cap);
            if my_len > max_opp_len {
                score += w.longest_bonus;
            }
        }

        // food
        let fd = vor.food_dist[0];
        let want =
            me.health < w.hunger_health || (opps > 0 && my_len < max_opp_len + w.want_length_lead);
        if want {
            let d = if fd == NONE {
                (rules.width + rules.height) as f32
            } else {
                fd as f32
            };
            let urgency = if me.health < w.hunger_health {
                1.0 + (w.hunger_health - me.health) as f32 / 20.0
            } else {
                1.0
            };
            score -= w.food_dist * urgency * d;
        }
        // impending starvation: no food we can reach in time
        let need = if fd == NONE { i32::MAX } else { fd as i32 };
        if me.health < need.min(1000) || (fd == NONE && me.health < 15) {
            score -= 200.0;
        }
        score -= w.health * (100 - me.health) as f32;
    }

    score -= w.opp_alive * opps as f32;

    if w.edge != 0.0 && !rules.wrapped {
        let (x, y) = rules.xy(me.head());
        if x == 0 || y == 0 || x == rules.width - 1 || y == rules.height - 1 {
            score -= w.edge;
        }
    }
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Rules, Snake};
    use std::sync::Arc;

    #[test]
    fn bitboard_matches_scalar_roughly() {
        let mut r = Rules::new(11, 11, false, false, 0);
        r.finalize();
        let r = Arc::new(r);
        let mk = |v: &[(i32, i32)]| Snake {
            body: v.iter().map(|&(x, y)| r.cell(x, y)).collect(),
            health: 90,
            alive: true,
        };
        let st = State {
            rules: r.clone(),
            snakes: vec![
                mk(&[(2, 2), (2, 1), (2, 0)]),
                mk(&[(8, 8), (8, 9), (8, 10)]),
            ],
            food: vec![r.cell(5, 5)],
            turn: 0,
        };
        let a = voronoi_bb(&st, 1.0);
        let mut rs = (*r).clone();
        rs.bitboard = false;
        let st2 = State {
            rules: Arc::new(rs),
            ..st.clone()
        };
        let b = voronoi(&st2, 1.0);
        // symmetric position: equal areas
        assert_eq!(a.cells[0], a.cells[1]);
        assert_eq!(b.cells[0], b.cells[1]);
        assert!(
            (a.cells[0] as i32 - b.cells[0] as i32).abs() <= 3,
            "{} vs {}",
            a.cells[0],
            b.cells[0]
        );
        assert_eq!(a.food_dist[0], NONE); // center food is contested
    }
}
