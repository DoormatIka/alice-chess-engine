extern crate vampirc_uci;

use mimalloc::MiMalloc;

use chess::{Board, BoardStatus, Game};
use std::str::FromStr;
use std::time::Duration;
use uci::conversion::uci_move_to_chess_move;
use vampirc_uci::{parse, UciInfoAttribute, UciMessage, UciTimeControl};

use std::io::stdin;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, RwLock};
use std::thread;

use crate::bots::basic_bot::BasicBot;
use crate::bots::bot_traits::Search;
use crate::uci::conversion;

pub mod bots;
pub mod fen;
pub mod moves;
pub mod piece_sq_tables;
pub mod types;
pub mod uci;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn output_thread(out: UciMessage, out_board: &mut Board, toggle_ready_ok: &Arc<RwLock<bool>>) {
    match out {
        UciMessage::Uci => {
            println!("id name Cirno");
            println!("id author twoleaflotus");
            println!("{}", UciMessage::UciOk);
        }

        UciMessage::IsReady => {
            *toggle_ready_ok.write().unwrap() = true;
        }

        UciMessage::Position {
            startpos,
            fen,
            moves,
        } => {
            let board = if startpos {
                Some(Board::default())
            } else {
                fen.and_then(|fen| Board::from_str(fen.0.as_str()).ok())
            };
            if let Some(board) = board {
                let mut new_board = board.clone();
                // println!("{}", new_board);
                for uci_move in moves {
                    if let Ok(chess_move) = uci_move_to_chess_move(&uci_move) {
                        if new_board.status() == BoardStatus::Ongoing && new_board.legal(chess_move)
                        {
                            let temp_board = new_board.make_move_new(chess_move);
                            new_board = temp_board;
                        }
                    }
                }
                *out_board = new_board;
            }
        }

        // engine responsibilities, so "go" has to be here
        UciMessage::Go {
            time_control,
            search_control,
        } => {
            if let Some(time_control) = time_control {
                match time_control {
                    // the act of thinking during the opponent's turn.
                    // we have separate threads so this should be easy to implement.
                    //      however, i don't really see a gain with
                    //      pondering (with the techniques rn), so no thanks.
                    //
                    // (opinion)
                    // also pondering can only be worth it *if*
                    //      the engine & player is on the same level.
                    //      since pondering searches through the tree and does a null-move (?)
                    //      and it only becomes worth if the player plays the expected move.
                    // https://www.chessprogramming.org/Pondering
                    UciTimeControl::Ponder => (),
                    UciTimeControl::Infinite => (),
                    UciTimeControl::MoveTime(_time) => (),
                    UciTimeControl::TimeLeft { .. } => (),
                };
            };
            if let Some(search_control) = search_control {
                if let Some(depth) = search_control.depth {
                    let mut bot = BasicBot::new(&out_board);
                    let (eval, chess_move) = bot.search(depth as u16);
                    let best_uci_move = conversion::chess_move_to_uci_move(&chess_move);

                    let depth_data = bot.uci.get_depth_data();
                    let debug_tree = bot.get_debug_tree();
                    bot.write_debug_tree_to_file();
                    println!("{:#?}", debug_tree);

                    for index in (0..depth_data.len()).rev() {
                        let data = &depth_data[index];

                        let mut info_vec: Vec<UciInfoAttribute> = vec![];
                        info_vec.reserve(3);

                        if let Some(chess_move) = data.best_move {
                            let chess_move = conversion::chess_move_to_uci_move(&chess_move);
                            info_vec.push(UciInfoAttribute::Pv(vec![chess_move]));
                        };
                        info_vec.push(UciInfoAttribute::Depth(data.depth as u8));
                        info_vec.push(UciInfoAttribute::Nodes(data.node_count as u64));

                        println!("{}", UciMessage::Info(info_vec));
                    }
                    let best_move = UciMessage::best_move(best_uci_move);
                    println!("{}", best_move);
                }
            };
        }
        _ => {}
    }
    if *toggle_ready_ok.read().unwrap() == true {
        println!("{}", UciMessage::ReadyOk);
    }
    *toggle_ready_ok.write().unwrap() = false;
}

fn main() {
    let (output_tx, output_rx): (Sender<UciMessage>, Receiver<UciMessage>) = mpsc::channel();
    let (input_tx, input_rx): (Sender<UciMessage>, Receiver<UciMessage>) = mpsc::channel();
    let toggle_ready_ok = Arc::new(RwLock::new(false));

    // INPUT
    thread::spawn(move || loop {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        let uci = parse(input.as_str());
        for command in uci {
            input_tx
                .send(command)
                .expect("Failed to send input to main thread.")
        }
    });

    // OUTPUT
    thread::spawn(move || {
        let mut board = Board::default();
        loop {
            thread::sleep(Duration::from_millis(100));
            match output_rx.try_recv() {
                Ok(out) => output_thread(out, &mut board, &toggle_ready_ok),
                Err(err) => match err {
                    TryRecvError::Disconnected => panic!("Disconnected from the main thread!"),
                    _ => {}
                },
            }
        }
    });

    loop {
        // this part might seem useless but its not.
        let uci_message = input_rx
            .recv()
            .expect("Failed to recieve from input thread.");

        thread::sleep(Duration::from_millis(100));

        match uci_message {
            UciMessage::Uci
            | UciMessage::IsReady
            | UciMessage::Position { .. }
            | UciMessage::Go { .. }
            | UciMessage::Stop => output_tx.send(uci_message),

            // not supported, use position startpos moves e2e4 ... instead.
            // https://stackoverflow.com/questions/56528420/basic-questions-on-uci-engine-ucinewgame-and-multiple-clients
            UciMessage::UciNewGame => Ok(()),
            _ => Ok(()),
        }
        .expect("Main thread can't send to output/process thread");
    }
}
