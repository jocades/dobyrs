use crate::board::{Board, Color::*, Piece, Role::*, Square, Undo, castle_mask};
use crate::movegen::{Kind, Move, is_attacked};
use crate::zobrist;

#[inline]
pub fn make(b: &mut Board, m: Move) -> bool {
    b.undos.push(Undo {
        m,
        epsq: b.epsq,
        castle: b.castle,
        halfmoves_clock: b.halfmoves_clock,
        fullmoves_count: b.fullmoves_count,
        key: b.key,
    });

    // Remove state components that are about to change. The new castling and
    // en-passant components are added after the move has been applied.
    b.key ^= zobrist::castle(b.castle);
    b.key ^= zobrist::en_passant_component(b);

    b.key ^= zobrist::piece(m.mov, m.src) ^ zobrist::piece(m.mov, m.dst);

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
                b.key ^= zobrist::piece(cap, m.dst);
            }
            if let Some(role) = m.promo {
                b.bitboards[m.mov].remove(m.dst);
                b.bitboards[b.side][role].insert(m.dst);
                let piece = role.of(b.side);
                b.squares[m.dst] = Some(piece);
                b.key ^= zobrist::piece(m.mov, m.dst);
                b.key ^= zobrist::piece(piece, m.dst);
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
            b.key ^= zobrist::piece(Piece::new(!b.side, Pawn), sq);
        }
        Kind::Castle => match m.dst {
            Square::G1 => move_piece(b, Square::H1, Square::F1, White.rook()),
            Square::C1 => move_piece(b, Square::A1, Square::D1, White.rook()),
            Square::G8 => move_piece(b, Square::H8, Square::F8, Black.rook()),
            Square::C8 => move_piece(b, Square::A8, Square::D8, Black.rook()),
            _ => unreachable!(),
        },
    }

    if m.mov.role() == Pawn || m.cap.is_some() {
        b.halfmoves_clock = 0;
    } else {
        b.halfmoves_clock += 1;
    }

    if b.side == Black {
        b.fullmoves_count += 1;
    }

    let king_sq = b.bitboards[b.side][King].first().unwrap();
    let in_check = is_attacked(b, king_sq, !b.side);

    b.side = !b.side;
    b.castle &= castle_mask(m.src) & castle_mask(m.dst);
    b.key ^= zobrist::side();
    b.key ^= zobrist::castle(b.castle);
    b.key ^= zobrist::en_passant_component(b);

    if in_check {
        unmake(b);
        return false;
    }

    true
}

#[inline]
pub fn unmake(b: &mut Board) {
    let Undo {
        m,
        epsq,
        castle,
        halfmoves_clock,
        fullmoves_count,
        key,
    } = b.undos.pop().unwrap();

    b.epsq = epsq;
    b.castle = castle;
    b.halfmoves_clock = halfmoves_clock;
    b.fullmoves_count = fullmoves_count;

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

    // Restoring the saved key is cheaper and less error-prone than reversing
    // every hash update performed by the move.
    b.key = key;
}

#[inline(always)]
fn move_piece(b: &mut Board, src: Square, dst: Square, p: Piece) {
    b.bitboards[p].replace(src, dst);
    b.occupancy[p.color()].replace(src, dst);

    b.squares[src] = None;
    b.squares[dst] = Some(p);
    b.key ^= zobrist::piece(p, src) ^ zobrist::piece(p, dst);
}

#[inline(always)]
pub fn remove_piece(b: &mut Board, sq: Square, p: Piece) {
    b.bitboards[p].remove(sq);
    b.occupancy[p.color()].remove(sq);

    b.squares[sq] = None;
    b.key ^= zobrist::piece(p, sq);
}

pub fn put_piece(b: &mut Board, sq: Square, p: Piece) {
    b.bitboards[p].insert(sq);
    b.occupancy[p.color()].insert(sq);

    b.squares[sq] = Some(p);
    b.key ^= zobrist::piece(p, sq);
}
