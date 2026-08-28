use crate::board::{Board, Color::*, Piece, Role::*, Square, Undo, castle_mask};
use crate::movegen::{Kind, Move, is_attacked};

#[inline]
pub fn make(b: &mut Board, m: Move) -> bool {
    b.undos.push(Undo {
        m,
        epsq: b.epsq,
        castle: b.castle,
    });

    b.bitboards[m.mov].replace(m.src, m.dst);
    b.occupancy[b.side].replace(m.src, m.dst);

    b.squares[m.src] = None;
    b.squares[m.dst] = Some(m.mov);

    b.epsq = None;

    match m.kind {
        Kind::Normal => {
            if let Some(cap) = m.cap {
                b.bitboards[cap].remove(m.dst);
                b.occupancy[!b.side].remove(m.dst);
            }
            if let Some(role) = m.promo {
                b.bitboards[m.mov].remove(m.dst);
                b.bitboards[b.side][role].insert(m.dst);
                b.squares[m.dst] = Some(Piece::new(b.side, role));
            }
        }
        Kind::DoublePush => {
            b.epsq = Some(m.dst.offset(b.side.fold(-8, 8)));
        }
        Kind::EnPassant => {
            let sq = m.dst.offset(b.side.fold(-8, 8));
            b.bitboards[!b.side][Pawn].remove(sq);
            b.occupancy[!b.side].remove(sq);
            b.squares[sq] = None;
        }
        Kind::Castle => match m.dst {
            Square::G1 => move_piece(b, Square::H1, Square::F1, White.rook()),
            Square::C1 => move_piece(b, Square::A1, Square::D1, White.rook()),
            Square::G8 => move_piece(b, Square::H8, Square::F8, Black.rook()),
            Square::C8 => move_piece(b, Square::A8, Square::D8, Black.rook()),
            _ => unreachable!(),
        },
    }

    let king_sq = b.bitboards[b.side][King].first().unwrap();
    let in_check = is_attacked(b, king_sq, !b.side);

    b.side = !b.side;
    b.castle &= castle_mask(m.src) & castle_mask(m.dst);

    if in_check {
        unmake(b);
        return false;
    }

    true
}

#[inline]
pub fn unmake(b: &mut Board) {
    let Undo { m, epsq, castle } = b.undos.pop().unwrap();
    b.epsq = epsq;
    b.castle = castle;

    b.side = !b.side;

    b.bitboards[m.mov].replace(m.dst, m.src);
    b.occupancy[b.side].replace(m.dst, m.src);

    b.squares[m.src] = Some(m.mov);
    b.squares[m.dst] = None;

    match m.kind {
        Kind::Normal => {
            if let Some(cap) = m.cap {
                b.bitboards[cap].insert(m.dst);
                b.occupancy[!b.side].insert(m.dst);
                b.squares[m.dst] = Some(cap);
            }
            if let Some(role) = m.promo {
                b.bitboards[m.mov].insert(m.src);
                b.bitboards[b.side][role].remove(m.dst);
            }
        }
        Kind::DoublePush => {}
        Kind::EnPassant => {
            let sq = m.dst.offset(b.side.fold(-8, 8));
            b.bitboards[!b.side][Pawn].insert(sq);
            b.occupancy[!b.side].insert(sq);
            b.squares[sq] = Some(Piece::new(!b.side, Pawn));
        }
        Kind::Castle => match m.dst {
            Square::G1 => move_piece(b, Square::F1, Square::H1, White.rook()),
            Square::C1 => move_piece(b, Square::D1, Square::A1, White.rook()),
            Square::G8 => move_piece(b, Square::F8, Square::H8, Black.rook()),
            Square::C8 => move_piece(b, Square::D8, Square::A8, Black.rook()),
            _ => unreachable!(),
        },
    }
}

#[inline(always)]
fn move_piece(b: &mut Board, src: Square, dst: Square, p: Piece) {
    b.bitboards[p].replace(src, dst);
    b.occupancy[p.color()].replace(src, dst);

    b.squares[src] = None;
    b.squares[dst] = Some(p);
}

#[inline(always)]
pub fn remove_piece(b: &mut Board, sq: Square, p: Piece) {
    b.bitboards[p].remove(sq);
    b.occupancy[p.color()].remove(sq);

    b.squares[sq] = None;
}

pub fn put_piece(b: &mut Board, sq: Square, p: Piece) {
    b.bitboards[p].insert(sq);
    b.occupancy[p.color()].insert(sq);

    b.squares[sq] = Some(p);
}
