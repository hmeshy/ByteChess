#![allow(unused)]
use std::fmt::Display;
mod board;
mod util;

fn main() {
    let _board = util::board_from_fen("rnbqkb1r/1pppp1Pp/5n2/p7/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 5");
    println!("Number of Moves: {}",_board.gen_moves().len());
    println!("{}",_board);
}