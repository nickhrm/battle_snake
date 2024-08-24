use serde::{Deserialize, Serialize};

use crate::{coord_utils::Coord, move_utils::Move};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Battlesnake {
    id: String,
    name: String,
    health: i32,
    pub body: Vec<Coord>,
    head: Coord,
    pub length: i32,
    latency: String,
    shout: Option<String>,
}

impl Battlesnake {
    //alle möglichen nächsten Head positionen berechnen
    fn next_heads(&self) -> Vec<Coord> {
        let head = self.head.clone();
        vec![
            head.get_next(Move::Left),
            head.get_next(Move::Right),
            head.get_next(Move::Up),
            head.get_next(Move::Down),
        ]
    }

    //Dor wo der Tail aktuell ist, wird zum nächst möglichen Zeitpunkt frei sein
    fn remove_tail(&self) -> Battlesnake {
        let mut new_snake: Battlesnake = self.clone();
        new_snake.body.pop();
        new_snake
    }

    pub fn next_rounds_snake(&self, you_length: i32) -> Battlesnake {
        let mut new_snake = self.clone();

        if self.length >= you_length {
            new_snake.body.append(&mut self.next_heads());
        }
        new_snake = self.remove_tail();
        new_snake
    }
}
