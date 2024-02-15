
extern crate vampirc_uci;

use mimalloc::MiMalloc;

use chess::{Board, Rank, File, ChessMove, Piece};
use std::str::FromStr;
use std::time::Duration;
use vampirc_uci::{parse, UciMove, UciMessage, UciTimeControl, UciSquare, UciInfoAttribute, Duration as TimeDelta, UciPiece};

use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::io::stdin;
use std::thread;
use std::sync::{Arc, RwLock};

use crate::bots::basic_bot::BasicBot;
use crate::bots::bot_traits::Search;

pub mod bots;
pub mod fen;
pub mod moves;
pub mod piece_sq_tables;
pub mod types;
pub mod game;
pub mod uci;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn rank_to_number(rank: &Rank) -> u8 {
    match rank {
        Rank::First => 1,
        Rank::Second => 2,
        Rank::Third => 3,
        Rank::Fourth => 4,
        Rank::Fifth => 5,
        Rank::Sixth => 6,
        Rank::Seventh => 7,
        Rank::Eighth => 8,
    }
}
fn file_to_string(file: &File) -> char {
    match file {
        File::A => 'a',
        File::B => 'b',
        File::C => 'c',
        File::D => 'd',
        File::E => 'e',
        File::F => 'f',
        File::G => 'g',
        File::H => 'h',
    }
}
fn chess_piece_to_uci_piece(piece: &Piece) -> UciPiece {
    match piece {
        Piece::Pawn => UciPiece::Pawn,
        Piece::Rook => UciPiece::Rook,
        Piece::King => UciPiece::King,
        Piece::Queen => UciPiece::Queen,
        Piece::Knight => UciPiece::Knight,
        Piece::Bishop => UciPiece::Bishop,
    }
}

fn chess_move_to_uci_move(chess_move: &ChessMove) -> UciMove {
    let (src_file, src_rank) = (chess_move.get_source().get_file(), chess_move.get_source().get_rank());
    let (dest_file, dest_rank) = (chess_move.get_dest().get_file(), chess_move.get_dest().get_rank());
    let promotion = match chess_move.get_promotion() {
        Some(chess_move) => Some(chess_piece_to_uci_piece(&chess_move)),
        None => None,
    };

    UciMove {
        from: UciSquare { file: file_to_string(&src_file), rank: rank_to_number(&src_rank) },
        to: UciSquare { file: file_to_string(&dest_file), rank: rank_to_number(&dest_rank) },
        promotion,
    }
}


fn output_thread(out: UciMessage, toggle_ready_ok: &Arc<RwLock<bool>>) {
    match out {
        UciMessage::Uci => {
            println!("id name Cirno");
            println!("id author twoleaflotus");
            println!("{}", UciMessage::UciOk);
        },

        // can be used by the GUI to check if the engine is ready or online
        // also used when the GUI send a LOT of commands and will take time to complete
        UciMessage::IsReady => {
            *toggle_ready_ok.write().unwrap() = true;
        },

        // sets up the board with a fen string and some moves
        // btw, this is where "position startpos moves" will go to
        UciMessage::Position { startpos, fen, moves } => {

        }

        // engine responsibilities, so "go" has to be here
        UciMessage::Go { time_control, search_control } => {
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

                    // Search until the "stop" command.
                    // Cannot be implemented at the moment.
                    UciTimeControl::Infinite => (),

                    // Search exactly for x milliseconds.
                    // Cannot be implemented at the moment.
                    UciTimeControl::MoveTime(_time) => (),

                    // Notifies the engine of how much time there is left.
                    UciTimeControl::TimeLeft { .. } => (),
                };
            };
            if let Some(search_control) = search_control {
                let board = Board::from_str("rnb1kb1r/ppp2ppp/8/3p1n2/4p3/3Qq1N1/PPPP1PPP/RNB1KB1R w KQkq - 0 1").expect("Die.");

                if let Some(depth) = search_control.depth {
                    let mut bot = BasicBot::new(&board);
                    let (eval, chess_move) = bot.search(depth as u16);
                    let best_uci_move = chess_move_to_uci_move(&chess_move);

                    let mut depth_data = bot.uci.get_depth_data();

                    depth_data.reverse();

                    for data in depth_data {
                        let mut info_vec: Vec<UciInfoAttribute> = vec![];
    
                        if let Some(chess_move) = data.best_move {
                            let chess_move = chess_move_to_uci_move(&chess_move);
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
        _ => {},
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
    thread::spawn(move || {
        loop {
            let mut input = String::new();
            stdin().read_line(&mut input).expect("Failed to read line");

            let uci = parse(input.as_str());
            for command in uci {
                input_tx.send(command).expect("Failed to send input to main thread.")
            }
        }
    });

    // OUTPUT
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            match output_rx.try_recv() {
                Ok(out) => output_thread(out, &toggle_ready_ok),
                Err(err) => {
                    match err {
                        TryRecvError::Disconnected => panic!("Disconnected from the main thread!"),
                        _ => {},
                    }
                },
            }
        }
    });


    loop { // this part might seem useless but its not.
        let uci_message = input_rx.recv().expect("Failed to recieve from input thread.");

        thread::sleep(Duration::from_millis(100));

        match uci_message {
            UciMessage::Uci 
                | UciMessage::IsReady
                | UciMessage::Position { .. }
                | UciMessage::Go { .. }
                | UciMessage::Stop => {
                output_tx.send(uci_message)
            },

            // not supported, use position startpos moves e2e4 ... instead.
            // https://stackoverflow.com/questions/56528420/basic-questions-on-uci-engine-ucinewgame-and-multiple-clients
            UciMessage::UciNewGame => {
                Ok(())
            }
            _ => {
                Ok(())
            },
        }.expect("Main thread can't send to output/process thread");
    }
}
