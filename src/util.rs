use crate::board::BBPiece;
use crate::board::Board;
use crate::{board, PIECE_VALUES, PIECE_VALUES_EG, MOBILITY_VALUES, MOBILITY_VALUES_EG};
const DOUBLED_PAWN_PENALTY: i32 = 1;
const ISOLATED_PAWN_PENALTY: i32 = 5;
const PAWN_ADVANCE_BONUS: i32 = 3;
const PAWN_ISLAND_PENALTY: i32 = 2;
// King safety constants
const KING_SAFETY_TABLE: [i32; 100] = [
    0,   0,   1,   2,   3,   5,   7,   9,  12,  15,
   18,  22,  26,  30,  35,  39,  44,  50,  56,  62,
   68,  75,  82,  85,  89,  97, 105, 113, 122, 131,
  140, 150, 169, 180, 191, 202, 213, 225, 237, 248,
  260, 272, 283, 295, 307, 319, 330, 342, 354, 366,
  377, 389, 401, 412, 424, 436, 448, 459, 471, 483,
  494, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
  500, 500, 500, 500, 500, 500, 500, 500, 500, 500
];

// Attack weights for different piece types
const ATTACK_WEIGHTS: [i32; 8] = [0, 0, 0, 1, 1, 2, 4, 0]; // Knight, Bishop, Rook, Queen

// Color Enum
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White = 1,
    Black = -1,
    None = 0, // Used for empty squares
}

// 4 bits are used for flags -- this is what they represent
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MoveFlag {
    Quiet = 0,
    DoublePush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    KnightPromotion = 8,
    BishopPromotion = 9,
    RookPromotion = 10,
    QueenPromotion = 11,
    KnightPromoCapture = 12,
    BishopPromoCapture = 13,
    RookPromoCapture = 14,
    QueenPromoCapture = 15,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Move {
    pub info: u16, // 6 bits for from and to, 4 bits for extra info (promotion, capture, en passant, castling)
}
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq)]
pub struct MoveStack {
    data: [Move; 218],
    len: usize,
}
impl MoveStack {
    pub fn new() -> Self {
        Self {
            data: [Move::null(); 218],
            len: 0,
        }
    }
    pub fn clear(&mut self) {
        for i in 0..self.len {
            self.data[i] = Move::null();
        }
        self.len = 0;
    }
    pub fn extend<I: IntoIterator<Item = Move>>(&mut self, iter: I) {
        for mv in iter {
            if self.len < self.data.len() {
                self.data[self.len] = mv;
                self.len += 1;
            } else {
                break; // Stop if capacity is reached
            }
        }
    }
    pub fn first(&self) -> Move {
        if self.len > 0 {
            self.data[0]
        } else {
            Move { info: 0 }
        }
    }
    pub fn push(&mut self, mv: Move) -> Result<(), &'static str> {
        if self.len < 218 {
            self.data[self.len] = mv;
            self.len += 1;
            Ok(())
        } else {
            Err("Stack full")
        }
    }

    pub fn pop(&mut self) -> Move {
        if self.len > 0 {
            self.len -= 1;
            self.data[self.len]
        } else {
            Move{info:0}
        }
    }
    pub fn move_to_front(&mut self, index: usize) {
        if index >= self.len {
            return;
        }
        // Remove the move at index
        let mv = self.remove(index);
        // Insert it at the front
        let _ = self.insert(0, mv);
    }
        /// Inserts a move at the given index, shifting elements to the right.
    pub fn insert(&mut self, index: usize, mv: Move) -> Result<(), &'static str> {
        if self.len >= self.data.len() {
            return Err("MoveStack is full");
        }
        if index > self.len {
            return Err("Index out of bounds");
        }
        // Shift elements to the right
        for i in (index..self.len).rev() {
            self.data[i + 1] = self.data[i];
        }
        self.data[index] = mv;
        self.len += 1;
        Ok(())
    }

    /// Removes and returns the move at the given index, shifting elements to the left.
    pub fn remove(&mut self, index: usize) -> Move {
        if index >= self.len {
            return Move { info: 0 };
        }
        let removed = self.data[index];
        for i in index..self.len - 1 {
            self.data[i] = self.data[i + 1];
        }
        self.len -= 1;
        self.data[self.len] = Move::null();
        removed
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.data[..self.len].iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Move> {
        self.data[..self.len].iter_mut()
    }
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Move) -> bool,
    {
        let mut new_len = 0;
        for i in 0..self.len {
            if f(&self.data[i]) {
                if new_len != i {
                    self.data[new_len] = self.data[i];
                }
                new_len += 1;
            }
        }
        // Set the rest to Move::null()
        for i in new_len..self.len {
            self.data[i] = Move::null();
        }
        self.len = new_len;
    }
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&Move, &Move) -> std::cmp::Ordering,
    {
        self.data[..self.len].sort_by(|a, b| compare(a, b));
    }
    pub fn sort_by_capture_value<F>(&mut self, mut captured_piece: F)
    where
        F: FnMut(&Move) -> Option<BBPiece>,
    {
        self.sort_by(|a, b| {
            let value_a = captured_piece(a)
                .map(|piece| PIECE_VALUES[piece as usize])
                .unwrap_or(0);
            let value_b = captured_piece(b)
                .map(|piece| PIECE_VALUES[piece as usize])
                .unwrap_or(0);
            value_b.cmp(&value_a)
        });
    }
    pub fn score_moves<F>(&mut self, mut score_fn: F)
    where
        F: FnMut(&Move) -> i32,
    {
        let mut scored: Vec<(i32, Move)> = self.data[..self.len]
            .iter()
            .map(|m| (score_fn(m), *m))
            .collect();
        scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));
        for (i, &(_, m)) in scored.iter().enumerate() {
            self.data[i] = m;
        }
    }
}

