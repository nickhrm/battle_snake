//! Compact game model + exact re-implementation of the official Battlesnake rules
//! (standard / royale / constrictor / wrapped) used by the search.

use std::collections::VecDeque;
use std::sync::Arc;

pub const NONE: u16 = u16::MAX;
pub const MAX_CELLS: usize = 32 * 32;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Dir {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

    pub fn name(self) -> &'static str {
        match self {
            Dir::Up => "up",
            Dir::Down => "down",
            Dir::Left => "left",
            Dir::Right => "right",
        }
    }

    #[inline]
    pub fn idx(self) -> usize {
        self as usize
    }
}

/// Static information about a game that does not change during a search.
#[derive(Clone, Debug)]
pub struct Rules {
    pub width: i32,
    pub height: i32,
    pub wrapped: bool,
    pub constrictor: bool,
    /// shrinking board; hazards can be absent during the first turns
    pub royale: bool,
    pub hazard_damage: i32,
    /// number of stacked hazards on each cell
    pub hazards: Vec<u8>,
    /// hazard bitboard (only valid when `bitboard` is true); set by `finalize`
    pub hazard_bb: u128,
    /// cells with even x + y (only valid when `bitboard` is true); set by `finalize`
    pub even_bb: u128,
    /// board fits into a u128 and is not wrapped -> fast bitboard evaluation
    pub bitboard: bool,
    /// neighbour table: nbr[cell*4 + dir] (NONE if off-board)
    pub nbr: Vec<u16>,
}

impl Rules {
    pub fn new(
        width: i32,
        height: i32,
        wrapped: bool,
        constrictor: bool,
        hazard_damage: i32,
    ) -> Rules {
        let cells = (width * height) as usize;
        assert!(cells <= MAX_CELLS, "board too large");
        let mut nbr = vec![NONE; cells * 4];
        for y in 0..height {
            for x in 0..width {
                let c = (y * width + x) as usize;
                for d in Dir::ALL {
                    let (mut nx, mut ny) = match d {
                        Dir::Up => (x, y + 1),
                        Dir::Down => (x, y - 1),
                        Dir::Left => (x - 1, y),
                        Dir::Right => (x + 1, y),
                    };
                    if wrapped {
                        nx = nx.rem_euclid(width);
                        ny = ny.rem_euclid(height);
                    }
                    if nx >= 0 && nx < width && ny >= 0 && ny < height {
                        nbr[c * 4 + d.idx()] = (ny * width + nx) as u16;
                    }
                }
            }
        }
        Rules {
            width,
            height,
            wrapped,
            constrictor,
            royale: false,
            hazard_damage,
            hazards: vec![0; cells],
            hazard_bb: 0,
            even_bb: 0,
            bitboard: cells <= 128 && !wrapped,
            nbr,
        }
    }

    /// Call after modifying `hazards`.
    pub fn finalize(&mut self) {
        self.hazard_bb = 0;
        if self.bitboard {
            self.even_bb = 0;
            for y in 0..self.height {
                for x in 0..self.width {
                    if (x + y) % 2 == 0 {
                        self.even_bb |= 1u128 << self.cell(x, y);
                    }
                }
            }
            for (i, &h) in self.hazards.iter().enumerate() {
                if h > 0 {
                    self.hazard_bb |= 1u128 << i;
                }
            }
        }
    }

    #[inline]
    pub fn cells(&self) -> usize {
        (self.width * self.height) as usize
    }

    #[inline]
    pub fn step(&self, c: u16, d: Dir) -> u16 {
        self.nbr[c as usize * 4 + d.idx()]
    }

    #[inline]
    pub fn xy(&self, c: u16) -> (i32, i32) {
        (c as i32 % self.width, c as i32 / self.width)
    }

    #[inline]
    pub fn cell(&self, x: i32, y: i32) -> u16 {
        (y * self.width + x) as u16
    }

    /// Manhattan distance (respecting wrap).
    #[inline]
    pub fn dist(&self, a: u16, b: u16) -> i32 {
        let (ax, ay) = self.xy(a);
        let (bx, by) = self.xy(b);
        let mut dx = (ax - bx).abs();
        let mut dy = (ay - by).abs();
        if self.wrapped {
            dx = dx.min(self.width - dx);
            dy = dy.min(self.height - dy);
        }
        dx + dy
    }

    pub fn has_hazards(&self) -> bool {
        self.hazard_damage != 0 && self.hazards.iter().any(|&h| h > 0)
    }
}

#[derive(Clone, Debug)]
pub struct Snake {
    /// body[0] is the head
    pub body: VecDeque<u16>,
    pub health: i32,
    pub alive: bool,
}

