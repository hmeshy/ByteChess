#![allow(unused)]
mod board;
mod util;

fn main() {
    let mut _board = board::Board {
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
        move_color: util::Color::White as i8,
        castling_rights: [true, true, true, true],
        en_passant: None,
        halfmove_clock: 0,
        fullmove_number: 1,
    };
    println!("Number of Moves: {}",_board.gen_moves().len());
}