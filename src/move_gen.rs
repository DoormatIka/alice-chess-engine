use std::io::stdin;

use chess::{Board, ChessMove, MoveGen, EMPTY};

/**
 * A tuple is a bad idea isn't it?
 *
 * Returns (capture_moves, non_capture_moves)
 */
pub fn generate_moves(board: &Board) -> (Vec<ChessMove>, Vec<ChessMove>) {
    let mut legal_iterable = MoveGen::new_legal(&board);
    let targets = board.color_combined(!board.side_to_move());

    let mut capture_moves: Vec<_> = vec![];
    let mut non_capture_moves: Vec<_> = vec![];

    legal_iterable.set_iterator_mask(*targets);
    for mov in &mut legal_iterable {
        capture_moves.push(mov);
    }

    legal_iterable.set_iterator_mask(!EMPTY);
    for mov in &mut legal_iterable {
        non_capture_moves.push(mov);
    }

    return (capture_moves, non_capture_moves);
}

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