impl Snake {
    #[inline]
    pub fn head(&self) -> u16 {
        self.body[0]
    }
    #[inline]
    pub fn len(&self) -> usize {
        self.body.len()
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }
    /// true when the tail will move away on the next turn (i.e. not stacked)
    #[inline]
    pub fn tail_moves(&self) -> bool {
        let n = self.body.len();
        n >= 2 && self.body[n - 1] != self.body[n - 2]
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub rules: Arc<Rules>,
    pub snakes: Vec<Snake>,
    pub food: Vec<u16>,
    pub turn: u32,
}

impl State {
    #[inline]
    pub fn me(&self) -> &Snake {
        &self.snakes[0]
    }

    pub fn alive_count(&self) -> usize {
        self.snakes.iter().filter(|s| s.alive).count()
    }

    pub fn alive_opponents(&self) -> usize {
        self.snakes.iter().skip(1).filter(|s| s.alive).count()
    }

    #[inline]
    pub fn has_food(&self, c: u16) -> bool {
        self.food.contains(&c)
    }

    /// Moves that do not lead into a wall, the own neck or a body segment that
    /// is guaranteed to still be there next turn. Never empty for a living snake
    /// unless it is fully boxed in, in which case the result is empty.
    pub fn safe_moves(&self, idx: usize, out: &mut [Dir; 4]) -> usize {
        let s = &self.snakes[idx];
        let head = s.head();
        let neck = if s.len() > 1 { s.body[1] } else { NONE };
        let mut n = 0;
        'dirs: for d in Dir::ALL {
            let c = self.rules.step(head, d);
            if c == NONE || (c == neck && neck != head) {
                continue;
            }
            for o in self.snakes.iter() {
                if !o.alive {
                    continue;
                }
                let len = o.len();
                // the last segment vacates if the tail is not stacked
                let check = if o.tail_moves() { len - 1 } else { len };
                for i in 0..check {
                    // (another snake's current head becomes its neck next turn)
                    if o.body[i] == c {
                        continue 'dirs;
                    }
                }
            }
            out[n] = d;
            n += 1;
        }
        n
    }

    /// Any move that at least stays on the board (used when everything is deadly).
    pub fn fallback_move(&self, idx: usize) -> Dir {
        let s = &self.snakes[idx];
        let neck = if s.len() > 1 { s.body[1] } else { NONE };
        for d in Dir::ALL {
            let c = self.rules.step(s.head(), d);
            if c != NONE && c != neck {
                return d;
            }
        }
        Dir::Up
    }

    /// Advance the game by one turn. `moves[i]` is used for snake `i` (ignored if dead).
    pub fn advance(&mut self, moves: &[Dir]) {
        // borrow the rules without touching the Arc's (shared, contended) refcount
        let rules: &Rules = &self.rules;
        let n = self.snakes.len();

        // 1. movement
        for i in 0..n {
            let s = &mut self.snakes[i];
            if !s.alive {
                continue;
            }
            let nh = rules.step(s.head(), moves[i]);
            if nh == NONE {
                s.alive = false;
                continue;
            }
            s.body.push_front(nh);
            s.body.pop_back();
        }

        // 2. starvation tick + 3. hazard damage
        for s in self.snakes.iter_mut() {
            if !s.alive {
                continue;
            }
            s.health -= 1;
            let h = s.head() as usize;
            let stack = rules.hazards[h] as i32;
            if stack > 0 && rules.hazard_damage != 0 && !self.food.contains(&s.head()) {
                s.health -= rules.hazard_damage * stack;
                s.health = s.health.clamp(0, 100);
            }
        }

        // 4. feeding
        if !self.food.is_empty() {
            let mut eaten: [u16; 16] = [NONE; 16];
            let mut ne = 0;
            for s in self.snakes.iter_mut() {
                if !s.alive {
                    continue;
                }
                let h = s.head();
                if self.food.contains(&h) {
                    s.health = 100;
                    let t = *s.body.back().unwrap();
                    s.body.push_back(t);
                    if ne < 16 {
                        eaten[ne] = h;
                        ne += 1;
                    }
                }
            }
            if ne > 0 {
                self.food.retain(|f| !eaten[..ne].contains(f));
            }
        }

        // 5. elimination
        let mut dead = [false; 16];
        for i in 0..n {
            if self.snakes[i].alive && self.snakes[i].health <= 0 {
                dead[i] = true;
            }
        }
        // snakes still "in the game" for collision purposes
        let mut present = [false; 16];
        for i in 0..n {
            present[i] = self.snakes[i].alive && !dead[i];
        }
        let mut coll = [false; 16];
        for i in 0..n {
            if !present[i] {
                continue;
            }
            let head = self.snakes[i].head();
            let my_len = self.snakes[i].len();
            // self collision
            if self.snakes[i].body.iter().skip(1).any(|&c| c == head) {
                coll[i] = true;
                continue;
            }
            for j in 0..n {
                if j == i || !present[j] {
                    continue;
                }
                let o = &self.snakes[j];
                if o.body.iter().skip(1).any(|&c| c == head) {
                    coll[i] = true;
                    break;
                }
                if o.head() == head && my_len <= o.len() {
                    coll[i] = true;
                    break;
                }
            }
        }
        for i in 0..n {
            if dead[i] || coll[i] {
                self.snakes[i].alive = false;
            }
        }

        // constrictor: everyone grows every turn and is always at full health
        if rules.constrictor {
            for s in self.snakes.iter_mut() {
                if s.alive {
                    s.health = 100;
                    let t = *s.body.back().unwrap();
                    s.body.push_back(t);
                }
            }
            self.food.clear();
        }

        self.turn += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(rules: Rules, snakes: Vec<(Vec<(i32, i32)>, i32)>, food: Vec<(i32, i32)>) -> State {
        let r = Arc::new(rules);
        let snakes = snakes
            .into_iter()
            .map(|(b, h)| Snake {
                body: b.iter().map(|&(x, y)| r.cell(x, y)).collect(),
                health: h,
                alive: true,
            })
            .collect();
        let food = food.iter().map(|&(x, y)| r.cell(x, y)).collect();
        State {
            rules: r,
            snakes,
            food,
            turn: 0,
        }
    }

    #[test]
    fn head_to_head_equal_both_die() {
        let s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![
                (vec![(2, 5), (1, 5), (0, 5)], 90),
                (vec![(4, 5), (5, 5), (6, 5)], 90),
            ],
            vec![],
        );
        let mut s2 = s.clone();
        s2.advance(&[Dir::Right, Dir::Left]);
        assert!(!s2.snakes[0].alive && !s2.snakes[1].alive);
    }

