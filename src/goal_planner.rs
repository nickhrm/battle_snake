use pathfinding::prelude::astar;

use crate::{battlesnake::Battlesnake, coord::Coord, danger::danger_matrix, Board};

pub fn goal_planner(food: Vec<Coord>, you: &Battlesnake, board: &Board) -> Vec<Coord> {
    let mut sorted_food = food.clone();
    sorted_food.sort_by_key(|a| a.distance(&you.head));


    let matrix = danger_matrix(board, you);

    let p = &you.head;

    let path_to_food: Option<(Vec<Coord>, u32)> = sorted_food.iter().find_map(|food| {
        astar(
            p,
            |p| p.successors(board, matrix),
            |p| p.distance(food),
            |p| p == food,
        )
    });

    match path_to_food {
        Some(path) => {
            let (mut coord_vec, _) = path;
            coord_vec.remove(0);
            coord_vec
        }
        None => {
            println!("Didnt find path to food. Choosing random");
            let path = you.head.successors(board, matrix);
            vec![path[0].0]
        }
    }
}
