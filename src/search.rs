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
}

impl Default for SearchConfig {
    fn default() -> Self {
        SearchConfig {
            weights: Weights::default(),
            parallel: true,
            max_depth: 60,
            max_relevant: 3,
            max_reach: 99,
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
        let s = &st.snakes[j];
        let r = &*st.rules;
        let hungry = s.health < 40;
        let mut best = moves[0];
        let mut best_v = i32::MIN;
        for &m in &moves[..n] {
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
        let reach = (2 * depth as i32 + 1).min(self.max_reach);
        for (rank, &(d, j)) in cand[..nc].iter().enumerate() {
            idxs[no] = j;
            if d <= reach && rank < self.max_relevant {
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
                    if child.snakes[0].len() > st.snakes[0].len() {
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
    let start_opps = st.alive_opponents();
    let stop = AtomicBool::new(false);
    let shared = AtomicU32::new((-INF).to_bits());
    let w = &cfg.weights;

    // per root move context (keeps killers across iterations)
    let far = Instant::now() + Duration::from_secs(3600);
    let mut ctxs: Vec<Ctx> = (0..n)
        .map(|_| {
            let mut c = Ctx::new(w, far, &stop, start_opps, cfg.max_relevant, cfg.max_reach);
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
                if v > alpha {
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
