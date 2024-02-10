use crate::{bots::bot_traits::Evaluation, moves::move_gen::generate_moves};
use crate::types::pieces_colored::PiecesColored;
use chess::{Board, ChessMove, Color};
use std::cmp;

pub struct BasicBot {
    pub board: Board,
}
impl BasicBot {
    pub fn new(board: &Board) -> Self {
        Self {
            board: board.clone(),
        }
    }

    pub fn count_material(&self, board: &Board) -> i32 {
        let white = PiecesColored::get_colored_pieces(&board, Color::White);
        let black = PiecesColored::get_colored_pieces(&board, Color::Black);

        let material_white = self.calculate_material(white) as i32;
        let material_black = self.calculate_material(black) as i32;

        let eval = material_white - material_black;
        let perspective = if board.side_to_move() == Color::White {
            1
        } else {
            -1
        };

        return eval * perspective;
    }

    pub fn calculate_material(&self, pieces: PiecesColored) -> u32 {
        let mut material = 0;

        material += pieces.pawns.popcnt() * 100;
        material += pieces.knights.popcnt() * 400;
        material += pieces.bishops.popcnt() * 300;
        material += pieces.rooks.popcnt() * 500;
        material += pieces.queens.popcnt() * 900;

        material
    }

    pub fn internal_search(
        &mut self,
        board: &Board,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        is_maximizing_player: bool,
    ) -> (i32, Option<ChessMove>) {
        // println!("Depth: {}, Alpha: {}, Beta: {}, Is Maximizing Player: {}", depth, alpha, beta, is_maximizing_player);

        if depth == 0 {
            let evaluation = self.evaluation(board);
            // println!("Leaf node, evaluation: {}", evaluation);
            return (evaluation, None);
        }

        let (mut capture_moves, mut non_capture_moves) = generate_moves(&board);
        let mut all_moves: Vec<ChessMove> = vec![];
        all_moves.append(&mut capture_moves);
        all_moves.append(&mut non_capture_moves);

        // println!("Generated moves: {:?}", all_moves);

        if all_moves.len() == 0 {
            if board.checkers().popcnt() != 0 {
                // println!("No moves and in check, Checkmate, returning -1000000");
                return (-1000000, None);
            }
            // println!("No moves, Stalemate, returning 0");
            return (0, None);
        }

        let mut best_move = Some(all_moves[0]); // Store the first move as the best move initially

        if is_maximizing_player {
            let mut best_val = -1000000;
            // println!("Maximizing player, initial best value: {}", best_val);

            for board_move in all_moves.iter() {
                let board = board.make_move_new(*board_move);
                let (eval, _) =
                    self.internal_search(&board, depth - 1, alpha, beta, !is_maximizing_player);

                if eval > beta {
                    // assuming the opponent would never let the player reach this position
                    //      i.e: "failing high"
                    return (beta, best_move);
                }

                if eval > best_val {
                    best_val = eval;
                    best_move = Some(*board_move);
                }
                alpha = cmp::max(alpha, best_val);

                // println!("Move: {:?}, Value: {}, Best Value: {}, Alpha: {}", board_move, value, best_val, alpha);

                if beta <= alpha {
                    // println!("Alpha >= Beta, pruning");
                    break;
                }
            }
            (best_val, best_move)
        } else {
            let mut best_val = 1000000;
            // println!("Minimizing player, initial best value: {}", best_val);

            for board_move in all_moves.iter() {
                let board = board.make_move_new(*board_move);
                let (eval, _) =
                    self.internal_search(&board, depth - 1, alpha, beta, !is_maximizing_player);

                if eval < alpha {
                    // assuming the opponent would never let the player reach this position
                    //      i.e: "failing high"
                    //
                    // same as the maximizing player but just in reverse.
                    return (alpha, best_move);
                }

                if eval < best_val {
                    best_val = eval;
                    best_move = Some(*board_move);
                }
                beta = cmp::min(beta, best_val);

                // println!("Move: {:?}, Value: {}, Best Value: {}, Beta: {}", board_move, value, best_val, beta);

                if beta <= alpha {
                    // println!("Beta <= Alpha, pruning");
                    break;
                }
            }
            (best_val, best_move)
        }
    }
}
