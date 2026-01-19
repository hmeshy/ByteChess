# Byte – A Rust Chess Engine Without Piece-Square Evaluation

**Byte** is an actively developed chess engine written in Rust.  
It’s my personal project to both:

1. Learn how to code in Rust.  
2. Explore how strong a chess engine can be **without** structural evaluation features like piece-square tables, instead focusing on dynamic positional factors like mobility and activity.

Currently, I estimate the engine’s strength to be between **2000-2100 Elo** (as of version 2.0).

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
- (As of v2.0) Tuned Parameters via Texel's Tuning Method

---

## Notes
- No piece-square tables are used in evaluation by design.
- Development is ongoing, with regular improvements to speed, evaluation, and search.

