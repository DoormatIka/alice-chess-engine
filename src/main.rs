use std::{cmp, io::stdin, str::FromStr, vec};

use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, Game, MoveGen, Piece, EMPTY};
use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;

pub mod fen;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let almost_mate = "6k1/1p3ppp/8/8/8/3q1bP1/5K1P/8 b - - 0 1    ";

    let mut game = Game::from_str(almost_mate).unwrap();
    let player_color: Option<Color> = Some(Color::Black);

    loop {
        let board = game.current_position();

        if game.can_declare_draw() {
            println!("Stalemated by three-fold repetition.");
            break;
        }

        match board.status() {
            BoardStatus::Stalemate | BoardStatus::Checkmate => {
                println!("Stalemated / Checkmated.");
                break;
            }
            _ => (),
        }

        let (mut capture_moves, mut non_capture_moves) = generate_moves(&board);

        print_board_from_fen(
            game.current_position().to_string().as_str(),
            &capture_moves,
            &non_capture_moves,
        );

        let mut all_move: Vec<ChessMove> = vec![];
        all_move.append(&mut capture_moves);
        all_move.append(&mut non_capture_moves);

        if let Some(player_color) = player_color {
            if game.side_to_move() == player_color {
                for chess_move in all_move {
                    print!("{} ", chess_move);
                }
                let chess_move = get_user_move(&board);
                match chess_move {
                    Ok(chess_move) => game.make_move(chess_move),
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
            } else {
                let mut bot = BasicBot::new(&board);
                let (eval, chess_move) = bot.search(3);
                game.make_move(chess_move);
                println!("Made move with eval {}, {}", eval, chess_move);
            }
        } else {
            let mut bot = BasicBot::new(&board);
            let (eval, chess_move) = bot.search(3);
            game.make_move(chess_move);
            println!("Made move with eval {}, {}", eval, chess_move);
        }

        let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
        println!("The max memory that was used: {}kb", peak_mem);
    }
}

struct BasicBot {
    board: Board,
    best_move: Option<ChessMove>,
    best_eval: i32,
}
trait Search {
    fn search(&mut self, depth: u16) -> (i32, ChessMove);
}
trait Evaluation {
    fn evaluation(&self, board: &Board) -> i32;
}

impl Search for BasicBot {
    // external function, interacts with self
    fn search(&mut self, depth: u16) -> (i32, ChessMove) {
        let board = self.board.clone();
        let alpha = -999999; // Negative infinity
        let beta = 999999; // Positive infinity

        let best_eval = self.internal_search(&board, depth, alpha, beta, true);
        (best_eval, self.best_move.unwrap())
    }
}
impl Evaluation for BasicBot {
    // internal function, doesn't interact with self
    fn evaluation(&self, board: &Board) -> i32 {
        // currently handles quiet moves like shit.
        // if there's no capture moves, then it'll be the first move in the movelist
        self.count_material(board)
    }
}
impl BasicBot {
    fn new(board: &Board) -> Self {
        Self {
            board: board.clone(),
            best_move: None,
            best_eval: -9999999,
        }
    }

