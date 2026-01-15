// used to parse training data
use std::fs::File;
use std::io::{BufReader, BufRead, Write};
use chess::{Board, ChessMove, MoveGen}; // Use `chess` crate
use pgn_reader::{BufferedReader, Visitor, Skip, SanPlus}; // Use `pgn-reader` crate

struct FenExtractor<W: Write> {
    output: W,
    board: Option<Board>,
    result: Option<f64>,
}

impl<W: Write> Visitor for FenExtractor<W> {
    type Result = ();

    fn begin_game(&mut self) {
        self.board = Some(Board::default());
        self.result = None;
    }

    fn header(&mut self, key: &[u8], value: &[u8]) {
        if key == b"Result" {
            self.result = match value {
                b"1-0" => Some(1.0),
                b"0-1" => Some(0.0),
                b"1/2-1/2" => Some(0.5),
                _ => None,
            };
        }
    }

    fn san(&mut self, san_plus: SanPlus) {
        if let Some(board) = &mut self.board {
            if let Ok(chess_move) = san_plus.san.to_move(board) {
                let fen = board.to_string();
                if let Some(result) = self.result {
                    writeln!(self.output, "{} [{}]", fen, result).unwrap();
                }
                board.make_move(chess_move);
            }
        }
    }

    fn end_game(&mut self) -> Self::Result {}
}

fn main() -> std::io::Result<()> {
    let file = File::open("tcec_games.pgn")?;
    let reader = BufReader::new(file);
    let output = File::create("positions.txt")?;

    let mut extractor = FenExtractor {
        output,
        board: None,
        result: None,
    };

    let mut buffered = BufferedReader::new(reader);
    buffered.read_all(&mut extractor, &mut Skip)?;
    Ok(())
}