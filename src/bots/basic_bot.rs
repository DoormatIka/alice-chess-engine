use std::cmp;

use crate::tables::piece_sq_tables::{create_pesto_piece_sqaure, ColoredTables};
use crate::tables::zobrist::{NodeInfo, ZobristHashMap};
use crate::uci::uci::Uci;
use crate::{bots::bot_traits::Evaluation, moves::move_gen::generate_moves};

use chess::{Board, ChessMove, Piece};

pub struct BasicBot {
    pub board: Board,
    pub uci: Uci,
    pub pesto: (ColoredTables, ColoredTables),
    pub killer_moves: Vec<Vec<Option<ChessMove>>>,
    tt_table: ZobristHashMap<NodeInfo>,
}

impl BasicBot {
    pub fn new(board: &Board, tt_byte_size: usize) -> Self {
        Self {
            board: board.clone(),
            pesto: create_pesto_piece_sqaure(),
            uci: Uci::default(),
            killer_moves: vec![vec![None; 4]; 15],
            tt_table: ZobristHashMap::new(tt_byte_size),
        }
    }

    pub fn change_board(&mut self, board: &Board) {
        self.board = board.clone();
    }

    pub fn reset(&mut self) {
        for i in 0..self.killer_moves.len() {
            for j in 0..self.killer_moves[i].len() {
                self.killer_moves[i][j] = None;
            }
        }
        self.uci.depth_data.clear();
        self.uci.nodes_total = 0;
        self.uci.ms_passed = 0;
    }

    /**
     * * Calculates the pieces in the board.
     *
     * (white_mg_score, white_eg_score, black_mg_score, black_eg_score) is the return type.
     * im sorry for using tuples again.
     */

    pub fn piece_to_int(&self, p: Piece) -> u8 {
        match p {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
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
        let all_moves = generate_moves(&board);

        let mut killer_moves: Vec<ChessMove> = Vec::new();
        let mut regular_moves: Vec<ChessMove> = Vec::new();

        for board_move in all_moves {
            if let Some(stored_killer_moves) = self.killer_moves.get(depth as usize) {
                if stored_killer_moves.contains(&Some(board_move)) {
                    killer_moves.push(board_move);
                } else {
                    regular_moves.push(board_move);
                }
            }
        }

        let sorted_moves = killer_moves
            .into_iter()
            .chain(regular_moves.into_iter())
            .collect();

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
                    let node_info = self.tt_table.get(&board).map(|v| v.clone()).unwrap();
                    // im assuming the deeper it is, the lower the depth is
                    if node_info.depth < depth {
                        let (eval, _) = self.internal_search(
                            &board,
                            max_depth,
                            depth - 1,
                            alpha,
                            beta,
                            !is_maximizing_player,
                            Some(*board_move),
                        );
                        NodeInfo { eval, best_move, depth }
                    } else {
                        node_info
                    }
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
                    NodeInfo { eval, best_move, depth }
                };

                if node_info.eval > best_val {
                    best_val = node_info.eval;
                    best_move = Some(*board_move);

                    self.tt_table.insert(
                        &board,
                        NodeInfo {
                            eval: node_info.eval,
                            best_move,
                            depth
                        },
                    );
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
                    let node_info = self.tt_table.get(&board).map(|v| v.clone()).unwrap();
                    // im assuming the deeper it is, the lower the depth is
                    if node_info.depth < depth {
                        let (eval, _) = self.internal_search(
                            &board,
                            max_depth,
                            depth - 1,
                            alpha,
                            beta,
                            !is_maximizing_player,
                            Some(*board_move),
                        );
                        NodeInfo { eval, best_move, depth }
                    } else {
                        node_info
                    }
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
                    NodeInfo { eval, best_move, depth }
                };

                if node_info.eval < best_val {
                    best_val = node_info.eval;
                    best_move = Some(*board_move);

                    self.tt_table.insert(
                        &board,
                        NodeInfo {
                            eval: node_info.eval,
                            best_move,
                            depth
                        },
                    );
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

    fn update_killer_move(&mut self, depth: u16, board_move: ChessMove) {
        self.killer_moves[depth as usize].rotate_right(1);
        self.killer_moves[depth as usize][0] = Some(board_move);
    }
}
