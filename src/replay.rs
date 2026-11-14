//! Arena replays as stored by `tools/arena.py` ({"meta": ..., "frames": [...]}): load a game and
//! rebuild the engine request any snake saw at any turn.

use crate::api::GameState;
use crate::game::Dir;
use serde_json::{json, Value};

pub struct Replay {
    pub id: String,
    pub game: Value,
    pub frames: Vec<Value>,
}

fn pt(p: &Value) -> Value {
    json!({"x": p["X"], "y": p["Y"]})
}

fn api_snake(s: &Value) -> Value {
    let body: Vec<Value> = s["Body"].as_array().map(|b| b.iter().map(pt).collect()).unwrap_or_default();
    json!({"id": s["ID"], "name": s["Name"], "health": s["Health"], "head": body.first(), "length": body.len(),
           "body": body, "latency": "0", "shout": ""})
}

pub fn is_dead(s: &Value) -> bool {
    !s["Death"].is_null()
}

impl Replay {
    pub fn load(path: &str) -> Option<Replay> {
        let txt = std::fs::read_to_string(path).ok()?;
        let mut v: Value = serde_json::from_str(&txt).ok()?;
        let frames = match v["frames"].take() {
            Value::Array(f) if f.len() >= 3 => f,
            _ => return None,
        };
        let game = v["meta"]["Game"].take();
        Some(Replay {
            id: game["ID"].as_str().unwrap_or("?").to_string(),
            game,
            frames,
        })
    }

    pub fn snakes(&self, frame: usize) -> &[Value] {
        self.frames[frame]["Snakes"].as_array().map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Number of snakes at the start of the game.
    pub fn players(&self) -> usize {
        self.snakes(0).len()
    }

    /// Name of the only survivor, if there is one.
    pub fn winner(&self) -> Option<&str> {
        let alive: Vec<&Value> = self.snakes(self.frames.len() - 1).iter().filter(|s| !is_dead(s)).collect();
        if alive.len() == 1 {
            alive[0]["Name"].as_str()
        } else {
            None
        }
    }

    pub fn find(&self, frame: usize, name: &str) -> Option<&Value> {
        self.snakes(frame).iter().find(|s| s["Name"] == name)
    }

    /// The engine request of the snake with this ID at `frame`, or None if it is dead there.
    pub fn request(&self, frame: usize, id: &Value) -> Option<GameState> {
        let f = &self.frames[frame];
        let cur: Vec<&Value> = self.snakes(frame).iter().filter(|s| !is_dead(s)).collect();
        let you = cur.iter().find(|s| &s["ID"] == id)?;
        let g = &self.game;
        let damage = g["Ruleset"]["damagePerTurn"].as_str().and_then(|x| x.parse::<i64>().ok()).unwrap_or(0);
        let settings = json!({"foodSpawnChance": 15, "minimumFood": 1, "hazardDamagePerTurn": damage});
        let list = |k: &str| f[k].as_array().map(|h| h.iter().map(pt).collect::<Vec<_>>()).unwrap_or_default();
        let gj = json!({
            "game": {"id": self.id, "ruleset": {"name": g["RulesetName"], "version": "1", "settings": settings},
                     "map": g["Map"], "timeout": 500, "source": "arena", "players": self.players()},
            "turn": f["Turn"],
            "board": {"height": g["Height"].as_i64().unwrap_or(11), "width": g["Width"].as_i64().unwrap_or(11),
                      "food": list("Food"), "hazards": list("Hazards"),
                      "snakes": cur.iter().map(|x| api_snake(x)).collect::<Vec<_>>()},
            "you": api_snake(you)});
        serde_json::from_value(gj).ok()
    }

    /// The move the snake played at `frame` (from its head in the next frame).
    pub fn played(&self, frame: usize, id: &Value) -> Option<Dir> {
        let a = self.snakes(frame).iter().find(|s| &s["ID"] == id)?;
        let b = self.snakes(frame + 1).iter().find(|s| &s["ID"] == id)?;
        let (a, b) = (&a["Body"][0], &b["Body"][0]);
        match (b["X"].as_i64()? - a["X"].as_i64()?, b["Y"].as_i64()? - a["Y"].as_i64()?) {
            (0, 1) => Some(Dir::Up),
            (0, -1) => Some(Dir::Down),
            (-1, 0) => Some(Dir::Left),
            (1, 0) => Some(Dir::Right),
            _ => None,
        }
    }
}
