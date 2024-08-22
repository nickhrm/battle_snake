   // We've included code to prevent your Battlesnake from moving backwards
    let my_head = &you.body[0]; // Coordinates of your head
    let my_neck = &you.body[1]; // Coordinates of your "neck"

    if my_neck.x < my_head.x {
        // Neck is left of head, don't move left
        safe_moves.remove_move(Moves::Left);
    } else if my_neck.x > my_head.x {
        // Neck is right of head, don't move right
        safe_moves.remove_move(Moves::Right);
    } else if my_neck.y < my_head.y {
        // Neck is below head, don't move down
        safe_moves.remove_move(Moves::Down);
    } else if my_neck.y > my_head.y {
        // Neck is above head, don't move up
        safe_moves.remove_move(Moves::Up);
    }

    // TODO: Step 1 - Prevent your Battlesnake from moving out of bounds
    let board_width = &_board.width;
    let board_height = &_board.height;

    //Snake is at left border of board
    if my_head.x <= 0 {
        safe_moves.remove_move(Moves::Left);
        info!("Snake is at left border of board")
    }

    //Snake is at right boarder of board
    if my_head.x >= (*board_width - 1) {
        safe_moves.remove_move(Moves::Right);
        info!("Snake is at right boarder of board")
    }

    //Snake is at top boarder of board
    if my_head.y >= (*board_height as i32 - 1) {
        safe_moves.remove_move(Moves::Up);
        info!("Snake is at top border of board")
    }

    //Snake is at bottom border of board
    if my_head.y <= 0 {
        safe_moves.remove_move(Moves::Down);
        info!("Snake is at bottom border of board")
    }

    // TODO: Step 2 - Prevent your Battlesnake from colliding with itself
    let my_body = &you.body;
    let moves_left = &Coord {
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
        safe_moves.remove_move(Moves::Left);
        info!("moving left would create a self-collision")
    }

    if my_body.contains(moves_right) {
        safe_moves.remove_move(Moves::Right);
        info!("moving right would create a self-collision")
    }

    if my_body.contains(moves_down) {
        safe_moves.remove_move(Moves::Down);
        info!("moving down would create a self-collision")
    }

    if my_body.contains(moves_up) {
        safe_moves.remove_move(Moves::Up);
        info!("moving up would create a self-collision")
    }

    // TODO: Step 3 - Prevent your Battlesnake from colliding with other Battlesnakes
    // let opponents = _board.snakes;
    // for snake in opponents.iter_mut() {

    // }

    // Are there any safe moves left?
