use crate::{coord_utils::Coord, move_utils::Move};

pub fn local_planner(current_pos: &Coord, next_pos: &Coord) -> Move {
    if next_pos.x > current_pos.x {
        return Move::Right;
    } else if next_pos.x < current_pos.x {
        return Move::Left;
    } else if next_pos.y > current_pos.y {
        return Move::Up;
    } else {
        return Move::Down;
    }
}
