
use peak_alloc::PeakAlloc;
use chess::{MoveGen, Board, EMPTY, ChessMove};
use colored::*;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn print_fen(fen: &str, capture_moves: Vec<ChessMove>, non_capture_moves: Vec<ChessMove>) {
    // psst, color the moves.
    let colored = format!("{}, {}", "green".green(), "red".red());
}

fn main() {
    let board = Board::default();
    let mut legal_iterable = MoveGen::new_legal(&board);

    let targets = board.color_combined(!board.side_to_move());

    let mut capture_moves: Vec<_> = vec![];
    legal_iterable.set_iterator_mask(*targets);
    for mov in &mut legal_iterable {
        capture_moves.push(mov);
    }

    let mut non_capture_moves: Vec<_> = vec![];
    legal_iterable.set_iterator_mask(!EMPTY);
    for mov in &mut legal_iterable {
        non_capture_moves.push(mov);
    }

    println!("Default position: {:?} \nCapture Moves:\n {:?}\n\n Non-capture Moves:\n {:?}\n\n", board.to_string(), capture_moves, non_capture_moves);

    let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
	println!("The max memory that was used: {}kb", peak_mem);
}
