use serde::{Deserialize, Serialize};

use crate::{move_utils::Move, Battlesnake, Board};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Hash, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn get_next(&self, mov: &Move) -> Coord {
        match mov {
            Move::Left => Coord {
                x: self.x - 1,
                y: self.y,
            },
            Move::Right => Coord {
                x: self.x + 1,
                y: self.y,
            },
            Move::Up => Coord {
                x: self.x,
                y: self.y + 1,
            },
            Move::Down => Coord {
                x: self.x,
                y: self.y - 1,
            },
        }
    }

    pub fn successors(&self, board: &Board, you: &Battlesnake) -> Vec<(Coord, u32)> {
        let mut all_moves = vec![
            self.get_next(&Move::Left),
            self.get_next(&Move::Right),
            self.get_next(&Move::Up),
            self.get_next(&Move::Down),
        ];

        //prevent collision with its own body
        let real_my_body = if you.get_reachable_apple(board.food.clone()).is_some() {
            you.clone()
        } else {
            you.remove_tail()
        };
        

        all_moves
            .retain(|future_coord| !real_my_body.body.iter().any(|coord| future_coord == coord));

        //calculate the real next snakes
        let real_snakes: Vec<Battlesnake> = board
            .snakes
            .iter()
            .map(|snake| snake.next_rounds_snake(you.length, board.food.clone()))
            .collect();

        //prevent collision with other snakes
        all_moves.retain(|future_coord| {
            !real_snakes
                .iter()
                .any(|snake| snake.body.iter().any(|coord| future_coord == coord))
        });

        //prevent going out of bounds
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
