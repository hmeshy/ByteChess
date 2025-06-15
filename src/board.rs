extern crate strum;
extern crate strum_macros;

use std::u8::MAX;

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
#[derive(Clone)]
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
        let combined_bb = self.combined([BBPiece::White, BBPiece::Black], false);
        for i in [BBPiece::Pawn, BBPiece::Knight, BBPiece::Bishop, BBPiece::Rook, BBPiece::Queen, BBPiece::King] {
            let mut pc_bb = self.combined([i, color_bb], true);
            // for all - generate start/end squares, get proper flag
            match i {
                BBPiece::Pawn => {
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        // Generate pawn moves
                        let mut _moves: Vec<Move> = Vec::new();
                        // Pushing 
                        // Single push (check if blocked))
                        let chk_sqr = if self.move_color == Color::White as i8 {
                            _square + 8
                        } else {
                            _square - 8
                        };
                        if !util::bb_get(combined_bb, chk_sqr) {
                            _moves.push(Move::from_parts(
                            _square as u8,
                            chk_sqr as u8,
                            MoveFlag::Quiet as u8,
                        ));   
                        // Double push (check if on inital rank, and if blocked)
                            let rank = _square / 8;
                            if (self.move_color == Color::White as i8 && rank == 1) || (self.move_color == Color::Black as i8 && rank == 6) {
                                let double_push_sqr = if self.move_color == Color::White as i8 {
                                    _square + 16
                                } else {
                                    _square - 16
                                };
                                if !util::bb_get(combined_bb, double_push_sqr) {
                                    _moves.push(Move::from_parts(
                                        _square as u8,
                                        double_push_sqr as u8,
                                        MoveFlag::DoublePush as u8,
                                    ));
                                }
                            }
                        }               
                        // Captures (check if opponent piece diagonal left or right, board wrapping checks & en passant)
                        let file = _square % 8;
                        // Note: left relative to white/up perspective
                        if file != 0 {
                            let left_capture_sqr = if self.move_color == Color::White as i8 {
                                _square + 7
                            } else {
                                _square - 9
                            };
                            // check opponent bitboard!
                            if util::bb_get(self.bitboards[1-(color_bb as usize)], left_capture_sqr) {
                                _moves.push(Move::from_parts(
                                    _square as u8,
                                    left_capture_sqr as u8,
                                    MoveFlag::Capture as u8,
                                ));
                            } else if self.en_passant == Some(left_capture_sqr) {
                                // En passant capture
                                _moves.push(Move::from_parts(
                                    _square as u8,
                                    left_capture_sqr as u8,
                                    MoveFlag::EnPassant as u8,
                                ));
                            }
                        }
                        if file != 7 {
                            let right_capture_sqr = if self.move_color == Color::White as i8 {
                                _square + 9
                            } else {
                                _square - 7
                            };
                            // check opponent bitboard!
                            if util::bb_get(self.bitboards[1-(color_bb as usize)], right_capture_sqr) {
                                _moves.push(Move::from_parts(
                                    _square as u8,
                                    right_capture_sqr as u8,
                                    MoveFlag::Capture as u8,
                                ));
                            } else if self.en_passant == Some(right_capture_sqr) {
                                // En passant capture
                                _moves.push(Move::from_parts(
                                    _square as u8,
                                    right_capture_sqr as u8,
                                    MoveFlag::EnPassant as u8,
                                ));
                            }
                        }
                        // Handle promotion if last rank
                        for _move in _moves.iter_mut() {
                            if (self.move_color == Color::White as i8 && _move.to_square() / 8 == 7) ||
                               (self.move_color == Color::Black as i8 && _move.to_square() / 8 == 0) {
                                // Add promotion flags
                                if _move.flags() & MoveFlag::Capture as u8 != 0 {
                                    // capture
                                    _move.set_flags(MoveFlag::KnightPromoCapture as u8);
                                    moves.push(_move.clone());
                                    _move.set_flags(MoveFlag::BishopPromoCapture as u8);
                                    moves.push(_move.clone());
                                    _move.set_flags(MoveFlag::RookPromoCapture as u8);
                                    moves.push(_move.clone());
                                    _move.set_flags(MoveFlag::QueenPromoCapture as u8);
                                    moves.push(_move.clone());
                                } else {
                                _move.set_flags(MoveFlag::KnightPromotion as u8);
                                moves.push(_move.clone());
                                _move.set_flags(MoveFlag::BishopPromotion as u8);
                                moves.push(_move.clone());
                                _move.set_flags(MoveFlag::RookPromotion as u8);
                                moves.push(_move.clone());
                                _move.set_flags(MoveFlag::QueenPromotion as u8);
                                moves.push(_move.clone());
                                }
                            }
                            else {
                                // Just a normal move
                                moves.push(_move.clone());
                            }
                        }
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Knight => {
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        // Generate knight moves
                        // Knight moves are L-shaped, 2 squares in one direction and 1 square perpendicular
                        let knight_moves = [
                            // Only include moves that are within 0..=63
                            if _square + 17 < 64 { Some((_square + 17) as u8) } else { None }, // Up2-Right1
                            if _square + 15 < 64 { Some((_square + 15) as u8) } else { None }, // Up2-Left1
                            if _square >= 17     { Some((_square - 17) as u8) } else { None }, // Down2-Left1
                            if _square >= 15     { Some((_square - 15) as u8) } else { None }, // Down2-Right1
                            if _square + 10 < 64 { Some((_square + 10) as u8) } else { None }, // Right2-Up1
                            if _square + 6  < 64 { Some((_square + 6)  as u8) } else { None }, // Left2-Up1
                            if _square >= 10     { Some((_square - 10) as u8) } else { None }, // Left2-Down1
                            if _square >= 6      { Some((_square - 6)  as u8) } else { None }, // Right2-Down1
                        ];
                        for &maybe_move_square in knight_moves.iter() {
                            if let(Some(move_square)) = maybe_move_square {
                                // Check if the square is empty or occupied by an opponent piece
                                // and not past border
                                if !util::bb_get(self.bitboards[color_bb as usize], move_square as usize) {
                                    // Check "wrapping"
                                    let rank_d = std::cmp::max(_square/8, (move_square / 8) as usize) - std::cmp::min(_square/8, (move_square / 8) as usize);
                                    let file_d = std::cmp::max(_square%8, (move_square % 8) as usize) - std::cmp::min(_square%8, (move_square % 8) as usize);
                                    if rank_d + file_d == 3 {
                                    // If the square is empty or occupied by an opponent piece
                                    let mut flags = MoveFlag::Quiet as u8;
                                    if util::bb_get(self.bitboards[1-(color_bb as usize)], move_square as usize) {
                                        flags = MoveFlag::Capture as u8; // Capture if opponent piece
                                    }
                                    moves.push(Move::from_parts(
                                        _square as u8,
                                        move_square,
                                        flags,
                                    ));}
                                }
                            }
                        }
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Bishop => {
                    // Generate bishop moves
                    // Queen move gen but just diagonals
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        let _moves = self.gen_sliding_moves(_square as usize, false, true);
                        for _move in _moves.iter() { moves.push(_move.clone());}
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::Rook => {
                    // Generate rook moves
                    // Queen move gen but just horizontals
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        let _moves: Vec<Move> = self.gen_sliding_moves(_square as usize, true, false);
                        for _move in _moves.iter() { moves.push(_move.clone());}
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
                        let _moves: Vec<Move> = self.gen_sliding_moves(_square as usize, true, true);
                        for _move in _moves.iter() { moves.push(_move.clone());}
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::King => {
                    // Generate king moves
                    let mut _square = util::bb_gs_low_bit(&mut pc_bb);
                    while _square != 64 {
                        // Check all adjacent squares (8 directions)
                        let king_moves = [
                            // Only include moves that are within 0..=63
                            if _square + 8 < 64 { Some((_square + 8) as u8) } else { None }, // Up
                            if _square >= 8     { Some((_square - 8) as u8) } else { None }, // Down
                            if _square % 8 != 0 { Some((_square - 1) as u8) } else { None }, // Left
                            if (_square + 1) % 8 != 0 { Some((_square + 1) as u8) } else { None }, // Right
                            if _square + 9 < 64 && (_square % 8 != 7) { Some((_square + 9) as u8) } else { None }, // Up-Right
                            if _square + 7 < 64 && (_square % 8 != 0) { Some((_square + 7) as u8) } else { None }, // Up-Left
                            if _square >= 9 && (_square % 8 != 7) { Some((_square - 9) as u8) } else { None }, // Down-Right
                            if _square >= 7 && (_square % 8 != 0) { Some((_square - 7) as u8) } else { None }, // Down-Left
                        ];
                        for &maybe_move_square in king_moves.iter() {
                            if let(Some(move_square)) = maybe_move_square {
                                // If the square is empty or occupied by an opponent piece (and not past border), add the move
                                // Check if the square is empty or occupied by an opponent piece
                                if !util::bb_get(self.bitboards[color_bb as usize], move_square as usize) {
                                    let mut flags = MoveFlag::Quiet as u8;
                                    if util::bb_get(self.bitboards[1-(color_bb as usize)], move_square as usize) {
                                        flags = MoveFlag::Capture as u8; // Capture if opponent piece
                                    }
                                    moves.push(Move::from_parts(
                                        _square as u8,
                                        move_square,
                                        flags,
                                    ));
                                }
                            }
                        }
                        // If castling rights exist, check if no pieces are in between the king and rook
                        if self.move_color == Color::White as i8 {
                            if self.castling_rights[0] && _square == util::sq_to_idx("e1") as usize && !util::bb_get(combined_bb, util::sq_to_idx("f1")) && !util::bb_get(combined_bb, util::sq_to_idx("g1")) {
                                // King-side castle
                                moves.push(Move::from_parts(_square as u8, util::sq_to_idx("g1") as u8, MoveFlag::KingCastle as u8));
                            }
                            if self.castling_rights[1] && _square == util::sq_to_idx("e1") as usize && !util::bb_get(combined_bb, util::sq_to_idx("d1")) && !util::bb_get(combined_bb, util::sq_to_idx("c1")) && !util::bb_get(combined_bb, util::sq_to_idx("b1")) {
                                // Queen-side castle
                                moves.push(Move::from_parts(_square as u8, util::sq_to_idx("c1") as u8, MoveFlag::QueenCastle as u8));
                            }
                        } else {
                            if self.castling_rights[2] && _square == util::sq_to_idx("e8") as usize && !util::bb_get(combined_bb, util::sq_to_idx("f8")) && !util::bb_get(combined_bb, util::sq_to_idx("g8")) {
                                // King-side castle
                                moves.push(Move::from_parts(_square as u8, util::sq_to_idx("g8") as u8, MoveFlag::KingCastle as u8));
                            }
                            if self.castling_rights[3] && _square == util::sq_to_idx("e8") as usize && !util::bb_get(combined_bb, util::sq_to_idx("d8")) && !util::bb_get(combined_bb, util::sq_to_idx("c8")) && !util::bb_get(combined_bb, util::sq_to_idx("b8")) {
                                // Queen-side castle
                                moves.push(Move::from_parts(_square as u8, util::sq_to_idx("c8") as u8, MoveFlag::QueenCastle as u8));
                            }
                        }
                        // we avoid any check logic here, just generate all moves
                        _square = util::bb_gs_low_bit(&mut pc_bb);
                    }
                }
                BBPiece::White | BBPiece::Black => unimplemented!()
            }
        }
        moves
    }
    fn gen_sliding_moves(&self, idx: usize, orth: bool, diag: bool ) -> Vec<Move>{
        let mut moves = Vec::new();
        let color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::White
        } else {
            BBPiece::Black
        };
        let rank = idx / 8;
        let file = idx % 8;
        let blockers = self.combined([BBPiece::White, BBPiece::Black], false);
        if orth {
            // Left moves
            for f in (0..file).rev() {
                let target_idx = rank * 8 + f;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
            // Right moves
            for f in (file + 1)..8 {
                let target_idx = rank * 8 + f;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
            // Down moves
            for r in (0..rank).rev() {
                let target_idx = r * 8 + file;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
            // Up moves
            for r in (rank + 1)..8 {
                let target_idx = r * 8 + file;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
        }
        if diag {
            // Up-Right moves
            for i in 1..8 {
                let target_idx = (rank + i) * 8 + (file + i);
                if target_idx >= 64 || (file + i) >= 8 { break; } // Out of bounds
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
            // Up-Left moves
            for i in 1..8 {
                if file < i { break; } // Out of bounds
                let target_idx = (rank + i) * 8 + (file - i);
                if target_idx >= 64 { break; } // Out of bounds
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                        // Capture opponent piece
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                    }
                    break; // Stop sliding in this direction
                } else {
                    moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                }
            }
            if rank > 0 {
                // Down-Right moves
                for i in 1..8 {
                    if rank < i { break; } // Out of bounds
                    let target_idx = (rank - i) * 8 + (file + i);
                    if (file + i) >= 8 || target_idx >= 64 { break; } // Out of bounds
                    if util::bb_get(blockers, target_idx) {
                        // Blocked by a piece
                        if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                            // Capture opponent piece
                            moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                        }
                        break; // Stop sliding in this direction
                    } else {
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                    }
                }
                // Down-Left moves
                for i in 1..8 {
                    if file < i || rank < i { break; } // Out of bounds
                    let target_idx = (rank - i) * 8 + (file - i);
                    if util::bb_get(blockers, target_idx) {
                        // Blocked by a piece
                        if util::bb_get(self.bitboards[1-(color_bb as usize)], target_idx) {
                            // Capture opponent piece
                            moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Capture as u8));
                        }
                        break; // Stop sliding in this direction
                    } else {
                        moves.push(Move::from_parts(idx as u8, target_idx as u8, MoveFlag::Quiet as u8));
                    }
                }
            }
        }
        moves
    }
    pub fn king_is_attacked(&self) -> bool {
        // Check if the square is attacked by any piece of current player (i.e., can we take the opponent king after they made their move)
        let mut color_bb: BBPiece = if self.move_color == Color::White as i8 {
            BBPiece::Black //reversed logic, we check if the opponent's king is attacked
        } else {
            BBPiece::White
        };
        let mut king_bb = self.combined([BBPiece::King, color_bb], true);
        color_bb = if self.move_color == Color::White as i8 {
            BBPiece::White //back to normal for our pieces
        } else {
            BBPiece::Black
        };
        let square = util::bb_gs_low_bit(&mut king_bb);
        let rank = square / 8;
        let file = square % 8;
        let blockers = self.combined([BBPiece::White, BBPiece::Black], false);
        // Check for pawn attacks
        if self.move_color == Color::White as i8 {
            // White pawns attack up-left and up-right
            if file > 0 && self.get([BBPiece::Pawn, BBPiece::White], square - 9) {
                return true
            }
            if file < 7 && self.get([BBPiece::Pawn, BBPiece::White], square - 7) {
                return true
            }
        } else {
            // Black pawns attack down-left and down-right
            if file > 0 && self.get([BBPiece::Pawn, BBPiece::Black], square + 7) {
                return true
            }
            if file < 7 && self.get([BBPiece::Pawn, BBPiece::Black], square + 9) {
                return true
            }
        }
        // Check for knight attacks
        let knight_moves = [
            // Only include moves that are within 0..=63
            if square + 17 < 64 { Some(square + 17) } else { None }, // Up2-Right1
            if square + 15 < 64 { Some(square + 15) } else { None }, // Up2-Left1
            if square >= 17     { Some(square - 17) } else { None }, // Down2-Left1
            if square >= 15     { Some(square - 15) } else { None }, // Down2-Right1
            if square + 10 < 64 { Some(square + 10) } else { None }, // Right2-Up1
            if square + 6  < 64 { Some(square + 6) } else { None }, // Left2-Up1
            if square >= 10     { Some(square - 10) } else { None }, // Left2-Down1
            if square >= 6      { Some(square - 6) } else { None }, // Right2-Down1
        ];
        for &maybe_move_square in knight_moves.iter() {
            if let(Some(move_square)) = maybe_move_square {
                // Check if the square is occupied by our knight
                if self.get([BBPiece::Knight, color_bb], move_square) {
                    // Check "wrapping"
                    let rank_d = std::cmp::max(square/8, (move_square / 8) as usize) - std::cmp::min(square/8, (move_square / 8) as usize);
                    let file_d = std::cmp::max(square%8, (move_square % 8) as usize) - std::cmp::min(square%8, (move_square % 8) as usize);
                    if rank_d + file_d == 3 {
                                        return true
                    }
                }
            }
        }
        // Check for sliding piece attacks (Bishop, Rook, Queen)
         // Left moves
            for f in (0..file).rev() {
                let target_idx = rank * 8 + f;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Rook], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            // Right moves
            for f in (file + 1)..8 {
                let target_idx = rank * 8 + f;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Rook], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            // Down moves
            for r in (0..rank).rev() {
                let target_idx = r * 8 + file;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Rook], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            // Up moves
            for r in (rank + 1)..8 {
                let target_idx = r * 8 + file;
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Rook], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            // Up-Right moves
            for i in 1..8 {
                let target_idx = (rank + i) * 8 + (file + i);
                if target_idx >= 64 || (file + i) >= 8 { break; } // Out of bounds
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Bishop], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            // Up-Left moves
            for i in 1..8 {
                if file < i { break; } // Out of bounds
                let target_idx = (rank + i) * 8 + (file - i);
                if target_idx >= 64 { break; } // Out of bounds
                if util::bb_get(blockers, target_idx) {
                    // Blocked by a piece
                    if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                        // Check if rook or queen
                        if self.get([BBPiece::Bishop], target_idx) || self.get([BBPiece::Queen], target_idx) {
                            return true
                        }
                    }
                    break; // Stop sliding in this direction
                } 
            }
            if rank > 0 {
                // Down-Right moves
                for i in 1..8 {
                    if rank < i { break; } // Out of bounds
                    let target_idx = (rank - i) * 8 + (file + i);
                    if (file + i) >= 8 || target_idx >= 64 { break; } // Out of bounds
                    if util::bb_get(blockers, target_idx) {
                        // Blocked by a piece
                        if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                            // Check if rook or queen
                            if self.get([BBPiece::Bishop], target_idx) || self.get([BBPiece::Queen], target_idx) {
                                return true
                            }
                        }
                        break; // Stop sliding in this direction
                    } 
                }
                // Down-Left moves
                for i in 1..8 {
                    if file < i || rank < i { break; } // Out of bounds
                    let target_idx = (rank - i) * 8 + (file - i);
                    if util::bb_get(blockers, target_idx) {
                        // Blocked by a piece
                        if util::bb_get(self.bitboards[color_bb as usize], target_idx) {
                            // Check if rook or queen
                            if self.get([BBPiece::Bishop], target_idx) || self.get([BBPiece::Queen], target_idx) {
                                return true
                            }
                        }
                        break; // Stop sliding in this direction
                    } 
                }
            }
        false
    }
}