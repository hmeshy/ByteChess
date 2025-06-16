#![allow(unused)]
use std::env;
use std::fmt::Display;
mod board;
mod util;

fn main() {
    use std::io::{self, Write, BufRead};
    let stdin = io::stdin();
    let mut board = util::board_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let mut input_fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");

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
                        let legal_moves = board.gen_moves();
                        let found = legal_moves.iter().find(|m| format!("{}", m) == *mv);
                        if let Some(m) = found {
                            board::make_move(&mut board, m).unwrap();
                        }
                    }
                }
            }
            "go" => {
                // Play the first legal move (pl with legality check)
                let moves = board.gen_moves();
                let mut found = false;
                for m in &moves {
                    let mut ghost = board.clone();
                    if board::make_move(&mut ghost, m).is_ok() && !ghost.king_is_attacked() {
                        println!("bestmove {}", m);
                        board::make_move(&mut board, m).unwrap();
                        found = true;
                        break;
                    }
                }
                if !found {
                    println!("bestmove (none)");
                }
            }
            "quit" | "exit" => {
                break;
            }
            _ => {}
        }
        io::stdout().flush().unwrap();
    }
}