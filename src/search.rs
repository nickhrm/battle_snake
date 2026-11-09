//! Iterative-deepening paranoid alpha-beta over simultaneous moves.
//!
//! Each ply we pick a move (max node), then all nearby opponents pick a joint
//! reply knowing our move (min node). This is pessimistic, which is exactly
//! what you want in Battlesnake: never walk into a position where some
//! combination of opponent moves kills us.

use crate::eval::{evaluate, Weights};
use crate::game::{Dir, State};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

pub const WIN: f32 = 1_000_000.0;
pub const LOSS: f32 = -1_000_000.0;
pub const DRAW: f32 = -100_000.0;
const INF: f32 = f32::INFINITY;
const MAX_PLY: usize = 128;

#[derive(Clone, Debug)]
pub struct SearchConfig {
    pub weights: Weights,
    pub parallel: bool,
    pub max_depth: u32,
    /// max number of opponents that get a full move set at a min node
    pub max_relevant: usize,
    /// opponents whose head is further away than this (Manhattan) get one heuristic move
    pub max_reach: i32,
    /// sequential only: search every root move with a full window, for analysis
    pub exact_root: bool,
    /// multi-player: from this ply on, only opponents whose head is within `deep_reach`
    /// of ours get a full move set (the others play their heuristic move)
    pub deep_ply: usize,
    pub deep_reach: i32,
    /// the ply limit only applies while at least this many opponents are alive at the root
    pub deep_min_opps: usize,
    /// share of the time for a full paranoid search first, whose proven losses are vetoed
    pub veto_frac: f32,
    /// heuristic move for opponents outside the search: space, head danger and food
    /// instead of free neighbours only
    pub smart_default: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            weights: Weights::default(),
            parallel: true,
            max_depth: 60,
            max_relevant: 3,
            max_reach: 99,
            exact_root: false,
            deep_ply: 99,
            deep_reach: 2,
            deep_min_opps: 2,
            veto_frac: 0.0,
            smart_default: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub best: Dir,
    pub score: f32,
    pub depth: u32,
    pub nodes: u64,
    pub scores: Vec<(Dir, f32)>,
}

pub fn is_win(v: f32) -> bool {
    v > WIN / 2.0
}
pub fn is_loss(v: f32) -> bool {
    v < LOSS / 2.0
}

struct Ctx<'a> {
    w: &'a Weights,
    deadline: Instant,
    stop: &'a AtomicBool,
    aborted: bool,
    nodes: u64,
    start_opps: usize,
    max_relevant: usize,
    max_reach: i32,
    deep_ply: usize,
    deep_reach: i32,
    deep_min_opps: usize,
    smart_default: bool,
    killer_me: [u8; MAX_PLY],
    killer_opp: [u32; MAX_PLY],
    /// best exact root score found so far by any thread (f32 bits), parallel root only
    shared_alpha: Option<&'a AtomicU32>,
    /// highest shared alpha this context has used during the current root search
    alpha_seen: f32,
}

/// Decay per ply of the eating bonus.
const EAT_DECAY: f32 = 0.85;

impl<'a> Ctx<'a> {
    fn new(
        w: &'a Weights,
        deadline: Instant,
        stop: &'a AtomicBool,
        start_opps: usize,
        max_relevant: usize,
        max_reach: i32,
    ) -> Self {
        Ctx {
            w,
            deadline,
            stop,
            aborted: false,
            nodes: 0,
            start_opps,
            max_relevant,
            max_reach,
            deep_ply: 99,
            deep_reach: 2,
            deep_min_opps: 2,
            smart_default: false,
            killer_me: [255; MAX_PLY],
            killer_opp: [u32::MAX; MAX_PLY],
            shared_alpha: None,
            alpha_seen: -INF,
        }
    }

    #[inline]
    fn check_time(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes & 255 == 0
            && (Instant::now() >= self.deadline || self.stop.load(Ordering::Relaxed))
        {
            self.aborted = true;
        }
        self.aborted
    }

