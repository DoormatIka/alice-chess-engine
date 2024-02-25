use std::{collections::HashMap, fmt::Debug};
use nohash_hasher::BuildNoHashHasher;
use std::collections::VecDeque;
use chess::{Piece, Board, Square, Color, ChessMove};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub eval: i32,
    pub best_move: Option<ChessMove>,
}

pub struct ZobristHashMap<V> {
    map: HashMap<u64, V, BuildNoHashHasher<u64>>,
    zobrist_table: ([[u64; 6]; 64], [[u64; 6]; 64]),
    keys: VecDeque<u64>,
    capacity: usize,
}

fn capacity_to_bytes<K, V>() -> usize {
    std::mem::size_of::<K>() + std::mem::size_of::<V>()
}
fn bytes_to_capacity<K, V>(total_bytes: usize) -> usize {
    total_bytes / (std::mem::size_of::<K>() + std::mem::size_of::<V>())
}

impl<V> ZobristHashMap<V> where V: Debug {
    pub fn new(byte_size: usize) -> Self {
        let capacity = bytes_to_capacity::<u64, V>(byte_size);
        ZobristHashMap { 
            map: HashMap::with_hasher(BuildNoHashHasher::default()),
            zobrist_table: init_zobrist(),
            keys: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    pub fn insert(&mut self, key: &Board, value: V) -> Option<V> {
        let hashd = hash(key, self.zobrist_table.0, self.zobrist_table.1);
        self.keys.push_back(hashd);
        let result = self.map.insert(hashd, value);
        self.remove_oldest_if_full();
        result
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
    pub fn remove_oldest_if_full(&mut self) {
        if self.keys.len() > self.capacity {
            if let Some(key) = self.keys.pop_front() {
                self.map.remove(&key);
            }
        }
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