impl std::fmt::Display for Move {
    /// Displays the move in UCI format (e.g., "a2a4")
     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = idx_to_sq(self.from_square() as usize);
        let to = idx_to_sq(self.to_square() as usize);
        let promo = match self.flags() {
            x if x == MoveFlag::KnightPromotion as u8 || x == MoveFlag::KnightPromoCapture as u8 => Some('n'),
            x if x == MoveFlag::BishopPromotion as u8 || x == MoveFlag::BishopPromoCapture as u8 => Some('b'),
            x if x == MoveFlag::RookPromotion as u8   || x == MoveFlag::RookPromoCapture as u8   => Some('r'),
            x if x == MoveFlag::QueenPromotion as u8  || x == MoveFlag::QueenPromoCapture as u8  => Some('q'),
            _ => None,
        };
        match promo {
            Some(p) => write!(f, "{}{}{}", from, to, p),
            None => write!(f, "{}{}", from, to),
        }
    }
}
// Methods to interact with the move data
impl Move {
    #[inline]
    pub fn from_parts(m_from: u8, m_to: u8, flags: u8) -> Self {
        let info = ((flags as u16 & 0xf) << 12) | ((m_from as u16 & 0x3f) << 6) | (m_to as u16 & 0x3f);
        Move { info }
    }
    #[inline]
    pub fn null() -> Self {
        Move { info: 0 }
    }
    #[inline]
    pub fn from_square(&self) -> u8 {
        ((self.info >> 6) & 0x3f) as u8
    }

    #[inline]
    pub fn to_square(&self) -> u8 {
        (self.info & 0x3f) as u8
    }

    #[inline]
    pub fn flags(&self) -> u8 {
        ((self.info >> 12) & 0xf) as u8
    }

    #[inline]
    pub fn set_from_square(&mut self, m_from: u8) {
        self.info = (self.info & !(0x3f << 6)) | (((m_from as u16) & 0x3f) << 6);
    }

    #[inline]
    pub fn set_to_square(&mut self, m_to: u8) {
        self.info = (self.info & !0x3f) | ((m_to as u16) & 0x3f);
    }