    #[inline]
    fn terminal(&self, parent: &State, st: &State, ply: usize) -> Option<f32> {
        if !st.snakes[0].alive {
            if self.start_opps > 0 && st.alive_count() == 0 {
                return Some(DRAW + ply as f32 * 10.0);
            }
            if self.w.mutual != 0.0 {
                let head = st.snakes[0].head();
                let traded = (1..st.snakes.len()).any(|j| {
                    !st.snakes[j].alive && parent.snakes[j].alive && st.snakes[j].head() == head
                });
                if traded {
                    return Some(self.w.mutual + ply as f32);
                }
            }
            return Some(LOSS + ply as f32 * 10.0);
        }
        if self.start_opps > 0 && st.alive_opponents() == 0 {
            return Some(WIN - ply as f32 * 10.0);
        }
        None
    }

    fn max_node(
        &mut self,
        st: &State,
        depth: u32,
        ply: usize,
        mut alpha: f32,
        beta: f32,
        bonus: f32,
    ) -> f32 {
        if depth == 0 {
            return evaluate(st, self.w) + bonus;
        }
        if self.check_time() {
            return 0.0;
        }
        let mut moves = [Dir::Up; 4];
        let n = st.safe_moves(0, &mut moves);
        if n == 0 {
            return LOSS + (ply as f32 + 1.0) * 10.0;
        }
        let p = ply.min(MAX_PLY - 1);
        let k = self.killer_me[p];
        if k != 255 {
            if let Some(pos) = moves[..n].iter().position(|&d| d as u8 == k) {
                moves[..n].swap(0, pos);
            }
        }
        let mut best = -INF;
        let mut best_m = moves[0];
        for &m in &moves[..n] {
            let v = self.min_node(st, m, depth, ply, alpha, beta, bonus);
            if self.aborted {
                return 0.0;
            }
            if v > best {
                best = v;
                best_m = m;
            }
            if v > alpha {
                alpha = v;
            }
            if alpha >= beta {
                break;
            }
        }
        self.killer_me[p] = best_m as u8;
        best
    }

    /// Choose a single plausible move for an opponent that is too far away to matter tactically.
    fn default_move(&self, st: &State, j: usize) -> Dir {
        let mut moves = [Dir::Up; 4];
        let n = st.safe_moves(j, &mut moves);
        if n == 0 {
            return st.fallback_move(j);
        }
        if n == 1 {
            return moves[0];
        }
        if self.smart_default && st.rules.bitboard {
            return smart_move(st, j, &moves[..n]);
        }
        simple_move(st, j, &moves[..n])
    }

