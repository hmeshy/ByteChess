use crate::util;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum enumPiece {
    bWhite,
    bBlack,
    bPawn,
    bKnight,
    bBishop,
    bRook,
    bQueen,
    bKing,
}
// squares, move color, castling rights, en passant square, halfmove clock (50 move rule), fullmove number
pub struct Board {
    bitboards: [U64; 8],
    move_color: Color,
    castling_rights: [bool; 4], // [White King, White Queen, Black King, Black Queen]
    en_passant: Option<usize>,
    halfmove_clock: u8,
    fullmove_number: u16,
}
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
// ...existing code...
pub struct Move {
    Move(u8 m_from, u8 m_to, u8 flags) {
      info = ((flags & 0xf)<<12) | ((m_from & 0x3f)<<6) | (m_to & 0x3f);
   }
    info: u16, // 6 bits for from and to, 4 bits for extra info (promotion, capture, en passant, castling)
}

pub enum Tags {
    QuietMove,
    DoublePush,
    KingSideCastle,
    QueenSideCastle,
    Capture,
    EnPassant,

}
// Print function for Board
impl std::fmt::Display for Board {
    // Prints in FEN format
    // Helper function to get FEN piece char for a given piece and color
    fn piece_to_fen(piece: enumPiece, color: Color) -> char {
            match (piece, color) {
                (enumPiece::bPawn, Color::White) => 'P',
                (enumPiece::bKnight, Color::White) => 'N',
                (enumPiece::bBishop, Color::White) => 'B',
                (enumPiece::bRook, Color::White) => 'R',
                (enumPiece::bQueen, Color::White) => 'Q',
                (enumPiece::bKing, Color::White) => 'K',
                (enumPiece::bPawn, Color::Black) => 'p',
                (enumPiece::bKnight, Color::Black) => 'n',
                (enumPiece::bBishop, Color::Black) => 'b',
                (enumPiece::bRook, Color::Black) => 'r',
                (enumPiece::bQueen, Color::Black) => 'q',
                (enumPiece::bKing, Color::Black) => 'k',
                _ => ' ',
            }
        }
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
                for (i, &bb) in board.bitboards.iter().enumerate() {
                    if i == enumPiece::bWhite as usize || i == enumPiece::bBlack as usize {
                        continue; // skip color bitboards
                    }
                    if (bb >> sq) & 1 == 1 {
                        // Determine color
                        let color = if (board.bitboards[enumPiece::bWhite as usize] >> sq) & 1 == 1 {
                            Color::White
                        } else {
                            Color::Black
                        };
                        let piece = piece_to_fen(enumPiece::from(i), color);
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
        // Helper to convert usize to enumPiece
        impl enumPiece {
            fn from(idx: usize) -> Self {
                match idx {
                    2 => enumPiece::bPawn,
                    3 => enumPiece::bKnight,
                    4 => enumPiece::bBishop,
                    5 => enumPiece::bRook,
                    6 => enumPiece::bQueen,
                    7 => enumPiece::bKing,
                    _ => panic!("Invalid piece index"),
                }
            }
        }
        // Active color
        write!(
            f,
            " {}",
            match self.move_color {
            Color::White => "w",
            Color::Black => "b",
            Color::None => "-", // Should not happen
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
            util::idx_to_sq(idx)
        } else {
            "-".to_string()
        };
        write!(f, " {}", en_passant)?;

        // Halfmove clock and fullmove number
        write!(f, " {} {}", self.halfmove_clock, self.fullmove_number)?;

        Ok(())
    }
}

// make move function (as UCI) - given a from and to square, move the piece to the new square, and empty the previous square (accepts square name inputs)
// assumes that a move is legal
pub fn make_move(board: &mut Board, move: & Move) -> Result<(), String> {
    let from_index = //from part of move;
    let to_index = //to part of move;;

    if board.squares[from_index]._type == PieceType::None {
        return Err(format!("No piece at {}", from));
    }

    // Move the piece
    board.squares[to_index] = board.squares[from_index];
    board.squares[from_index] = Piece { _type: PieceType::None, color: Color::None };

    // Update other board state information
    board.move_color = if board.move_color == Color::White { Color::Black } else { Color::White };
    board.fullmove_number += if board.move_color == Color::White { 1 } else { 0 };
    // TODO: Update castling rights, en passant square, and halfmove clock

    Ok(())
}

// MOVE GENERATION
const JUMP_OFFSETS = [
    [(1,0)], // Pawn
    [(-1,2),(-2,1),(1,2),(2,1),(-1,-2),(-2,-1),(1,-2),(2,-1)], // Knight
    [], //Bishop
    [], // Rook
    [], // Queen
    [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)], // King
]
const SLIDE_OFFSETS = [
    [], // Pawn
    [], // Knight
    [(-1,-1),(-1,1),(1,-1),(1,1)], // Bishop
    [(-1,0),(0,-1),(0,1),(1,0)], // Rook
    [(-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)], // Queen
    [], // King
]
// Note: We will use a PSEUDOLEGAL move generator and check legality later