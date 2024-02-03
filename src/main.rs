
use std::{str::FromStr, vec, io::stdin};

use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use chess::{MoveGen, EMPTY, Game, ChessMove, BoardStatus, Color, Board};
use rand::seq::SliceRandom;

use std::time::Instant;

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

    let start = Instant::now();
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
        legal_iterable.set_iterator_mask(*targets);
        for mov in &mut legal_iterable {
            capture_moves.push(mov);
        }

        let mut non_capture_moves: Vec<_> = vec![];
        legal_iterable.set_iterator_mask(!EMPTY);
        for mov in &mut legal_iterable {
            non_capture_moves.push(mov);
        }

        let mut all_move: Vec<ChessMove> = vec![];
        all_move.append(&mut capture_moves);
        all_move.append(&mut non_capture_moves);

        print_board_from_fen(game.current_position().to_string().as_str(), &capture_moves, &non_capture_moves);

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
    }

    let elapsed = start.elapsed();
    println!("Duration: {} ms", elapsed.as_millis());

    let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
    println!("The max memory that was used: {}kb", peak_mem);
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
