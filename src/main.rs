
extern crate vampirc_uci;

use peak_alloc::PeakAlloc;
use std::fmt::Display;
use std::time::Duration;
use vampirc_uci::{parse, UciMove, UciMessage, UciTimeControl, UciSearchControl, UciSquare, UciPiece};

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

fn main() {
    let starting = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let almost_mate = "6k1/1p3ppp/8/8/8/3q1bP1/5K1P/8 b - - 0 1    ";

    let (output_tx, output_rx): (Sender<UciMessage>, Receiver<UciMessage>) = mpsc::channel();
    let (input_tx, input_rx): (Sender<UciMessage>, Receiver<UciMessage>) = mpsc::channel();

    // TODO: not really sure what isready's behavior is..
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
                        UciMessage::UciOk => {
                            println!("id name Cirno");
                            println!("id author twoleaflotus");
                            println!("uciok");
                        },

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
                                search_control.depth; // the only supported option we have.
                                
                                let best_move = UciMessage::best_move(UciMove {
                                    from: UciSquare { file: 'e', rank: 5 }, 
                                    to: UciSquare { file: 'e', rank: 7 }, 
                                    promotion: None,
                                });

                                thread::sleep(Duration::from_secs(3));
                                println!("{}", best_move);
                            };
                        }
                        _ => {},
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
                if *is_ready.read().unwrap() {
                    println!("{}", UciMessage::ReadyOk);
                }
                Ok(())
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
