#![allow(unused)]
use std::fmt::Display;
mod board;
mod util;

fn main() {
    let mut _board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    println!("Number of Moves: {}",perft(&mut _board, 1));
    println!("Number of Moves: {}",perft(&mut _board, 2));
    println!("Number of Moves: {}",perft(&mut _board, 3));
    println!("Number of Moves: {}",perft(&mut _board, 4));
    println!("Number of Moves: {}",perft(&mut _board, 5));

}
fn perft(bd: &mut board::Board, depth: u8) -> u64 {
    let mut count = 0;
    for m in bd.gen_moves() {
        let mut bd_copy = bd.clone();
        board::make_move(&mut bd_copy,&m);
        if !bd_copy.king_is_attacked(){
            if depth > 1 {
                count += perft(&mut bd_copy,depth - 1);
            } else {
                count += 1;
            }    
        }
    }
    count
}