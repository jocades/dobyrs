use std::io;
use std::time::Instant;

use doby::board::{Board, Piece};
use doby::{perft, search::search};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE_FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq";

fn main() -> io::Result<()> {
    let mut board = Board::from_fen("1k6/4r2Q/5R2/8/8/8/8/6K1 w - - 0 1").unwrap();
    println!("{}", doby::evaluate::evaluate(&board));
    // perft::divide(&mut board, 3);
    println!("{board}");
    let best_move = search(&mut board, 3);
    println!("{best_move:?}");

    // let score = doby::search::negamax(&mut board, 5);
    // println!("{score}");

    // let depth = 1;
    // let start = Instant::now();
    // let total = perft::divide(&mut board, depth);
    // println!("\nSearched {total} nodes in {:?}", start.elapsed());

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
