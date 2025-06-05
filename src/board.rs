use crate::util;
use crate::util::{Color,Move,MoveFlag};

// Enum for bitboard piece tables
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BBPiece {
    White,
    Black,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

// Board structure
pub struct Board {
    pub bitboards: [u64; 8], //8 bitboards, accessed via the enum
    pub move_color: Color,
    pub castling_rights: [bool; 4], // [White King, White Queen, Black King, Black Queen]
    pub en_passant: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

pub const STARTING_POSITION: Board = Board {
    bitboards: [
        0x000000000000FFFF, // White Pieces
        0xFFFF000000000000, // Black Pieces
        0x00FF00000000FF00, // Pawns
        0x4200000000000042, // Knights
        0x2400000000000024, // Bishops
        0x8100000000000081, // Rooks
        0x0800000000000008, // Queens
        0x1000000000000010, // Kings
    ],
    move_color: Color::White,
    castling_rights: [true, true, true, true],
    en_passant: None,
    halfmove_clock: 0,
    fullmove_number: 1,
};

//Move tags
pub enum Tags {
    QuietMove,
    DoublePush,
    KingSideCastle,
    QueenSideCastle,
    Capture,
    EnPassant,

}

// make move function (as UCI) - given a from and to square, move the piece to the new square, and empty the previous square (accepts square name inputs)
// assumes that a move is legal
pub fn make_move(board: &mut Board, _move: & Move) -> Result<(), String> {
    let from_index = _move.from_square();
    let to_index = _move.to_square();
    let flags = _move.flags();

    // Check if castling eligibility has changed
    // Check which piece has moved
    if board.bitboards[BBPiece::King as usize] & (1 << from_index) != 0 {
        // King moved, update castling rights
        if board.move_color == Color::White {
            board.castling_rights[0] = false; // White King-side
            board.castling_rights[1] = false; // White Queen-side
        } else {
            board.castling_rights[2] = false; // Black King-side
            board.castling_rights[3] = false; // Black Queen-side
        }
    } else if board.bitboards[BBPiece::Rook as usize] & (1 << from_index) != 0 {
        // Rook moved, update castling rights
        if board.move_color == Color::White {
            if from_index == util::sq_to_idx("a1") as u8 {
                board.castling_rights[1] = false; // White Queen-side
            } else if from_index == util::sq_to_idx("h1") as u8 {
                board.castling_rights[0] = false; // White King-side
            }
        } else {
            if from_index == util::sq_to_idx("a8") as u8 {
                board.castling_rights[3] = false; // Black Queen-side
            } else if from_index == util::sq_to_idx("h8") as u8 {
                board.castling_rights[2] = false; // Black King-side
            }
        }
    }
    
    // Zero the from_index and replace the to_index in every bitboard
    for i in 0..8 {
        // Set the to square to the value of the from square
        if board.bitboards[i] & (1 << from_index) != 0 {
            board.bitboards[i] &= !(1 << from_index); // Clear the from square
            board.bitboards[i] |= 1 << to_index; // Set the to square
        } else if board.bitboards[i] & (1 << to_index) != 0 {
            // We are capturing from a different bitboard, clear the to square
            board.bitboards[i] &= !(1 << to_index);
        }
    }
    if flags & 0x8 != 0 { // Pawn Promotion
        match (flags & 0x3) { // Get Promotion Piece
            0 => board.bitboards[BBPiece::Knight as usize] |= 1 << to_index, // Promote to Queen
            1 => board.bitboards[BBPiece::Bishop as usize] |= 1 << to_index, // Promote to Rook
            2 => board.bitboards[BBPiece::Rook as usize] |= 1 << to_index, // Promote to Bishop
            3 => board.bitboards[BBPiece::Queen as usize] |= 1 << to_index, // Promote to Knight
            _ => return Err("Invalid promotion type".to_string()),
        }
        board.bitboards[BBPiece::Pawn as usize] &= !(1 << from_index); // Remove the pawn from the board
    }
    if flags == MoveFlag::EnPassant as u8 { // En passant
        // Clear the captured pawn
        let captured_pawn_index = if board.move_color == Color::White {
            to_index + 8 // Captured pawn is one rank below
        } else {
            to_index - 8 // Captured pawn is one rank above
        };
        board.bitboards[BBPiece::Pawn as usize] &= !(1 << captured_pawn_index);
    }
    if flags & 0x2 != 0 && flags & 0xC == 0 { // Castling
        if from_index == util::sq_to_idx("e1") as u8 && to_index == util::sq_to_idx("g1") as u8 { // White King-side castle
            board.bitboards[BBPiece::Rook as usize] &= !(1 << util::sq_to_idx("h1")); // Clear the rook
            board.bitboards[BBPiece::Rook as usize] |= 1 << util::sq_to_idx("f1"); // Move the rook to f1
        } else if from_index == util::sq_to_idx("e1") as u8 && to_index == util::sq_to_idx("c1") as u8 { // White Queen-side castle
            board.bitboards[BBPiece::Rook as usize] &= !(1 << util::sq_to_idx("a1")); // Clear the rook
            board.bitboards[BBPiece::Rook as usize] |= 1 << util::sq_to_idx("d1"); // Move the rook to d1
        } else if from_index == util::sq_to_idx("e8") as u8 && to_index == util::sq_to_idx("g8") as u8 { // Black King-side castle
            board.bitboards[BBPiece::Rook as usize] &= !(1 << util::sq_to_idx("h8")); // Clear the rook
            board.bitboards[BBPiece::Rook as usize] |= 1 << util::sq_to_idx("f8"); // Move the rook to f8
        } else if from_index == util::sq_to_idx("e8") as u8 && to_index == util::sq_to_idx("c8") as u8 { // Black Queen-side castle
            board.bitboards[BBPiece::Rook as usize] &= !(1 << util::sq_to_idx("a8")); // Clear the rook
            board.bitboards[BBPiece::Rook as usize] |= 1 << util::sq_to_idx("d8"); // Move the rook to d8
        }
    }

    // Update other board state information
    board.move_color = if board.move_color == Color::White { Color::Black } else { Color::White };
    board.fullmove_number += if board.move_color == Color::White { 1 } else { 0 };
    // TODO: Update castling rights, en passant square, and halfmove clock

    Ok(())
}
// Note: We will use a PSEUDOLEGAL move generator and check legality later