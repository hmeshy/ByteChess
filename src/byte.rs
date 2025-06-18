#![allow(unused)]
use std::env;
use std::fmt::Display;

use crate::util::get_ordered_moves;
mod board;
mod util;
pub const PIECE_VALUES: [i32; 8] = [0, 0, 71, 293, 300, 456, 905, 10000];
pub const MOBILITY_VALUES: [i32; 8] = [0, 0, 0, 10, 10, 3, 2, 0];
/*fn main() // perft profiling debugging as necessary
{
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    } // debug code
    let mut board = util::board_from_fen("r3k2r/qp2bpp1/p2p4/N2Pp1P1/P4n2/5P2/1PPQB3/R3K2R b KQkq - 0 20");
    println!("{}", board);
    let legal_moves = board.gen_moves(false);
    let found = legal_moves.iter().find(|m| format!("{}", m) == "h8h1");
    if let Some(m) = found {
        board::make_move(&mut board, m).unwrap();
    }
    println!("{}", board);
    /*let start = std::time::Instant::now();
    println!("{}", util::perft(&mut board, 4, false));
    let duration = start.elapsed();
    println!("perft took {} ms", duration.as_millis());
    let start = std::time::Instant::now();
    println!("{}", util::perft(&mut board, 5, false));
    let duration = start.elapsed();
    println!("perft took {} ms", duration.as_millis());
    let start = std::time::Instant::now();
    println!("{}", util::perft(&mut board, 6, false));
    let duration = start.elapsed();
    println!("perft took {} ms", duration.as_millis());
    let start = std::time::Instant::now();
    println!("{}", util::perft(&mut board, 7, false));
    let duration = start.elapsed();
    println!("perft took {} ms", duration.as_millis());*/
}*/

