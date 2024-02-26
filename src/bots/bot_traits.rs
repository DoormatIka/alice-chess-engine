use crate::{bots::basic_bot::BasicBot, types::pieces_colored::PiecesColored};
use chess::{Board, ChessMove, Color, Piece, ALL_SQUARES};

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

        let (best_eval, best_move) =
            self.internal_search(&board, depth, depth, alpha, beta, true, None);
        self.uci.set_ms_passed(start.elapsed().as_millis() as u64);

        let best_move = match best_move {
            Some(best_move) => best_move,
            None => panic!("Something went wrong with searching the best move."),
        };

        (best_eval, best_move)
    }
}

pub trait Evaluation {
    fn evaluation(&self, board: &Board, moves: &Vec<ChessMove>, is_maximizing_player: bool) -> i32;
}

impl Evaluation for BasicBot {
    fn evaluation(&self, board: &Board, moves: &Vec<ChessMove>, is_maximizing_player: bool) -> i32 {
        // all of these functions subtract from black and white and vice versa
        // should we pass in the "maximizing_player" boolean instead of praying White will be the
        // maximizing player?
        let material = self.evaluate_material_advantage(board);
        let position = self.evaluate_piece_sq_table(board);
        let check = self.evaluate_mates(board, moves, is_maximizing_player);

        material + position as i32 + check
    }
}

pub trait BoardEvaluator {
    fn evaluate_material_advantage(&self, board: &Board) -> i32;
    fn evaluate_piece_sq_table(&self, board: &Board) -> f32;
    fn evaluate_mates(
        &self,
        board: &Board,
        moves: &Vec<ChessMove>,
        is_maximizing_player: bool,
    ) -> i32;
}

impl BoardEvaluator for BasicBot {
    fn evaluate_mates(
        &self,
        board: &Board,
        moves: &Vec<ChessMove>,
        is_maximizing_player: bool,
    ) -> i32 {
        let perspective = if is_maximizing_player { -1 } else { 1 };
        let check = if moves.len() == 0 {
            let checkers = board.checkers();
            if checkers.popcnt() >= 1 {
                // checkmate
                999999 * perspective
            } else {
                // stalemate
                555555 * perspective
            }
        } else {
            0
        };

        check
    }
    fn evaluate_material_advantage(&self, board: &Board) -> i32 {
        let white = PiecesColored::get_colored_pieces(&board, Color::White);
        let black = PiecesColored::get_colored_pieces(&board, Color::Black);

        let material_white = self.calculate_material(white) as i32;
        let material_black = self.calculate_material(black) as i32;

        let eval = if board.side_to_move() == Color::White {
            material_white - material_black
        } else {
            material_black - material_white
        };
        return eval;
    }

    fn evaluate_piece_sq_table(&self, board: &Board) -> f32 {
        let (white_mg_score, white_eg_score, black_mg_score, black_eg_score) =
            self.calculate_piece_sq_with_board(board);
        let (mg_score, eg_score) = if board.side_to_move() == Color::White {
            (
                white_mg_score - black_mg_score,
                white_eg_score - black_eg_score,
            )
        } else {
            (
                black_mg_score - white_mg_score,
                black_eg_score - white_eg_score,
            )
        };

        let score = self.calculate_score(board, mg_score, eg_score);

        score
    }
}

pub trait ChessScoring {
    fn mvv_lva_score(&self, chess_move: &ChessMove, board: &Board) -> Option<i32>;
    fn piece_value(&self, piece: Piece) -> i32;
}

impl ChessScoring for BasicBot {
    fn mvv_lva_score(&self, chess_move: &ChessMove, board: &Board) -> Option<i32> {
        let victim_piece = board.piece_on(chess_move.get_dest());
        let aggressor_piece = board.piece_on(chess_move.get_source());

        match (victim_piece, aggressor_piece) {
            (Some(victim), Some(aggressor)) => {
                let victim_value = self.piece_value(victim);
                let aggressor_value = self.piece_value(aggressor);
                Some(victim_value - aggressor_value)
            }
            _ => None,
        }
    }

    fn piece_value(&self, piece: Piece) -> i32 {
        match piece {
            Piece::Pawn => 1,
            Piece::Knight | Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            Piece::King => std::i32::MAX,
        }
    }
}

pub trait ScoreCalculator {
    fn calculate_score(&self, board: &Board, mg_score: i32, eg_score: i32) -> f32;
    fn calculate_material(&self, pieces: PiecesColored) -> u32;
    fn calculate_piece_sq_with_board(&self, board: &Board) -> (i32, i32, i32, i32);
}

impl ScoreCalculator for BasicBot {
    fn calculate_piece_sq_with_board(&self, board: &Board) -> (i32, i32, i32, i32) {
        let mut white_mg = 0;
        let mut black_mg = 0;
        let mut white_eg = 0;
        let mut black_eg = 0;
        let (mg_pesto, eg_pesto) = &self.pesto;

        for sq in ALL_SQUARES {
            let piece = board.piece_on(sq);
            let color = board.color_on(sq);

            if let Some(piece) = piece {
                let piece = self.piece_to_int(piece);
                if let Some(color) = color {
                    match color {
                        Color::White => {
                            white_mg += mg_pesto.white[piece as usize][sq.to_int() as usize];
                            white_eg += eg_pesto.white[piece as usize][sq.to_int() as usize];
                        }
                        Color::Black => {
                            black_mg += mg_pesto.black[piece as usize][sq.to_int() as usize];
                            black_eg += eg_pesto.black[piece as usize][sq.to_int() as usize];
                        }
                    }
                }
            }
        }

        (white_mg, white_eg, black_mg, black_eg)
    }

    fn calculate_material(&self, pieces: PiecesColored) -> u32 {
        let mut material = 0;

        material += pieces.pawns.popcnt() * 100;
        material += pieces.knights.popcnt() * 300;
        material += pieces.bishops.popcnt() * 300;
        material += pieces.rooks.popcnt() * 500;
        material += pieces.queens.popcnt() * 900;

        material
    }

    fn calculate_score(&self, board: &Board, mg_score: i32, eg_score: i32) -> f32 {
        let white_pieces = PiecesColored::get_colored_pieces(board, Color::White);
        let black_pieces = PiecesColored::get_colored_pieces(board, Color::Black);
        let white_material = self.calculate_material(white_pieces);
        let black_material = self.calculate_material(black_pieces);

        let total_material = white_material + black_material;

        let max_material = 7800.0;

        let game_phase = total_material as f32 / max_material;

        let mg_phase = game_phase;
        let eg_phase = 1.0 - game_phase;

        let weighted_mg_score = mg_phase * mg_score as f32;
        let weighted_eg_score = eg_phase * eg_score as f32;

        weighted_mg_score + weighted_eg_score
    }
}
