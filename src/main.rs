
extern crate vampirc_uci;

use chess::{BoardStatus, ChessMove, Color, Game};
use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use std::time::Duration;
use std::{str::FromStr, vec};
use vampirc_uci::{parse, UciMove, UciMessage};

use crate::bots::basic_bot::BasicBot;
use crate::bots::bot_traits::Search;
use crate::moves::move_gen::generate_moves;
use crate::moves::user_move::get_user_move;

use std::sync::mpsc::{self, Sender, Receiver};
use std::io::stdin;
use std::thread;
use std::sync::{Mutex, Arc};

pub mod bots;
pub mod fen;
pub mod moves;
pub mod piece_sq_tables;
pub mod types;
pub mod uci;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

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

    let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
    println!("The max memory that was used: {}kb", peak_mem);
}

fn main() {
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let almost_mate = "6k1/1p3ppp/8/8/8/3q1bP1/5K1P/8 b - - 0 1    ";

    let (output_tx, output_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let (input_tx, input_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    let is_ready = Arc::new(Mutex::new(true));
    
    let out_ready = Arc::clone(&is_ready);

    thread::spawn(move || {
        loop {
            let out = output_rx.recv().expect("Failed to recieve from main thread.");
            *out_ready.lock().unwrap() = false;
            thread::sleep(Duration::from_secs(2));
            *out_ready.lock().unwrap() = true;
            println!("Output thread: {}", out);
        }
    });
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Failed to read line");
            input_tx.send(input).expect("Failed to send input to main thread.");
            /*
            let uci = parse(input.as_str());

            for command in uci {
                input_tx.send(command).expect("Failed to send input to main thread.")
            }
            */
        }
    });

    // game(starting);

    loop {
        let input = input_rx.recv().expect("Failed to recieve from input thread.");

        if *is_ready.lock().unwrap() {
            output_tx.send(format!("Recieved: {}", input)).expect("Failed to send to output_tx");
        } else {
            println!("Main thread: Not ready yet!");
        }

        /*
        match input {
            UciMessage::IsReady => {
                let is_ready = if *is_ready.lock().unwrap() {
                    "ready"
                } else {
                    "not ready"
                };
                println!("{}", is_ready);
            }
            _ => {},
        }
        */
    }
}