    #[test]
    fn head_to_head_longer_wins() {
        let mut s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![
                (vec![(2, 5), (1, 5), (0, 5), (0, 4)], 90),
                (vec![(4, 5), (5, 5), (6, 5)], 90),
            ],
            vec![],
        );
        s.advance(&[Dir::Right, Dir::Left]);
        assert!(s.snakes[0].alive && !s.snakes[1].alive);
    }

    #[test]
    fn tail_chasing_is_safe_but_not_after_eating() {
        // snake in a little loop chasing its own tail
        let mut s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![(vec![(1, 1), (1, 2), (2, 2), (2, 1)], 90)],
            vec![],
        );
        let mut m = [Dir::Up; 4];
        let n = s.safe_moves(0, &mut m);
        assert!(m[..n].contains(&Dir::Right));
        s.advance(&[Dir::Right]);
        assert!(s.snakes[0].alive);
        // stacked tail: not safe
        let s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![(vec![(1, 1), (1, 2), (2, 2), (2, 1), (2, 1)], 90)],
            vec![],
        );
        let n = s.safe_moves(0, &mut m);
        assert!(!m[..n].contains(&Dir::Right));
    }

    #[test]
    fn eating_grows_and_heals() {
        let mut s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![(vec![(1, 1), (1, 0), (0, 0)], 1)],
            vec![(1, 2)],
        );
        s.advance(&[Dir::Up]);
        assert!(s.snakes[0].alive);
        assert_eq!(s.snakes[0].health, 100);
        assert_eq!(s.snakes[0].len(), 4);
        assert!(s.food.is_empty());
    }

    #[test]
    fn starvation() {
        let mut s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![(vec![(1, 1), (1, 0), (0, 0)], 1)],
            vec![],
        );
        s.advance(&[Dir::Up]);
        assert!(!s.snakes[0].alive);
    }

    #[test]
    fn hazard_damage() {
        let mut r = Rules::new(11, 11, false, false, 14);
        let c = r.cell(1, 2);
        r.hazards[c as usize] = 1;
        let mut s = mk(r, vec![(vec![(1, 1), (1, 0), (0, 0)], 50)], vec![]);
        s.advance(&[Dir::Up]);
        assert_eq!(s.snakes[0].health, 35);
    }

    #[test]
    fn wrapped_moves() {
        let mut s = mk(
            Rules::new(11, 11, true, false, 0),
            vec![(vec![(0, 5), (1, 5), (2, 5)], 50)],
            vec![],
        );
        s.advance(&[Dir::Left]);
        assert!(s.snakes[0].alive);
        assert_eq!(s.rules.xy(s.snakes[0].head()), (10, 5));
    }

    #[test]
    fn body_collision() {
        let mut s = mk(
            Rules::new(11, 11, false, false, 0),
            vec![
                (vec![(2, 4), (1, 4), (0, 4)], 90),
                (vec![(3, 5), (2, 5), (1, 5), (0, 5)], 90),
            ],
            vec![],
        );
        s.advance(&[Dir::Up, Dir::Right]);
        assert!(!s.snakes[0].alive);
        assert!(s.snakes[1].alive);
    }
}
