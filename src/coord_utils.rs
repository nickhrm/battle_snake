use serde::{Deserialize, Serialize};

use crate::{Battlesnake, Board};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn successors(&self, board: &Board, you: &Battlesnake) -> Vec<(Coord, u32)> {
        let all_moves = vec![
            Coord {
                x: self.x + 1,
                y: self.y,
            },
            Coord {
                x: self.x - 1,
                y: self.y,
            },
            Coord {
                x: self.x,
                y: self.y - 1,
            },
            Coord {
                x: self.x,
                y: self.y + 1,
            },
        ];

        return all_moves.into_iter().map(|p| (p, 1)).collect();
    }

    pub fn distance(&self, other: &Coord) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}
