extern crate strum;
extern crate strum_macros;

use crate::util;
use crate::util::{Color,Move,MoveFlag};
use self::strum_macros::EnumIter;
use self::strum::IntoEnumIterator;

// Enum for bitboard piece tables
#[derive(Copy, Clone, PartialEq, Eq, EnumIter)]
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

// Helper to convert usize to BBPiece
impl BBPiece {
    pub(crate) fn from(idx: usize) -> Self {
        match idx {
            2 => BBPiece::Pawn,
            3 => BBPiece::Knight,
            4 => BBPiece::Bishop,
            5 => BBPiece::Rook,
            6 => BBPiece::Queen,
            7 => BBPiece::King,
            _ => panic!("Invalid piece index"),
        }
    }
}

// Board structure
pub struct Board {
    pub bitboards: [u64; 8], //8 bitboards, accessed via the enum
    pub move_color: i8, // 1 for White, -1 for Black
    pub castling_rights: [bool; 4], // [White King, White Queen, Black King, Black Queen]
    pub en_passant: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
}

impl Board {
    // Bitboard util functions
    #[inline]
    pub fn get(&self, pieces: impl IntoIterator<Item = BBPiece>, square: impl Into<usize>) -> bool {
        let square = square.into();
        for piece in pieces {
            if !util::bb_get(self.bitboards[piece as usize], square) {
                return false;
            }
        }
        true
    }

    #[inline]
    pub fn set(&mut self, pieces: impl IntoIterator<Item = BBPiece>, square: impl Into<usize>, value: bool) {
        let square = square.into();
        for piece in pieces {
            util::bb_set(&mut self.bitboards[piece as usize], square, value);
        }
    }

    #[inline]
    pub fn move_piece(&mut self, pieces: impl IntoIterator<Item = BBPiece>, from: impl Into<usize>, to: impl Into<usize>) {
        let from = from.into();
        let to = to.into();
        for piece in pieces {
            self.set([piece], from, false);
            self.set([piece], to, true);
        }
    }

    #[inline]
    pub fn combined(&self, pieces: impl IntoIterator<Item = BBPiece>, exclusive: bool) -> u64 {
        let mut iter = pieces.into_iter();
        let mut combined = if let Some(first) = iter.next() {
            self.bitboards[first as usize]
        } else {
            return 0;
        };
        for piece in iter {
            if exclusive {
            combined &= self.bitboards[piece as usize];
            } else {
            combined |= self.bitboards[piece as usize];
            }
        }
        combined
    }
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
    move_color: Color::White as i8,
    castling_rights: [true, true, true, true],
    en_passant: None,
    halfmove_clock: 0,
    fullmove_number: 1,
};

