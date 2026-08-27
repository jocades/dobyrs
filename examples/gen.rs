use dobyrs::attacks::pawn_attacks;
use dobyrs::bitboard::Bitboard;
use dobyrs::board::Role;
use dobyrs::board::{
    Board,
    Color::{self, *},
    Role::*,
    Square,
};
use dobyrs::make::{make, take};
use dobyrs::movegen::{Array, Kind, gen_pawn_moves, generate};
use dobyrs::perft::{self, perft};

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let mut b = Board::from_fen(START_FEN).unwrap();
    println!("{}", perft(&mut b, 3));
    println!("{b}");

    // let moves = generate(&b);
    // let m = moves[0];
    // println!("{m:?}");
    // make(&mut b, m);
    // println!("{b}");
    // println!("last = {:?}", b.history.last());
    // take(&mut b);
    // println!("{b}");
    // let mut moves = Array::new();
    // gen_pawn_moves(&b, &mut moves, Bitboard(0));
    // println!("{}", b.bitboards[Black][Pawn]);
    // println!("{:#?}", &moves[..2]);
}
