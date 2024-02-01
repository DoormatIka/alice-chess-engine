
use chess::ChessMove;
use colored::*;

pub fn print_board_from_fen(fen: &str, targets: &Vec<ChessMove>, moves: &Vec<ChessMove>) {
    let parts: Vec<&str> = fen.split(' ').collect();
    let mut rows: Vec<&str> = parts[0].split('/').collect();

    rows.reverse();

    for (i, row) in rows.iter().enumerate() { // row number
        for c in row.chars() {
            match c {
                '1'..='8' => {
                    let num = c.to_digit(10).unwrap();
                    for j in 0..num { // column
                        let s = if let Some(mov) = moves.iter().find(|v| v.get_dest().to_index() == (8 * i) + j as usize) {
                            "[ ]".green()
                        } else {
                            "[ ]".into()
                        };
                        print!("{}{}", s, (8 * i) + j as usize);
                    }
                }
                _ => print!("{}", ColoredString::from(format!("[{}]", c))),
            }
        }
        println!(" {}", 8-i);
    }
    println!(" a  b  c  d  e  f  g  h");

    let turn = match parts[1] {
        "w" => "White's turn",
        "b" => "Black's turn",
        _ => "Unknown",
    };
    println!("{}", turn);
}
