#![allow(unused)]
use std::fmt::Display;
mod board;
mod util;

fn main() {
    let _board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("Number of Moves: {}",_board.gen_moves().len());
    println!("{}",_board);
}