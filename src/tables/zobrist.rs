use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chess::{Piece, Board, Square, Color};
use rand::Rng;

struct BoardHasher {
    hash: u64,
}

impl BoardHasher {
    pub fn new() -> Self {
        Self { hash: 0 }
    }
}

impl Hasher for BoardHasher {
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        
    }
}

#[derive(Default)]
struct ZobristHash<K, V> {
    map: HashMap<K, V>,
}

impl<K, V> ZobristHash<K, V> where K: Eq + Hash {
    fn new() -> Self {
        ZobristHash { map: HashMap::new() }
    }
    fn custom_hash(&self, key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        0 // TODO: MAKE THIS IMPLEMENT THE ZOBRIST HASHHH
    }
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.map.insert(key, value)
    }
    fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    fn len(&self) -> usize {
        self.map.len()
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
