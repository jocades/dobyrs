use std::io;
use std::time::Instant;

use dobyrs::board::Board;
use dobyrs::perft::perft;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() -> io::Result<()> {
    let mut board = Board::from_fen(START_FEN).unwrap();

    let depth = 6;
    let mut total = 0;
    let start = Instant::now();

    for m in board.generate() {
        if board.make(m) {
            let leafs = perft(&mut board, depth - 1);
            println!("{}{}: {}", m.src, m.dst, leafs);
            total += leafs;
        }
        board.take();
    }

    println!("\nSearched {total} nodes in {:?}", start.elapsed());

    // let mut board = Board::from_fen(START_FEN).unwrap();

    // // basic uci protocol for perft and position
    // for line in io::stdin().lines() {
    //     let line = line?;
    //     let mut parts = line.split_whitespace();
    //     match parts.next() {
    //         Some("uci") => {
    //             println!("id name dobyrs");
    //             println!("uciok");
    //         }
    //         Some("isready") => {
    //             println!("readyok");
    //         }
    //         Some("position") => match parts.next() {
    //             Some("startpos") => {
    //                 board = Board::from_fen(START_FEN).unwrap();
    //             }
    //             Some("fen") => {
    //                 let fen = parts.collect::<Vec<_>>().join(" ");
    //                 match Board::from_fen(&fen) {
    //                     Some(b) => board = b,
    //                     None => eprintln!("Invalid FEN: {fen}"),
    //                 }
    //             }
    //             _ => eprintln!("Invalid position command"),
    //         },
    //         Some("go") => match parts.next() {
    //             Some("perft") => {
    //                 if let Some(depth) = parts.next().and_then(|d| d.parse::<u32>().ok()) {
    //                     if depth == 0 {
    //                         println!("0: 1");
    //                         continue;
    //                     }
    //                     let mut total = 0;
    //                     let start = Instant::now();
    //                     for m in board.generate() {
    //                         if board.make(m) {
    //                             let leafs = perft(&mut board, depth - 1);
    //                             println!("{}{}: {}", m.src, m.dst, leafs);
    //                             total += leafs;
    //                         }
    //                         board.take();
    //                     }
    //                     println!("\nSearched {total} nodes in {:?}", start.elapsed());
    //                 } else {
    //                     eprintln!("Invalid depth for go perft command");
    //                 }
    //             }
    //             _ => eprintln!("Unknown go command"),
    //         },
    //         Some("d") => {
    //             println!("{board}");
    //         }
    //         Some("quit") => break,
    //         _ => eprintln!("Unknown command"),
    //     }
    // }

    Ok(())
}
