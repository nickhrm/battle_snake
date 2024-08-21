use std::{collections::HashMap, fmt};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Moves {
    Left,
    Right,
    Up,
    Down,
}


impl fmt::Display for Moves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f,"Left"),
            Self::Right => write!(f,"Right"),
            Self::Up => write!(f,"Up"),
            Self::Down => write!(f,"Down"),
        }
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
        *self.get_safe_moves().get( rand::thread_rng()).unwrap()
    }
}
