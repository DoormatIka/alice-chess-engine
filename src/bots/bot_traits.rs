use crate::bots::basic_bot::BasicBot;
use chess::{Board, ChessMove};

use std::time::Instant;

pub trait Search {
    fn search(&mut self, depth: u16) -> (i32, ChessMove);
}

impl Search for BasicBot {
    // external function, interacts with self
    fn search(&mut self, depth: u16) -> (i32, ChessMove) {
        let board = self.board.clone();
        let alpha = -999999; // Negative infinity
        let beta = 999999; // Positive infinity

        let start = Instant::now();
        let (best_eval, best_move) = self.internal_search(&board, depth, depth, alpha, beta, true);
        self.uci.set_ms_passed(start.elapsed().as_millis() as u64);

        (best_eval, best_move.unwrap())
    }
}

pub trait Evaluation {
    fn evaluation(&self, board: &Board) -> i32;
}

impl Evaluation for BasicBot {
    // internal function, doesn't interact with self
    fn evaluation(&self, board: &Board) -> i32 {
        // currently handles quiet moves like shit.
        // if there's no capture moves, then it'll be the first move in the movelist
        let material = self.evaluate_material_advantage(board);
        let position = self.evaluate_piece_sq_table(board);

        material + position as i32
    }
}
