use crate::bots::basic_bot::BasicBot;
use chess::{Board, ChessMove, Color};

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

        let (best_eval, best_move) = self.internal_search(&board, depth, depth, alpha, beta, true, None);
        self.uci.set_ms_passed(start.elapsed().as_millis() as u64);

        (best_eval, best_move.unwrap())
    }
}

pub trait Evaluation {
    fn evaluation(&self, board: &Board, moves: &Vec<ChessMove>) -> i32;
}

impl Evaluation for BasicBot {
    fn evaluation(&self, board: &Board, moves: &Vec<ChessMove>) -> i32 {
        // all of these functions subtract from black and white and vice versa
        // should we pass in the "maximizing_player" boolean instead of praying White will be the
        // maximizing player?
        let material = self.evaluate_material_advantage(board);
        let position = self.evaluate_piece_sq_table(board);
        let check = if moves.len() == 0 {
            let checkers = board.checkers();
            if checkers.popcnt() >= 1 {
                // let sq = checkers.to_square();
                1000
            } else { 0 }
        } else { 0 };
        material + position as i32 + check
    }
}
