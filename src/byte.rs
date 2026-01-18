#![allow(unused)]
use core::hash;
use std::cmp::max;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use crate::magic::ROOK_MAGICS;
use crate::table::PawnTable;
use crate::table::{TranspositionTable, TTEntry, Bound};
use util::Score;
mod board;
mod util;
mod magic;
mod zobrist;
mod table;
mod tuner;
mod tunereval;
pub const PIECE_VALUES: [Score; 8] = [
    Score::new(0,0), // Empty
    Score::new(0,0), // None
    Score::new(73, 109), // Pawn
    Score::new(306, 314), // Knight
    Score::new(362, 325), // Bishop
    Score::new(457, 625), // Rook
    Score::new(1100, 1010), // Queen
    Score::new(100000, 100000) // King
];// === Mobility Weights ===
pub const MOBILITY_VALUES: [Score; 8] = [
    Score::new(0,0), Score::new(0,0), Score::new(0,0),
    Score::new(9, 11), // Knight
    Score::new(4, 10), // Bishop
    Score::new(4, 6), // Rook
    Score::new(-1, 12), // Queen
    Score::new(-11, 13), // King
];
pub const WINDOW: i32 = 33; // Search window for aspiration
// A simple pawn transposition table using a hash map.
// Key: zobrist hash of pawn structure, Value: evaluation score (i32)
pub struct SearchInfo {
    pub killer_moves: [[util::Move; 2]; 64], // Two killer moves per depth
    pub nodes: u64,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self {
            killer_moves: [[util::Move::from_parts(
            0 as u8,
            0 as u8,
            util::MoveFlag::Quiet as u8,
            ); 2]; 64], // Max depth 64
            nodes: 0,
        }
    }
    
    pub fn update_killer(&mut self, depth: usize, mv: util::Move) {
        if self.killer_moves[depth][0] != mv {
            self.killer_moves[depth][1] = self.killer_moves[depth][0];
            self.killer_moves[depth][0] = mv;
        }
    }
    
    pub fn next_move(&mut self) {
        // Reset killer moves and history table for the next move
        self.nodes = 0; // Reset node count for the next move
    }
}
fn main() {
    use std::io::{self, Write, BufRead};
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "tune"
    {
        println!("Starting Texel-based Tuning...");
        tuner::main();
        return;
    }
    let stdin = io::stdin();
    let mut board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    board.zobrist_hash = zobrist::zobrist_hash(&board);
    let mut search_info = SearchInfo::new();
    let mut hash_size_mb = 256;
    let mut tt = TranspositionTable::new(hash_size_mb);
    let mut input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut my_time: u64 = 1000 * 160;      // Bot's remaining time in ms
    let mut my_inc: u64 = 1000 * 0;       // Bot's increment in ms, keep at 0 if updating from uci
    let mut opp_time: u64 = 0;     // Opponent's remaining time in ms
    let mut opp_inc: u64 = 0;      // Opponent's increment in ms
    let mut mate_eval = 99900; // Evaluation to find checkmates, can be adjusted
    let mut pawn_tt = table::PawnTable::new(); // Initialize pawn transposition table

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
                println!("option name Hash type spin default 256 min 1 max 1024");
                println!("uciok");
            }
            "isready" => {
                println!("readyok");
            }
            "setoption" => {
                if tokens.len() >= 5 && tokens[1] == "name" && tokens[2] == "Hash" && tokens[3] == "value" {
                    if let Ok(value) = tokens[4].parse::<usize>() {
                        hash_size_mb = value;
                        tt = TranspositionTable::new(hash_size_mb);
                    }
                }
            }
            "testeval" => {
                // Print the evaluation of the current position
                util::print_eval(&board);
            }
            "ucinewgame" => {
                mate_eval = 99900; // Reset mate evaluation for new game
                board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                board.zobrist_hash = zobrist::zobrist_hash(&board);
                tt = TranspositionTable::new(hash_size_mb);
                input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
                let mut board_hist: Vec<String> = Vec::new();
                board_hist.push(input_fen.clone());
                search_info = SearchInfo::new();
            }
            "position" => {
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
                let m = think(&mut board, think_time, start, &mut tt, &mut mate_eval, &mut search_info, &mut pawn_tt);
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
fn think(board: &mut board::Board, think_time: u64, timer: std::time::Instant, tt: &mut TranspositionTable, mate_eval: &mut i32, search_info: &mut SearchInfo, pawn_tt: &mut PawnTable) -> util::Move {
    // Thinking logic
    tt.next_age();
    search_info.next_move();
    let mut depth = 0;    
    let mut moves = board.get_ordered_moves(false,true, false, None, &search_info.killer_moves[0]);
    let inf = i32::MIN + 1;
    let mut alpha = inf;
    let mut best_move = moves.first().clone(); // Save the first (ordered) legal move as a placeholder
    if moves.len() == 1 { // If there's only one possible move, return it immediately
        return best_move;
    }
    let eg = board.is_pawn_endgame();
    let mut previous_best_move = best_move.clone();
    let mut prev_eval = 0;
    let mut pv = Vec::new();
    if let Some(entry) = tt.probe(board.zobrist_hash) {
        if let Some(mv) = entry.get_best_move() {
            best_move = mv;
            previous_best_move = best_move;
        }
    }
    while timer.elapsed().as_millis() < think_time as u128 {
        moves = board.get_ordered_moves(false,true, false, Some(previous_best_move), &search_info.killer_moves[0]);
        let mut local_pv = Vec::new();
        for m in moves.iter()
        {
            board::make_move(board,&m);
            let mut child_pv = Vec::new();
            let mut eval = -minimax(board, depth, 0, -prev_eval-WINDOW, -prev_eval+WINDOW, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
            if eval <= prev_eval - WINDOW || eval >= prev_eval + WINDOW {
                // If the evaluation is outside the window, we need to re-search with a wider window
                eval = -minimax(board, depth, 0, alpha, -alpha, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
            }
            /*println!(
            "info move_ {} depth {} val {} nodes {}",
            m, depth, eval, search_info.nodes,
         );*/
            if timer.elapsed().as_millis() > think_time as u128 {
                // Time is up, break the loop
                if alpha == i32::MIN + 1 {
                    // A new alpha value was never set
                    alpha = prev_eval; // Use the previous evaluation
                }
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
            alpha, depth, search_info.nodes, elapsed, pv_string, best_move
        );
        if alpha >= *mate_eval || alpha <= -*mate_eval {
                // If the evaluation is a checkmate, return the move we found
                *mate_eval = alpha + 2; // Make sure to raise the mate threshold so we only return a faster checkmate next time
                return best_move;
        }
        previous_best_move = best_move.clone();
        prev_eval = alpha;
        alpha = i32::MIN + 1;
        depth += 1;
        if depth > 64 {
            // Limit the search depth to prevent excessive computation
            // Mainly used to prevent crashes in positions that are "dead" draws without accidentally blundering mate
            return best_move;
        }
    }
    best_move
}
fn minimax(board: &mut board::Board, depth: i32, depth_searched: i32, mut alpha: i32, beta: i32, think_time: u64, timer: std::time::Instant, tt: &mut TranspositionTable, pv: &mut Vec<util::Move>, search_info: &mut SearchInfo, eg: bool, pawn_tt: &mut PawnTable) -> i32 {
    search_info.nodes += 1;
    let r = 3; // Reduction factor
    if board.is_draw() {
        pv.clear();
        return 0; // Draw by repetition or 50 move or drawn endgame; checked before hash to avoid draws on decreasing depth!
    }
    // TT probe
    let mut tt_best_move = None;
    if let Some(entry) = tt.probe(board.zobrist_hash) {
        if entry.get_depth() >= depth {
            match entry.get_bound() {
                Bound::Exact => {   
                    pv.clear();
                    if let Some(mv) = entry.get_best_move() {
                        pv.push(mv);
                    }
                    return entry.score;}
                Bound::Lower => if entry.score >= beta { return entry.score; },
                Bound::Upper => if entry.score <= alpha { return entry.score; },
            }
        }
        tt_best_move = entry.get_best_move();
    }
    let killer_moves = if (depth_searched as usize) < search_info.killer_moves.len() {
        search_info.killer_moves[depth_searched as usize]
    } else {
        [util::Move::from_parts(
        0 as u8,
        0 as u8,
        util::MoveFlag::Quiet as u8,
        ); 2]};
    if depth == 0 {
            pv.clear();
            return minimax_captures(board, depth_searched, alpha, beta, depth_searched, search_info, pawn_tt);
    }
    let is_check = board::is_check(board);
    if !eg && depth >= r && !is_check { //null move conditions met
        // Perform null move pruning
        board::make_null_move(board);
        let mut null_pv = Vec::new();
        let eval = -minimax(board, depth - r, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut null_pv, search_info, eg, pawn_tt);
        board::undo_null_move(board);
        if eval >= beta {
            tt.store(TTEntry {
                zobrist: board.zobrist_hash,
                best_move: 0,
                depth: depth as u8,
                score: beta,
                bound: Bound::Lower.to_u8(),
                age: tt.age,
                _pad: 0,
            });
            pv.clear();
            return beta; // Beta cut-off
        }
        if eval > alpha {
            alpha = eval; // Update alpha
        }
    }
    let mut has_moves = false;
    let mut best_score = i32::MIN + 1;
    let mut best_move: Option<util::Move> = None;
    let mut best_pv: Vec<util::Move> = Vec::new();
    // If a hash move exists, try it first
    if let Some(hash_move) = tt_best_move {
        // Only try if the hash move is legal in this position
        let m = hash_move;
        board::make_move(board, &m);
        if !board.king_is_attacked() {
            has_moves = true;
            let mut child_pv = Vec::new();
            let mut eval;
            // late move reduction not applied to hash move
            eval = -minimax(board, depth - 1, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
            if (search_info.nodes & 0x3FF) == 0 && timer.elapsed().as_millis() > think_time as u128 {
                board::undo_move(board);
                pv.clear();
                pv.extend(best_pv.iter());
                return alpha;
            }
            if eval >= beta {
                board::undo_move(board);
                if m.flags() & 8 as u8 == 0 {
                    search_info.update_killer(depth_searched as usize, m);
                }
                pv.clear();
                pv.push(m);
                pv.extend(child_pv);
                tt.store(TTEntry {
                    zobrist: board.zobrist_hash,
                    best_move: m.info,
                    depth: depth as u8,
                    score: beta,
                    bound: Bound::Lower.to_u8(),
                    age: tt.age,
                    _pad: 0,
                });
                return beta;
            }
            if eval > alpha {
                alpha = eval;
                best_score = eval;
                best_move = Some(m);
                best_pv.clear();
                best_pv.push(m);
                best_pv.extend(child_pv);
            } 
        }
        board::undo_move(board);
    }
    let mut moves = board.get_ordered_moves(false, false, false, tt_best_move, &killer_moves);
    for (m_index, m) in moves.iter().enumerate(){
        if m_index == 0 && let Some(hash_move) = tt_best_move
        {
            continue; // Skip the hash move if it was already tried
        }
        board::make_move(board, &m);
        if !board.king_is_attacked()
        {
            has_moves = true;
            let mut child_pv = Vec::new();
            let mut eval;
            // late move reduction
            if depth >= 3 && m_index >= 4 
            {
                // Reduce the depth for later moves
                eval = -minimax(board, depth - 2, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
                if eval > alpha { // a promising move, so search deeper
                eval = -minimax(board, depth - 1, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
                }

            } else {
                // Normal search depth
                eval = -minimax(board, depth - 1, depth_searched + 1, -beta, -alpha, think_time, timer, tt, &mut child_pv, search_info, eg, pawn_tt);
            }
            if (search_info.nodes & 0x3FF) == 0 && timer.elapsed().as_millis() > think_time as u128 {
                board::undo_move(board);
                pv.clear();
                pv.extend(best_pv.iter());
                return alpha
            }
            if eval >= beta {
                board::undo_move(board);
                if m.flags() & 8 as u8 == 0 {
                    search_info.update_killer(depth_searched as usize, *m);
                }
                pv.clear();
                pv.push(*m);
                pv.extend(child_pv);
                tt.store(TTEntry {
                    zobrist: board.zobrist_hash,
                    best_move: m.info,
                    depth: depth as u8,
                    score: beta,
                    bound: Bound::Lower.to_u8(),
                    age: tt.age,
                    _pad: 0,
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
        best_move: best_move.map(|m| m.info).unwrap_or(0),
        depth: depth as u8,
        score: alpha,
        bound: if best_score > i32::MIN + 1 { Bound::Exact } else { Bound::Upper }.to_u8(),
        age: tt.age,
        _pad: 0,
    });
    pv.clear();
    pv.extend(best_pv.iter());
    alpha
}
// to_do -> include checks to make eval a truly quiet position
fn minimax_captures(board: &mut board::Board, depth_searched: i32, mut alpha: i32, beta: i32, depth: i32, search_info: &mut SearchInfo, pawn_tt: &mut PawnTable) -> i32 {
    search_info.nodes += 1;
    let eval = util::evaluate(board, pawn_tt);
    if eval >= beta {
        return beta;
    } else if eval >= alpha {
        alpha = eval;
    }
    let mut moves = board.get_ordered_moves(false, false, true, None, &[util::Move::from_parts(
        0 as u8,
        0 as u8,
        util::MoveFlag::Quiet as u8,
        ); 2]);
    if depth_searched <= 2 * depth && moves.len() != 0
    {
        for m in moves.iter(){
            board::make_move(board, &m);
            let eval = -minimax_captures(board, depth_searched + 1, -beta, -alpha, depth, search_info, pawn_tt);
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