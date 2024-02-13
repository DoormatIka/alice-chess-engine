
use std::sync::mpsc;
use std::io::stdin;
use std::thread;
use vampirc_uci::parse;
use vampirc_uci::{UciMessage, MessageList, UciTimeControl, Serializable};

fn p() {
    let (tx, rx) = mpsc::channel();
    
    let t = thread::spawn(move || {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");

        // this thread sends stdin into a channel
        tx.send(input).expect("Failed to send input to main thread");
    });

    // waits for something from the channel
    let input = rx.recv().expect("Failed to receive input from other thread");
    
    // the main thread waits for the spawned thread.
    t.join().expect("Failed to join stdin reading thread.");

    println!("Input received from other thread: {}", input);
}
