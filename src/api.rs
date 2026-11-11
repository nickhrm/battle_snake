//! Battlesnake webhook JSON objects and conversion into the internal model.

use crate::game::{Rules, Snake, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct RoyaleSettings {
    #[serde(rename = "shrinkEveryNTurns")]
    pub shrink_every_n_turns: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct RulesetSettings {
    #[serde(rename = "foodSpawnChance")]
    pub food_spawn_chance: i32,
    #[serde(rename = "minimumFood")]
    pub minimum_food: i32,
    #[serde(rename = "hazardDamagePerTurn")]
    pub hazard_damage_per_turn: i32,
    pub royale: RoyaleSettings,
}

impl Default for RulesetSettings {
    fn default() -> Self {
        RulesetSettings {
            food_spawn_chance: 15,
            minimum_food: 1,
            hazard_damage_per_turn: 14,
            royale: RoyaleSettings::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    pub settings: RulesetSettings,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub map: String,
    pub timeout: u64,
    pub source: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct ApiSnake {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub body: Vec<Coord>,
    pub head: Option<Coord>,
    pub length: i32,
    pub latency: serde_json::Value,
    pub shout: Option<String>,
    pub squad: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes: Vec<ApiSnake>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: ApiSnake,
}

impl ApiSnake {
    /// The direction of our last move, which the engine repeats when our answer is late
    /// ("up" before the first move).
    pub fn last_move(&self) -> crate::game::Dir {
        use crate::game::Dir;
        match (self.body.first(), self.body.get(1)) {
            (Some(h), Some(n)) => match (h.x - n.x, h.y - n.y) {
                (0, -1) => Dir::Down,
                (-1, 0) => Dir::Left,
                (1, 0) => Dir::Right,
                // wrapped boards: a step over the edge shows up as a jump
                (dx, 0) if dx > 1 => Dir::Left,
                (dx, 0) if dx < -1 => Dir::Right,
                (0, dy) if dy > 1 => Dir::Down,
                _ => Dir::Up,
            },
            _ => Dir::Up,
        }
    }

    /// latency (ms) the engine measured for our previous response
    pub fn latency_ms(&self) -> Option<u64> {
        match &self.latency {
            serde_json::Value::String(s) => s.trim().parse().ok(),
            serde_json::Value::Number(n) => n.as_u64(),
            _ => None,
        }
    }
}

impl GameState {
    pub fn is_wrapped(&self) -> bool {
        self.game.ruleset.name == "wrapped" || self.game.map.contains("wrapped")
    }
    pub fn is_constrictor(&self) -> bool {
        self.game.ruleset.name.contains("constrictor")
    }

    /// Convert into the search model. Our own snake is always index 0.
    pub fn to_state(&self) -> State {
        let w = self.board.width.max(1);
        let h = self.board.height.max(1);
        let dmg = self.game.ruleset.settings.hazard_damage_per_turn;
        let mut rules = Rules::new(w, h, self.is_wrapped(), self.is_constrictor(), dmg);
        // not shrinkEveryNTurns: our sim sends it in every mode, and the engine may do the same
        rules.royale = self.game.map == "royale" || !self.board.hazards.is_empty();
        let inb = |c: &Coord| c.x >= 0 && c.y >= 0 && c.x < w && c.y < h;
        for c in self.board.hazards.iter().filter(|c| inb(c)) {
            let i = rules.cell(c.x, c.y) as usize;
            rules.hazards[i] = rules.hazards[i].saturating_add(1);
        }
        rules.finalize();
        let rules = Arc::new(rules);
        let conv = |s: &ApiSnake| Snake {
            body: s
                .body
                .iter()
                .filter(|c| inb(c))
                .map(|c| rules.cell(c.x, c.y))
                .collect(),
            health: s.health,
            alive: !s.body.is_empty(),
        };
        let mut snakes = vec![conv(&self.you)];
        for s in &self.board.snakes {
            if s.id != self.you.id {
                let sn = conv(s);
                if !sn.body.is_empty() && snakes.len() < 16 {
                    snakes.push(sn);
                }
            }
        }
        let food = self
            .board
            .food
            .iter()
            .filter(|c| inb(c))
            .map(|c| rules.cell(c.x, c.y))
            .collect();
        State {
            rules,
            snakes,
            food,
            turn: self.turn,
        }
    }
}