    fn min_node(
        &mut self,
        st: &State,
        my_move: Dir,
        depth: u32,
        ply: usize,
        alpha: f32,
        beta: f32,
        bonus: f32,
    ) -> f32 {
        let n = st.snakes.len();
        let rules = &*st.rules;
        let my_head = st.snakes[0].head();
        let my_next = rules.step(my_head, my_move);

        // opponent candidate moves
        let mut opt: [[Dir; 4]; 16] = [[Dir::Up; 4]; 16];
        let mut cnt: [usize; 16] = [1; 16];
        let mut idxs: [usize; 16] = [0; 16];
        let mut no = 0;

        // pick relevant opponents (closest first)
        let mut cand: [(i32, usize); 16] = [(i32::MAX, 0); 16];
        let mut nc = 0;
        for j in 1..n.min(16) {
            let s = &st.snakes[j];
            if !s.alive {
                continue;
            }
            cand[nc] = (rules.dist(my_head, s.head()), j);
            nc += 1;
        }
        cand[..nc].sort_unstable();
        let mut reach = (2 * depth as i32 + 1).min(self.max_reach);
        if ply >= self.deep_ply && self.start_opps >= self.deep_min_opps.max(2) {
            reach = reach.min(self.deep_reach);
        }
        for (rank, &(d, j)) in cand[..nc].iter().enumerate() {
            idxs[no] = j;
            // A proven win must not rest on a guessed move, so the last opponent is always
            // searched in full. It costs little: there is only one snake to enumerate.
            if (d <= reach || self.start_opps == 1) && rank < self.max_relevant {
                let c = st.safe_moves(j, &mut opt[no]);
                if c == 0 {
                    opt[no][0] = st.fallback_move(j);
                    cnt[no] = 1;
                } else {
                    // aggressive moves first: closest to our next head
                    let head = st.snakes[j].head();
                    opt[no][..c].sort_by_key(|&m| {
                        let t = rules.step(head, m);
                        if my_next == crate::game::NONE {
                            0
                        } else {
                            rules.dist(t, my_next)
                        }
                    });
                    cnt[no] = c;
                }
            } else {
                opt[no][0] = self.default_move(st, j);
                cnt[no] = 1;
            }
            no += 1;
        }

        let total: usize = cnt[..no].iter().product();
        let p = ply.min(MAX_PLY - 1);
        let killer = self.killer_opp[p];

        let mut moves = [Dir::Up; 16];
        moves[0] = my_move;
        let mut v = INF;
        let mut best_code = u32::MAX;

        // iterate joint moves, killer first
        let mut order_first: Option<usize> = None;
        if killer != u32::MAX && total > 1 {
            // decode killer into an index if all moves are available
            let mut idx = 0usize;
            let mut mul = 1usize;
            let mut ok = true;
            for k in 0..no {
                let d = ((killer >> (2 * k)) & 3) as u8;
                match opt[k][..cnt[k]].iter().position(|&m| m as u8 == d) {
                    Some(pos) => idx += pos * mul,
                    None => {
                        ok = false;
                        break;
                    }
                }
                mul *= cnt[k];
            }
            if ok {
                order_first = Some(idx);
            }
        }

        let mut alpha = alpha;
        for step in 0..total {
            if ply == 0 {
                if let Some(sa) = self.shared_alpha {
                    let a = f32::from_bits(sa.load(Ordering::Relaxed));
                    if a > alpha {
                        alpha = a;
                    }
                    if alpha > self.alpha_seen {
                        self.alpha_seen = alpha;
                    }
                    if v <= alpha {
                        break;
                    }
                }
            }
            let ji = match order_first {
                Some(f) => {
                    if step == 0 {
                        f
                    } else if step <= f {
                        step - 1
                    } else {
                        step
                    }
                }
                None => step,
            };
            let mut rem = ji;
            let mut code = 0u32;
            for k in 0..no {
                let m = opt[k][rem % cnt[k]];
                rem /= cnt[k];
                moves[idxs[k]] = m;
                code |= (m as u32) << (2 * k);
            }
            let mut child = st.clone();
            child.advance(&moves[..n]);
            let val = match self.terminal(st, &child, ply + 1) {
                Some(t) => t,
                None => {
                    let mut b = bonus;
                    if child.snakes[0].len() > st.snakes[0].len()
                        && (self.w.eat_want == 0.0 || crate::eval::wants_food(st, self.w))
                    {
                        b += self.w.eat_early * EAT_DECAY.powi(ply as i32);
                    }
                    self.max_node(&child, depth - 1, ply + 1, alpha, beta.min(v), b)
                }
            };
            if self.aborted {
                return 0.0;
            }
            if val < v {
                v = val;
                best_code = code;
            }
            if v <= alpha {
                break;
            }
        }
        if best_code != u32::MAX {
            self.killer_opp[p] = best_code;
        }
        v
    }
}

