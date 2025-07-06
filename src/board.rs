extern crate strum;
extern crate strum_macros;

use std::u8::MAX;

use crate::magic;
use crate::util;
use crate::util::{Color,Move,MoveFlag,Squares};
use self::strum_macros::EnumIter;
use self::strum::IntoEnumIterator;
use crate::{board, PIECE_VALUES};
use crate::zobrist::{ZOBRIST_CASTLING,ZOBRIST_EP,ZOBRIST_PIECES,ZOBRIST_SIDE};

pub const KNIGHT_ATTACKS: [u64; 64] =[0x0000000000020400, 0x0000000000050800, 0x00000000000a1100, 0x0000000000142200, 0x0000000000284400, 0x0000000000508800, 0x0000000000a01000, 0x0000000000402000,
0x0000000002040004, 0x0000000005080008, 0x000000000a110011, 0x0000000014220022, 0x0000000028440044, 0x0000000050880088, 0x00000000a0100010, 0x0000000040200020,
0x0000000204000402, 0x0000000508000805, 0x0000000a1100110a, 0x0000001422002214, 0x0000002844004428, 0x0000005088008850, 0x000000a0100010a0, 0x0000004020002040,
0x0000020400040200, 0x0000050800080500, 0x00000a1100110a00, 0x0000142200221400, 0x0000284400442800, 0x0000508800885000, 0x0000a0100010a000, 0x0000402000204000,
0x0002040004020000, 0x0005080008050000, 0x000a1100110a0000, 0x0014220022140000, 0x0028440044280000, 0x0050880088500000, 0x00a0100010a00000, 0x0040200020400000,
0x0204000402000000, 0x0508000805000000, 0x0a1100110a000000, 0x1422002214000000, 0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000,
0x0400040200000000, 0x0800080500000000, 0x1100110a00000000, 0x2200221400000000, 0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000,
0x0004020000000000, 0x0008050000000000, 0x00110a0000000000, 0x0022140000000000, 0x0044280000000000, 0x0088500000000000, 0x0010a00000000000, 0x0020400000000000];
pub const KING_ATTACKS: [u64; 64] =[0x0000000000000302, 0x0000000000000705, 0x0000000000000e0a, 0x0000000000001c14, 0x0000000000003828, 0x0000000000007050, 0x000000000000e0a0, 0x000000000000c040,
0x0000000000030203, 0x0000000000070507, 0x00000000000e0a0e, 0x00000000001c141c, 0x0000000000382838, 0x0000000000705070, 0x0000000000e0a0e0, 0x0000000000c040c0,
0x0000000003020300, 0x0000000007050700, 0x000000000e0a0e00, 0x000000001c141c00, 0x0000000038283800, 0x0000000070507000, 0x00000000e0a0e000, 0x00000000c040c000,
0x0000000302030000, 0x0000000705070000, 0x0000000e0a0e0000, 0x0000001c141c0000, 0x0000003828380000, 0x0000007050700000, 0x000000e0a0e00000, 0x000000c040c00000,
0x0000030203000000, 0x0000070507000000, 0x00000e0a0e000000, 0x00001c141c000000, 0x0000382838000000, 0x0000705070000000, 0x0000e0a0e0000000, 0x0000c040c0000000,
0x0003020300000000, 0x0007050700000000, 0x000e0a0e00000000, 0x001c141c00000000, 0x0038283800000000, 0x0070507000000000, 0x00e0a0e000000000, 0x00c040c000000000,
0x0302030000000000, 0x0705070000000000, 0x0e0a0e0000000000, 0x1c141c0000000000, 0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000,
0x0203000000000000, 0x0507000000000000, 0x0a0e000000000000, 0x141c000000000000, 0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000];


