use crate::{coord::Coord, Board};
use std::collections::VecDeque;

// Status-Definitionen
const UNVISITED: i8 = -1;
const CONTESTED: i8 = -2;
const WIDTH: usize = 11;
const HEIGHT: usize = 11;
const BOARD_SIZE: usize = 121;

impl Board {
    pub fn voronoi_you(&self) -> Coord {
        // 1. Schlangen vorbereiten (Eigene Schlange identifizieren)
        // Wir nehmen an, self.you ist der Index oder die ID deiner Schlange
        let my_id = self.you.id.clone();

        // 2. Board für Distanzen und Besitz initialisieren
        // Wir speichern den Index der Schlange, die das Feld kontrolliert
        let mut owner_board: [i8; BOARD_SIZE] = [UNVISITED; BOARD_SIZE];
        let mut distance_board: [u32; BOARD_SIZE] = [0; BOARD_SIZE];

        // Queue für BFS: (Koordinate, Schlangen_Index, Distanz)
        let mut queue: VecDeque<(Coord, usize, u32)> = VecDeque::new();

        // 3. Startpunkte: Alle Schlangenköpfe in die Queue
        for (idx, snake) in self.snakes.iter().enumerate() {
            let pos = snake.head;
            let board_idx = (pos.y * WIDTH as i32 + pos.x) as usize;
            owner_board[board_idx] = idx as i8;
            queue.push_back((pos, idx, 0));
        }

        // 4. BFS / Flood Fill
        while let Some((curr_pos, snake_idx, dist)) = queue.pop_front() {
            let next_dist = dist + 1;

            for neighbor in self.get_neighbors(curr_pos) {
                let n_idx = (neighbor.y * WIDTH as i32 + neighbor.x) as usize;

                if owner_board[n_idx] == UNVISITED {
                    // Feld ist noch frei
                    owner_board[n_idx] = snake_idx as i8;
                    distance_board[n_idx] = next_dist;
                    queue.push_back((neighbor, snake_idx, next_dist));
                } else if distance_board[n_idx] == next_dist
                    && owner_board[n_idx] != snake_idx as i8
                {
                    // Zwei Schlangen erreichen das Feld gleichzeitig
                    // Hier könnte man Länge prüfen, vereinfacht: CONTESTED
                    owner_board[n_idx] = CONTESTED;
                }
            }
        }

        // 5. Score pro möglichem Zug berechnen
        let possible_moves = self.get_neighbors(self.you.head);
        let mut best_move = possible_moves[0];
        let mut max_score = -1;

        for move_coord in possible_moves {
            let score = self.count_controlled_fields(&owner_board, move_coord, my_id_idx);
            if score > max_score {
                max_score = score;
                best_move = move_coord;
            }
        }

        best_move
    }

    // Hilfsfunktion: Zählt, wie viele Felder von einem Startpunkt aus erreichbar sind,
    // die uns im owner_board gehören.
    fn count_controlled_fields(&self, board: &[i8; BOARD_SIZE], start: Coord, my_idx: i8) -> i32 {
        // Hier zählen wir einfach alle Felder im board, die == my_idx sind
        board.iter().filter(|&&owner| owner == my_idx).count() as i32
    }

    fn get_neighbors(&self, p: Coord) -> Vec<Coord> {
        let mut neighbors = Vec::new();
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dx, dy) in dirs {
            let next = Coord {
                x: p.x + dx,
                y: p.y + dy,
            };
            if next.x >= 0 && next.x < WIDTH as i32 && next.y >= 0 && next.y < HEIGHT as i32 {
                // Hier zusätzlich prüfen, ob dort ein Schlangenkörper ist!
                if !self.is_occupied(next) {
                    neighbors.push(next);
                }
            }
        }
        neighbors
    }
}
