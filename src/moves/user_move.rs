use std::io::stdin;

use chess::{Board, ChessMove};

/**
 * Returns the pieces but colored.
 */

/**
 * h8Q = promotions
 * Qh6 = more specific movement (Queen goes to h6)
 * Qh6xh8 = capturing
 */
pub fn get_user_move(board: &Board) -> Result<ChessMove, chess::Error> {
    let mut input = String::new();
    println!("Enter your move (e.g. e4, f4e2):");
    stdin().read_line(&mut input).expect("Failed to read line");

    let input = input.trim();

    ChessMove::from_san(&board, input)
}
