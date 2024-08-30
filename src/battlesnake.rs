use serde::{Deserialize, Serialize};

use crate::{coord::Coord, r#move::Move};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: i32,
    pub latency: String,
    pub shout: Option<String>,
}

impl Battlesnake {
    pub fn get_direction(&self) -> Move {
        let neck = &self.body[1];

        let moves = vec![Move::Left, Move::Right, Move::Up, Move::Down];

        for mov in &moves {
            if neck.get_next(mov) == self.head {
                return mov.clone();
            }
        }
        Move::Down
    }

    pub fn get_reachable_apple(&self, food: Vec<Coord>) -> Option<Coord> {
        let all_next_heads = vec![
            self.head.get_next(&Move::Left),
            self.head.get_next(&Move::Right),
            self.head.get_next(&Move::Up),
            self.head.get_next(&Move::Down),
        ];

        all_next_heads.into_iter().find(|&next_head| food.contains(&next_head))
    }

    //Dor wo der Tail aktuell ist, wird zum nächst möglichen Zeitpunkt frei sein
    pub fn remove_tail(&self) -> Battlesnake {
        let mut new_snake: Battlesnake = self.clone();
        new_snake.body.pop();
        new_snake
    }

    
}
