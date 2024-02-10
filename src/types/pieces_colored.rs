use chess::{BitBoard, Board, Color, Piece};

pub struct PiecesColored {
    pub color: Color,
    pub kings: BitBoard,
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub queens: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
}

impl PiecesColored {
    pub fn get_colored_pieces(board: &Board, color: Color) -> Self {
        let black = board.color_combined(color).0;

        let kings = board.pieces(Piece::King).0;
        let pawns = board.pieces(Piece::Pawn).0;
        let rooks = board.pieces(Piece::Rook).0;
        let queens = board.pieces(Piece::Queen).0;
        let knights = board.pieces(Piece::Knight).0;
        let bishops = board.pieces(Piece::Bishop).0;

        let color_kings = BitBoard::new(black & kings);
        let color_pawns = BitBoard::new(black & pawns);
        let color_rooks = BitBoard::new(black & rooks);
        let color_queens = BitBoard::new(black & queens);
        let color_knights = BitBoard::new(black & knights);
        let color_bishops = BitBoard::new(black & bishops);

        PiecesColored {
            color,
            kings: color_kings,
            pawns: color_pawns,
            rooks: color_rooks,
            queens: color_queens,
            knights: color_knights,
            bishops: color_bishops,
        }
    }
}