/// The original heuristic move: free neighbours, food when hungry or short, no hazards.
pub fn simple_move(st: &State, j: usize, moves: &[Dir]) -> Dir {
        let s = &st.snakes[j];
        let r = &*st.rules;
        let hungry = s.health < 40;
        let mut best = moves[0];
        let mut best_v = i32::MIN;
        for &m in moves {
            let c = r.step(s.head(), m);
            let mut v = 0;
            for d in Dir::ALL {
                let nb = r.step(c, d);
                if nb != crate::game::NONE
                    && !st.snakes.iter().any(|o| o.alive && o.body.contains(&nb))
                {
                    v += 10;
                }
            }
            if st.has_food(c) && (hungry || s.len() < 8) {
                v += 15;
            }
            if r.hazards[c as usize] > 0 {
                v -= 20;
            }
            if v > best_v {
                best_v = v;
                best = m;
            }
        }
        best
}

/// Heuristic move for snake `j` among its safe `moves` (bitboard boards only): avoid cells next
/// to heads that win or tie a head-to-head, avoid pockets smaller than the body, go for food
/// when hungry or short, and lean towards the centre.
pub fn smart_move(st: &State, j: usize, moves: &[Dir]) -> Dir {
    smart_move_p(st, j, moves, &SmartParams::default())
}

/// Tunable constants of [`smart_move`].
#[derive(Clone, Debug)]
pub struct SmartParams {
    pub danger: f32,
    pub trap: f32,
    pub trap_cell: f32,
    pub room: f32,
    pub cap_extra: u32,
    pub hunger_low: f32,
    pub hunger_mid: f32,
    pub hunger_full: f32,
    pub eat: f32,
    pub center: f32,
}

impl Default for SmartParams {
    fn default() -> Self {
        SmartParams {
            danger: 60.0,
            trap: 40.0,
            trap_cell: 3.0,
            room: 0.5,
            cap_extra: 4,
            hunger_low: 4.0,
            hunger_mid: 2.0,
            hunger_full: 0.5,
            eat: 3.0,
            center: 0.25,
        }
    }
}

pub fn smart_move_p(st: &State, j: usize, moves: &[Dir], p: &SmartParams) -> Dir {
    let r = &*st.rules;
    let s = &st.snakes[j];
    let len = s.len();
    // cells that stay blocked next turn: every body segment except tails that move away
    let mut blocked: u128 = 0;
    let mut danger: u128 = 0;
    for (k, o) in st.snakes.iter().enumerate() {
        if !o.alive {
            continue;
        }
        let keep = if o.tail_moves() { o.len() - 1 } else { o.len() };
        for &c in o.body.iter().take(keep) {
            blocked |= 1u128 << c;
        }
        if k != j && o.len() >= len {
            danger |= r.bb_expand(1u128 << o.head());
        }
    }
    let mut food_bb: u128 = 0;
    for &f in &st.food {
        food_bb |= 1u128 << f;
    }
    let max_other = st
        .snakes
        .iter()
        .enumerate()
        .filter(|(k, o)| *k != j && o.alive)
        .map(|(_, o)| o.len())
        .max()
        .unwrap_or(0);
    let hunger = if s.health < 30 {
        p.hunger_low
    } else if s.health < 60 || len <= max_other + 1 {
        p.hunger_mid
    } else {
        p.hunger_full
    };
    let free = r.full_bb & !blocked;
    let cap = len as u32 + p.cap_extra;
    let mut best = moves[0];
    let mut best_v = f32::MIN;
    for &m in moves {
        let c = r.step(s.head(), m);
        let cb = 1u128 << c;
        let mut v = 0.0f32;
        if danger & cb != 0 {
            v -= p.danger;
        }
        // bounded flood fill: room reachable from the new head, and the distance to food
        let mut seen = cb;
        let mut frontier = cb;
        let mut food_d = if food_bb & cb != 0 { 0 } else { 99 };
        let mut steps = 0;
        while frontier != 0 && seen.count_ones() < cap && steps < 12 {
            steps += 1;
            frontier = r.bb_expand(frontier) & free & !seen;
            seen |= frontier;
            if food_d == 99 && frontier & food_bb != 0 {
                food_d = steps;
            }
        }
        let room = seen.count_ones();
        if room < len as u32 {
            v -= p.trap + p.trap_cell * (len as u32 - room) as f32;
        }
        v += room.min(cap) as f32 * p.room;
        if food_d < 99 {
            v -= hunger * food_d as f32;
            if food_d == 0 {
                v += p.eat * hunger;
            }
        } else {
            v -= hunger * 14.0;
        }
        let (x, y) = r.xy(c);
        let dc = (2 * x - (r.width - 1)).abs() + (2 * y - (r.height - 1)).abs();
        v -= p.center * dc as f32;
        if r.hazards[c as usize] > 0 {
            v -= 20.0;
        }
        if v > best_v {
            best_v = v;
            best = m;
        }
    }
    best
}

