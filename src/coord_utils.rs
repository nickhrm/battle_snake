use serde::{Deserialize, Serialize};

use crate::{Battlesnake, Board};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn successors(&self, board: &Board, you: &Battlesnake) -> Vec<(Coord, u32)> {
        let mut all_moves = vec![
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

        //prevent collision with its own body
        all_moves.retain(|future_coord| !you.body.iter().any(|coord| future_coord == coord));

        // //prevent collision with other snakes
        all_moves.retain(|future_coord| {
            !board
                .snakes
                .iter()
                .any(|snake| snake.body.iter().any(|coord| future_coord == coord))
        });

        
        let board_width = board.width;
        let board_height = board.height;

        all_moves.retain(|future_coord| {
            if future_coord.x >= board_width {
                return false;
            }
            if future_coord.x < 0 {
                return false;
            }
            if future_coord.y < 0 {
                return false;
            }
            if future_coord.y >= board_height as i32 {
                return false;
            }
            return true;
        });
        
        return all_moves.into_iter().map(|p| (p, 1)).collect();
    }

    pub fn distance(&self, other: &Coord) -> u32 {
        (self.x.abs_diff(other.x) + self.y.abs_diff(other.y)) as u32
    }
}
