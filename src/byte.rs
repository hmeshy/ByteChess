mod board;
mod util;

fn main() {
    let mut _board = board::STARTING_POSITION;
    println!("{}",_board);
    board::make_move(&mut _board, "e2", "e4");
    println!("{}",_board);
    board::make_move(&mut _board, "e1", "e8");
    println!("{}",_board);
}