    #[inline]
    pub fn set_flags(&mut self, flags: u8) {
        self.info = (self.info & !(0xf << 12)) | (((flags as u16) & 0xf) << 12);
    }
    pub fn score_move(&self, m: &Move, tt_move: Option<Move>, killer_moves: &[Move; 2], attacking_piece: Option<BBPiece>, captured_piece: Option<BBPiece>) -> i32 {
        let mut score = 0;
        
        // 1. Hash/TT move gets highest priority
        if let Some(tt_move) = tt_move {
            if *m == tt_move {
                return 1_000_000;
            }
        }
        
        // 2. Winning captures (MVV-LVA: Most Valuable Victim - Least Valuable Attacker)
        if m.flags() & MoveFlag::Capture as u8 != 0 {
            if let Some(captured) = captured_piece && let Some(attacker) = attacking_piece {
                let victim_value = PIECE_VALUES[captured as usize];
                let attacker_value = PIECE_VALUES[attacker as usize];
                
                // MVV-LVA scoring
                score += 10000 + victim_value * 10 - attacker_value;
                
                // Bonus for capturing with less valuable pieces
                if victim_value >= attacker_value {
                    score += 5000; // Good capture
                }
            }
        }
        
        // 3. Promotions
        if m.flags() & 8 as u8 != 0 {
            score += 8000;
        }
        
        // 4. Killer moves (non-captures that caused beta cutoffs)
        if m.flags() & MoveFlag::Capture as u8 == 0 {
            if *m == killer_moves[0] {
                score += 7000;
            } else if *m == killer_moves[1] {
                score += 6000;
            }
        }
        score
    }
}

// Converts a position to an integer index ("a1" --> 0, "h8" --> 63)
pub(crate) fn sq_to_idx(pos: &str) -> usize {
    let mut chars = pos.chars();
    let col = chars.next().unwrap() as usize - 'a' as usize;
    let row = chars.next().unwrap() as usize - '1' as usize;
    (row * 8 + col) as usize
}

