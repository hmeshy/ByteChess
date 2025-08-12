use std::collections::HashMap;
use crate::util::Move;

// The type of bound stored in the table
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Bound {
    Exact,      // PV-Node: exact score
    Lower,      // Cut-Node: lower bound (beta cutoff)
    Upper,      // All-Node: upper bound (fail high)
}

// A transposition table entry
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub zobrist: u64,      // Zobrist hash of the position
    pub best_move: Option<Move>, // Best/refutation move found
    pub depth: i32,        // Search depth at which this entry was stored
    pub score: i32,        // Score (can be mate distance encoded)
    pub bound: Bound,      // Node type (Exact, Lower, Upper)
    pub age: u8,           // For replacement strategy
}
#[derive(Copy, Clone)]
pub struct PawnEntry {
    pub zobrist: u64,      // Zobrist hash of the position
    pub score: i32,        // Score 
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
        let size = 1 << 22; // 4M entries
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
    pub fn new() -> Self {
        let size = 1 << 24; // 16M entries (adjust as needed)
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