
use std::{str::FromStr, vec, io::stdin};

use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use chess::{MoveGen, EMPTY, Game, ChessMove, BoardStatus, Color, Board, Piece};
use rand::seq::SliceRandom;

pub mod fen;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let black_on_check = "rnb1kbnr/5ppp/p7/1p6/2p5/8/PPP1QPPP/RNB1KBNR b KQkq - 0 1";
    let capture = "rnbqkbnr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let en_passant = "rnbqkbnr/ppppp1pp/8/8/4Pp2/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";

    let mut game = Game::from_str(black_on_check).unwrap();
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

        let mut legal_iterable = MoveGen::new_legal(&board);
        let targets = board.color_combined(!game.side_to_move());

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

        print_board_from_fen(game.current_position().to_string().as_str(), &capture_moves, &non_capture_moves);

        let mut all_move: Vec<ChessMove> = vec![];
        all_move.append(&mut capture_moves);
        all_move.append(&mut non_capture_moves);

        if let Some(player_color) = player_color {
            if game.side_to_move() == player_color {
                let chess_move = get_user_move(&board);
                match chess_move {
                    Ok(chess_move) => game.make_move(chess_move),
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
            } else {
                let bot = BasicBot { board: board.clone() };
                bot.count_material();

                let chosen = all_move.choose(&mut rand::thread_rng());
                if let Some(chosen) = chosen {
                    game.make_move(*chosen);
                }
            }
        } else {
            let chosen = all_move.choose(&mut rand::thread_rng());
            if let Some(chosen) = chosen {
                game.make_move(*chosen);
            }
        }

        let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
        println!("The max memory that was used: {}kb", peak_mem);
    }
}

struct BasicBot {
    board: Board,
}
impl BasicBot {
    fn count_material(&self) {
        let material = 0;

        // currently returns both black and white's pieces.
        let kings = self.board.pieces(Piece::King).popcnt();
        let pawns = self.board.pieces(Piece::Pawn).popcnt();
        let rooks = self.board.pieces(Piece::Rook).popcnt();
        let queens = self.board.pieces(Piece::Queen).popcnt();
        let knights = self.board.pieces(Piece::Knight).popcnt();
        let bishops = self.board.pieces(Piece::Bishop).popcnt();
    }
}
impl Search for BasicBot {
    
}
impl Evaluation for BasicBot {

}
trait Search {
    fn search() {}
}
trait Evaluation {
    fn evaluation() {}
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
