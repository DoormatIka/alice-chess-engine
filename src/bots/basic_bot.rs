
use crate::tables::piece_sq_tables::{create_pesto_piece_sqaure, ColoredTables};
use crate::tables::zobrist::{ZobristHashMap, NodeInfo};
use crate::bots::bot_traits::ChessScoring;
use crate::types::pieces_colored::PiecesColored;
use crate::uci::uci::Uci;
use crate::{bots::bot_traits::Evaluation, moves::move_gen::generate_moves};

use chess::{Board, ChessMove, Color, Piece, ALL_SQUARES};
use std::cmp;


pub struct BasicBot {
    pub board: Board,
    pub uci: Uci,
    pesto: (ColoredTables, ColoredTables),
    killer_moves: Vec<Vec<Option<ChessMove>>>,
    history_table: [[i32; 64]; 64], 
    tt_table: ZobristHashMap<NodeInfo>,
}

impl BasicBot {
    pub fn new(board: &Board, tt_byte_size: usize) -> Self {
        Self {
            board: board.clone(),
            pesto: create_pesto_piece_sqaure(),
            uci: Uci::default(),
            killer_moves: vec![vec![None; 4]; 15], // Fix this later, Make dynamic setting of this based on depth
            history_table: [[0; 64]; 64],
            tt_table: ZobristHashMap::new(tt_byte_size),
        }
    }

    pub fn change_board(&mut self, board: &Board) {
        self.board = board.clone();
    }

    pub fn evaluate_material_advantage(&self, board: &Board) -> i32 {
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

    pub fn evaluate_piece_sq_table(&self, board: &Board) -> f32 {
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

    /**
     * * Calculates the pieces in the board.
     *
     * (white_mg_score, white_eg_score, black_mg_score, black_eg_score) is the return type.
     * im sorry for using tuples again.
     */
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

    fn piece_to_int(&self, p: Piece) -> u8 {
        match p {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
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

    pub fn evaluate_mates(
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

    pub fn internal_search(
        &mut self,
        board: &Board,
        max_depth: u16,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        is_maximizing_player: bool,
        _previous_move: Option<ChessMove>,
    ) -> (i32, Option<ChessMove>) {
        let max_depth_history = 10;
        let all_moves = generate_moves(&board);
        let mut killer_moves: Vec<ChessMove> = Vec::new();
        let mut regular_moves: Vec<ChessMove> = Vec::new();
        
        for board_move in all_moves {
            if self.killer_moves[depth as usize].contains(&Some(board_move)) {
                killer_moves.push(board_move);
            } else {
                regular_moves.push(board_move);
            }
        }

        regular_moves.sort_by_key(|chess_move: &ChessMove| {
            let source = chess_move.get_source().to_int() as usize;
            let dest = chess_move.get_dest().to_int() as usize;
            let history_score = if depth <= max_depth_history {
                -self.history_table[source][dest]
            } else {
                0
            };
            let mvv_lva_score = -self.mvv_lva_score(chess_move, &board).unwrap_or(0);
            (history_score, mvv_lva_score)
        });
        let sorted_moves = killer_moves
            .into_iter()
            .chain(regular_moves.into_iter())
            .collect::<Vec<ChessMove>>();


        if depth == 0 {
            let evaluation = self.evaluation(board, &sorted_moves, is_maximizing_player);
            return (evaluation, None);
        }

        let mut best_move = sorted_moves.get(0).map(|f| f.clone()); // Store the first move as the best move initially

        if is_maximizing_player {
            let mut best_val = -1000000;

            for board_move in sorted_moves.iter() {
                let board = board.make_move_new(*board_move);

                let node_info = if self.tt_table.contains(&board) {
                    self.tt_table.get(&board).map(|v| v.clone()).unwrap()
                } else {
                    let (eval, _) = self.internal_search(
                        &board,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                        !is_maximizing_player,
                        Some(*board_move),
                    );
                    NodeInfo { eval, best_move }
                };

                if node_info.eval > best_val {
                    best_val = node_info.eval;
                    best_move = Some(*board_move);

                    self.tt_table.insert(&board, NodeInfo { eval: node_info.eval, best_move });
                    self.uci.update_depth_data(depth, max_depth, best_move);
                }
                alpha = cmp::max(alpha, best_val);

                if beta <= alpha {
                    self.update_killer_move(depth, *board_move);
                    break;
                }
            }
            (best_val, best_move)
        } else {
            let mut best_val = 1000000;

            for board_move in sorted_moves.iter() {
                let board = board.make_move_new(*board_move);

                let node_info = if self.tt_table.contains(&board) {
                    self.tt_table.get(&board).map(|v| v.clone()).unwrap()
                } else {
                    let (eval, _) = self.internal_search(
                        &board,
                        max_depth,
                        depth - 1,
                        alpha,
                        beta,
                        !is_maximizing_player,
                        Some(*board_move),
                    );
                    NodeInfo { eval, best_move }
                };

                if node_info.eval < best_val {
                    best_val = node_info.eval;
                    best_move = Some(*board_move);

                    self.tt_table.insert(&board, NodeInfo { eval: node_info.eval, best_move });
                    self.uci.update_depth_data(depth, max_depth, best_move);
                }
                beta = cmp::min(beta, best_val);

                if beta <= alpha {
                    self.update_killer_move(depth, *board_move);
                    break;
                }
            }

            (best_val, best_move)
        }
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

    fn update_killer_move(&mut self, depth: u16, board_move: ChessMove) {
        self.killer_moves[depth as usize].rotate_right(1);
        self.killer_moves[depth as usize][0] = Some(board_move);

        let source = board_move.get_source().to_int() as usize;
        let dest = board_move.get_dest().to_int() as usize;
        self.history_table[source][dest] += 1;
    }
}
