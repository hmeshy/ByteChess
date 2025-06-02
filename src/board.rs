use crate::util;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White = 1,
    Black = -1,
    None = 0,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Piece {
    _type: PieceType,
    color: Color,
}

// squares, move color, castling rights, en passant square, halfmove clock (50 move rule), fullmove number
pub struct Board {
    squares: [Piece; 64],
    move_color: Color,
    castling_rights: [bool; 4], // [White King, White Queen, Black King, Black Queen]
    en_passant: Option<usize>,
    halfmove_clock: u8,
    fullmove_number: u16,
}

// Print function for Board
impl std::fmt::Display for Board {
    // Prints in FEN format
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Piece placement
        for row in (0..8).rev() {
            let mut empty_count = 0;
            for col in 0..8 {
            let piece = &self.squares[row * 8 + col];
            if piece._type == PieceType::None {
                empty_count += 1;
            } else {
                if empty_count > 0 {
                write!(f, "{}", empty_count)?;
                empty_count = 0;
                }
                let symbol = match piece._type {
                PieceType::Pawn => 'P',
                PieceType::Knight => 'N',
                PieceType::Bishop => 'B',
                PieceType::Rook => 'R',
                PieceType::King => 'K',
                PieceType::Queen => 'Q',
                PieceType::None => '.', // Should not happen here
                };
                let symbol = match piece.color {
                Color::White => symbol,
                Color::Black => symbol.to_ascii_lowercase(),
                Color::None => '.', // Should not happen here
                };
                write!(f, "{}", symbol)?;
            }
            }
            if empty_count > 0 {
            write!(f, "{}", empty_count)?;
            }
            if row != 0 {
            write!(f, "/")?;
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

pub const STARTING_POSITION: Board = Board {
    squares: [
        Piece { _type: PieceType::Rook, color: Color::White },
        Piece { _type: PieceType::Knight, color: Color::White },
        Piece { _type: PieceType::Bishop, color: Color::White },
        Piece { _type: PieceType::Queen, color: Color::White },
        Piece { _type: PieceType::King, color: Color::White },
        Piece { _type: PieceType::Bishop, color: Color::White },
        Piece { _type: PieceType::Knight, color: Color::White },
        Piece { _type: PieceType::Rook, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::Pawn, color: Color::White },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::None, color: Color::None },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Pawn, color: Color::Black },
        Piece { _type: PieceType::Rook, color: Color::Black },
        Piece { _type: PieceType::Knight, color: Color::Black },
        Piece { _type: PieceType::Bishop, color: Color::Black },
        Piece { _type: PieceType::Queen, color: Color::Black },
        Piece { _type: PieceType::King, color: Color::Black },
        Piece { _type: PieceType::Bishop, color: Color::Black },
        Piece { _type: PieceType::Knight, color: Color::Black },
        Piece { _type: PieceType::Rook, color: Color::Black }
    ],
    move_color: Color::White,
    castling_rights: [true, true, true, true],
    en_passant: None,
    halfmove_clock: 0,
    fullmove_number: 1,
};

// make move function (as UCI) - given a from and to square, move the piece to the new square, and empty the previous square (accepts square name inputs)
// assumes that a move is legal
pub fn make_move(board: &mut Board, from: &str, to: &str) -> Result<(), String> {
    let from_index = util::sq_to_idx(from);
    let to_index = util::sq_to_idx(to);

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