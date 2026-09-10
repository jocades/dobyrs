use std::io;

use doby::board::{Board, Piece};
use doby::search::alphabeta;
use doby::search::search;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE_FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
// const MATE_IN_3: &str = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
const MATE_IN_3: &str = "1k4rr/pppq1p2/2p2Q2/8/4N3/3P4/PPP2PPP/4RRK1 b - - 0 1";

fn main() -> io::Result<()> {
    let mut board = Board::from_fen(MATE_IN_3).unwrap();
    println!("{board}");

    let best = search(&mut board, 6);
    dbg!(best);

    Ok(())
}
