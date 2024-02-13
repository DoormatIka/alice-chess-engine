
extern crate vampirc_uci;

use peak_alloc::PeakAlloc;
use std::fmt::Display;
use std::time::Duration;
use vampirc_uci::{parse, UciMove, UciMessage, UciTimeControl, UciSearchControl};

use std::sync::mpsc::{self, Sender, Receiver, TryRecvError};
use std::io::stdin;
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

pub mod bots;
pub mod fen;
pub mod moves;
pub mod piece_sq_tables;
pub mod types;
pub mod game;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

/**
 * For the write thread to get.
 */
enum Command {
    // name
    UciOk(String),
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::UciOk(name) => write!(f, "{}", name),
        }
        
    }
}

fn main() {
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let almost_mate = "6k1/1p3ppp/8/8/8/3q1bP1/5K1P/8 b - - 0 1    ";

    let (output_tx, output_rx): (Sender<Command>, Receiver<Command>) = mpsc::channel();
    let (input_tx, input_rx): (Sender<UciMessage>, Receiver<UciMessage>) = mpsc::channel();

    let is_ready = Arc::new(RwLock::new(true));
    let out_ready = Arc::clone(&is_ready);

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
                Ok(out) => {
                    *out_ready.write().unwrap() = false;
                    match out {
                        Command::UciOk(ready) => {
                            thread::sleep(Duration::from_secs(2));
                            println!("uciok");
                        },
                    }
                    *out_ready.write().unwrap() = true;
                },
                Err(err) => {
                    match err {
                        TryRecvError::Disconnected => panic!("Disconnected from the main thread!"),
                        _ => {},
                    }
                },
            }
        }
    });


    loop {
        let input = input_rx.recv().expect("Failed to recieve from input thread.");

        match input {
            // switches the executable from not UCI to UCI-mode
            UciMessage::Uci => {
                output_tx.send(Command::UciOk(String::from("Cirno")))
            },

            // can be used by the GUI to check if the engine is ready or online
            // also used when the GUI send a LOT of commands and will take time to complete
            UciMessage::IsReady => {
                if *is_ready.read().unwrap() {
                    println!("{}", UciMessage::ReadyOk);
                } else {
                    println!("readynotok (this is not standard UCI)");
                }
                Ok(())
            },

            // not supported, use position startpos moves e2e4 ... instead.
            // https://stackoverflow.com/questions/56528420/basic-questions-on-uci-engine-ucinewgame-and-multiple-clients
            UciMessage::UciNewGame => {
                Ok(())
            }
            
            // sets up the board with a fen string and some moves
            //
            // btw, this is where "position startpos moves" will go to
            UciMessage::Position { startpos, fen, moves } => { Ok(()) },
            
            // allows the engine to start calculating on current position
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
                    search_control.depth; // the only supported option we have.
                };
                Ok(()) 
            }
            // stops all calculations.
            UciMessage::Stop => { Ok(()) }
            _ => {
                println!("Invalid command.");
                Ok(())
            },
        }.expect("Main thread can't send to output/process thread");
        
        // let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
        // println!("The max memory that was used: {}kb", peak_mem);
    }
}