/// Pick a move. `deadline` is a hard limit; depth 1 is always completed.
pub fn search(st: &State, cfg: &SearchConfig, deadline: Instant) -> SearchResult {
    let mut root = [Dir::Up; 4];
    let n = st.safe_moves(0, &mut root);
    if n == 0 {
        let m = st.fallback_move(0);
        return SearchResult {
            best: m,
            score: LOSS,
            depth: 0,
            nodes: 0,
            scores: vec![(m, LOSS)],
        };
    }
    if n == 1 {
        return SearchResult {
            best: root[0],
            score: 0.0,
            depth: 0,
            nodes: 0,
            scores: vec![(root[0], 0.0)],
        };
    }
    let opps = st.alive_opponents();
    let limited = cfg.deep_ply < 99 && opps >= cfg.deep_min_opps.max(2);
    if !limited || cfg.veto_frac <= 0.0 {
        return search_root(st, cfg, deadline, &root[..n]);
    }
    // Phase 1: the full paranoid search vetoes root moves it proves lost. The limited search
    // plays better positionally but cannot see traps set by opponents away from our head.
    let now = Instant::now();
    let d1 = now + deadline.saturating_duration_since(now).mul_f32(cfg.veto_frac);
    let full = SearchConfig {
        deep_ply: 99,
        ..cfg.clone()
    };
    let r1 = search_root(st, &full, d1, &root[..n]);
    if is_win(r1.score) {
        return r1;
    }
    let allowed: Vec<Dir> = r1
        .scores
        .iter()
        .filter(|(_, v)| !is_loss(*v))
        .map(|(m, _)| *m)
        .collect();
    if allowed.len() == 1 {
        return SearchResult {
            best: allowed[0],
            ..r1
        };
    }
    let moves: &[Dir] = if allowed.is_empty() { &root[..n] } else { &allowed };
    let mut r2 = search_root(st, cfg, deadline, moves);
    r2.nodes += r1.nodes;
    for &(m, v) in &r1.scores {
        if !r2.scores.iter().any(|x| x.0 == m) {
            r2.scores.push((m, v));
        }
    }
    r2
}

