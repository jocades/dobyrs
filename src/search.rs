use crate::{board::Board, evaluate::evaluate, movegen::Move};

#[derive(Debug)]
pub struct Info {
    nodes: u64,
}

pub fn search(board: &mut Board, depth: u32) -> (Option<Move>, i32) {
    let mut best_score = -INF;
    let mut best_move = None;

    let mut info = Info { nodes: 0 };

    for m in board.generate() {
        if !board.make(m) {
            continue;
        }

        // let score = -negamax(board, depth - 1, 1, &mut info);

        let score = -alphabeta(board, depth - 1, 1, -INF, INF, &mut info);

        println!("{m}: {score}");

        if score > best_score {
            best_score = score;
            best_move = Some(m)
        }

        board.unmake();

        #[cfg(debug_assertions)]
        board.check();
    }

    println!("{info:?}");

    (best_move, best_score)
}

pub fn searc_iterative_deepen(board: &mut Board, depth: u32) {
    let mut best = -INF;
    let mut info = Info { nodes: 0 };

    for d in 1..=depth {
        best = alphabeta(board, d, 1, -INF, INF, &mut info);
    }
}

const INF: i32 = 32_000;
const MATE: i32 = 30_000;

pub fn alphabeta(
    board: &mut Board,
    depth: u32,
    ply: i32,
    mut alpha: i32,
    beta: i32,
    info: &mut Info,
) -> i32 {
    if depth == 0 {
        info.nodes += 1;
        return evaluate(board);
    }

    let mut legal = 0;

    for m in board.generate() {
        if !board.make(m) {
            continue;
        }

        legal += 1;

        let score = -alphabeta(board, depth - 1, ply + 1, -beta, -alpha, info);

        board.unmake();

        if score >= beta {
            return score;
        }

        alpha = alpha.max(score);
    }

    if legal == 0 {
        return if board.in_check() { -MATE + ply } else { 0 };
    }

    alpha
}

pub fn negamax(board: &mut Board, depth: u32, ply: i32, info: &mut Info) -> i32 {
    if depth == 0 {
        info.nodes += 1;
        return evaluate(board);
    }

    let mut best = -INF;
    let mut legal = 0;

    for m in board.generate() {
        if !board.make(m) {
            continue;
        }

        legal += 1;

        let score = -negamax(board, depth - 1, ply + 1, info);

        board.unmake();

        if score > best {
            best = score;
        }
    }

    if legal == 0 {
        return if board.in_check() { -MATE + ply } else { 0 };
    }

    best
}
