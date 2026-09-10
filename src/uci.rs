use std::io::{self, BufRead, BufWriter, Write};

use crate::board::{Board, Square};

pub fn run() -> io::Result<()> {
    let mut board = Board::default();

    let stdin = io::stdin().lock();
    let mut stdout = BufWriter::new(io::stdout());

    for line in stdin.lines() {
        let line = line?;
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("uci") => {
                writeln!(stdout, "id name doby")?;
                writeln!(stdout, "uciok")?;
                stdout.flush()?;
            }

            Some("isready") => {
                writeln!(stdout, "readyok")?;
                stdout.flush()?;
            }

            Some("position") => match parts.next() {
                Some("startpos") => {
                    board = Board::default();
                    if parts.next() == Some("moves") {
                        for lan in parts {
                            let bytes = lan.as_bytes();
                            let src = Square::from_coords(bytes[0] - b'a', bytes[1] - b'1');
                            let dst = Square::from_coords(bytes[2] - b'a', bytes[3] - b'1');
                            let m = board
                                .generate()
                                .into_iter()
                                .find(|m| m.src == src && m.dst == dst)
                                .unwrap();
                            if !board.make(m) {
                                panic!("illegal move")
                            }
                        }
                    }
                }
                Some("fen") => {
                    let fen = parts.collect::<Vec<_>>().join(" ");
                    board = Board::from_fen(&fen).unwrap();
                }
                _ => {}
            },

            Some("go") => match parts.next() {
                Some("perft") => {
                    if let Some(depth) = parts.next().and_then(|d| d.parse::<u32>().ok()) {
                        if depth == 0 {
                            writeln!(stdout, "0: 1")?;
                            stdout.flush()?;
                            continue;
                        }

                        let mut nodes = 0;
                        let start = std::time::Instant::now();

                        for m in board.generate() {
                            if !board.make(m) {
                                continue;
                            }

                            let leafs = crate::perft::perft(&mut board, depth - 1);
                            writeln!(stdout, "{m}: {leafs}")?;
                            nodes += leafs;
                            board.unmake();

                            #[cfg(debug_assertions)]
                            board.check();
                        }

                        writeln!(stdout, "\nSearched {nodes} nodes in {:?}", start.elapsed())?;
                        stdout.flush()?
                    }
                }

                Some("depth") => {
                    if let Some(depth) = parts.next().and_then(|d| d.parse::<u32>().ok()) {
                        if depth == 0 {
                            continue;
                        }

                        let (best_move, score) = crate::search::search(&mut board, depth);
                        if let Some(best) = best_move {
                            writeln!(stdout, "bestmove {best} score {score}")?;
                            stdout.flush()?;
                        }
                    }
                }
                _ => {}
            },

            Some("d") => {
                writeln!(stdout, "{board}")?;
                stdout.flush()?;
            }

            Some("quit") => break,

            _ => {}
        }
    }

    Ok(())
}
