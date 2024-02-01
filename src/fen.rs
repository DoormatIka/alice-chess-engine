pub fn print_board_from_fen(fen: &str) {
    let parts: Vec<&str> = fen.split(' ').collect();
    let rows: Vec<&str> = parts[0].split('/').collect();

    for (i, row) in rows.iter().enumerate() {
        let mut line = String::new();
        for c in row.chars() {
            match c {
                '1'..='8' => {
                    let num = c.to_digit(10).unwrap();
                    for _ in 0..num {
                        line.push_str("[ ]");
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