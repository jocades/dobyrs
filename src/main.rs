use std::time::Instant;

use dobyrs::board::{ALGEBRAIC, Board};
use dobyrs::make::{make, take};
use dobyrs::movegen::generate;
use dobyrs::perft::perft;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let mut b = Board::from_fen(START_FEN).unwrap();

    let start = Instant::now();

    let depth = 5;

    let mut total = 0;

    for &m in generate(&b).iter() {
        if make(&mut b, m) {
            let leafs = perft(&mut b, depth - 1);
            println!("{}{}: {}", ALGEBRAIC[m.src], ALGEBRAIC[m.dst], leafs);
            total += leafs;
        }
        take(&mut b);
    }

    println!("\nSearched {total} nodes in {:?}", start.elapsed());
}
