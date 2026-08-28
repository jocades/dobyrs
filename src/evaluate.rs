use crate::board::{Board, Color::*, Role};

const ROLE_SCORE: [i32; 6] = [100, 320, 330, 500, 900, 0];

pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0;

    for role in Role::ALL {
        let value = ROLE_SCORE[role as usize];
        score += value * board.bitboards[White][role].count() as i32;
        score -= value * board.bitboards[Black][role].count() as i32;
    }

    board.side.fold(score, -score)
}
