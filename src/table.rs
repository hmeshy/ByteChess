use std::collections::HashMap;
use crate::util::Move;

// The type of bound stored in the table
#[derive(Copy, Clone, PartialEq, Eq)]
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

// The transposition table itself
pub struct TranspositionTable {
    pub table: HashMap<u64, TTEntry>,
    pub age: u8,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::with_capacity(1 << 20), // 1M entries, adjust as needed
            age: 0,
        }
    }

    pub fn store(&mut self, entry: TTEntry) {
        // Replace if new entry is deeper or from a newer search
        let replace = match self.table.get(&entry.zobrist) {
            Some(old) => entry.depth > old.depth || entry.age > old.age,
            None => true,
        };
        if replace {
            self.table.insert(entry.zobrist, entry);
        }
    }

    pub fn probe(&self, zobrist: u64) -> Option<&TTEntry> {
        self.table.get(&zobrist)
    }

    pub fn next_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }
}