// make move function (as UCI) - given a from and to square, move the piece to the new square, and empty the previous square (accepts square name inputs)
// assumes that a move is legal, tracks other FEN changes
pub fn make_move(board: &mut Board, _move: & Move) -> Result<(), String> {
    let from_index = _move.from_square();
    let to_index = _move.to_square();
    let flags = _move.flags();

    // Check if castling eligibility has changed
    // Check which piece has moved
    if board.get([BBPiece::King], from_index) {
        // King moved, update castling rights
        board.castling_rights[(1-board.move_color) as usize] = false; // First castling rights for color
        board.castling_rights[(2-board.move_color) as usize] = false; // Second castling rights for color
    } else if board.get([BBPiece::Rook], from_index) {
        // Rook moved, update castling rights
        // Todo optimize with color var
        if board.move_color == Color::White as i8 {
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
    for i in BBPiece::iter() {
        if board.get([i], from_index) {
            board.move_piece([i], from_index, to_index);
        } else if board.get([i], to_index) {
            // We are capturing from a different bitboard, clear the to square
            board.set([i], to_index, false);
        }
    }
    if flags & 0x8 != 0 { // Pawn Promotion
        match flags & 0x3 { // Get Promotion Piece
            0 => board.set([BBPiece::Knight], to_index, true),
            1 => board.set([BBPiece::Bishop], to_index, true),
            2 => board.set([BBPiece::Rook], to_index, true),
            3 => board.set([BBPiece::Queen], to_index, true),
            _ => return Err("Invalid promotion type".to_string()),
        }
        board.set([BBPiece::Pawn], to_index, false); // Remove the pawn from the board
    }
    if flags == MoveFlag::EnPassant as u8 { // En passant
        // Clear the captured pawn
        let captured_pawn_index = to_index - 8 * board.move_color as u8;
        board.set([BBPiece::Pawn, BBPiece::White, BBPiece::Black], captured_pawn_index, false); // Clear the captured pawn
    }
    if flags & 0x2 != 0 && flags & 0xC == 0 { // Castling
        if from_index == util::sq_to_idx("e1") as u8 && to_index == util::sq_to_idx("g1") as u8 { // White King-side castle
            board.move_piece([BBPiece::Rook, BBPiece::White], util::sq_to_idx("h1"), util::sq_to_idx("f1")); 
        } else if from_index == util::sq_to_idx("e1") as u8 && to_index == util::sq_to_idx("c1") as u8 { // White Queen-side castle
            board.move_piece([BBPiece::Rook, BBPiece::White], util::sq_to_idx("a1"), util::sq_to_idx("d1"));
        } else if from_index == util::sq_to_idx("e8") as u8 && to_index == util::sq_to_idx("g8") as u8 { // Black King-side castle
            board.move_piece([BBPiece::Rook, BBPiece::Black], util::sq_to_idx("h8"), util::sq_to_idx("f8")); 
        } else if from_index == util::sq_to_idx("e8") as u8 && to_index == util::sq_to_idx("c8") as u8 { // Black Queen-side castle
            board.move_piece([BBPiece::Rook, BBPiece::Black], util::sq_to_idx("a8"), util::sq_to_idx("d8")); 
        }
    }

    // Update other board state information
    board.move_color *= -1;
    if board.move_color == Color::White as i8 {
        board.fullmove_number += 1;
    }
    
    // Check En Passant square
    if flags == MoveFlag::DoublePush as u8 {
        //TODO: Check if en passant is legal in the next move, first
        board.en_passant = Some((from_index as i8 - 8 * board.move_color) as usize); // Set en passant target square
    } else {
        board.en_passant = None; // Clear en passant target square
    }

    // Reset halfmove clock if a pawn is moved or a capture is made
    if flags & (MoveFlag::Capture as u8) != 0 || board.get([BBPiece::Pawn], to_index) {
        board.halfmove_clock = 0; // Reset halfmove clock
    } else {
        board.halfmove_clock += 1; // Increment halfmove clock
    }
    Ok(())
}

// Pseudolegal move generator
impl Board {
    pub fn gen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        for i in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
            let combined_bb = self.combined([i, color_bb], true);
            util::bb_print(combined_bb);
            // for all - generate start/end squares, get proper flag
            match i {
                BBPiece::Pawn => {
                    // Generate pawn moves
                    // Pushing 
                    // Single push (check if blocked))
                    // Double push (check if on inital rank, and if blocked)
                    // Captures (check if opponent piece diagonal left or right, board wrapping checks)
                    // En passant (check if en passant can happen))
                    // Handle promotion if last rank
                }
                BBPiece::Knight => {
                    // Generate knight moves
                    // Use the geometry of the knight to find all possible moves, filter for moving out of the board
                }
                BBPiece::Bishop => {
                    // Generate bishop moves
                    // Queen move gen but just diagonals
                }
                BBPiece::Rook => {
                    // Generate rook moves
                    // Queen move gen but just horizontals
                }
                BBPiece::Queen => {
                    // Generate queen moves
                    // Queen / slider move generation
                    // Check all directions (horizontal, vertical, diagonal) until hitting a piece or the edge of the board
                    // If hitting a piece, check if it's an opponent piece to capture
                }
                BBPiece::King => {
                    // Generate king moves
                    // Check all adjacent squares (8 directions)
                    // If the square is empty or occupied by an opponent piece (and not past border), add the move
                    // If castling rights exist, check if no pieces are in between the king and rook
                    // we avoid any check logic here, just generate all moves
                }
            }
        }
        moves
    }
}