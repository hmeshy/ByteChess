# ByteChess Engine Writeup

This document explains the current engine behavior in the codebase: what is in evaluation, and how search is structured.

*This document was generated with AI (GPT 5.3 Codex)*

## Overview

ByteChess is a bitboard engine with:

- Magic-bitboard move generation
- Negamax alpha-beta search
- Iterative deepening at root
- Transposition table (TT) and pawn-hash table
- Quiescence search
- Null move pruning
- Late move reduction (LMR)
- Aspiration windows
- Tapered middlegame/endgame evaluation

The main runtime search path is in `src/byte.rs`, with evaluation in `src/util.rs` and move generation/order support in `src/board.rs`.

## Search Structure

### 1. Root Search (`think`)

At `go`, the engine calls `think(...)` and:

1. Advances TT age and resets per-move search counters.
2. Generates ordered root moves.
3. Runs iterative deepening from depth 0 upward until time runs out (or depth cap is reached).
4. Uses aspiration windows centered on previous iteration score.
5. Prints UCI info including score, depth, nodes, time, and PV.

If only one legal move exists, it returns immediately.

### 2. Core Recursive Search (`minimax`)

`minimax` is negamax alpha-beta:

- Node accounting increments every call.
- Draw detection is checked early.
- TT is probed before move search:
  - `Exact` nodes can return immediately.
  - `Lower` and `Upper` bounds can cutoff if they prove beta/alpha.
  - TT move is retained for move ordering / hash-move try-first logic.
- At depth 0 it transitions to quiescence (`minimax_captures`).

### 3. Pruning and Reductions

Current pruning/reduction components:

- **Null move pruning**
  - Enabled when not in pawn endgame, depth is high enough, and side is not in check.
  - Makes a null move, searches reduced depth, and beta-cuts if the null result is high enough.

- **Late move reduction (LMR)**
  - For later moves (index-based) at adequate depth, first searches reduced depth.
  - If promising (beats alpha), re-searches full depth.

- **Time checks inside search**
  - Every 1024 nodes, checks if allocated think time is exceeded and exits safely.

### 4. Move Ordering

Move ordering combines:

- TT move priority
- MVV-LVA style capture scoring
- Promotion bonus
- Killer moves (two per ply)

Non-capture ordering uses a score function in `Move::score_move`; capture-only quiescence ordering uses captured-piece value sorting.

### 5. Quiescence (`minimax_captures`)

At leaf depth:

- Evaluates stand-pat static score.
- Performs alpha-beta checks on stand-pat.
- Searches capture-only continuations (with a depth guard based on searched depth).

This reduces horizon effects versus pure fixed-depth static evaluation.

### 6. TT and Pawn Hash Tables

- **Main TT (`TranspositionTable`)**
  - Stores zobrist key, best move, depth, score, bound type, and age.
  - Uses replacement based on deeper or newer entries.
  - Age increments each root move.

- **Pawn table (`PawnTable`)**
  - Caches pawn-structure evaluation by pawn hash.
  - Avoids recomputing expensive pawn-structure terms at many nodes.

## Evaluation Structure

Top-level evaluation is:

`(material + mobility + king_safety + king_edge + pawn_structure).taper(phase) * side_to_move`

Where:

- `Score` stores middlegame (`mg`) and endgame (`eg`) values.
- `phase` (0..255) blends mg/eg.
- Final score is from side-to-move perspective.

### 1. Material

Material is maintained incrementally on make/undo move via piece values and reused directly in eval.

### 2. Mobility

Mobility is computed for both sides, then differenced:

- Counts available attacks/moves per piece type
- Uses per-piece mobility weights
- Includes king mobility term

### 3. Pawn Structure

Pawn structure evaluation includes:

- Pawn advancement bonus (rank-based growth)
- Passed pawn detection (same file + adjacent files ahead clear of enemy pawns)
- Passed pawn scoring:
  - Base value
  - Rank bonus (more advanced passed pawns get more)
  - Protected passed pawn bonus
- Doubled pawn penalty (scaled with count)
- Isolated pawn penalty

The final pawn score is white minus black and is cached in the pawn hash table.

### 4. King Safety

King safety is computed per side and differenced:

1. Build king zone = king square plus adjacent king-neighbor squares.
2. For each enemy knight/bishop/rook/queen:
   - Generate attacks
   - If attacks hit king zone, add weighted attack units.
3. Add extra attack units when multiple attackers are present.
4. Add pawn-shelter penalties near king files:
   - Missing shield pawn penalty
   - Far shield pawn penalty
5. Convert accumulated attack units through a non-linear king safety lookup table.

This produces larger penalties as local king pressure grows.

### 5. King Edge Term

King-edge logic evaluates each king's distance to nearest corner and uses a tuned center/corner weight.

Current behavior is phase-gated:

- No effect before late-game threshold.
- Smooth ramp-in across configured late phases.
- Fully active in deeper endgames.

This keeps king-edge guidance mostly endgame-focused.

## Move Generation and Board Core

Board representation and movegen details:

- Separate piece and color bitboards
- Magic attack tables for sliding pieces
- Precomputed attack maps for king/knight
- Full state tracking for make/undo:
  - Castling rights
  - En passant
  - Halfmove/fullmove counters
  - Position history
  - Pawn hash and zobrist hash

Legality checks are done by making moves and verifying own king safety.

## Tuning Support

The repository also includes a tuner path (`src/tuner.rs`, `src/tunereval.rs`) for parameter optimization (Texel-style workflow).  
The main play path uses `util::evaluate`, while tuning uses `tunereval::evaluate` with tunable parameter structs.

## Current Character of the Engine

In practical terms, the engine currently emphasizes:

- Fast bitboard move generation
- Tactical pruning/search efficiency
- Dynamic activity (mobility + king pressure)
- Pawn-structure and king-safety heuristics without piece-square tables

This aligns with the project goal of strong play driven by dynamic factors rather than PST-heavy static structure.
