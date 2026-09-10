use crate::board::{BKSC, BQSC, Board, Square, WKSC, WQSC};
use std::fmt::Write;

pub fn create(board: &Board) -> String {
    let mut buf = String::with_capacity(64);

    for y in (0..8).rev() {
        let mut empty = 0;
        for x in 0..8 {
            let sq = Square::from_coords(x, y);
            if let Some(p) = board.squares[sq] {
                if empty != 0 {
                    _ = write!(buf, "{empty}");
                    empty = 0;
                }
                let mut ch = match p.role() {
                    crate::board::Role::Pawn => 'p',
                    crate::board::Role::Knight => 'n',
                    crate::board::Role::Bishop => 'b',
                    crate::board::Role::Rook => 'r',
                    crate::board::Role::Queen => 'q',
                    crate::board::Role::King => 'k',
                };
                if p.is_white() {
                    ch = ch.to_ascii_uppercase();
                }
                buf.push(ch);
            } else {
                empty += 1;
                continue;
            }
        }
        if empty != 0 {
            _ = write!(buf, "{empty}");
        }

        if y != 0 {
            buf.push('/');
        }
    }

    _ = write!(buf, " {} ", board.side.fold('w', 'b'));

    if board.castle == 0 {
        buf.push('-');
    } else {
        if board.castle & WKSC != 0 {
            buf.push('K');
        }
        if board.castle & WQSC != 0 {
            buf.push('Q');
        }
        if board.castle & BKSC != 0 {
            buf.push('k');
        }
        if board.castle & BQSC != 0 {
            buf.push('q');
        }
    }

    if let Some(epsq) = board.epsq {
        _ = write!(buf, " {epsq}");
    } else {
        buf.push_str(" -");
    }

    buf
}
