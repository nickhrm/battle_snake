use std::{collections::HashMap, fmt};

use rand::Rng;
use serde::Serialize;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Moves {
    Left,
    Right,
    Up,
    Down,
}

impl fmt::Display for Moves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Up => "up",
            Self::Down => "down",
        };
        write!(f, "{}", direction)
    }
}

impl Serialize for Moves {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
#[derive(Clone)]
pub struct SafeMoves {
    moves: HashMap<Moves, bool>,
}

impl SafeMoves {
    pub fn init() -> SafeMoves {
        SafeMoves {
            moves: vec![
                (Moves::Left, true),
                (Moves::Right, true),
                (Moves::Up, true),
                (Moves::Down, true),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn remove_move(&mut self, mov: Moves) {
        self.moves.insert(mov, false);
    }

    pub fn get_safe_moves(&self) -> Vec<Moves> {
        self.moves
            .iter()
            .filter(|&(_, &v)| v)
            .map(|(&ref k, _)| k.clone())
            .collect()
    }

    pub fn get_safe_move(&self) -> Moves {
        let safe_moves = self.get_safe_moves();
        let len = safe_moves.len();
        let index = rand::thread_rng().gen_range(0..len);
        safe_moves[index].clone()
    }
}
