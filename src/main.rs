use chess::{BoardStatus, ChessMove, Color, Game};
use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use std::{str::FromStr, vec};

use crate::bots::basic_bot::BasicBot;
use crate::bots::bot_traits::Search;
use crate::moves::move_gen::generate_moves;
use crate::moves::user_move::get_user_move;

pub mod bots;
pub mod fen;
pub mod moves;
pub mod piece_sq_tables;
pub mod types;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let almost_mate = "6k1/1p3ppp/8/8/8/3q1bP1/5K1P/8 b - - 0 1    ";

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
