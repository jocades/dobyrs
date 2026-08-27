use crate::{
    board::Board,
    make::{make, take},
    movegen::generate,
};

pub fn perft(b: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut leafs = 0;

    for m in generate(b) {
        if make(b, m) {
            leafs += perft(b, depth - 1);
        }
        take(b);

        #[cfg(debug_assertions)]
        b.check();
    }

    leafs
}
