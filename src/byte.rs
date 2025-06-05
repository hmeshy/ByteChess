mod board;
mod util;

fn main() {
    let mut _board = board::STARTING_POSITION;
    println!("{}",_board);
    board::make_move(&mut _board, &util::Move::from_parts(util::sq_to_idx("e2") as u8, util::sq_to_idx("e4") as u8, 0));
    println!("{}",_board);
    board::make_move(&mut _board, &util::Move::from_parts(util::sq_to_idx("e1") as u8, util::sq_to_idx("e2") as u8, 0));
    println!("{}", _board);
}