// Enum for bitboard piece tables
#[derive(Copy, Clone, PartialEq, Eq, EnumIter, Debug)]
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
#[derive(Clone)]
#[derive(PartialEq, Eq)]
pub struct Board {
    pub bitboards: [u64; 8], // 8 bitboards, accessed via the enum
    pub move_color: i8, // 1 for White, -1 for Black
    pub castling_rights: [bool; 4], // [White King, White Queen, Black King, Black Queen]
    pub en_passant: Option<usize>,
    pub halfmove_clock: u8,
    pub fullmove_number: u16,
    pub zobrist_hash: u64,
    pub moves: util::MoveStack,
    pub state_history: Vec<([bool; 4], Option<usize>, u8)>,
    // Move history: stack of all moves
    pub move_history: Vec<Move>,
    // Captures history: stack of (from_piece, to_piece) for each capture
    pub captures_history: Vec<(BBPiece, BBPiece)>,
    pub position_history: Vec<u64>,
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

//info to store for unmaking moves
//bitboards - need to undo
//move color - just * -1
//fullmove_number - same rules but in reverse
//castling rights & en passant, half_move clock, move history, captured pieces - need to be stored and retrieve
pub fn is_check(board: &mut Board) -> bool {
        // Check if the current player is in checkmate
        board.move_color = -board.move_color; // Reverse the move color to check if the opponent's king is attacked
        let mut ret = true;
        if !board.king_is_attacked() {
            ret = false
        }
        board.move_color = -board.move_color; // Reverse the move color back!!
        ret
    }
// Helper: get piece index for Zobrist table
fn zobrist_piece_index(piece: BBPiece, color: BBPiece) -> usize {
    let piece_idx = match piece {
        BBPiece::Pawn => 0,
        BBPiece::Knight => 1,
        BBPiece::Bishop => 2,
        BBPiece::Rook => 3,
        BBPiece::Queen => 4,
        BBPiece::King => 5,
        _ => panic!("Invalid piece for zobrist"),
    };
    let color_idx = match color {
        BBPiece::White => 0,
        BBPiece::Black => 1,
        _ => panic!("Invalid color for zobrist"),
    };
    piece_idx + 6 * color_idx
}
/// Converts [White King, White Queen, Black King, Black Queen] castling rights to a usize (0..16)
pub fn castling_rights_to_bits(castling_rights: &[bool; 4]) -> usize {
    (castling_rights[0] as usize) << 0 |
    (castling_rights[1] as usize) << 1 |
    (castling_rights[2] as usize) << 2 |
    (castling_rights[3] as usize) << 3
}
// make move function (as UCI) - given a from and to square, move the piece to the new square, and empty the previous square (accepts square name inputs)
// assumes that a move is legal, tracks other FEN changes
pub fn make_move(board: &mut Board, _move: & Move) -> Result<(), String> {
    let from_index = _move.from_square();
    let to_index = _move.to_square();
    let flags = _move.flags();
    board.state_history.push((
        board.castling_rights,
        board.en_passant,
        board.halfmove_clock,
    )); //update state history
    // Check if castling eligibility has changed
    // Check which piece has moved
    board.position_history.push(board.zobrist_hash);
    board.zobrist_hash ^= ZOBRIST_CASTLING[castling_rights_to_bits(&board.castling_rights)];
    if board.get([BBPiece::King], from_index) {
        // King moved, update castling rights
        board.castling_rights[(1-board.move_color) as usize] = false; // First castling rights for color
        board.castling_rights[(2-board.move_color) as usize] = false; // Second castling rights for color
    } if board.get([BBPiece::Rook], from_index) {
        // Rook moved, update castling rights
        // Todo optimize with color var
        if board.move_color == Color::White as i8 {
            if from_index == Squares::A1 as u8 {
                board.castling_rights[1] = false; // White Queen-side
            } else if from_index == Squares::H1 as u8 {
                board.castling_rights[0] = false; // White King-side
            }
        } else {
            if from_index == Squares::A8 as u8 {
                board.castling_rights[3] = false; // Black Queen-side
            } else if from_index == Squares::H8 as u8 {
                board.castling_rights[2] = false; // Black King-side
            }
        }
    } if board.get([BBPiece::Rook], to_index) {
        // Capturing a rook possibly on a corner square, update castling rights for the opponent
        if to_index == Squares::A1 as u8 {
            board.castling_rights[1] = false; // White Queen-side
        } else if to_index == Squares::H1 as u8 {
            board.castling_rights[0] = false; // White King-side
        } else if to_index == Squares::A8 as u8 {
            board.castling_rights[3] = false; // Black Queen-side
        } else if to_index == Squares::H8 as u8 {
            board.castling_rights[2] = false; // Black King-side
        }
    }
    board.zobrist_hash ^= ZOBRIST_CASTLING[castling_rights_to_bits(&board.castling_rights)];
    let mut captured: Option<(BBPiece, BBPiece)> = None;

    // Handle capture and move the piece
    for color in [BBPiece::White, BBPiece::Black] {
        if board.get([color], to_index) {
            // If a color bitboard has a piece on the destination, that's the color of the captured piece
            for piece in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
                if board.get([piece], to_index) {
                    captured = Some((color, piece));
                    board.set([piece], to_index, false); // Remove captured piece
                    board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(piece, color)][to_index as usize];
                    break;
                }
            }
            // Remove the color bit as well
            board.set([color], to_index, false);
        }
    }

    // Move the piece from from_index to to_index on its respective bitboard
    let mut _piece = BBPiece::Pawn;
    for piece in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
        if board.get([piece], from_index) {
            board.set([piece], from_index, false);
            board.set([piece], to_index, true);
            _piece = piece;
            break;
        }
    }
    // Move the color bit as well
    let mut _color = BBPiece::White;
    let mut opp_color = BBPiece::White;
    for color in [BBPiece::White, BBPiece::Black] {
        if board.get([color], from_index) {
            board.set([color], from_index, false);
            board.set([color], to_index, true);
            _color = color;
            break;
        }
    }
    if _color == BBPiece::White {
        opp_color = BBPiece::Black;
    }
    board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(_piece, _color)][from_index as usize];
    board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(_piece, _color)][to_index as usize];

    if let Some(capture) = captured {
        board.captures_history.push(capture);
    }
    if flags & 0x8 != 0 { // Pawn Promotion
    // Remove pawn from hash
    board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Pawn, _color)][to_index as usize];
    // Remove pawn from board
    board.set([BBPiece::Pawn], to_index, false);

    // Add promoted piece to board and hash
    match flags & 0x3 { // Get Promotion Piece
        0 => {
            board.set([BBPiece::Knight], to_index, true);
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Knight, _color)][to_index as usize];
        },
        1 => {
            board.set([BBPiece::Bishop], to_index, true);
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Bishop, _color)][to_index as usize];
        },
        2 => {
            board.set([BBPiece::Rook], to_index, true);
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, _color)][to_index as usize];
        },
        3 => {
            board.set([BBPiece::Queen], to_index, true);
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Queen, _color)][to_index as usize];
        },
        _ => return Err("Invalid promotion type".to_string()),
    }
}
    if flags == MoveFlag::EnPassant as u8 { // En passant
        // Clear the captured pawn
        let captured_pawn_index = if board.move_color == Color::White as i8 {
            // Black just moved, so en passant capture is one rank down
            to_index.wrapping_sub(8)
        } else {
            // White just moved, so en passant capture is one rank up
            to_index.wrapping_add(8)
        };        
        board.set([BBPiece::Pawn, BBPiece::White, BBPiece::Black], captured_pawn_index, false); // Clear the captured pawn
        board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Pawn, opp_color)][captured_pawn_index as usize];
    }
    if flags & 0x2 != 0 && flags & 0xC == 0 { // Castling
        if from_index == Squares::E1 as u8 && to_index == Squares::G1 as u8 { // White King-side castle
            board.move_piece([BBPiece::Rook, BBPiece::White], Squares::H1, Squares::F1); 
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::White)][Squares::H1 as usize];
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::White)][Squares::F1 as usize];
        } else if from_index == Squares::E1 as u8 && to_index == Squares::C1 as u8 { // White Queen-side castle
            board.move_piece([BBPiece::Rook, BBPiece::White], Squares::A1, Squares::D1);
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::White)][Squares::A1 as usize];
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::White)][Squares::D1 as usize];
        } else if from_index == Squares::E8 as u8 && to_index == Squares::G8 as u8 { // Black King-side castle
            board.move_piece([BBPiece::Rook, BBPiece::Black], Squares::H8, Squares::F8); 
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::Black)][Squares::H8 as usize];
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::Black)][Squares::F8 as usize];
        } else if from_index == Squares::E8 as u8 && to_index == Squares::C8 as u8 { // Black Queen-side castle
            board.move_piece([BBPiece::Rook, BBPiece::Black], Squares::A8, Squares::D8); 
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::Black)][Squares::A8 as usize];
            board.zobrist_hash ^= ZOBRIST_PIECES[zobrist_piece_index(BBPiece::Rook, BBPiece::Black)][Squares::D8 as usize];
        }
    }

    // Update other board state information
    board.move_color *= -1;
    board.zobrist_hash ^= ZOBRIST_SIDE;
    if board.move_color == Color::White as i8 {
        board.fullmove_number += 1;
    }
    
    if let Some(old_ep) = board.en_passant {
        board.zobrist_hash ^= ZOBRIST_EP[old_ep % 8];
    }
    // Check En Passant square
    if flags == MoveFlag::DoublePush as u8 {
        //TODO: Check if en passant is legal in the next move, first
        let ep_square = (from_index as i8 - 8 * board.move_color) as usize;
        board.en_passant = Some(ep_square);
        // Add new en passant to hash
        board.zobrist_hash ^= ZOBRIST_EP[ep_square % 8];    
    } 
    else {
        board.en_passant = None; // Clear en passant target square
    }

    // Reset halfmove clock if a pawn is moved or a capture is made
    if flags & (MoveFlag::Capture as u8) != 0 || board.get([BBPiece::Pawn], to_index) {
        board.halfmove_clock = 0; // Reset halfmove clock
    } else {
        board.halfmove_clock += 1; // Increment halfmove clock
    }
    board.move_history.push(*_move);
    Ok(())
}
pub fn undo_move(board: &mut Board) -> Result<(), String> {
    // Pop the last move
    let _move = match board.move_history.pop() {
        Some(m) => m,
        None => return Err("No move to undo".to_string()),
    };

    let from_index = _move.from_square();
    let to_index = _move.to_square();
    let flags = _move.flags();
    board.zobrist_hash = match board.position_history.pop() {
        Some(hash) => hash,
        None => return Err("No position history to undo".to_string()),
    };
    // Undo move_color and fullmove_number
    if board.move_color == Color::White as i8 {
        // If it's white's turn now, it was black's turn before
        board.fullmove_number -= 1;
    }
    board.move_color *= -1;
    //now, board.move_color is the color of the player who moved the original piece!
    // Undo en passant, castling rights, and halfmove clock from state_history
    if let Some((castling_rights, en_passant, halfmove_clock)) = board.state_history.pop() {
        board.castling_rights = castling_rights;
        board.en_passant = en_passant;
        board.halfmove_clock = halfmove_clock;
    } else {
        return Err("No state history to undo".to_string());
    }
    // Undo pawn promotion
    if flags & 0x8 != 0 {
        // Remove promoted piece
        match flags & 0x3 {
            0 => {
                board.set([BBPiece::Knight], to_index, false);
            },
            1 => {
                board.set([BBPiece::Bishop], to_index, false);
            },
            2 => {
                board.set([BBPiece::Rook], to_index, false);
            }
            3 => {
                board.set([BBPiece::Queen], to_index, false);
            }
            _ => return Err("Invalid promotion type".to_string()),
        }
        // Restore pawn
        board.set([BBPiece::Pawn], to_index, false);
        board.set([BBPiece::Pawn], from_index, true);
        // Restore color bit
        if board.move_color == Color::White as i8 {
            board.set([BBPiece::White], to_index, false);
            board.set([BBPiece::White], from_index, true);
        } else {
            board.set([BBPiece::Black], to_index, false);
            board.set([BBPiece::Black], from_index, true);
        }
    } else if flags == MoveFlag::EnPassant as u8 {
        // Undo en passant capture
        board.move_piece([BBPiece::Pawn, if board.move_color == Color::White as i8 { BBPiece::White } else { BBPiece::Black }], to_index, from_index);
        if board.move_color == Color::White as i8 {
            board.set([BBPiece::Black, BBPiece::Pawn], to_index.wrapping_sub(8), true);
        } else {
            board.set([BBPiece::White, BBPiece::Pawn], to_index.wrapping_add(8), true);
        }
    } else if flags & 0x2 != 0 && flags & 0xC == 0 {
        // Undo castling
        board.move_piece([BBPiece::King, if board.move_color == Color::White as i8 { BBPiece::White } else { BBPiece::Black }], to_index, from_index);
        if board.move_color == Color::White as i8 {
            board.set([BBPiece::White], to_index, false);
            board.set([BBPiece::White], from_index, true);
        } else {
            board.set([BBPiece::Black], to_index, false);
            board.set([BBPiece::Black], from_index, true);
        }
        // Undo rook move
        match (from_index, to_index) {
            (x, y) if x == Squares::E1 as u8 && y == Squares::G1 as u8 => {
                board.move_piece([BBPiece::Rook, BBPiece::White], Squares::F1, Squares::H1);
            }
            (x, y) if x == Squares::E1 as u8 && y == Squares::C1 as u8 => {
                board.move_piece([BBPiece::Rook, BBPiece::White], Squares::D1, Squares::A1);
            }
            (x, y) if x == Squares::E8 as u8 && y == Squares::G8 as u8 => {
                board.move_piece([BBPiece::Rook, BBPiece::Black], Squares::F8, Squares::H8);
            }
            (x, y) if x == Squares::E8 as u8 && y == Squares::C8 as u8 => {
                board.move_piece([BBPiece::Rook, BBPiece::Black], Squares::D8, Squares::A8);
            }
            _ => {}
        }
    } else {
        // Normal move
        for piece in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
            if board.get([piece], to_index) {
                board.move_piece([piece], to_index, from_index);
                break;
            }
        }
        // Move color bit
        if board.move_color == Color::White as i8 {
            if board.get([BBPiece::White], to_index) {
                board.set([BBPiece::White], to_index, false);
                board.set([BBPiece::White], from_index, true);
            }
        } else {
            if board.get([BBPiece::Black], to_index) {
                board.set([BBPiece::Black], to_index, false);
                board.set([BBPiece::Black], from_index, true);
            }
        }
    }

    // Restore captured piece if there was a capture
    if flags & (MoveFlag::Capture as u8) != 0 && flags != MoveFlag::EnPassant as u8 { //we don't store / restore en passant captures this way
        if let Some((color, piece)) = board.captures_history.pop() {
                board.set([piece], to_index, true);
                board.set([color], to_index, true);
        }
    }

    Ok(())
}
fn rook_attacks(square: usize, occupancy: u64) -> u64
{
    let magic = &magic::ROOK_MAGICS[square];
    let blockers = occupancy & magic.mask;
    let hash = ((blockers.wrapping_mul(magic.magic))>> magic.shift) as usize;
    let index = hash as usize + magic.offset;
    magic::ROOK_ATTACKS[index]
}
fn bishop_attacks(square: usize, occupancy: u64) -> u64
{
    let magic = &magic::BISHOP_MAGICS[square];
    let blockers = occupancy & magic.mask;
    let hash = ((blockers.wrapping_mul(magic.magic))>> magic.shift) as usize;
    let index = hash as usize + magic.offset;
    magic::BISHOP_ATTACKS[index]
}
impl Board {
    pub fn get_ordered_moves(&mut self, is_generated: bool, legal_only: bool) -> util::MoveStack {
        if !is_generated{
        self.gen_moves(legal_only);}
        let mut _moves = self.moves;
        _moves.order_by_capture_value(|m: &Move| self.captured_piece(m));
        _moves
    }
    fn captured_piece(&self, m: &Move) -> Option<BBPiece> {
        if m.flags() & MoveFlag::Capture as u8 != 0 {
            let to_square = m.to_square() as usize;
            for (i, &bb) in self.bitboards.iter().enumerate() {
                if i == BBPiece::White as usize || i == BBPiece::Black as usize {
                    continue; // Skip color bitboards
                } else if util::bb_get(bb, to_square) {
                    return Some(BBPiece::from(i));
                }
            }
        }
        None
    }
    // Pseudolegal move generator
    fn is_legal_move(&mut self, m: &Move) -> bool
    {
        board::make_move(self, m);
        let is_legal = !self.king_is_attacked();
        board::undo_move(self);
        is_legal
    }
    fn add_move(&mut self, m: &Move, legal_only: bool)
    {
        if !legal_only || self.is_legal_move(m)
        {
            self.moves.push(*m);
        }
    }
    pub fn captures_only(&mut self)
    {
        self.moves.retain(|m| m.flags() & MoveFlag::Capture as u8 != 0);
    }
    pub fn gen_moves(&mut self, legal_only: bool) {
        self.moves.clear();
        let white = self.move_color == Color::White as i8;
        let color_bb: BBPiece = if white {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let combined_bb: u64 = self.combined([BBPiece::White, BBPiece::Black], false);
        let empty_bb = !combined_bb;
        for i in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
            let mut pc_bb = self.combined([i, color_bb], true);
            // for all - generate start/end squares, get proper flag
            match i {
                BBPiece::Pawn => {
                    let not_a_file = 0xfefefefefefefefeu64;
                    let not_h_file = 0x7f7f7f7f7f7f7f7fu64;
                    let mut single_push = 0 as u64;
                    let mut double_push = 0 as u64;
                    let mut captures_left = 0 as u64;
                    let mut captures_right = 0 as u64;
                    let mut en_passant_left = 0u64;
                    let mut en_passant_right = 0u64;
                    if white
                    {
                        single_push = (pc_bb << 8) & empty_bb;
                        let rank3 = 0x00000000FF0000u64;
                        double_push = ((single_push & rank3) << 8) & empty_bb;
                        let black_pieces = self.bitboards[BBPiece::Black as usize];
                        //code here to add possible en passant square
                        captures_left  = ((pc_bb & not_a_file) << 7) & black_pieces;
                        captures_right = ((pc_bb & not_h_file) << 9) & black_pieces;
                        if let Some(ep_sq) = self.en_passant {
                            let ep_bb = 1u64 << ep_sq;
                            // Left en passant: pawn must be on the file to the right of ep_sq and able to capture left
                            en_passant_left = ((pc_bb & not_a_file) << 7) & ep_bb;
                            // Right en passant: pawn must be on the file to the left of ep_sq and able to capture right
                            en_passant_right = ((pc_bb & not_h_file) << 9) & ep_bb;
                        }   
                        while single_push != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut single_push) as u8;
                            if to_sq >= 56 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq - 8 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromotion as u8,
                                ), legal_only); 
                                self.add_move(&Move::from_parts(
                                to_sq - 8 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromotion as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 8 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromotion as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 8 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromotion as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq - 8 as u8,
                                to_sq as u8,
                                MoveFlag::Quiet as u8,
                                ), legal_only);
                            }

                        }
                        while double_push != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut double_push) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq - 16 as u8,
                                to_sq as u8,
                                MoveFlag::DoublePush as u8,
                                ), legal_only);
                        }
                        while captures_left != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut captures_left) as u8;
                            if to_sq >= 56 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromoCapture as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::Capture as u8,
                                ), legal_only);
                            }
                        }
                        while captures_right != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut captures_right) as u8;
                            if to_sq >= 56 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromoCapture as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::Capture as u8,
                                ), legal_only);
                            }
                        }
                        while en_passant_left != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut en_passant_left) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq - 7 as u8,
                                to_sq as u8,
                                MoveFlag::EnPassant as u8,
                                ), legal_only);
                        }
                        while en_passant_right != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut en_passant_right) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq - 9 as u8,
                                to_sq as u8,
                                MoveFlag::EnPassant as u8,
                                ), legal_only);
                        }
                    }
                    else {
                        single_push = (pc_bb >> 8) & empty_bb;
                        let rank6 = 0x0000FF0000000000u64;
                        double_push = ((single_push & rank6) >> 8 & empty_bb);
                        let white_pieces = self.bitboards[BBPiece::White as usize];
                        captures_left  = ((pc_bb & not_h_file) >> 7) & white_pieces;
                        captures_right = ((pc_bb & not_a_file) >> 9) & white_pieces;
                        if let Some(ep_sq) = self.en_passant {
                            let ep_bb = 1u64 << ep_sq;
                            // Left en passant: pawn must be on the file to the right of ep_sq and able to capture left
                            en_passant_left = ((pc_bb & not_h_file) >> 7) & ep_bb;
                            // Right en passant: pawn must be on the file to the left of ep_sq and able to capture right
                            en_passant_right = ((pc_bb & not_a_file) >> 9) & ep_bb;
                        }   
                        while single_push != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut single_push) as u8;
                            if to_sq <= 7 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq + 8 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromotion as u8,
                                ), legal_only); 
                                self.add_move(&Move::from_parts(
                                to_sq + 8 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromotion as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 8 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromotion as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 8 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromotion as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq + 8 as u8,
                                to_sq as u8,
                                MoveFlag::Quiet as u8,
                                ), legal_only);
                            }

                        }
                        while double_push != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut double_push) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq + 16 as u8,
                                to_sq as u8,
                                MoveFlag::DoublePush as u8,
                                ), legal_only);
                        }
                        while captures_left != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut captures_left) as u8;
                            if to_sq <= 7 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromoCapture as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::Capture as u8,
                                ), legal_only);
                            }
                        }
                        while captures_right != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut captures_right) as u8;
                            if to_sq <= 7 // first or last rank
                            {
                                self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::QueenPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::RookPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::KnightPromoCapture as u8,
                                ), legal_only);
                                self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::BishopPromoCapture as u8,
                                ), legal_only);
                            }
                            else {
                                self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::Capture as u8,
                                ), legal_only);
                            }
                        }
                        while en_passant_left != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut en_passant_left) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq + 7 as u8,
                                to_sq as u8,
                                MoveFlag::EnPassant as u8,
                                ), legal_only);
                        }
                        while en_passant_right != 0
                        {
                            let to_sq = util::bb_gs_low_bit(&mut en_passant_right) as u8;
                            self.add_move(&Move::from_parts(
                                to_sq + 9 as u8,
                                to_sq as u8,
                                MoveFlag::EnPassant as u8,
                                ), legal_only);
                        }
                    }
                }
                BBPiece::Knight => {
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        // Generate knight moves
                        // Knight moves are L-shaped, 2 squares in one direction and 1 square perpendicular
                        let mut attacks = KNIGHT_ATTACKS[_square] & !self.bitboards[color_bb as usize];
                        let mut move_square = util::bb_gs_low_bit(&mut attacks);
                        while move_square != 64 {
                                    let mut flags = MoveFlag::Quiet as u8;
                                    if util::bb_get(self.bitboards[1-(color_bb as usize)], move_square as usize) {
                                        flags = MoveFlag::Capture as u8; // Capture if opponent piece
                                    }
                                    self.add_move(&Move::from_parts(
                                        _square as u8,
                                        move_square as u8,
                                        flags,
                                    ), legal_only);
                                    move_square = util::bb_gs_low_bit(&mut attacks);
                                }
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Bishop => {
                    // Generate bishop moves
                    // Queen move gen but just diagonals
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        self.gen_sliding_moves(_square as usize, legal_only, false, true);
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Rook => {
                    // Generate rook moves
                    // Queen move gen but just horizontals
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        self.gen_sliding_moves(_square as usize, legal_only, true, false);
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Queen => {
                    // Generate queen moves
                    // Queen / slider move generation
                    // Check all directions (horizontal, vertical, diagonal) until hitting a piece or the edge of the board
                    // If hitting a piece, check if it's an opponent piece to capture
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        self.gen_sliding_moves(_square as usize, legal_only, true, true);
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::King => {
                    // Generate king moves
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    // If castling rights exist, check if no pieces are in between the king and rook
                    if self.move_color == Color::White as i8 {
                        if self.castling_rights[0] && _square == Squares::E1 as usize && !util::bb_get(combined_bb, Squares::F1 as usize) && !util::bb_get(combined_bb, Squares::G1 as usize) {
                            // King-side castle
                            self.add_move(&Move::from_parts(_square as u8, Squares::G1 as u8, MoveFlag::KingCastle as u8), legal_only);
                        }
                        if self.castling_rights[1] && _square == Squares::E1 as usize && !util::bb_get(combined_bb, Squares::D1 as usize) && !util::bb_get(combined_bb, Squares::C1 as usize) && !util::bb_get(combined_bb, Squares::B1 as usize) {
                            // Queen-side castle
                            self.add_move(&Move::from_parts(_square as u8, Squares::C1 as u8, MoveFlag::QueenCastle as u8), legal_only);
                        }
                    } else {
                        if self.castling_rights[2] && _square == Squares::E8 as usize && !util::bb_get(combined_bb, Squares::F8 as usize) && !util::bb_get(combined_bb, Squares::G8 as usize) {
                            // King-side castle
                            self.add_move(&Move::from_parts(_square as u8, Squares::G8 as u8, MoveFlag::KingCastle as u8), legal_only);
                        }
                        if self.castling_rights[3] && _square == Squares::E8 as usize && !util::bb_get(combined_bb, Squares::D8 as usize) && !util::bb_get(combined_bb, Squares::C8 as usize) && !util::bb_get(combined_bb, Squares::B8 as usize) {
                            // Queen-side castle
                            self.add_move(&Move::from_parts(_square as u8, Squares::C8 as u8, MoveFlag::QueenCastle as u8), legal_only);
                        }
                    }
                    while _square != 64 {
                        // Generate king moves
                        let mut attacks = KING_ATTACKS[_square] & !self.bitboards[color_bb as usize];
                        let mut move_square = util::bb_gs_low_bit(&mut attacks);
                        while move_square != 64 {
                                    let mut flags = MoveFlag::Quiet as u8;
                                    if util::bb_get(self.bitboards[1-(color_bb as usize)], move_square as usize) {
                                        flags = MoveFlag::Capture as u8; // Capture if opponent piece
                                    }
                                    self.add_move(&Move::from_parts(
                                        _square as u8,
                                        move_square as u8,
                                        flags,
                                    ), legal_only);
                                    move_square = util::bb_gs_low_bit(&mut attacks);
                                }
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::White | BBPiece::Black => unimplemented!()
            }
        }
    }
    fn gen_sliding_moves(&mut self, idx: usize, legal_only: bool, orth: bool, diag: bool) {
        let color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let own_pieces = self.bitboards[color_bb as usize];
        let opp_pieces = self.bitboards[1 - (color_bb as usize)];
        let blockers = self.combined([BBPiece::White, BBPiece::Black], false);

        let mut attacks = 0u64;
        if orth {
            attacks |= rook_attacks(idx, blockers);
        }
        if diag {
            attacks |= bishop_attacks(idx, blockers);
        }

        // Remove own pieces from attack set
        let targets = attacks & !own_pieces;

        // Iterate over all target squares
        let mut targets_bb = targets;
        while targets_bb != 0 {
            let to = targets_bb.trailing_zeros() as usize;
            let flag = if (opp_pieces & (1u64 << to)) != 0 {
                MoveFlag::Capture as u8
            } else {
                MoveFlag::Quiet as u8
            };
            self.add_move(&Move::from_parts(idx as u8, to as u8, flag), legal_only);
            targets_bb &= targets_bb - 1; // Clear the lowest set bit
        }
    }
    pub fn king_is_attacked(&self) -> bool {
        // Check if the square is attacked by any piece of current player (i.e., can we take the opponent king after they made their move)
        let mut color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::Black //reversed logic, we check if the opponent's king is attacked
        } else {
            BBPiece::White
        };
        let mut king_bb = self.combined([BBPiece::King, color_bb], true);
        if let Some(_move) = self.move_history.last() {
            if _move.flags() & 0x2 != 0 && _move.flags() & 0xC == 0 { // Castling
            // Get squares moved through
            let from_index = _move.from_square();
            let to_index = _move.to_square();
            if from_index == Squares::E1 as u8 && to_index == Squares::G1 as u8 { // White King-side castle
                if self.square_is_attacked(Squares::E1 as usize) || self.square_is_attacked(Squares::F1 as usize) {
                return true;
                }
            } else if from_index == Squares::E1 as u8 && to_index == Squares::C1 as u8 { // White Queen-side castle
                if self.square_is_attacked(Squares::E1 as usize) || self.square_is_attacked(Squares::D1 as usize) {
                return true;
                }
            } else if from_index == Squares::E8 as u8 && to_index == Squares::G8 as u8 { // Black King-side castle
                if self.square_is_attacked(Squares::E8 as usize) || self.square_is_attacked(Squares::F8 as usize) {
                return true;
                } 
            } else if from_index == Squares::E8 as u8 && to_index == Squares::C8 as u8 { // Black Queen-side castle
                if self.square_is_attacked(Squares::E8 as usize) || self.square_is_attacked(Squares::D8 as usize) {
                return true;
                }
            }
            }
        }
        let square = util::bb_gs_low_bit(&mut king_bb);
        return self.square_is_attacked(square);
    }
    // Checks if a square is attacked
    pub fn square_is_attacked(&self, square: usize) -> bool {
        // Check if the square is attacked by any piece of current player (i.e., can we take the opponent king after they made their move)
        let mut color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let mut opp_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::Black
        } else {
            BBPiece::White
        };
        let rank = square / 8;
        let file = square % 8;
        let blockers = self.combined([BBPiece::White, BBPiece::Black], false);
        // Check for pawn attacks
        if self.move_color == Color::White as i8 {
            // White pawns attack up-left and up-right
            if file > 0 && rank > 0 && self.get([BBPiece::Pawn, BBPiece::White], square - 9) {
                return true


            }
            if file < 7 && rank > 0 && self.get([BBPiece::Pawn, BBPiece::White], square - 7) {
                return true

            }
        } else {
            // Black pawns attack down-left and down-right
            if file > 0 && rank < 7 && self.get([BBPiece::Pawn, BBPiece::Black], square + 7) {
                return true


            }
            if file < 7 && rank < 7 && self.get([BBPiece::Pawn, BBPiece::Black], square + 9) {
                return true

            }
        }
        // Check for knight attacks
        if KNIGHT_ATTACKS[square] & self.bitboards[color_bb as usize] & self.bitboards[BBPiece::Knight as usize] != 0
        {
            return true
        }
        let rook_attackers = self.bitboards[BBPiece::Rook as usize] | self.bitboards[BBPiece::Queen as usize];
        let rook_attacks = rook_attacks(square, blockers) & self.bitboards[color_bb as usize] & rook_attackers;
        if rook_attacks != 0 {
            return true;
        }

        // Bishop/Queen (diagonal)
        let bishop_attackers = self.bitboards[BBPiece::Bishop as usize] | self.bitboards[BBPiece::Queen as usize];
        let bishop_attacks = bishop_attacks(square, blockers) & self.bitboards[color_bb as usize] & bishop_attackers;
        if bishop_attacks != 0 {
            return true;
        }
        if KING_ATTACKS[square] & self.bitboards[color_bb as usize] & self.bitboards[BBPiece::King as usize] != 0
        {
            return true
        }
        false
    }
    pub fn is_draw(&self) -> bool {
        // 50-move rule
        if self.halfmove_clock >= 100 {
            return true;
        }

        // Threefold repetition: count occurrences of current Zobrist hash in position history
        let current_hash = self.zobrist_hash;
        for &hash in self.position_history.iter().rev() {
            if hash == current_hash {
                return true;
            }
        }
        false
    }
    pub fn compute_mobility(&self) -> ([u32; 8], [u32; 8]) {
        let mut white_mobility = [0u32; 8];
        let mut black_mobility = [0u32; 8];

        // For each piece type, count pseudo-legal moves for both sides
        for piece in 3..8 { // Assuming 3..8 are piece types AND pawns are treated differently!
            // White
            let mut white_bb = self.bitboards[piece] & self.bitboards[BBPiece::White as usize];
            white_mobility[piece] = self.count_piece_mobility(piece, &mut white_bb, true);

            // Black
            let mut black_bb = self.bitboards[piece] & self.bitboards[BBPiece::Black as usize];
            black_mobility[piece] = self.count_piece_mobility(piece, &mut black_bb, false);
        }
        (white_mobility, black_mobility)
    }
    fn count_piece_mobility(&self, piece: usize, bb: &mut u64, is_white: bool) -> u32
    {
        let color_bb: BBPiece = if is_white {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let mut attacks = 0;
        match piece {
            3 => // Knight
            {
                let mut _square = util::bb_gs_low_bit(bb);
                while _square != 64 {
                    // Generate knight moves
                    // Knight moves are L-shaped, 2 squares in one direction and 1 square perpendicular
                    let k_attacks = KNIGHT_ATTACKS[_square] & !self.bitboards[color_bb as usize];
                    attacks += k_attacks.count_ones();
                    _square = util::bb_gs_low_bit(bb);
                }
            }
            4 => // Bishop
            {
                let mut _square = util::bb_gs_low_bit(bb);
                while _square != 64 {
                    attacks += self.gen_sliding_mobility(_square as usize, false, true, is_white);
                    _square = util::bb_gs_low_bit(bb);
                }
            }
            5 => // Rook
            {
                let mut _square = util::bb_gs_low_bit(bb);
                while _square != 64 {
                    attacks += self.gen_sliding_mobility(_square as usize, true, false, is_white);
                    _square = util::bb_gs_low_bit(bb);
                }
            }
            6 => // Queen
            {
                let mut _square = util::bb_gs_low_bit(bb);
                while _square != 64 {
                    attacks += self.gen_sliding_mobility(_square as usize, true, true, is_white);
                    _square = util::bb_gs_low_bit(bb);
                }
            }
            7 => // King
            {
                // Generate king moves
                let _square = util::bb_gs_low_bit(bb);
                if _square != 64 {
                    let mut k_attacks = KING_ATTACKS[_square] & !self.bitboards[color_bb as usize];
                    //later, add EFFICIENT! check to see if king can move to these squares - i.e., by generating opp attacks
                    attacks += k_attacks.count_ones();
                }
            }
            _ => unimplemented!()
        }
        attacks
    }
    fn gen_sliding_mobility(&self, idx: usize, orth: bool, diag: bool, is_white: bool) -> u32 {
        let color_bb: BBPiece = if is_white {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let own_pieces = self.bitboards[color_bb as usize];
        let opp_pieces = self.bitboards[1 - (color_bb as usize)];
        let blockers = self.combined([BBPiece::White, BBPiece::Black], false);

        let mut attacks = 0u64;
        if orth {
            attacks |= rook_attacks(idx, blockers);
        }
        if diag {
            attacks |= bishop_attacks(idx, blockers);
        }
        // Remove own pieces from attack set
        let targets = attacks & !own_pieces;
        // Iterate over all target squares
        targets.count_ones()
    }
}