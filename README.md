# Byte – A Rust Chess Engine Without Piece-Square Evaluation

**Byte** is an actively developed chess engine written in Rust.  
It’s my personal project to both:

1. Learn how to code in Rust.  
2. Explore how strong a chess engine can be **without** piece-square evaluation features like piece-square tables.

Currently, I estimate the engine’s strength to be around **~1800 Elo** (as of version 1.3).

---

## Getting Started

**Download & Run**

- **Option 1:** Download the `.exe` file from the latest release and run it directly.
- **Option 2:** Clone the repository and run:

```bash
cargo run --release
```
## Features

### **Move Generation**
- Bitboards
- Magic Bitboards
- Position History & State Tracking

### **Search**
- Negamax with Alpha-Beta Pruning
- Zobrist Hash Tables
- Quiescence Search
- Null Move Pruning
- Late Move Reduction (LMR)
- Aspiration Windows
- Move Ordering via:
  - MVV-LVA
  - Killer Heuristic

### **Evaluation**
- Piece Values
- Mobility
- Basic Pawn Structure
- Basic King Safety

---

## Notes
- No piece-square tables are used in evaluation by design.
- Development is ongoing, with regular improvements to speed, evaluation, and search.

