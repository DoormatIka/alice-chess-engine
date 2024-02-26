use chess::{Board, ChessMove, Color, Piece, Square};
use nohash_hasher::BuildNoHashHasher;
use rand::Rng;
use std::collections::VecDeque;
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub eval: i32,
    pub best_move: Option<ChessMove>,
    pub depth: u16,
}

struct ZobristValues {
    white_pieces: [[u64; 6]; 64],
    black_pieces: [[u64; 6]; 64],
    white_turn: u64,
    black_turn: u64,
}

pub struct ZobristHashMap<V> {
    map: HashMap<u64, V, BuildNoHashHasher<u64>>,
    zobrist_table: ZobristValues,
    keys: VecDeque<u64>,
    capacity: usize,
}

fn capacity_to_bytes<K, V>() -> usize {
    std::mem::size_of::<K>() + std::mem::size_of::<V>()
}
fn bytes_to_capacity<K, V>(total_bytes: usize) -> usize {
    total_bytes / (std::mem::size_of::<K>() + std::mem::size_of::<V>())
}

impl<V> ZobristHashMap<V>
where
    V: Debug,
{
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
        let hashd = hash(key, &self.zobrist_table);
        self.keys.push_back(hashd);
        let result = self.map.insert(hashd, value);
        self.remove_oldest_if_full();
        result
    }
    pub fn contains(&self, key: &Board) -> bool {
        let hashd = hash(key, &self.zobrist_table);
        self.map.contains_key(&hashd)
    }
    pub fn get(&self, key: &Board) -> Option<&V> {
        let hashd = hash(key, &self.zobrist_table);
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

fn init_zobrist() -> ZobristValues {
    let mut rng = rand::thread_rng();
    let mut black_zobrist_table: [[u64; 6]; 64] = [[0; 6]; 64];
    let mut white_zobrist_table: [[u64; 6]; 64] = [[0; 6]; 64];
    for sq in 0..64 {
        // 0 to 63 (inclusive)
        for piece_index in Piece::Pawn.to_index()..Piece::King.to_index() {
            white_zobrist_table[sq][piece_index] = rng.gen_range(0..u64::MAX);
            black_zobrist_table[sq][piece_index] = rng.gen_range(0..u64::MAX);
        }
    }

    ZobristValues {
        white_pieces: white_zobrist_table,
        black_pieces: black_zobrist_table,
        white_turn: rng.gen_range(0..u64::MAX),
        black_turn: rng.gen_range(0..u64::MAX),
    }
}

fn hash(
    board: &Board,
    zobrist_values: &ZobristValues,
) -> u64 {
    let mut final_hash = 0;
    let color_value = match board.side_to_move() {
        Color::White => zobrist_values.white_turn,
        Color::Black => zobrist_values.black_turn
    };

    for i in 0..64 {
        if let Some(piece) = board.piece_on(unsafe { Square::new(i) }) {
            if let Some(color) = board.color_on(unsafe { Square::new(i) }) {
                match color {
                    Color::White => {
                        final_hash = final_hash ^ zobrist_values.white_pieces[i as usize][piece.to_index()] ^ color_value
                    }
                    Color::Black => {
                        final_hash = final_hash ^ zobrist_values.black_pieces[i as usize][piece.to_index()] ^ color_value
                    }
                };
            }
        }
    }

    final_hash
}
