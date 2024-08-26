use pathfinding::prelude::astar;

use crate::{battlesnake_utils::Battlesnake, coord_utils::Coord, Board};




fn goal_planner(food:Vec<Coord>, you: &Battlesnake, _board: &Board) -> Coord {
    let mut sorted_food = food.clone();
    sorted_food.sort_by(|a,b|a.distance(&you.head).cmp(&b.distance(&you.head)) );

    let p = &you.head;

    let path_to_food: Option<(Vec<Coord>, u32)> = sorted_food.iter().find_map(|food| {
        astar(
            p,
            |p| p.successors(_board, you),
            |p| p.distance(food),
            |p| p == food,
        )
    });

    match path_to_food {
        Some(path) => {
            let (mut coord_vec, _) = path;
            coord_vec.remove(0);
            coord_vec[0]
        },  
        None => {
            let path = you.head.successors(_board, you);
            path[0].0
        }
    }


    

}