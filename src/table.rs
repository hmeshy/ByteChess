use std::collections::HashMap;
use crate::util::{Move, Score};

const DEFAULT_TT_SIZE_MB: usize = 256;

// The type of bound stored in the table
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Bound {
    Exact,      // PV-Node: exact score
    Lower,      // Cut-Node: lower bound (beta cutoff)
    Upper,      // All-Node: upper bound (fail high)
}

impl Bound {
    pub fn to_u8(self) -> u8 {
        match self {
            Bound::Exact => 0,
            Bound::Lower => 1,
            Bound::Upper => 2,
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Bound::Exact,
            1 => Bound::Lower,
            _ => Bound::Upper,
        }
    }
}

// A transposition table entry
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub zobrist: u64,      // 8 bytes
    pub best_move: u16,    // 2 bytes
    pub depth: u8,         // 1 byte
    pub bound: u8,         // 1 byte
    pub age: u8,           // 1 byte
    pub _pad: u8,          // 1 byte (explicit padding)
    pub score: i32,        // 4 bytes
}

impl TTEntry {
    pub fn get_bound(&self) -> Bound {
        Bound::from_u8(self.bound)
    }

    pub fn get_best_move(&self) -> Option<Move> {
        if self.best_move == 0 {
            None
        } else {
            Some(Move { info: self.best_move })
        }
    }

    pub fn get_depth(&self) -> i32 {
        self.depth as i32
    }
}
#[derive(Copy, Clone)]
pub struct PawnEntry {
    pub zobrist: u64,      // Zobrist hash of the position
    pub score: Score,        // Score 
}
// The transposition table itself
pub struct TranspositionTable {
    pub table: Vec<Option<TTEntry>>,
    pub mask: usize,
    pub age: u8,
}
pub struct PawnTable {
    pub table: Vec<Option<PawnEntry>>,
    pub mask: usize,
}

impl PawnTable {
    pub fn new() -> Self {
        let size = 1 << 20; // 1M entries
        Self {
            table: vec![None; size],
            mask: size - 1,
        }
    }

    pub fn store(&mut self, entry: PawnEntry) {
        let idx = (entry.zobrist as usize) & self.mask;
        self.table[idx] = Some(entry);
    }

    pub fn probe(&self, zobrist: u64) -> Option<&PawnEntry> {
        let idx = (zobrist as usize) & self.mask;
        self.table[idx].as_ref().filter(|e| e.zobrist == zobrist)
    }
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let target_size_bytes = size_mb * 1024 * 1024;
        let size = target_size_bytes / entry_size;
        Self {
            table: vec![None; size],
            mask: size - 1,
            age: 0,
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        let idx = (entry.zobrist as usize) & self.mask;
        // Replace if new entry is deeper or from a newer search
        let replace = match &self.table[idx] {
            Some(old) => entry.depth > old.depth || entry.age > old.age,
            None => true,
        };
        if replace {
            self.table[idx] = Some(entry);
        }
    }

    pub fn probe(&self, zobrist: u64) -> Option<&TTEntry> {
        let idx = (zobrist as usize) & self.mask;
        self.table[idx].as_ref().filter(|e| e.zobrist == zobrist)
    }

    pub fn next_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
}