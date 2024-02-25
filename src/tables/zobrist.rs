use std::{collections::HashMap, fmt::Debug};
use nohash_hasher::BuildNoHashHasher;

use chess::{Piece, Board, Square, Color, ChessMove};
use rand::Rng;

#[derive(Debug)]
pub struct NodeInfo {
    pub eval: i32,
    pub best_move: ChessMove,
}

pub struct ZobristHashMap<V> {
    map: HashMap<u64, V, BuildNoHashHasher<u64>>,
    zobrist_table: ([[u64; 6]; 64], [[u64; 6]; 64]),
}

impl<V> Default for ZobristHashMap<V> {
    fn default() -> Self {
        Self { map: Default::default(), zobrist_table: init_zobrist() }
    }
}

impl<V> ZobristHashMap<V> where V: Debug {
    pub fn new() -> Self {
        ZobristHashMap { 
            map: HashMap::with_hasher(BuildNoHashHasher::default()),
            zobrist_table: init_zobrist(),
        }
    }
    pub fn insert(&mut self, key: &Board, value: V) -> Option<V> {
        let hashd = hash(key, self.zobrist_table.0, self.zobrist_table.1);
        self.map.insert(hashd, value)
    }
    pub fn contains(&self, key: &Board) -> bool {
        let hashd = hash(key, self.zobrist_table.0, self.zobrist_table.1);
        self.map.contains_key(&hashd)
    }
    pub fn get(&self, key: &Board) -> Option<&V> {
        let hashd = hash(key, self.zobrist_table.0, self.zobrist_table.1);
        self.map.get(&hashd)
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn print(&self) {
        println!("{:#?}", self.map);
    }
}

pub fn init_zobrist() -> ([[u64; 6]; 64], [[u64; 6]; 64]) {
    let mut rng = rand::thread_rng();
    let mut black_zobrist_table: [[u64; 6]; 64] = [[0; 6]; 64];
    let mut white_zobrist_table: [[u64; 6]; 64] = [[0; 6]; 64];
    for sq in 0..64 { // 0 to 63 (inclusive)
        for piece_index in Piece::Pawn.to_index()..Piece::King.to_index() {
            white_zobrist_table[sq][piece_index] = rng.gen_range(0..u64::MAX);
            black_zobrist_table[sq][piece_index] = rng.gen_range(0..u64::MAX);
        }
    }

    (white_zobrist_table, black_zobrist_table)
}

pub fn hash(board: &Board, white_zobrist_table: [[u64; 6]; 64], black_zobrist_table: [[u64; 6]; 64]) -> u64 {
    let mut h = 0;
    for i in 0..64 {
        if let Some(piece) = board.piece_on(unsafe { Square::new(i) }) {
            match board.side_to_move() {
                Color::White => h = h ^ white_zobrist_table[i as usize][piece.to_index()],
                Color::Black => h = h ^ black_zobrist_table[i as usize][piece.to_index()],
            };
        }
    }
    
    h
}
