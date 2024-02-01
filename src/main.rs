
use std::str::FromStr;

use fen::print_board_from_fen;
use peak_alloc::PeakAlloc;
use chess::{MoveGen, Board, EMPTY, Color, get_king_moves, ChessMove};
pub mod fen;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

fn main() {
    let board = Board::from_str("rnb1kbnr/5ppp/p7/1p6/2p5/8/PPP1QPPP/RNB1KBNR b KQkq - 0 1").unwrap();
    let mut legal_iterable = MoveGen::new_legal(&board);

    let targets = board.color_combined(!board.side_to_move());

    let mut capture_moves: Vec<_> = vec![];
    legal_iterable.set_iterator_mask(*targets);
    for mov in &mut legal_iterable {
        capture_moves.push(mov);
    }

    let mut non_capture_moves: Vec<_> = vec![];
    if board.checkers().0 == 0 {
        legal_iterable.set_iterator_mask(!EMPTY);
        for mov in &mut legal_iterable {
            non_capture_moves.push(mov);
        }
    } else {
        let king = get_king_moves(board.king_square(board.side_to_move()));
        legal_iterable.set_iterator_mask(king);
        for mov in &mut legal_iterable {
            non_capture_moves.push(mov);
        }
    }



    // Prints board from fen...
    print_board_from_fen(board.to_string().as_str(), &capture_moves, &non_capture_moves);

    println!("Default position: {:?} \nCapture Moves:\n {:?}\n\n Non-capture Moves:\n {:?}\n\n", board.to_string(), capture_moves, non_capture_moves);

    let peak_mem = PEAK_ALLOC.peak_usage_as_kb();
	println!("The max memory that was used: {}kb", peak_mem);
}
