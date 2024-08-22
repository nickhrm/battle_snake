use std::{collections::HashMap, fmt};

use rand::Rng;
use serde::Serialize;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
}

impl fmt::Display for Move {
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

impl Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
#[derive(Clone)]
pub struct SafeMoves {
    moves: HashMap<Move, bool>,
}

impl SafeMoves {
    pub fn init() -> SafeMoves {
        SafeMoves {
            moves: vec![
                (Move::Left, true),
                (Move::Right, true),
                (Move::Up, true),
                (Move::Down, true),
            ]
            .into_iter()
            .collect(),
        }
    }

    pub fn remove_move(&mut self, mov: Move) {
        self.moves.insert(mov, false);
    }

    pub fn get_safe_moves(&self) -> Vec<Move> {
        self.moves
            .iter()
            .filter(|&(_, &v)| v)
            .map(|(&ref k, _)| k.clone())
            .collect()
    }

    pub fn get_safe_move(&self) -> Move {
        let safe_moves = self.get_safe_moves();
        let len = safe_moves.len();
        let index = rand::thread_rng().gen_range(0..len);
        safe_moves[index].clone()
    }
}
