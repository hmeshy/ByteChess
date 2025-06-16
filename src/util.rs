use crate::board::BBPiece;
use crate::board;

// Color Enum
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    White = 1,
    Black = -1,
    None = 0, // Used for empty squares
}

// Moves

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
pub fn perft(bd: &mut board::Board, depth: u8) -> u64 {
    let mut count = 0;
    for m in bd.gen_moves() {
        let mut bd_copy = bd.clone();
        board::make_move(&mut bd_copy,&m);
        if !bd_copy.king_is_attacked(){
            //println!("{}",m);
            if depth > 1 {
                count += perft(&mut bd_copy,depth - 1);
            } else {
                count += 1;
            }    
        }
    }
    count
}