fn search_root(st: &State, cfg: &SearchConfig, deadline: Instant, root: &[Dir]) -> SearchResult {
    let n = root.len();
    let start_opps = st.alive_opponents();
    let stop = AtomicBool::new(false);
    let shared = AtomicU32::new((-INF).to_bits());
    let w = &cfg.weights;

    // per root move context (keeps killers across iterations)
    let far = Instant::now() + Duration::from_secs(3600);
    let mut ctxs: Vec<Ctx> = (0..n)
        .map(|_| {
            let mut c = Ctx::new(w, far, &stop, start_opps, cfg.max_relevant, cfg.max_reach);
            c.deep_ply = cfg.deep_ply;
            c.deep_reach = cfg.deep_reach;
            c.deep_min_opps = cfg.deep_min_opps;
            c.smart_default = cfg.smart_default;
            if cfg.parallel {
                c.shared_alpha = Some(&shared);
            }
            c
        })
        .collect();

    // order: moves sorted by last iteration's score
    let mut order: Vec<usize> = (0..n).collect();
    let mut prev: Vec<f32> = vec![0.0; n];
    let mut best_idx = 0usize;
    let mut best_score = -INF;
    let mut done_depth = 0;

    for depth in 1..=cfg.max_depth {
        let dl = if depth == 1 { far } else { deadline };
        for c in ctxs.iter_mut() {
            c.deadline = dl;
            c.aborted = false;
            c.alpha_seen = -INF;
        }
        shared.store((-INF).to_bits(), Ordering::Relaxed);
        // (value, exact): non-exact values are upper bounds (the move failed low)
        let mut res: Vec<Option<(f32, bool)>> = vec![None; n];

        if cfg.parallel {
            let shared = &shared;
            let results: Vec<(usize, Option<(f32, bool)>)> = std::thread::scope(|scope| {
                let mut handles = Vec::new();
                for (i, c) in ctxs.iter_mut().enumerate() {
                    let m = root[i];
                    handles.push(scope.spawn(move || {
                        // private copy of the rules: every State clone bumps the Arc
                        // refcount, which would otherwise bounce between cores
                        let local = State {
                            rules: std::sync::Arc::new((*st.rules).clone()),
                            ..st.clone()
                        };
                        let v = c.min_node(&local, m, depth, 0, -INF, INF, 0.0);
                        if c.aborted {
                            return (i, None);
                        }
                        let exact = v > c.alpha_seen;
                        if exact {
                            // raise the shared bound
                            let mut cur = shared.load(Ordering::Relaxed);
                            while v > f32::from_bits(cur) {
                                match shared.compare_exchange_weak(
                                    cur,
                                    v.to_bits(),
                                    Ordering::Relaxed,
                                    Ordering::Relaxed,
                                ) {
                                    Ok(_) => break,
                                    Err(x) => cur = x,
                                }
                            }
                        }
                        (i, Some((v, exact)))
                    }));
                }
                handles.into_iter().map(|h| h.join().unwrap()).collect()
            });
            for (i, v) in results {
                res[i] = v;
            }
        } else {
            // sequential: first move with full window, others against best (exact when better)
            let mut alpha = -INF;
            for &i in &order {
                let c = &mut ctxs[i];
                let v = c.min_node(st, root[i], depth, 0, alpha, INF, 0.0);
                if c.aborted {
                    break;
                }
                res[i] = Some((v, v > alpha));
                if v > alpha && !cfg.exact_root {
                    alpha = v;
                }
            }
        }

        // better(a, b): a is preferred over b (higher value; exact wins ties)
        let better = |a: (f32, bool), b: (f32, bool)| a.0 > b.0 || (a.0 == b.0 && a.1 && !b.1);

        let complete = res.iter().all(|r| r.is_some());
        if complete {
            let scores: Vec<f32> = res.iter().map(|r| r.unwrap().0).collect();
            // In multi-player games a paranoid "everything loses" is mostly an artefact of
            // assuming all opponents cooperate: keep the last iteration that still had hope.
            if start_opps >= 2
                && depth > 1
                && scores.iter().all(|&v| is_loss(v))
                && !is_loss(best_score)
            {
                break;
            }
            // best: max score; ties broken by previous iteration
            let mut bi = order[0];
            for &i in &order {
                if better(res[i].unwrap(), res[bi].unwrap()) {
                    bi = i;
                }
            }
            best_idx = bi;
            best_score = scores[bi];
            prev = scores;
            done_depth = depth;
            order.sort_by(|&a, &b| {
                prev[b]
                    .partial_cmp(&prev[a])
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if is_win(best_score) || is_loss(best_score) {
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
        } else {
            // partial iteration: only trust it if the previous best was re-searched
            if let Some(v_prev_best) = res[best_idx] {
                let mut bi = best_idx;
                let mut bv = v_prev_best;
                for i in 0..n {
                    if let Some(v) = res[i] {
                        if better(v, bv) {
                            bv = v;
                            bi = i;
                        }
                    }
                }
                let mut bv = bv.0;
                if is_loss(bv) {
                    // our best turned out to lose: prefer an unfinished move that looked fine before
                    let mut alt = None;
                    for &i in &order {
                        if res[i].is_none() && !is_loss(prev[i]) {
                            alt = Some(i);
                            break;
                        }
                    }
                    if let Some(a) = alt {
                        bi = a;
                        bv = prev[a];
                    }
                }
                best_idx = bi;
                best_score = bv;
            }
            break;
        }
    }

    let nodes = ctxs.iter().map(|c| c.nodes).sum();
    SearchResult {
        best: root[best_idx],
        score: best_score,
        depth: done_depth,
        nodes,
        scores: (0..n).map(|i| (root[i], prev[i])).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Rules, Snake};
    use std::sync::Arc;

    fn mk(snakes: Vec<Vec<(i32, i32)>>, food: Vec<(i32, i32)>) -> State {
        let mut r = Rules::new(11, 11, false, false, 0);
        r.finalize();
        let r = Arc::new(r);
        State {
            snakes: snakes
                .iter()
                .map(|b| Snake {
                    body: b.iter().map(|&(x, y)| r.cell(x, y)).collect(),
                    health: 90,
                    alive: true,
                })
                .collect(),
            food: food.iter().map(|&(x, y)| r.cell(x, y)).collect(),
            rules: r,
            turn: 10,
        }
    }

    fn best(st: &State, ms: u64) -> SearchResult {
        let cfg = SearchConfig {
            parallel: false,
            ..Default::default()
        };
        search(st, &cfg, Instant::now() + Duration::from_millis(ms))
    }

    #[test]
    fn avoids_head_to_head_with_bigger_snake() {
        // we are at (5,5) heading up; a longer opponent head at (5,7) could meet us at (5,6)
        let st = mk(
            vec![
                vec![(5, 5), (5, 4), (5, 3)],
                vec![(5, 7), (5, 8), (6, 8), (7, 8), (8, 8)],
            ],
            vec![],
        );
        let r = best(&st, 50);
        assert_ne!(r.best, Dir::Up, "{:?}", r);
    }

    #[test]
    fn finds_forced_kill() {
        // opponent squeezed between the wall and our body; its only move is (0,1),
        // which we can take with a winning head-to-head
        let st = mk(
            vec![
                vec![(1, 1), (1, 2), (1, 3), (1, 4), (1, 5)],
                vec![(0, 2), (0, 3), (0, 4)],
            ],
            vec![],
        );
        let r = best(&st, 100);
        assert!(is_win(r.score), "{:?}", r);
        assert_eq!(r.best, Dir::Left);
    }

    #[test]
    fn distant_last_opponent_is_not_a_proven_win() {
        // arena game d3087934, turn 434: the opponent is ten cells away in a pocket and survives
        // by following its own tail, but its heuristic default move runs into the dead end
        let js = std::fs::read_to_string("tests/fixtures/false_win_duel.json").unwrap();
        let st = serde_json::from_str::<crate::api::GameState>(&js)
            .unwrap()
            .to_state();
        let r = best(&st, 100);
        assert!(!is_win(r.score), "{:?}", r);
        assert!(r.depth > 5, "{:?}", r);
    }

    #[test]
    fn escapes_trap() {
        // only Right leads to open space; Up leads into a dead-end pocket
        let st = mk(
            vec![
                vec![(1, 5), (1, 4), (1, 3), (1, 2), (1, 1)],
                vec![
                    (0, 7),
                    (1, 7),
                    (2, 7),
                    (2, 6),
                    (3, 6),
                    (3, 5),
                    (3, 4),
                    (4, 4),
                    (5, 4),
                    (6, 4),
                ],
            ],
            vec![],
        );
        let r = best(&st, 100);
        assert!(!is_loss(r.score), "{:?}", r);
    }
}
