use chess::ChessMove;
use colored::*;

pub fn print_board_from_fen(fen: &str, targets: &Vec<ChessMove>, moves: &Vec<ChessMove>) {
    let fen_parts: Vec<&str> = fen.split(' ').collect();
    let board_rows: Vec<&str> = fen_parts[0].split('/').collect();

    for (row_index, row) in board_rows.iter().enumerate() {
        let mut line = String::new();
        let mut char_index = 0;
        for character in row.chars() {
            let mut square_index = (8 * (7 - row_index)) + char_index;
            match character {
                '1'..='8' => {
                    let num_spaces = character.to_digit(10).unwrap();
                    for _ in 0..num_spaces {
                        let piece = format!("[ ]{}", square_index);
                        if moves.iter().any(|chess_move| chess_move.get_dest().to_index() == square_index) {
                            line.push_str(&piece.green().to_string());
                        } else {
                            line.push_str(&piece);
                        }
                        square_index += 1;
                    }
                }
                _ => {
                    let piece = format!("[{}]{}", character, square_index);
                    if targets.iter().any(|chess_move| chess_move.get_dest().to_index() == square_index) {
                        line.push_str(&piece.red().to_string());
                    } else {
                        line.push_str(&piece);
                    }
                    square_index += 1;
                },
            }
            char_index = square_index - (8 * (7 - row_index));
        }
        println!("{} {}", 8-row_index, line);
    }
    println!("   a  b  c  d  e  f  g  h");

    let turn = match fen_parts[1] {
        "w" => "White's turn",
        "b" => "Black's turn",
        _ => "Unknown",
    };
    println!("{}", turn);
}