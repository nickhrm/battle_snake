use crate::{battlesnake_utils::Battlesnake, coord_utils::Coord, Board, GameState};

pub fn print_board(board: &Board, you: &Battlesnake, path: &Vec<Coord>) {
    let mut board_vec = vec![vec!['.'; board.width as usize]; board.height as usize];

    // Place food on the board
    for food in &board.food {
        board_vec[food.y as usize][food.x as usize] = 'F';
    }

    // Place hazards on the board
    for hazard in &board.hazards {
        board_vec[hazard.y as usize][hazard.x as usize] = 'H';
    }

    // Place snakes on the board
    for snake in &board.snakes {
        board_vec[snake.head.y as usize][snake.head.x as usize] = 'S';
        for (i, coord) in snake.body.iter().enumerate() {
            board_vec[coord.y as usize][coord.x as usize] = 's';
        }
    }

    //print myself
    board_vec[you.head.y as usize][you.head.x as usize] = 'Y';
    for (i, coord) in you.body.iter().enumerate() {
        board_vec[coord.y as usize][coord.x as usize] = 'y';
    }

    for (i, pos) in path.iter().enumerate() {
        let symb = if i == path.len() - 1 { 'P' } else { 'p' };
        board_vec[pos.y as usize][pos.x as usize] = symb;
    }

    // Print the board
    for row in board_vec.iter().rev() {
        // Reverse to print from top to bottom
        for cell in row {
            print!("{}", cell);
        }
        println!();
    }
}
