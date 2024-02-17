
use chess::{ChessMove, File, Piece, Rank, Square};
use vampirc_uci::{UciMove, UciPiece, UciSquare};

#[derive(Debug)]
pub enum ConversionError {
    NumberUndefined,
    CharUndefined,

    ChessMoveToUciMove,
    UciMoveToChessMove,
}

pub fn rank_to_number(rank: &Rank) -> u8 {
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
pub fn file_to_string(file: &File) -> char {
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
pub fn char_to_file(file: char) -> Result<File, ConversionError> {
    match file {
        'a' => Ok(File::A),
        'b' => Ok(File::B),
        'c' => Ok(File::C),
        'd' => Ok(File::D),
        'e' => Ok(File::E),
        'f' => Ok(File::F),
        'g' => Ok(File::G),
        'h' => Ok(File::H),
        _ => Err(ConversionError::CharUndefined),
    }
}

pub fn number_to_rank(rank: u8) -> Result<Rank, ConversionError> {
    match rank {
        1 => Ok(Rank::First),
        2 => Ok(Rank::Second),
        3 => Ok(Rank::Third),
        4 => Ok(Rank::Fourth),
        5 => Ok(Rank::Fifth),
        6 => Ok(Rank::Sixth),
        7 => Ok(Rank::Seventh),
        8 => Ok(Rank::Eighth),
        _ => Err(ConversionError::NumberUndefined),
    }
}

pub fn chess_piece_to_uci_piece(piece: &Piece) -> UciPiece {
    match piece {
        Piece::Pawn => UciPiece::Pawn,
        Piece::Rook => UciPiece::Rook,
        Piece::King => UciPiece::King,
        Piece::Queen => UciPiece::Queen,
        Piece::Knight => UciPiece::Knight,
        Piece::Bishop => UciPiece::Bishop,
    }
}

pub fn uci_piece_to_chess_piece(piece: &UciPiece) -> Piece {
    match piece {
        UciPiece::Pawn => Piece::Pawn,
        UciPiece::Rook => Piece::Rook,
        UciPiece::King => Piece::King,
        UciPiece::Queen => Piece::Queen,
        UciPiece::Knight => Piece::Knight,
        UciPiece::Bishop => Piece::Bishop,
    }
}

pub fn chess_move_to_uci_move(chess_move: &ChessMove) -> UciMove {
    let (src_file, src_rank) = (
        chess_move.get_source().get_file(),
        chess_move.get_source().get_rank(),
    );
    let (dest_file, dest_rank) = (
        chess_move.get_dest().get_file(),
        chess_move.get_dest().get_rank(),
    );
    let promotion = chess_move.get_promotion().map(|promotion| chess_piece_to_uci_piece(&promotion));

    UciMove {
        from: UciSquare {
            file: file_to_string(&src_file),
            rank: rank_to_number(&src_rank),
        },
        to: UciSquare {
            file: file_to_string(&dest_file),
            rank: rank_to_number(&dest_rank),
        },
        promotion,
    }
}

pub fn uci_move_to_chess_move(uci_move: &UciMove) -> Result<ChessMove, ConversionError> {
    let (from_file, from_rank) = ( char_to_file(uci_move.from.file)?, number_to_rank(uci_move.from.rank)? );
    let (to_file, to_rank) = ( char_to_file(uci_move.to.file)?, number_to_rank(uci_move.to.rank)? );
    let promotion = uci_move.promotion.map(|promotion| uci_piece_to_chess_piece(&promotion));

    Ok(ChessMove::new(
        Square::make_square(from_rank, from_file), 
        Square::make_square(to_rank, to_file), 
        promotion
    ))
}
