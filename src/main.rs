
use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use chess::{MoveGen, Board, EMPTY, ChessMove};
use colored::*;
pub mod fen;

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

    // Prints board from fen...
    print_board_from_fen("r1bqkb1r/p1p2npp/2pp2n1/p7/3B4/1R1QPK1P/PPNP1PP1/5BNR b - - 0 1");

    println!("Default position: {:?} \nCapture Moves:\n {:?}\n\n Non-capture Moves:\n {:?}\n\n", board.to_string(), capture_moves, non_capture_moves);

    let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
	println!("The max memory that was used: {}kb", peak_mem);
}
