
pub struct PieceSquare {
    pawn_table: [i16; 64],
    knight_table: [i16; 64],
    bishop_table: [i16; 64],
    rook_table: [i16; 64],
    queen_table: [i16; 64],
    king_table: [i16; 64],
}

pub struct PeSTOPieceSquare {
    middle_game: PieceSquare,
    end_game: PieceSquare,
}
