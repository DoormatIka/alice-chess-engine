
extern crate vampirc_uci;

use chess::{Board, Rank, File};
use peak_alloc::PeakAlloc;
use std::fmt::Display;
use std::str::FromStr;
use std::time::Duration;
use vampirc_uci::{parse, UciMove, UciMessage, UciTimeControl, UciSquare, UciInfoAttribute, Duration as TimeDelta};

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

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;


fn output_thread(out: UciMessage, toggle_ready_ok: &Arc<RwLock<bool>>) {
    match out {
        UciMessage::UciOk => {
            println!("id name Cirno");
            println!("id author twoleaflotus");
            println!("uciok");
        },

        UciMessage::IsReady => {
            *toggle_ready_ok.write().unwrap() = true;
        },

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
                let board = Board::from_str("rnb1kbnr/ppp1pppp/8/3p4/2q1P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1").expect("Die.");

                if let Some(depth) = search_control.depth {
                    let mut bot = BasicBot::new(&board);
                    let (eval, chess_move) = bot.search(depth as u16);
                    let (src_rank, src_file) = (chess_move.get_source().get_rank(), chess_move.get_source().get_file());
                    let (dest_rank, dest_file) = (chess_move.get_dest().get_rank(), chess_move.get_dest().get_file());

                    let chess_move = UciMove {
                        from: UciSquare { file: file_to_string(&src_file), rank: rank_to_number(&src_rank) }, 
                        to: UciSquare { file: file_to_string(&dest_file), rank: rank_to_number(&dest_rank) }, 
                        promotion: None,
                    };

                    for depth in 0..depth {
                        // pull this out from the search function in basic bot
                        let depth = UciInfoAttribute::Depth(depth);
                        let pv = UciInfoAttribute::Pv(vec![chess_move]);
                        let nodes = UciInfoAttribute::Nodes(bot.get_nodes_per_second() as u64);
                        println!("{}", UciMessage::Info(vec![depth, pv, nodes]));

                        thread::sleep(Duration::from_millis(200));
                    }
                    let best_move = UciMessage::best_move(chess_move);
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

        match uci_message {
            // switches the executable from not UCI to UCI-mode
            UciMessage::Uci => {
                output_tx.send(UciMessage::UciOk)
            },

            // can be used by the GUI to check if the engine is ready or online
            // also used when the GUI send a LOT of commands and will take time to complete
            UciMessage::IsReady => {
                // readyok will now be sent if the chess engine is ready again.
                output_tx.send(uci_message)
            },

            // not supported, use position startpos moves e2e4 ... instead.
            // https://stackoverflow.com/questions/56528420/basic-questions-on-uci-engine-ucinewgame-and-multiple-clients
            UciMessage::UciNewGame => {
                Ok(())
            }
            
            // sets up the board with a fen string and some moves
            // btw, this is where "position startpos moves" will go to
            UciMessage::Position { .. } => { 
                output_tx.send(uci_message)
            },
            // allows the engine to start calculating on current position
            UciMessage::Go { .. } => {
                output_tx.send(uci_message)
            }
            // stops all calculations.
            UciMessage::Stop => { Ok(()) }
            _ => {
                Ok(())
            },
        }.expect("Main thread can't send to output/process thread");
        
        // let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
        // println!("The max memory that was used: {}kb", peak_mem);
    }
}
