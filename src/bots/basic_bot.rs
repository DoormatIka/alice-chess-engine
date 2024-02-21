use crate::piece_sq_tables::{create_pesto_piece_sqaure, ColoredTables};
use crate::types::pieces_colored::PiecesColored;
use crate::uci::uci::Uci;
use crate::{bots::bot_traits::Evaluation, moves::move_gen::generate_moves};

use chess::{Board, ChessMove, Color, Piece, ALL_SQUARES};
use serde_json::to_string;
use std::cmp;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct BasicBot {
    pub board: Board,
    pub uci: Uci,
    pesto: (ColoredTables, ColoredTables),
    node_connections: HashMap<String, Vec<String>>, // [fen + moves]
    node_information: HashMap<String, NodeInfo>
}

struct NodeInfo {
    evaluation: i32,
    best_move: Option<ChessMove>,
    alpha: i32,
    beta: i32, 
    is_maximizing_player: bool,
}

impl BasicBot {
    pub fn new(board: &Board) -> Self {
        Self {
            board: board.clone(),
            pesto: create_pesto_piece_sqaure(),
            uci: Uci::default(),
            node_connections: HashMap::new(),
            node_information: HashMap::new(),
        }
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

        let white_pieces = PiecesColored::get_colored_pieces(board, Color::White); // works
        let black_pieces = PiecesColored::get_colored_pieces(board, Color::Black); // works
        let white_material = self.calculate_material(white_pieces); // works
        let black_material = self.calculate_material(black_pieces); // works

        let total_material = white_material + black_material;

        let max_material = 7800.0; // Maximum possible material at the start of the game
        let game_phase = total_material as f32 / max_material;

        let mg_phase = game_phase;
        let eg_phase = game_phase - 1.0;

        let weighted_mg_score = mg_phase * mg_score as f32;
        let weighted_eg_score = eg_phase * eg_score as f32;

        let score = weighted_mg_score + weighted_eg_score;

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
        material += pieces.knights.popcnt() * 400;
        material += pieces.bishops.popcnt() * 300;
        material += pieces.rooks.popcnt() * 500;
        material += pieces.queens.popcnt() * 900;

        material
    }

    pub fn get_debug_tree(&self) -> &HashMap<String, Vec<String>> {
        &self.node_connections
    }
    pub fn write_debug_tree_to_file(&self) -> std::io::Result<()> {
        let mut connections = std::fs::File::create("debug_tree_connections.json")?;
        let mut information = std::fs::File::create("debug_tree_information.json")?;

        let node_ids_string = to_string(&self.node_connections).unwrap();
        let node_ids_string = format!("{}\n", node_ids_string);

        // needs to implement serde::Serialize...
        let info_ids_string = to_string(&self.node_information).unwrap();
        let info_ids_string = format!("{}\n", info_ids_string);
    
        connections.write_all(node_ids_string.as_bytes())?;

        Ok(())
    }

    pub fn internal_search(
        &mut self,
        board: &Board,
        max_depth: u16,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        is_maximizing_player: bool,
        previous_move: Option<ChessMove>,
    ) -> (i32, Option<ChessMove>) {
        let all_moves = generate_moves(&board);

        if depth == 0 {
            let evaluation = self.evaluation(board, &all_moves);
            return (evaluation, None);
        }

        let mut best_move = all_moves.get(0).map(|f| f.clone()); // Store the first move as the best move initially
        if is_maximizing_player {
            let mut best_val = -1000000;

            for board_move in all_moves.iter() {

                let previous_board = board.clone();
                let board = board.make_move_new(*board_move);

                // Note: this has an extremely hard performance impact. this is only enabled for
                // debugging purposes.
                self.push_node(&previous_board, previous_move, &board, board_move, depth);

                let (eval, best) = self.internal_search(
                    &board,
                    max_depth,
                    depth - 1,
                    alpha,
                    beta,
                    !is_maximizing_player,
                    Some(*board_move),
                );
                
                let node_info = NodeInfo { 
                    evaluation: eval, 
                    best_move: best, 
                    alpha, 
                    beta, 
                    is_maximizing_player,
                };
                self.node_information.insert(format!(
                    "{}-{}-{}",
                    board.to_string().replace("/", "#"),
                    board_move.to_string(),
                    cmp::max(depth - 1, 0)
                ), node_info);

                if eval > best_val {
                    best_val = eval;
                    best_move = Some(*board_move);
                    self.uci.update_depth_data(depth, max_depth, best_move);
                }
                alpha = cmp::max(alpha, best_val);

                if beta <= alpha {
                    break;
                }
            }
            (best_val, best_move)
        } else {
            let mut best_val = 1000000;

            for board_move in all_moves.iter() {
                let previous_board = board.clone();
                let board = board.make_move_new(*board_move);

                self.push_node(&previous_board, previous_move, &board, board_move, depth);

                let (eval, _) = self.internal_search(
                    &board,
                    max_depth,
                    depth - 1,
                    alpha,
                    beta,
                    !is_maximizing_player,
                    Some(*board_move),
                );

                if eval < best_val {
                    best_val = eval;
                    best_move = Some(*board_move);
                    self.uci.update_depth_data(depth, max_depth, best_move);
                }
                beta = cmp::min(beta, best_val);

                if beta <= alpha {
                    break;
                }
            }

            (best_val, best_move)
        }
    }
    
    fn push_node(
        &mut self,
        previous_board: &Board,
        previous_move: Option<ChessMove>,
        board: &Board,
        board_move: &ChessMove,
        depth: u16,
    ) {
        if let Some(previous_move) = previous_move {
            let node_id = self.node_connections.get_mut(&format!(
                "{}-{}-{}",
                previous_board.to_string().replace("/", "#"),
                previous_move.to_string(),
                depth
            ));
            match node_id {
                Some(node_id) => node_id.push(format!(
                    "{}-{}-{}",
                    board.to_string().replace("/", "#"),
                    board_move.to_string(),
                    cmp::max(depth - 1, 0)
                )),
                None => {
                    self.node_connections.insert(
                        format!(
                            "{}-{}-{}",
                            previous_board.to_string().replace("/", "#"),
                            previous_move.to_string(),
                            depth
                        ),
                        vec![format!(
                            "{}-{}-{}",
                            board.to_string().replace("/", "#"),
                            board_move.to_string(),
                            cmp::max(depth - 1, 0)
                        )],
                    );
                }
            };
        } else {
            // it won't have a previous move if it's the very first node in the tree.
            let node_id = self.node_connections.get_mut(&format!("top"));
            match node_id {
                Some(node_id) => node_id.push(format!(
                    "{}-{}-{}",
                    board.to_string().replace("/", "#"),
                    board_move.to_string(),
                    cmp::max(depth - 1, 0)
                )),
                None => {self.node_connections.insert(
                    format!("top"),
                    vec![format!(
                        "{}-{}-{}",
                        board.to_string().replace("/", "#"),
                        board_move.to_string(),
                        cmp::max(depth - 1, 0)
                    )],
                );},
            };
        }
    }
}
