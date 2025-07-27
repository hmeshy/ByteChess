#![allow(unused)]
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use crate::magic::ROOK_MAGICS;
use crate::table::{TranspositionTable, TTEntry, Bound};
mod board;
mod util;
mod magic;
mod zobrist;
mod table;
pub const PIECE_VALUES: [i32; 8] = [0, 0, 71, 293, 300, 456, 905, 100000];
pub const MOBILITY_VALUES: [i32; 8] = [0, 0, 0, 10, 10, 3, 2, 0];

fn main() {
    use std::io::{self, Write, BufRead};
    let stdin = io::stdin();
    let mut board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    board.zobrist_hash = zobrist::zobrist_hash(&board);
    let mut tt = TranspositionTable::new();
    let mut input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut my_time: u64 = 1000 * 160;      // Bot's remaining time in ms
    let mut my_inc: u64 = 1000 * 0;       // Bot's increment in ms, keep at 0 if updating from uci
    let mut opp_time: u64 = 0;     // Opponent's remaining time in ms
    let mut opp_inc: u64 = 0;      // Opponent's increment in ms
    //let mut board_hist: Vec<String> = Vec::new();
    //board_hist.push(input_fen.clone());
    println!("id name ByteChess");
    println!("id author H&LM");
    println!("uciok");

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        if tokens.is_empty() { continue; }

        match tokens[0] {
            "uci" => {
                println!("id name ByteChess");
                println!("id author H&LM");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                board.zobrist_hash = zobrist::zobrist_hash(&board);
                tt = TranspositionTable::new();
                input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                let mut board_hist: Vec<String> = Vec::new();
                board_hist.push(input_fen.clone());
            }
            "position" => {
                // position [fen <fenstring> | startpos ]  moves <move1> ... <movei>
                let mut idx = 1;
                if tokens.len() > 1 && tokens[1] == "startpos" {
                    board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                    board.zobrist_hash = zobrist::zobrist_hash(&board);
                    input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                    idx += 1;
                } else if tokens.len() > 2 && tokens[1] == "fen" {
                    let moves_idx = tokens.iter().position(|&s| s == "moves").unwrap_or(tokens.len());
                    input_fen = tokens[2..moves_idx].join(" ");
                    board = util::board_from_fen(&input_fen);
                    board.zobrist_hash = zobrist::zobrist_hash(&board);
                    // Split the FEN string to count its parts (should be 6 for a full FEN)
                    let fen_parts: Vec<&str> = input_fen.split_whitespace().collect();
                    idx = 2 + fen_parts.len();
                }
                // Play moves if present
                if let Some(moves_idx) = tokens.iter().position(|&s| s == "moves") {
                    for mv in &tokens[moves_idx + 1..] {
                        board.gen_moves(false); //we can trust the move to compare against to be legal!
                        let legal_moves = board.moves;
                        let found = legal_moves.iter().find(|m| format!("{}", m) == *mv);
                        if let Some(m) = found {
                            board::make_move(&mut board, m).unwrap();
                        }
                    }
                }
            }
            "go" => {           
                // Parse time controls from the command
                //if input_fen != "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
                //    board_hist.push(input_fen.clone());
                //}
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
                let m = think(&mut board, think_time, start, &mut tt);

                // After move selection, update the bot's time
                let elapsed = start.elapsed().as_millis() as u64;
                my_time = my_time.saturating_sub(elapsed).saturating_add(my_inc);

                // Play the first legal move (pl with legality check)
                println!("bestmove {}", m);
            }
            "quit" | "exit" => {
                break;
            }
            _ => {}
        }
        io::stdout().flush().unwrap();
    }
}
fn think(board: &mut board::Board, think_time: u64, timer: std::time::Instant, tt: &mut TranspositionTable) -> util::Move {
    // Placeholder for thinking logic
    // This function should implement the logic to find the best move
    // based on the current board state and the given time limit.
    tt.next_age();
    let mut depth = 0;
    let mut nodes: u64 = 0;
    let mut moves = board.get_ordered_moves(false,true, false);
    let mut alpha = i32::MIN + 1;
    let mut best_move = moves.first().clone(); // Return the first legal move as a placeholder
    let mut previous_best_move = best_move.clone();
    let mut prev_eval = alpha;
    let mut pv = Vec::new();
    if let Some(entry) = tt.probe(board.zobrist_hash) {
        if let Some(mv) = entry.best_move {
            best_move = mv;
            previous_best_move = best_move;
        }
    }
    while timer.elapsed().as_millis() < think_time as u128 {
        // remove and insert previous best move at the beginning of moves
        let pos = moves.iter().position(|m| *m == previous_best_move);
        if let Some(pos) = pos {
            let mv = moves.remove(pos);
            moves.insert(0, mv);
        }
        let mut local_pv = Vec::new();
        for m in moves.iter()
        {
            board::make_move(board,&m);
            let mut child_pv = Vec::new();
            let eval = -minimax(board, depth, 0, i32::MIN + 1, -alpha, think_time, timer, tt, &mut child_pv, &mut nodes);
            if timer.elapsed().as_millis() > think_time as u128 {
                // Time is up, break the loop
                alpha = prev_eval; // Return the previous evaluation if time is up
                break;
            }
            if eval > alpha {
                alpha = eval;
                best_move = m.clone();
                local_pv.clear();
                local_pv.push(m.clone());
                local_pv.extend(child_pv)
            }
            board::undo_move(board);
        }
        pv = local_pv;
        let pv_string = pv.iter().map(|mv| format!("{}", mv)).collect::<Vec<_>>().join(" ");
        let elapsed = timer.elapsed().as_millis(); // <-- Add this line
        println!(
            "info score cp {} depth {} nodes {} time {} pv {} move {}",
            alpha, depth, nodes, elapsed, pv_string, best_move
        );
        previous_best_move = best_move.clone();
        prev_eval = alpha;
        alpha = i32::MIN + 1;
        depth += 1;
    }
    best_move
}
fn minimax(board: &mut board::Board, depth: i32, depth_searched: i32, mut alpha: i32, beta: i32, think_time: u64, timer: std::time::Instant, tt: &mut TranspositionTable, pv: &mut Vec<util::Move>, nodes: &mut u64) -> i32 {
    *nodes += 1;
    let r = 3; // Reduction factor
    if board.is_draw() {
        pv.clear();
        return 0; // Draw by repetition or 50 move; checked before hash to avoid draws on decreasing depth!
    }
    // TT probe
    let mut tt_best_move = None;
    if let Some(entry) = tt.probe(board.zobrist_hash) {
        if entry.depth >= depth /*&& entry.age == tt.age*/ {
            match entry.bound {
                Bound::Exact => {   
                    pv.clear();
                    if let Some(mv) = entry.best_move {
                        pv.push(mv);
                    }
                    return entry.score;}
                Bound::Lower => if entry.score >= beta { return entry.score; },
                Bound::Upper => if entry.score <= alpha { return entry.score; },
            }
        }
        tt_best_move = entry.best_move;
    }
    if depth == 0 {
            pv.clear();
            return minimax_captures(board, depth_searched, alpha, beta, depth_searched, nodes);
    }
    let is_check = board::is_check(board);
    if depth >= r && !is_check && !board.is_pawn_endgame() { //null move conditions met
        // Perform null move pruning
        board::make_null_move(board);
        let mut null_pv = Vec::new();
        let eval = -minimax(board, depth - r, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut null_pv, nodes);
        board::undo_null_move(board);
        if eval >= beta {
            tt.store(TTEntry {
                zobrist: board.zobrist_hash,
                best_move: None,
                depth,
                score: beta,
                bound: Bound::Lower,
                age: tt.age,
            });
            pv.clear();
            return beta; // Beta cut-off
        }
        if eval > alpha {
            alpha = eval; // Update alpha
        }
    }
    let mut moves = board.get_ordered_moves(false, false, false); // <-- no &
    if let Some(bm) = tt_best_move {
        let pos = moves.iter().position(|m| *m == bm);
        if let Some(pos) = pos {
            moves.move_to_front(pos);
        }
    }
    let mut has_moves = false;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<util::Move> = None;
    let mut best_pv: Vec<util::Move> = Vec::new();
    for m in moves.iter(){
        board::make_move(board, &m);
        if !board.king_is_attacked()
        {
            has_moves = true;
            let mut child_pv = Vec::new();
            let eval = -minimax(board, depth - 1, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut child_pv, nodes);
            if timer.elapsed().as_millis() > think_time as u128 {
                board::undo_move(board);
                pv.clear();
                pv.extend(best_pv.iter());
                return alpha
            }
            if eval >= beta {
                board::undo_move(board);
                pv.clear();
                pv.push(*m);
                pv.extend(child_pv);
                tt.store(TTEntry {
                    zobrist: board.zobrist_hash,
                    best_move: Some(*m),
                    depth,
                    score: beta,
                    bound: Bound::Lower,
                    age: tt.age,
                });
                return beta; // Beta cut-off
            }
            if eval > alpha {
                alpha = eval; // Update alpha
                best_score = eval;
                best_move = Some(*m);
                best_pv.clear();
                best_pv.push(*m);
                best_pv.extend(child_pv);
            }
        }
        board::undo_move(board);
    }
    if !has_moves {
        pv.clear();
        if is_check {
            return (depth_searched - 100000); // Checkmate
        } else {
            return 0; // Stalemate
        }
    }
    // Store TT entry (exact or upper bound)
    tt.store(TTEntry {
        zobrist: board.zobrist_hash,
        best_move,
        depth,
        score: alpha,
        bound: if best_score > i32::MIN + 1 { Bound::Exact } else { Bound::Upper },
        age: tt.age,
    });
    pv.clear();
    pv.extend(best_pv.iter());
    alpha
}
// to_do -> include checks to make eval a truly quiet position
fn minimax_captures(board: &mut board::Board, depth_searched: i32, mut alpha: i32, beta: i32, depth: i32, nodes: &mut u64) -> i32 {
    *nodes += 1;
    let eval = util::evaluate(board);
    if eval >= beta {
        return beta;
    } else if eval >= alpha {
        alpha = eval;
    }
    let mut moves = board.get_ordered_moves(false, false, true);
    if depth_searched <= 2 * depth && moves.len() != 0
    {
        for m in moves.iter(){
            board::make_move(board, &m);
            let eval = -minimax_captures(board, depth_searched + 1, -beta, -alpha, depth, nodes);
            if eval >= beta {
                board::undo_move(board);
                return beta; // Beta cut-off
            }
            if eval > alpha {
                alpha = eval; // Update alpha
            }
            board::undo_move(board);
        }
    }
    alpha
}