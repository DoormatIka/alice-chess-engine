use chess::ChessMove;
use colored::*;

pub fn print_board_from_fen(fen: &str, targets: &Vec<ChessMove>, moves: &Vec<ChessMove>) {
    let parts: Vec<&str> = fen.split(' ').collect();
    let rows: Vec<&str> = parts[0].split('/').collect();

    for (i, row) in rows.iter().enumerate() {
        let mut line = String::new();
        for (j, c) in row.chars().enumerate() {
            match c {
                '1'..='8' => {
                    let num = c.to_digit(10).unwrap();
                    for k in 0..num {
                        if moves.iter().any(|v| v.get_dest().to_index() == (8 * (7 - i)) + (7 - (j + k as usize)) as usize) {
                            line.push_str(&"[ ]".green().to_string());
                        } else {
                            line.push_str("[ ]");
                        }
                    }
                }
                _ => line.push_str(&format!("[{}]", c)),
            }
        }
        println!("{} {}", 8-i, line);
    }
    println!("   a  b  c  d  e  f  g  h");

    let turn = match parts[1] {
        "w" => "White's turn",
        "b" => "Black's turn",
        _ => "Unknown",
    };
    println!("{}", turn);
}