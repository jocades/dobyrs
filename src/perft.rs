use crate::board::Board;

pub fn perft(b: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut leafs = 0;

    for m in b.generate() {
        if !b.make(m) {
            continue;
        }

        leafs += perft(b, depth - 1);
        b.unmake();

        #[cfg(debug_assertions)]
        b.check();
    }

    leafs
}

pub fn divide(b: &mut Board, depth: u32) -> u64 {
    let mut total = 0;

    for m in b.generate() {
        if !b.make(m) {
            continue;
        }

        let leafs = perft(b, depth - 1);
        println!("{m}: {leafs}");
        total += leafs;
        b.unmake();

        #[cfg(debug_assertions)]
        b.check();
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://chessprogramming.org/Perft_Results
    // Might want to run tests in `release` mode to avoid `board.check()` overhead.

    macro_rules! perft {
        ($name:ident, $fen:expr, $($depth:expr => $expected:expr),* $(,)?) => {
            #[test]
            fn $name() {
                let mut b = Board::from_fen($fen).unwrap();
                $(
                    assert_eq!(perft(&mut b, $depth), $expected);
                )*
            }
        };
    }

    perft!(startpos, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        1 => 20,
        2 => 400,
        3 => 8_902,
        4 => 197_281,
        5 => 4_865_609,
        6 => 119_060_324,
    );

    perft!(kiwipete, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        1 => 48,
        2 => 2_039,
        3 => 97_862,
        4 => 4_085_603,
        5 => 193_690_690,
        // 6 => 8_031_647_685,
    );
}
