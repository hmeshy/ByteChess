use crate::board::BBPiece;
use crate::board::Board;
use crate::{board, PIECE_VALUES};
use crate::MOBILITY_VALUES;


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
fn captured_piece(board: &Board, m: &Move) -> Option<BBPiece> {
    if m.flags() & MoveFlag::Capture as u8 != 0 {
        let to_square = m.to_square() as usize;
        for (i, &bb) in board.bitboards.iter().enumerate() {
            if i == BBPiece::White as usize || i == BBPiece::Black as usize {
                continue; // Skip color bitboards
            } else if bb_get(bb, to_square) {
                return Some(BBPiece::from(i));
            }
        }
    }
    None
}

pub fn get_ordered_moves(board: &Board, captures_only: bool) -> Vec<Move> {
    let mut moves = board.gen_moves(captures_only);
    moves.sort_by(|a, b| {
        let value_a = captured_piece(&board, a)
            .map(|piece| PIECE_VALUES[piece as usize])
            .unwrap_or(0);
        let value_b = captured_piece(&board, b)
            .map(|piece| PIECE_VALUES[piece as usize])
            .unwrap_or(0);
        value_b.cmp(&value_a)
    });
    moves
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
        last_move: None,
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

#[inline]
pub(crate) fn bb_print(bb: u64) -> () {
    let mut result = String::new();
    for rank in (0..8).rev() {
        for file in 0..8 {
            let i = rank * 8 + file;
            if bb_get(bb, i) {
                result.push('1');
            } else {
                result.push('0');
            }
            if file != 7 {
                result.push(' ');
            }
        }
        result.push('\n');
    }
    print!("{}", result);
}

pub fn evaluate(board: &board::Board) -> i32 {
    let mut score = 0;
    let mut w_board = board.clone();
    let mut b_board = board.clone();
    w_board.move_color = 1; // Set to white for evaluation
    b_board.move_color = -1; // Set to black for evaluation
    let w_moves = w_board.gen_moves(false);
    let b_moves = b_board.gen_moves(false);
    let w_attacks = get_piece_dist(w_moves, &w_board);
    let b_attacks = get_piece_dist(b_moves, &b_board);
    for i in 0..12{
        // for now piece values, next mobility too!
        let piece = BBPiece::from((i%6)+2);
        let colorbb = if i < 6 { BBPiece::White } else { BBPiece::Black };
        let is_white = colorbb == BBPiece::White;
        let piece_value = PIECE_VALUES[piece as usize];
        let mobility_value = MOBILITY_VALUES[piece as usize];
        let piece_attacks = if is_white { w_attacks } else { b_attacks };
        let attack_value = piece_attacks[piece as usize];
        let mut bitboard = board.combined([piece, colorbb], true);
        let mut partial_score = 0;
        partial_score += piece_value * bitboard.count_ones() as i32 + mobility_value * attack_value as i32;
        if (piece == BBPiece::Pawn) {
            while bitboard != 0 {
                let square = bb_gs_low_bit(&mut bitboard);
                partial_score += 3 * if is_white { (square / 8 - 1) as i32 } else { (7 - square / 8) as i32 };
            }
        }
        score += if is_white { partial_score } else { -partial_score };
        partial_score = 0; // Reset for next piece
    }
    score * board.move_color as i32 // Adjust score based on the current player's color
}
fn get_piece_dist(moves: Vec<Move>, bd: &board::Board) -> [u32; 8] {
    let mut piece_attacks = [0u32; 8];
    for m in moves {
        let from_square = m.from_square() as usize;
        // find which piece is moving
        let mut piece = BBPiece::Pawn;
        for i in 2..8 {
            if bb_get(bd.bitboards[i], from_square) {
                piece = BBPiece::from(i);
                break;
            }
        }
        let piece_index = (piece as usize);
        piece_attacks[piece_index] += 1;
    }
    piece_attacks
}
pub fn perft(bd: &mut board::Board, depth: u8, captures_only: bool) -> u64 {
    let mut count = 0;
    for m in bd.gen_moves(captures_only) {
        let mut bd_copy = bd.clone();
        board::make_move(&mut bd_copy,&m);
        if !bd_copy.king_is_attacked(){
            //println!("{}",m);
            if depth > 1 {
                count += perft(&mut bd_copy,depth - 1, captures_only);
            } else {
                count += 1;
            }    
        }
    }
    count
}