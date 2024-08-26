// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use crate::{
    coord_utils::Coord, local_planner::local_planner, move_utils::Move, print_util::print_board,
};
use log::info;
use pathfinding::prelude::astar;
use serde_json::{json, Value};

use crate::{Battlesnake, Board, Game};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": "#ff5f3b", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {
    println!("w: {},h: {}", _board.width, _board.height);

    let p: &Coord = &you.body[0]; // Coordinates of your head
    let goal: &Coord = &_board.food[0];

    println!("Goal: {:?}, Current Pos: {:?}", goal, p);
    let path: Option<(Vec<Coord>, u32)> = _board.food.iter().find_map(|food| {
        astar(
            p,
            |p| p.successors(_board, you),
            |p| p.distance(food),
            |p| p == food,
        )
    });

    match path {
        Some(res) => {
            let (mut coord_vec, _) = res;
            coord_vec.remove(0);

            let next_move = local_planner(p, &coord_vec[0]);

            println!("{:?}", you);
            print_board(_board, you, &coord_vec);

            info!("MOVE {}: {}", turn, next_move);
            return json!({ "move": next_move });
        }

        None => {
            info!("MOVE {}: No move found. Moving down as default", turn);
            return json!({ "move": Move::Left });
        }
    }
}
