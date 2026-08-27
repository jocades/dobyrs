use crate::{
    attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks},
    board::{Board, Color, Piece, Role::*, Square, Undo},
    movegen::{Kind, Move},
};

pub fn make(b: &mut Board, m: Move) -> bool {
    b.history.push(Undo { m, epsq: b.epsq });

    b.bitboards[m.mov].movbit(m.src, m.dst);
    b.occupancy[b.side].movbit(m.src, m.dst);

    b.squares[m.src] = None;
    b.squares[m.dst] = Some(m.mov);

    b.epsq = None;

    match m.kind {
        Kind::Normal => {
            if let Some(cap) = m.cap {
                b.bitboards[cap].clrbit(m.dst);
                b.occupancy[!b.side].clrbit(m.dst);
            }
            if let Some(role) = m.promo {
                b.bitboards[m.mov].clrbit(m.dst);
                b.bitboards[b.side][role].setbit(m.dst);
                b.squares[m.dst] = Some(Piece::new(b.side, role));
            }
        }
        Kind::DoublePush => {
            b.epsq = Some(m.dst.offset(b.side.fold(-8, 8)));
        }
        Kind::EnPassant => {
            let sq = m.dst.offset(b.side.fold(-8, 8));
            b.bitboards[!b.side][Pawn].clrbit(sq);
            b.occupancy[!b.side].clrbit(sq);
            b.squares[sq] = None;
        }
        Kind::Castle => todo!(),
    }

    let king_sq = b.bitboards[b.side][King].first().unwrap();
    let king_attacked = is_attacked(b, king_sq, !b.side);

    b.side = !b.side;

    !king_attacked
}

pub fn take(b: &mut Board) {
    let Undo { m, epsq } = b.history.pop().unwrap();
    b.epsq = epsq;

    b.side = !b.side;

    b.bitboards[m.mov].clrbit(m.dst);
    b.bitboards[m.mov].setbit(m.src);

    b.occupancy[b.side].clrbit(m.dst);
    b.occupancy[b.side].setbit(m.src);

    b.squares[m.dst] = None;
    b.squares[m.src] = Some(m.mov);

    match m.kind {
        Kind::Normal => {
            if let Some(cap) = m.cap {
                b.bitboards[cap].setbit(m.dst);
                b.occupancy[!b.side].setbit(m.dst);
                b.squares[m.dst] = Some(cap);
            }
            if let Some(role) = m.promo {
                b.bitboards[m.mov].setbit(m.src);
                b.bitboards[b.side][role].clrbit(m.dst);
            }
        }
        Kind::DoublePush => {}
        Kind::EnPassant => {
            let sq = m.dst.offset(b.side.fold(-8, 8));
            b.bitboards[!b.side][Pawn].setbit(sq);
            b.occupancy[!b.side].setbit(sq);
            b.squares[sq] = Some(Piece::new(!b.side, Pawn));
        }
        Kind::Castle => todo!(),
    }
}

#[rustfmt::skip]
pub fn is_attacked(b: &Board, sq: Square, by_side: Color) -> bool {
    let pawns = b.bitboards[by_side][Pawn];
    if (pawn_attacks(!by_side, sq) & pawns).any() { return true; }

    let knights = b.bitboards[by_side][Knight];
    if (knight_attacks(sq) & knights).any() { return true; }

    let king = b.bitboards[by_side][King];
    if (king_attacks(sq) & king).any() { return true; }

    let occupied = b.occupancy[Color::White] | b.occupancy[Color::Black];

    let bishops_queens = b.bitboards[by_side][Bishop] | b.bitboards[by_side][Queen];
    if (bishop_attacks(sq, occupied) & bishops_queens).any() { return true; }

    let rooks_queens = b.bitboards[by_side][Rook] | b.bitboards[by_side][Queen];
    if (rook_attacks(sq, occupied) & rooks_queens).any() { return true; }

    false
}