// Constants for all squares
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Squares {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

// Implement conversion from Squares to usize
impl From<Squares> for usize {
    fn from(sq: Squares) -> Self {
        sq as u8 as usize
    }
}

impl From<Squares> for u8 {
    fn from(sq: Squares) -> Self {
        sq as u8
    }
}

// And vice versa
pub(crate) fn idx_to_sq(idx: usize) -> String {
    let row = idx / 8;
    let col = idx % 8;
    format!("{}{}", (col as u8 + 'a' as u8) as char, (row as u8 + '1' as u8) as char)
}

// Helper function to get FEN piece char for a given piece and color
pub(crate) fn piece_to_fen(piece: BBPiece, color: Color) -> char {
    match (piece, color) {
        (BBPiece::Pawn, Color::White) => 'P',
        (BBPiece::Knight, Color::White) => 'N',
        (BBPiece::Bishop, Color::White) => 'B',
        (BBPiece::Rook, Color::White) => 'R',
        (BBPiece::Queen, Color::White) => 'Q',
        (BBPiece::King, Color::White) => 'K',
        (BBPiece::Pawn, Color::Black) => 'p',
        (BBPiece::Knight, Color::Black) => 'n',
        (BBPiece::Bishop, Color::Black) => 'b',
        (BBPiece::Rook, Color::Black) => 'r',
        (BBPiece::Queen, Color::Black) => 'q',
        (BBPiece::King, Color::Black) => 'k',
        _ => ' ',
    }
}

//Make function for board
pub fn board_from_fen(fen: &str) -> board::Board {
    use crate::board::{Board, BBPiece};
    let mut bitboards = [0u64; 8];
    let mut piece_moves = [0u32; 8];
    let mut castling_rights = [false; 4];
    let mut en_passant = None;
    let mut halfmove_clock = 0u8;
    let mut fullmove_number = 1u16;
    let mut move_color = 1;

    let parts: Vec<&str> = fen.split_whitespace().collect();
    assert!(parts.len() >= 4, "Invalid FEN string");

    // Piece placement
    let mut sq = 56; // Start at a8
    for c in parts[0].chars() {
        match c {
            '/' => sq -= 16,
            '1'..='8' => sq += c.to_digit(10).unwrap() as usize,
            'P' => { bitboards[BBPiece::Pawn as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'N' => { bitboards[BBPiece::Knight as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'B' => { bitboards[BBPiece::Bishop as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'R' => { bitboards[BBPiece::Rook as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'Q' => { bitboards[BBPiece::Queen as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'K' => { bitboards[BBPiece::King as usize] |= 1 << sq; bitboards[BBPiece::White as usize] |= 1 << sq; sq += 1; }
            'p' => { bitboards[BBPiece::Pawn as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            'n' => { bitboards[BBPiece::Knight as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            'b' => { bitboards[BBPiece::Bishop as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            'r' => { bitboards[BBPiece::Rook as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            'q' => { bitboards[BBPiece::Queen as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            'k' => { bitboards[BBPiece::King as usize] |= 1 << sq; bitboards[BBPiece::Black as usize] |= 1 << sq; sq += 1; }
            _ => {}
        }
    }

    // Active color
    move_color = match parts[1] {
        "w" => 1,
        "b" => -1,
        _ => 1,
    };

    // Castling rights
    if parts[2].contains('K') { castling_rights[0] = true; }
    if parts[2].contains('Q') { castling_rights[1] = true; }
    if parts[2].contains('k') { castling_rights[2] = true; }
    if parts[2].contains('q') { castling_rights[3] = true; }

    // En passant
    if parts[3] != "-" {
        en_passant = Some(crate::util::sq_to_idx(parts[3]));
    }

    // Halfmove clock
    if parts.len() > 4 {
        halfmove_clock = parts[4].parse().unwrap_or(0);
    }

    // Fullmove number
    if parts.len() > 5 {
        fullmove_number = parts[5].parse().unwrap_or(1);
    }

    Board {
        bitboards,
        move_color,
        castling_rights,
        en_passant,
        halfmove_clock,
        fullmove_number,
        zobrist_hash: 0u64, //generated AFTER making a board
        moves: MoveStack::new(),
        state_history: Vec::new(),
        move_history: Vec::new(),
        captures_history: Vec::new(),
        position_history: Vec::new(),
    }
}
// Print function for Board
impl std::fmt::Display for board::Board {
    // Prints in FEN format

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Piece placement
        // To be implemented using the bitboards
        // Returns the FEN piece placement string for the board
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq = rank * 8 + file;
                let mut found = false;
                for (i, &bb) in self.bitboards.iter().enumerate() {
                    if i == BBPiece::White as usize || i == BBPiece::Black as usize {
                        continue; // skip color bitboards
                    }
                    if (bb >> sq) & 1 == 1 {
                        // Determine color
                        let color = if (self.bitboards[BBPiece::White as usize] >> sq) & 1 == 1 {
                            Color::White
                        } else {
                            Color::Black
                        };
                        let piece = piece_to_fen(BBPiece::from(i), color);
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }
                        fen.push(piece);
                        found = true;
                        break;
                    }
                }
                if !found {
                    empty += 1;
                }
            }
            if empty > 0 {
                fen.push_str(&empty.to_string());
            }
            if rank != 0 {
                fen.push('/');
            }
        }
        write!(f, "{}", fen)?;
        // Active color
        write!(
            f,
            " {}",
            match self.move_color {
                1 => "w",
                -1 => "b",
                _ => "-", // Should not happen
            }
        )?;

        // Castling rights
        let mut castling = String::new();
        if self.castling_rights[0] { castling.push('K'); }
        if self.castling_rights[1] { castling.push('Q'); }
        if self.castling_rights[2] { castling.push('k'); }
        if self.castling_rights[3] { castling.push('q'); }
        if castling.is_empty() {
            castling.push('-');
        }
        write!(f, " {}", castling)?;

        // En passant target square
        let en_passant = if let Some(idx) = self.en_passant {
            idx_to_sq(idx)
        } else {
            "-".to_string()
        };
        write!(f, " {}", en_passant)?;

        // Halfmove clock and fullmove number
        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)?;

        Ok(())
    }
}

// Bitboard util functions
#[inline]
pub(crate) fn bb_get(bb: u64, square: usize) -> bool {
    if square >= 64 {
        panic!("Square index out of bounds: {}", square);
    }
    (bb & (1 << square)) != 0
}
#[inline]
pub(crate) fn bb_set(bb: &mut u64, square: usize, value: bool) {
    if value {
        *bb |= 1 << square;
    } else {
        *bb &= !(1 << square);
    }
}
#[inline]
pub(crate) fn bb_gs_low_bit(bb: &mut u64) -> usize {
    if *bb == 0 {
        return 64; // Return an invalid index if the bitboard is empty
    }
    let low_bit = bb.trailing_zeros() as usize;
    // Clear the lowest bit
    *bb &= !(1 << low_bit);
    low_bit
}
pub fn evaluate(board: &board::Board) -> i32 {
    let phase = board.get_phase();
    let material_score = material_score(board, phase);
    let mobility_score = mobility_score(board, phase);
    let king_safety_score = ((1.0 - phase) * king_safety_score(board) as f32).round() as i32;
    let king_edge_score = (phase * king_edge(board) as f32).round() as i32;
    let pawn_structure_score = pawn_struct_score(board);
    return (material_score + mobility_score + king_safety_score + pawn_structure_score) * board.move_color as i32;
}
pub fn king_edge(board: &board::Board) -> i32 {
    let white_distance = king_distance_to_corner(board, true);
    let black_distance = king_distance_to_corner(board, false);
    
    // Return difference (closer to corner = higher penalty)
    (white_distance - black_distance) * 20
}

fn king_distance_to_corner(board: &board::Board, is_white: bool) -> i32 {
    let king_color = if is_white { BBPiece::White } else { BBPiece::Black };
    
    // Find king position
    let king_bb = board.bitboards[BBPiece::King as usize] & board.bitboards[king_color as usize];
    if king_bb == 0 {
        return 0; // No king (shouldn't happen)
    }
    
    let mut king_bb_copy = king_bb;
    let king_square = bb_gs_low_bit(&mut king_bb_copy);
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    
    // Calculate distance to each corner and return the minimum
    let corners = [
        (0, 0),  // a1
        (0, 7),  // a8  
        (7, 0),  // h1
        (7, 7),  // h8
    ];
    
    let mut min_distance = 3;
    
    for &(corner_file, corner_rank) in &corners {
        // Use Chebyshev distance (king move distance)
        let distance = (king_file as i32 - corner_file).abs().max(
                      (king_rank as i32 - corner_rank).abs());
        min_distance = min_distance.min(distance);
    }
    
    min_distance
}
pub fn print_eval(board: &board::Board) {
    let phase = board.get_phase();
    let material_score = material_score(board, phase);
    let mobility_score = mobility_score(board, phase);
    let king_safety_score = king_safety_score(board);
    let pawn_structure_score = pawn_struct_score(board);
    println!("Material Score: {}", material_score* board.move_color as i32);
    println!("Mobility Score: {}", mobility_score* board.move_color as i32);
    println!("Pawn Structure Score: {}", pawn_structure_score * board.move_color as i32);
    println!("King Safety Score: {}", king_safety_score * board.move_color as i32);
    println!("Total Evaluation: {}", (material_score + mobility_score + king_safety_score + pawn_structure_score)* board.move_color as i32);
}
fn material_score(board: &board::Board, phase: f32) -> i32 {
    let mut score = 0;
    
    // More efficient: iterate through piece types directly
    for piece_type in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
        let piece_value;
        piece_value = ((phase * PIECE_VALUES_EG[piece_type as usize] as f32) + ((1.0 - phase) * PIECE_VALUES[piece_type as usize] as f32)).round() as i32;
        let white_pieces = board.combined([piece_type, BBPiece::White], true);
        let black_pieces = board.combined([piece_type, BBPiece::Black], true);
        let white_count = white_pieces.count_ones() as i32;
        let black_count = black_pieces.count_ones() as i32;
        score += piece_value * (white_count - black_count);
    }
    score
}

fn mobility_score(board: &board::Board, phase: f32) -> i32 {
    let mut score = 0;
    let (w_attacks, b_attacks) = board.compute_mobility(phase);
    for i in 3..7 {
        let white_mobility = w_attacks[i] as i32;
        let black_mobility = b_attacks[i] as i32;
        score += white_mobility - black_mobility;
    }
    score
}
fn pawn_struct_score(board: &board::Board) -> i32 {
    let white_pawns = board.combined([BBPiece::Pawn, BBPiece::White], true);
    let black_pawns = board.combined([BBPiece::Pawn, BBPiece::Black], true);
    pawn_evaluation(white_pawns, true) - pawn_evaluation(black_pawns, false)
}
fn eg_pawn_score(board: &board::Board) -> i32 {
    let white_pawns = board.combined([BBPiece::Pawn, BBPiece::White], true);
    let black_pawns = board.combined([BBPiece::Pawn, BBPiece::Black], true);
    eg_pawn_evaluation(white_pawns, true) - eg_pawn_evaluation(black_pawns, false)
}
fn eg_pawn_evaluation(pawn_bb: u64, is_white: bool) -> i32 {
    if pawn_bb == 0 {
        return 0;
    }
    
    let mut score = 0;
    let mut pawns_per_file = [0u8; 8];
    let mut bitboard = pawn_bb;
    
    // First pass: count pawns per file and add advancement bonuses
    while bitboard != 0 {
        let square = bb_gs_low_bit(&mut bitboard);
        let rank = (square / 8) as u8;
        let file = (square % 8) as usize;
        
        pawns_per_file[file] += 1;
        
        // Pawn advancement bonus =
        let advancement = if is_white {
            2_i32.pow((rank as u32).saturating_sub(1))
        } else {
            2_i32.pow((6u32).saturating_sub(rank as u32))
        };
        score += PAWN_ADVANCE_BONUS * 3 * (advancement as i32);
    }
    
    // Second pass: evaluate pawn structure
    for file in 0..8 {
        let pawn_count = pawns_per_file[file];
        if pawn_count == 0 {
            continue;
        }
        
        // Doubled/tripled pawn penalty (exponential)
        if pawn_count > 1 {
            score -= DOUBLED_PAWN_PENALTY * 3 *  (pawn_count as i32 - 1) * (pawn_count as i32 - 1);
        }
        
        // Isolated pawn penalty
        let has_support = (file > 0 && pawns_per_file[file - 1] > 0) || 
                         (file < 7 && pawns_per_file[file + 1] > 0);
        if !has_support {
            score -= ISOLATED_PAWN_PENALTY;
        }
    }
    score
}
fn pawn_evaluation(pawn_bb: u64, is_white: bool) -> i32 {
    if pawn_bb == 0 {
        return 0;
    }
    
    let mut score = 0;
    let mut pawns_per_file = [0u8; 8];
    let mut bitboard = pawn_bb;
    
    // First pass: count pawns per file and add advancement bonuses
    while bitboard != 0 {
        let square = bb_gs_low_bit(&mut bitboard);
        let rank = (square / 8) as u8;
        let file = (square % 8) as usize;
        
        pawns_per_file[file] += 1;
        
        // Pawn advancement bonus =
        let advancement = if is_white {
            2_i32.pow((rank as u32).saturating_sub(1))
        } else {
            2_i32.pow((6u32).saturating_sub(rank as u32))
        };
        score += PAWN_ADVANCE_BONUS * (advancement as i32);
    }
    
    // Second pass: evaluate pawn structure
    for file in 0..8 {
        let pawn_count = pawns_per_file[file];
        if pawn_count == 0 {
            continue;
        }
        
        // Doubled/tripled pawn penalty (exponential)
        if pawn_count > 1 {
            score -= DOUBLED_PAWN_PENALTY * (pawn_count as i32 - 1) * (pawn_count as i32 - 1);
        }
        
        // Isolated pawn penalty
        let has_support = (file > 0 && pawns_per_file[file - 1] > 0) || 
                         (file < 7 && pawns_per_file[file + 1] > 0);
        if !has_support {
            score -= ISOLATED_PAWN_PENALTY;
        }
    }
    
    // Pawn islands penalty
    let pawn_islands = count_pawn_islands(&pawns_per_file);
    if pawn_islands > 1 {
        score -= PAWN_ISLAND_PENALTY * (pawn_islands as i32 - 1);
    }
    
    score
}

fn count_pawn_islands(pawns_per_file: &[u8; 8]) -> u8 {
    let mut islands = 0;
    let mut in_island = false;
    
    for &pawn_count in pawns_per_file {
        if pawn_count > 0 {
            if !in_island {
                islands += 1;
                in_island = true;
            }
        } else {
            in_island = false;
        }
    }
    islands
}
pub fn king_safety_score(board: &board::Board) -> i32 {
    let white_score = evaluate_king_safety(board, true);
    let black_score = evaluate_king_safety(board, false);
    white_score - black_score
}

fn evaluate_king_safety(board: &board::Board, is_white: bool) -> i32 {
    let king_color = if is_white { BBPiece::White } else { BBPiece::Black };
    let enemy_color = if is_white { BBPiece::Black } else { BBPiece::White };
    
    // Find king position
    let king_bb = board.bitboards[BBPiece::King as usize] & board.bitboards[king_color as usize];
    if king_bb == 0 {
        return 0; // No king (shouldn't happen)
    }
    
    let mut king_bb_copy = king_bb;
    let king_square = bb_gs_low_bit(&mut king_bb_copy);
    
    let mut attack_units = 0;
    let mut attackers = 0;
    
    // Get king zone (king + surrounding squares)
    let king_zone = get_king_zone(king_square);
    
    // Check attacks from enemy pieces
    for piece_type in 3..7 { // Knight, Bishop, Rook, Queen
        let enemy_pieces = board.bitboards[piece_type] & board.bitboards[enemy_color as usize];
        let mut piece_bb = enemy_pieces;
        
        while piece_bb != 0 {
            let piece_square = bb_gs_low_bit(&mut piece_bb);
            let attacks = board.get_piece_attacks(piece_type, piece_square);
            
            if attacks & king_zone != 0 {
                attackers += 1;
                let zone_attacks = (attacks & king_zone).count_ones() as i32;
                attack_units += ATTACK_WEIGHTS[piece_type] * zone_attacks;
            }
        }
    }
    
    // Bonus for multiple attackers
    if attackers >= 2 {
        attack_units += 3;
    }
    if attackers >= 3 {
        attack_units += 5 * (attackers - 2); // More attackers, more bonus;
    }
    
    // Pawn shelter bonus/penalty
    let shelter_penalty = evaluate_pawn_shelter(board, king_square, is_white);
    attack_units += shelter_penalty;
    
    // Convert attack units to score using safety table
    let index = std::cmp::min(attack_units as usize, 99);
    -KING_SAFETY_TABLE[index] // Negative because this is penalty for our king
}

fn get_king_zone(king_square: usize) -> u64 {
    let mut zone = board::KING_ATTACKS[king_square];
    zone |= 1u64 << king_square; // Include king square itself
    zone
}

fn evaluate_pawn_shelter(board: &board::Board, king_square: usize, is_white: bool) -> i32 {
    let king_file = king_square % 8;
    let king_rank = king_square / 8;
    let mut penalty = 0;
    if king_file >= 3 && king_file <= 5 { // on d,e,f files / not castled
        penalty += 3; 
    }
    let own_pawns = if is_white {
        board.combined([BBPiece::Pawn, BBPiece::White], true)
    } else {
        board.combined([BBPiece::Pawn, BBPiece::Black], true)
    };
    
    // Check files around king (king file and adjacent files)
    for file_offset in -1i32..=1i32 {
        let file = (king_file as i32 + file_offset) as usize;
        if file >= 8 { continue; }
        
        let mut has_pawn = false;
        let mut closest_pawn_distance = 8;
        
        // Look for pawns in front of king on this file
        for rank in 0..8 {
            let square = rank * 8 + file;
            if own_pawns & (1u64 << square) != 0 {
                has_pawn = true;
                let distance = if is_white {
                    (rank as i32 - king_rank as i32).abs()
                } else {
                    (king_rank as i32 - rank as i32).abs()
                };
                if distance < closest_pawn_distance {
                    closest_pawn_distance = distance;
                }
            }
        }
        
        if !has_pawn {
            penalty += 6; // No pawn shield on this file
        } else if closest_pawn_distance > 2 {
            penalty += 3 * (closest_pawn_distance - 2); // Pawn too far away
        }
    }
    penalty
}

pub fn perft(bd: &mut board::Board, depth: u8) -> u64 {
    let mut count = 0;
    bd.gen_moves(true);
    let moves = bd.moves;
    if depth <= 1
    {
        return moves.len() as u64
    }
    for m in moves.iter() {
        let orig = bd.clone();
        board::make_move(bd, m);
        if depth > 1 {
            count += perft(bd, depth - 1);
        } else {
            count += 1;
        }
        board::undo_move(bd);
    }
    count
}