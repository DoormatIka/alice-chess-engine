
use std::{str::FromStr, vec, io::stdin, cmp};

use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use chess::{MoveGen, EMPTY, Game, ChessMove, BoardStatus, Color, Board, Piece, BitBoard};
use rand::seq::SliceRandom;

pub mod fen;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let black_on_check = "rnb1kbnr/5ppp/p7/1p6/2p5/8/PPP1QPPP/RNB1KBNR b KQkq - 0 1";
    let capture = "rnbqkbnr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let en_passant = "rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

    let mut game = Game::from_str(starting).unwrap();
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
            _ => ()
        }

        let (mut capture_moves, mut non_capture_moves) = generate_moves(&board);

        print_board_from_fen(game.current_position().to_string().as_str(), &capture_moves, &non_capture_moves);

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
    fn search(&mut self, depth: u8) -> (i32, ChessMove);
}
trait Evaluation {
    fn evaluation(&self, board: &Board) -> i32;
}

impl Search for BasicBot {
    // external function, interacts with self
    fn search(&mut self, depth: u8) -> (i32, ChessMove) {
        let board = self.board.clone();
        let best_eval = self.internal_search(&board, depth);
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
        Self { board: board.clone(), best_move: None, best_eval: -9999999 }
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

    fn internal_search(&mut self, board: &Board, depth: u8) -> i32 {

        let negative_infinity = -1000000;

        if depth == 0 {
            return self.evaluation(board);
        }
        
        // generate moves here
        let (mut capture_moves, mut non_capture_moves) = generate_moves(&board);

        let mut all_move: Vec<ChessMove> = vec![];
        all_move.append(&mut capture_moves);
        all_move.append(&mut non_capture_moves);

        if all_move.len() == 0 {
            if board.checkers().popcnt() != 0 {
                return negative_infinity;
            }
            return 0;
        }

        let mut best_eval = negative_infinity;
        let mut best_move = Some(all_move[0]);

        for board_move in all_move.iter() {
            let board = board.make_move_new(*board_move);
            let eval = -self.internal_search(&board, depth - 1);

            /*
            if let Some(best_move) = self.best_move {
                println!("{}, {}", best_move, &all_move[0]);
            } else {
                println!("_, {}", &all_move[0]);
            }
            */

            if best_eval < eval {
                best_move = Some(*board_move);
            }
            best_eval = cmp::max(eval, best_eval);
        }

        self.best_eval = best_eval;
        self.best_move = best_move;

        self.best_eval
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
