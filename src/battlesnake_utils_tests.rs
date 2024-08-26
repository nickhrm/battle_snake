

#[cfg(test)]
mod tests {
    use crate::{battlesnake_utils::Battlesnake, coord_utils::Coord, move_utils::Move};

    use super::*;

    #[test]
    fn test_get_direction() {
        let moving_left = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 5,  y: 5 },
            body: vec![Coord{x:6, y: 5}],
            health:1,
            latency: "1".to_string(),
            length:1,
            name:"tester".to_string(),
            shout: None,
        };
        let moving_up = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 5,  y: 5 },
            body: vec![Coord{x:5, y: 4}],
            health:1,
            latency: "1".to_string(),
            length:1,
            name:"tester".to_string(),
            shout: None,
        };


        let direction_left = moving_left.get_direction();
        let direction_up = moving_up.get_direction();

        assert_eq!(Move::Left,direction_left);
        assert_eq!(Move::Up, direction_up);

    }


    #[test]
    fn test_get_reachable_apple() {
        let snake = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 5,  y: 5 },
            body: vec![Coord{x:6, y: 5}],
            health:1,
            latency: "1".to_string(),
            length:1,
            name:"tester".to_string(),
            shout: None,
        }; 
        let apples = vec![Coord {x: 5, y: 6}];
        let result = Some(Coord {x: 5, y: 6});

        assert_eq!(result, snake.get_reachable_apple(apples));

    }

    #[test]
    fn test_next_rounds_snake() {
        let last_round_snake = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 5,  y: 5 },
            body: vec![Coord{x:6, y: 5}],
            health:1,
            latency: "1".to_string(),
            length:20,
            name:"tester".to_string(),
            shout: None,
        }; 
        let can_reach_apples = vec![Coord {x: 5, y: 6}];
        let cannot_reach_apples = vec![Coord {x: 6, y: 6}];

        let next_rounds_snake_can_reach_apple = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 5,  y: 6 },
            body: vec![Coord { x: 5,  y: 5 }, Coord{x:6, y: 5}],
            health:1,
            latency: "1".to_string(),
            length:20,
            name:"tester".to_string(),
            shout: None,
        }; 

        let next_rounds_snake_cannot_reach_apple = Battlesnake {
            id: "1".to_string(),
            head: Coord { x: 4,  y: 5 },
            body: vec![Coord{x:5, y: 5}],
            health:1,
            latency: "1".to_string(),
            length:20,
            name:"tester".to_string(),
            shout: None,
        }; 

        assert_eq!(next_rounds_snake_can_reach_apple, last_round_snake.next_rounds_snake(1, can_reach_apples), "Snake can reach apple");
        assert_eq!(next_rounds_snake_cannot_reach_apple, last_round_snake.next_rounds_snake(1, cannot_reach_apples), "Snake cannot reach apple");

    }

}
