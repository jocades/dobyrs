use crate::{board::Board, evaluate::evaluate, movegen::Move};

const INF: i32 = 32_000;
const MATE: i32 = 30_000;

pub fn negamax(board: &mut Board, depth: u32, ply: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut best = -INF;
    let mut legal = 0;

    for m in board.generate() {
        if !board.make(m) {
            continue;
        }

        legal += 1;

        let score = -negamax(board, depth - 1, ply + 1);

        if score > best {
            best = score;
        }

        board.unmake();
    }

    if legal == 0 {
        return if board.in_check() { -MATE + ply } else { 0 };
    }

    best
}

pub fn search(board: &mut Board, depth: u32) -> (Move, i32) {
    let mut best_score = -INF;
    let mut best_move = None;

    for m in board.generate() {
        if !board.make(m) {
            continue;
        }

        let score = -negamax(board, depth - 1, 0);

        if score > best_score {
            best_score = score;
            best_move = Some(m)
        }

        board.unmake();

        #[cfg(debug_assertions)]
        board.check();
    }

    (best_move.unwrap(), best_score)
}
