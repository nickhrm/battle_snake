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

use log::info;
use rand::seq::SliceRandom;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{Battlesnake, Board, Coord, Game};

// info is called when you create your Battlesnake on play.battlesnake.com
// and controls your Battlesnake's appearance
// TIP: If you open your Battlesnake URL in a browser you should see this data
pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "", // TODO: Your Battlesnake Username
        "color": "#888888", // TODO: Choose color
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
    
    let mut is_move_safe: HashMap<_, _> = vec![
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]
    .into_iter()
    .collect();

    // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1]; // Coordinates of your "neck"
    
    if my_neck.x < my_head.x { // Neck is left of head, don't move left
        is_move_safe.insert("left", false);

    } else if my_neck.x > my_head.x { // Neck is right of head, don't move right
        is_move_safe.insert("right", false);

    } else if my_neck.y < my_head.y { // Neck is below head, don't move down
        is_move_safe.insert("down", false);
    
    } else if my_neck.y > my_head.y { // Neck is above head, don't move up
        is_move_safe.insert("up", false);
    }

    // TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
    let board_width = &_board.width;
    let board_height = &_board.height;

    //Snake is at left border of board
    if my_head.x <= 0 {
        is_move_safe.insert("left", false);
        info!("Snake is at left border of board")
    }

    //Snake is at right boarder of board
    if my_head.x >= (*board_width - 1) {
        is_move_safe.insert("right", false);
        info!("Snake is at right boarder of board")
    }

    //Snake is at top boarder of board
    if my_head.y >= (*board_height as i32 - 1) {
        is_move_safe.insert("up", false);
        info!("Snake is at top border of board")
    }

    //Snake is at bottom border of board
    if my_head.y <= 0 {
        is_move_safe.insert("down", false);
        info!("Snake is at bottom border of board")
    }

    // TODO: Step 2 - Prevent your Battlesnake from colliding with itself
    let my_body = &you.body;
    let moves_left =  &Coord {
        x: my_head.x - 1,
        y: my_head.y,
    };
    let moves_right = &Coord {
        x: my_head.x + 1,
        y: my_head.y,
    };
    let moves_down = &Coord {
        x: my_head.x,
        y: my_head.y - 1,
    };
    let moves_up = &Coord {
        x: my_head.x,
        y: my_head.y + 1,
    };

    if my_body.contains(moves_left) {
        is_move_safe.insert("left", false);
        info!("moving left would create a self-collision")
    }

    if my_body.contains(moves_right) {
        is_move_safe.insert("right", false);
        info!("moving right would create a self-collision")
    }

    if my_body.contains(moves_down) {
        is_move_safe.insert("down", false);
        info!("moving down would create a self-collision")
    }

    if my_body.contains(moves_up) {
        is_move_safe.insert("up", false);
        info!("moving up would create a self-collision")
    }

    // TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    // let opponents = _board.snakes;
    // for snake in opponents.iter_mut() {

    // }

    // Are there any safe moves left?
    let safe_moves = is_move_safe
        .into_iter()
        .filter(|&(_, v)| v)
        .map(|(k, _)| k)
        .collect::<Vec<_>>();
    
    info!("Moves left: {:#?} ", safe_moves);
    // Choose a random move from the safe ones
    let chosen = safe_moves.choose(&mut rand::thread_rng()).unwrap();

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    // let food = &board.food;

    info!("MOVE {}: {}", turn, chosen);
    return json!({ "move": chosen });
}