    pub fn count_material(&self, board: &Board) -> i32 {
        let white = get_colored_pieces(&board, Color::White);
        let black = get_colored_pieces(&board, Color::Black);

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

    fn calculate_material(&self, pieces: PiecesColored) -> u32 {
        let mut material = 0;

        material += pieces.pawns.popcnt() * 100;
        material += pieces.knights.popcnt() * 400;
        material += pieces.bishops.popcnt() * 300;
        material += pieces.rooks.popcnt() * 500;
        material += pieces.queens.popcnt() * 900;

        material
    }

    fn internal_search(
        &mut self,
        board: &Board,
        depth: u16,
        mut alpha: i32,
        mut beta: i32,
        is_maximizing_player: bool,
    ) -> i32 {
        // If leaf node, We return value of node.
        if depth == 0 {
            return self.evaluation(board);
        }

        // She's generating moves here.
        let (mut capture_moves, mut non_capture_moves) = generate_moves(&board);
        let mut all_moves: Vec<ChessMove> = vec![];
        all_moves.append(&mut capture_moves);
        all_moves.append(&mut non_capture_moves);

        if all_moves.len() == 0 {
            if board.checkers().popcnt() != 0 {
                return -1000000;
            }
            return 0;
        }
        // End of generating moves.

        // If maximizing player, it will try to maximize the evaluation score.
        if is_maximizing_player {
            // Initialize best_val to negative infinity.
            let mut best_val = -1000000;

            // Iterate over each possible move.
            for board_move in all_moves.iter() {
                // Create a new board with the current move applied.
                let board = board.make_move_new(*board_move);
                // Recursively call the internal_search function, reducing the depth by 1 each time and switching the player.
                // We pass false for is_maximizing_player because we're now considering the opponent's moves, and they will try to minimize the score.
                let value = -self.internal_search(&board, depth - 1, -beta, -alpha, false);
                // Update best_val if the current move leads to a higher score.
                best_val = cmp::max(best_val, value);
                // Update alpha, which represents the best score that the maximizing player can guarantee at this point.
                alpha = cmp::max(alpha, best_val);
                // If alpha is greater than or equal to beta, prune the remaining branches.
                if beta <= alpha {
                    break;
                }
            }
            best_val
        // If not maximizing player, it will try to minimize the evaluation score.
        } else {
            // Initialize best_val to positive infinity.
            let mut best_val = 1000000;

            // Iterate over each possible move.
            for board_move in all_moves.iter() {
                // Create a new board with the current move applied.
                let board = board.make_move_new(*board_move);
                // Recursively call the internal_search function, reducing the depth by 1 each time and switching the player.
                // We pass true for is_maximizing_player because we're now considering the opponent's moves, and they will try to maximize the score.
                let value = -self.internal_search(&board, depth - 1, -beta, -alpha, true);
                // Update best_val if the current move leads to a lower score.
                best_val = cmp::min(best_val, value);
                // Update beta, which represents the best score that the minimizing player can guarantee at this point.
                beta = cmp::min(beta, best_val);
                // If beta is less than or equal to alpha, prune the remaining branches.
                if beta <= alpha {
                    break;
                }
            }
            best_val
        }
    }
}
struct PiecesColored {
    color: Color,
    kings: BitBoard,
    pawns: BitBoard,
    rooks: BitBoard,
    queens: BitBoard,
    knights: BitBoard,
    bishops: BitBoard,
}

/**
 * A tuple is a bad idea isn't it?
 *
 * Returns (capture_moves, non_capture_moves)
 */
fn generate_moves(board: &Board) -> (Vec<ChessMove>, Vec<ChessMove>) {
    let mut legal_iterable = MoveGen::new_legal(&board);
    let targets = board.color_combined(!board.side_to_move());

    let mut capture_moves: Vec<_> = vec![];
    let mut non_capture_moves: Vec<_> = vec![];

    legal_iterable.set_iterator_mask(*targets);
    for mov in &mut legal_iterable {
        capture_moves.push(mov);
    }

    legal_iterable.set_iterator_mask(!EMPTY);
    for mov in &mut legal_iterable {
        non_capture_moves.push(mov);
    }

    return (capture_moves, non_capture_moves);
}

/**
 * Returns the pieces but colored.
 */
fn get_colored_pieces(board: &Board, color: Color) -> PiecesColored {
    // returns every black piece in the board..
    // but they're all treated the same....
    //
    // the nuance that a piece is a bishop or a rook is gone
    let black = board.color_combined(color).0;

    // currently returns both black and white's pieces.
    let kings = board.pieces(Piece::King).0;
    let pawns = board.pieces(Piece::Pawn).0;
    let rooks = board.pieces(Piece::Rook).0;
    let queens = board.pieces(Piece::Queen).0;
    let knights = board.pieces(Piece::Knight).0;
    let bishops = board.pieces(Piece::Bishop).0;

    // do a "mask" to select the kings from the black pieces
    let color_kings = BitBoard::new(black & kings); // 001 & 101 = 001
    let color_pawns = BitBoard::new(black & pawns);
    let color_rooks = BitBoard::new(black & rooks);
    let color_queens = BitBoard::new(black & queens);
    let color_knights = BitBoard::new(black & knights);
    let color_bishops = BitBoard::new(black & bishops);

    PiecesColored {
        color,
        kings: color_kings,
        pawns: color_pawns,
        rooks: color_rooks,
        queens: color_queens,
        knights: color_knights,
        bishops: color_bishops,
    }
}

/**
 * h8Q = promotions
 * Qh6 = more specific movement (Queen goes to h6)
 * Qh6xh8 = capturing
 */
fn get_user_move(board: &Board) -> Result<ChessMove, chess::Error> {
    let mut input = String::new();
    println!("Enter your move (e.g. e4, f4e2):");
    stdin().read_line(&mut input).expect("Failed to read line");

    let input = input.trim();

    ChessMove::from_san(&board, input)
}
