

use std::str::FromStr;

use chess::{BoardStatus, ChessMove, Color, Game};

use crate::fen::print_board_from_fen;
use crate::bots::basic_bot::BasicBot;
use crate::bots::bot_traits::Search;
use crate::moves::move_gen::generate_moves;
use crate::moves::user_move::get_user_move;

/**
 * Move this to a separate thread.
 */
fn game(fen: &str) {
    let mut game = Game::from_str(fen).unwrap();
    let player_color: Option<Color> = Some(Color::Black);

    let board = game.current_position();

    if game.can_declare_draw() {
        println!("Stalemated by three-fold repetition.");
        return;
    }

    match board.status() {
        BoardStatus::Stalemate => {
            println!("Stalemated");
            return;
        }
        BoardStatus::Checkmate => {
            println!("Stalemated");
            return;
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
                    return;
                }
            };
        } else {
            let mut bot = BasicBot::new(&board);
            let (eval, chess_move) = bot.search(4);
            game.make_move(chess_move);
            println!("Made move with eval {}, {}", eval, chess_move);
        }
    } else {
        let mut bot = BasicBot::new(&board);
        let (eval, chess_move) = bot.search(4);
        game.make_move(chess_move);
        println!("Made move with eval {}, {}", eval, chess_move);
    }
}