fn main() {
    use std::io::{self, Write, BufRead};
    let stdin = io::stdin();
    let mut board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut my_time: u64 = 1000 * 160;      // Bot's remaining time in ms
    let mut my_inc: u64 = 1000 * 0;       // Bot's increment in ms, keep at 0 if updating from uci
    let mut opp_time: u64 = 0;     // Opponent's remaining time in ms
    let mut opp_inc: u64 = 0;      // Opponent's increment in ms
    let mut board_hist: Vec<String> = Vec::new();
    board_hist.push(input_fen.clone());
    println!("id name ByteChess");
    println!("id author github-copilot");
    println!("uciok");

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() { continue; }

        match tokens[0] {
            "uci" => {
                println!("id name ByteChess");
                println!("id author github-copilot");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                let mut board_hist: Vec<String> = Vec::new();
                board_hist.push(input_fen.clone());
            }
            "position" => {
                // position [fen <fenstring> | startpos ]  moves <move1> ... <movei>
                let mut idx = 1;
                if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                    input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                    idx += 1;
                } else if tokens.len() > 2 && tokens[1] == "fen" {
                    let moves_idx = tokens.iter().position(|&s| s == "moves").unwrap_or(tokens.len());
                    input_fen = tokens[2..moves_idx].join(" ");
                    board = util::board_from_fen(&input_fen);
                    // Split the FEN string to count its parts (should be 6 for a full FEN)
                    let fen_parts: Vec<&str> = input_fen.split_whitespace().collect();
                    idx = 2 + fen_parts.len();
                }
                // Play moves if present
                if let Some(moves_idx) = tokens.iter().position(|&s| s == "moves") {
                    for mv in &tokens[moves_idx + 1..] {
                        let legal_moves = board.gen_moves(false);
                        let found = legal_moves.iter().find(|m| format!("{}", m) == *mv);
                        if let Some(m) = found {
                            board::make_move(&mut board, m).unwrap();
                        }
                    }
                }
            }
            "go" => {           
                // Parse time controls from the command
                if input_fen != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
                    board_hist.push(input_fen.clone());
                }
                let mut i = 1;
                while i < tokens.len() {
                    match tokens[i] {
                        "wtime" => {
                            if board.move_color == util::Color::White as i8 {
                                my_time = tokens[i + 1].parse().unwrap_or(0);
                            } else {
                                opp_time = tokens[i + 1].parse().unwrap_or(0);
                            }
                            i += 2;
                        }
                        "btime" => {
                            if board.move_color == util::Color::Black as i8 {
                                my_time = tokens[i + 1].parse().unwrap_or(0);
                            } else {
                                opp_time = tokens[i + 1].parse().unwrap_or(0);
                            }
                            i += 2;
                        }
                        "winc" => {
                            if board.move_color == util::Color::White as i8 {
                                my_inc = tokens[i + 1].parse().unwrap_or(0);
                            } else {
                                opp_inc = tokens[i + 1].parse().unwrap_or(0);
                            }
                            i += 2;
                        }
                        "binc" => {
                            if board.move_color == util::Color::Black as i8 {
                                my_inc = tokens[i + 1].parse().unwrap_or(0);
                            } else {
                                opp_inc = tokens[i + 1].parse().unwrap_or(0);
                            }
                            i += 2;
                        }
                        _ => { i += 1; }
                    }
                }

                // Record the start time before move calculation
                let start = std::time::Instant::now();
                let think_time = my_time/20 + my_inc/2; // 5% of time + half increment for thinking time
                // ... your move selection logic here ...
                let m = think(&mut board, &board_hist, think_time, start);

                // After move selection, update the bot's time
                let elapsed = start.elapsed().as_millis() as u64;
                my_time = my_time.saturating_sub(elapsed).saturating_add(my_inc);

                // Play the first legal move (pl with legality check)
                println!("bestmove {}", m);
                board_hist.push(format!("{}", board));
            }
            "quit" | "exit" => {
                break;
            }
            _ => {}
        }
        io::stdout().flush().unwrap();
    }
}
fn think(board: &mut board::Board, board_hist: &Vec<String>,think_time: u64, timer: std::time::Instant) -> util::Move {
    // Placeholder for thinking logic
    // This function should implement the logic to find the best move
    // based on the current board state and the given time limit.
    let mut depth = 0;
    let mut moves = get_ordered_moves(board, false);
    let mut alpha = i32::MIN + 1;
    let mut best_move = moves[0].clone(); // Return the first legal move as a placeholder
    let mut previous_best_move = best_move.clone();
    let mut prev_eval = alpha;
    while timer.elapsed().as_millis() < think_time as u128 {
        // remove and insert previous best move at the beginning of moves
        if let Some(pos) = moves.iter().position(|m| *m == previous_best_move) {
            let mv = moves.remove(pos);
            moves.insert(0, mv);
        }
        for m in &moves
        {
            board::make_move(board,&m);
            if !board.king_is_attacked(){
                // move IS legal
                let eval = -minimax(board, board_hist, depth, 0, i32::MIN + 1, -alpha, think_time, timer);
                if timer.elapsed().as_millis() > think_time as u128 {
                    // Time is up, break the loop
                    alpha = prev_eval; // Return the previous evaluation if time is up
                    break;
                }
                if eval > alpha {
                    alpha = eval;
                    best_move = m.clone();
                }
            }
            board::undo_move(board);
        }
        println!("info score cp {} depth {} pv {} move {}", alpha, depth, best_move, best_move);
        previous_best_move = best_move.clone();
        prev_eval = alpha; // Store the previous evaluation
        alpha = i32::MIN + 1; // Reset alpha for the next iteration
        depth += 1;
    }
    best_move
}
fn minimax(board: &mut board::Board, board_hist: &Vec<String>, depth: i32, depth_searched: i32, mut alpha: i32, beta: i32, think_time: u64, timer: std::time::Instant) -> i32 {
    if depth == 0 {
        if util::perft(board, 1, true) == 0 {
            let eval = util::evaluate(board);
            if eval >= beta {
                return beta;
            } else if eval <= alpha {
                return alpha;
            } else {
                return eval;
            }
        } else {
            return minimax_captures(board, depth_searched, alpha, beta, true);
        }
    }
    if util::perft(board, 1, false) == 0 {
        if board::is_check(board) {
            return (depth_searched - 100000); // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    if util::is_repetition(board, board_hist) {
        return 0; // Draw by repetition
    }
    let mut moves = get_ordered_moves(board, false);
    for m in &moves{
        board::make_move(board, &m);
        if !board.king_is_attacked() {
            // move IS legal
            let eval = -minimax(board, board_hist, depth - 1, depth_searched + 1, -beta, -alpha, think_time, timer);
            if eval >= beta {
                board::undo_move(board);
                return beta; // Beta cut-off
            }
            if eval > alpha {
                alpha = eval; // Update alpha
            }
            if timer.elapsed().as_millis() > think_time as u128 {
                board::undo_move(board);
                return alpha
            }
        }
        board::undo_move(board);
    }
    alpha
}
fn minimax_captures(board: &mut board::Board, depth_searched: i32, mut alpha: i32, beta: i32, call_color: bool) -> i32 {
    let eval = util::evaluate(&board);
    if eval >= beta {
        return beta;
    } else if eval >= alpha {
        alpha = eval;
    }
    let mut moves = get_ordered_moves(board, true);
    for m in &moves{
        board::make_move(board, &m);
        if !board.king_is_attacked() {
            // move IS legal
            let eval = -minimax_captures(board, depth_searched + 1, -beta, -alpha, !call_color);
            if eval >= beta {
                board::undo_move(board);
                return beta; // Beta cut-off
            }
            if eval > alpha {
                alpha = eval; // Update alpha
            }
        }
        board::undo_move(board);
    }
    alpha
}