use std::convert::TryInto;

use crate::{battlesnake::Battlesnake, coord::Coord, r#move::Move, Board};

pub fn danger_matrix(board: &Board, you: &Battlesnake) -> Vec<(Coord, i32)> {
    let mut matrix = vec![];

    for x in 0..board.width {
        for y in 0..board.height {
            matrix.push((Coord { x, y: y.try_into().unwrap() }, 0));
        }
    }


    for snake in board.snakes.clone().into_iter() {
        if snake.id == you.id {
            continue;
        }
        let snakes_next_positions = vec![
            snake.head.get_next(&Move::Up),
            snake.head.get_next(&Move::Down),
            snake.head.get_next(&Move::Left),
            snake.head.get_next(&Move::Right),
        ];
        println!("Snakes next posititons: {:?}", snakes_next_positions);


        for pos in snakes_next_positions.into_iter() {
            for val in matrix.iter_mut() { 
                if pos == val.0 {
                    val.1 -= 100;
                    println!("Changing value");
                }
            }
        }
    }

    for apple in board.food.clone().into_iter(){
        for val in matrix.iter_mut() { 
            if apple == val.0 {
                val.1 += 10;
                println!("Changing value");
            }
        }
    } 




    println!("Printing danger matrix");
    for val in matrix.iter_mut() {
        if val.1 != 0 {
            println!("{:?}", val);
        }
    }


    matrix
}
