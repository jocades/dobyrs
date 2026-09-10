use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks};
use crate::bitboard::{Bitboard, Direction};
use crate::board::{
    BKSC, BQSC, Board,
    Color::{self, *},
    Piece,
    Role::{self, *},
    Square, WKSC, WQSC,
};

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub src: Square,
    pub dst: Square,
    pub mov: Piece,
    pub cap: Option<Piece>,
    pub promo: Option<Role>,
    pub kind: Kind,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Kind {
    Normal,
    DoublePush,
    EnPassant,
    Castle,
}

#[inline]
pub(crate) fn generate(b: &Board) -> Array<Move, 256> {
    let mut moves = Array::new();

    let not_us = !b.occupancy[b.side]; // empty | enemy
    let occupied = b.occupied();

    gen_pawn_moves(b, &mut moves, occupied);

    let knights = b.bitboards[b.side][Knight];
    for src in knights {
        let dsts = knight_attacks(src) & not_us;
        for dst in dsts {
            moves.push(Move {
                src,
                dst,
                mov: b.side.knight(),
                cap: b.squares[dst],
                promo: None,
                kind: Kind::Normal,
            });
        }
    }

    let bishops = b.bitboards[b.side][Bishop];
    for src in bishops {
        let dsts = bishop_attacks(src, occupied) & not_us;
        for dst in dsts {
            moves.push(Move {
                src,
                dst,
                mov: b.side.bishop(),
                cap: b.squares[dst],
                promo: None,
                kind: Kind::Normal,
            });
        }
    }

    let rooks = b.bitboards[b.side][Rook];
    for src in rooks {
        let dsts = rook_attacks(src, occupied) & not_us;
        for dst in dsts {
            moves.push(Move {
                src,
                dst,
                mov: b.side.rook(),
                cap: b.squares[dst],
                promo: None,
                kind: Kind::Normal,
            });
        }
    }

    let queens = b.bitboards[b.side][Queen];
    for src in queens {
        let mut dsts = rook_attacks(src, occupied) | bishop_attacks(src, occupied);
        dsts &= not_us;
        for dst in dsts {
            moves.push(Move {
                src,
                dst,
                mov: b.side.queen(),
                cap: b.squares[dst],
                promo: None,
                kind: Kind::Normal,
            });
        }
    }

    let king = b.bitboards[b.side][King];
    let src = king.first().unwrap();
    let dsts = king_attacks(src) & not_us;
    for dst in dsts {
        moves.push(Move {
            src,
            dst,
            mov: b.side.king(),
            cap: b.squares[dst],
            promo: None,
            kind: Kind::Normal,
        });
    }

    if b.side.is_white() {
        const OO: Bitboard = Bitboard::from_squares([Square::F1, Square::G1]);
        const OOO: Bitboard = Bitboard::from_squares([Square::D1, Square::C1, Square::B1]);

        if b.castle & WKSC != 0
            && (occupied & OO).is_empty()
            && !is_attacked(b, Square::E1, Black)
            && !is_attacked(b, Square::F1, Black)
            && !is_attacked(b, Square::G1, Black)
        {
            moves.push(Move {
                src: Square::E1,
                dst: Square::G1,
                mov: White.king(),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }

        if b.castle & WQSC != 0
            && (occupied & OOO).is_empty()
            && !is_attacked(b, Square::E1, Black)
            && !is_attacked(b, Square::D1, Black)
            && !is_attacked(b, Square::C1, Black)
        {
            moves.push(Move {
                src: Square::E1,
                dst: Square::C1,
                mov: White.king(),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }
    } else {
        const OO: Bitboard = Bitboard::from_squares([Square::F8, Square::G8]);
        const OOO: Bitboard = Bitboard::from_squares([Square::D8, Square::C8, Square::B8]);

        if b.castle & BKSC != 0
            && (occupied & OO).is_empty()
            && !is_attacked(b, Square::E8, White)
            && !is_attacked(b, Square::F8, White)
            && !is_attacked(b, Square::G8, White)
        {
            moves.push(Move {
                src: Square::E8,
                dst: Square::G8,
                mov: Black.king(),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }

        if b.castle & BQSC != 0
            && (occupied & OOO).is_empty()
            && !is_attacked(b, Square::E8, White)
            && !is_attacked(b, Square::D8, White)
            && !is_attacked(b, Square::C8, White)
        {
            moves.push(Move {
                src: Square::E8,
                dst: Square::C8,
                mov: Black.king(),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }
    }

    moves
}

#[inline]
pub fn gen_pawn_moves(b: &Board, moves: &mut Array<Move, 256>, occupied: Bitboard) {
    use crate::bitboard::{BACKRANKS, RANK_3, RANK_6};

    #[inline(always)]
    fn push_promos(
        moves: &mut Array<Move, 256>,
        src: Square,
        dst: Square,
        side: Color,
        cap: Option<Piece>,
    ) {
        for role in [Queen, Rook, Bishop, Knight] {
            moves.push(Move {
                src,
                dst,
                mov: side.pawn(),
                cap,
                promo: Some(role),
                kind: Kind::Normal,
            })
        }
    }

    let side = b.side;
    let empty = !occupied;
    let pawns = b.bitboards[side][Pawn];
    let enemy = b.occupancy[!side];

    let dir = side.fold(Direction::North, Direction::South);
    let singles = dir.translate(pawns) & empty;

    for dst in singles & !BACKRANKS {
        let src = dst.offset(-dir.offset());
        moves.push(Move {
            src,
            dst,
            mov: side.pawn(),
            cap: None,
            promo: None,
            kind: Kind::Normal,
        })
    }

    for dst in singles & BACKRANKS {
        let src = dst.offset(-dir.offset());
        push_promos(moves, src, dst, side, None)
    }

    let doubles = dir.translate(side.fold(singles & RANK_3, singles & RANK_6)) & empty;
    for dst in doubles {
        let src = dst.offset(-dir.offset() * 2);
        moves.push(Move {
            src,
            dst,
            mov: side.pawn(),
            cap: None,
            promo: None,
            kind: Kind::DoublePush,
        })
    }

    let dir = side.fold(Direction::NorthEast, Direction::SouthEast);
    let east_caps = dir.translate(pawns) & enemy;

    for dst in east_caps & !BACKRANKS {
        let src = dst.offset(-dir.offset());
        moves.push(Move {
            src,
            dst,
            mov: side.pawn(),
            cap: b.squares[dst],
            promo: None,
            kind: Kind::Normal,
        })
    }

    for dst in east_caps & BACKRANKS {
        let src = dst.offset(-dir.offset());
        push_promos(moves, src, dst, side, b.squares[dst]);
    }

    let dir = side.fold(Direction::NorthWest, Direction::SouthWest);
    let west_caps = dir.translate(pawns) & enemy;

    for dst in west_caps & !BACKRANKS {
        let src = dst.offset(-dir.offset());
        moves.push(Move {
            src,
            dst,
            mov: side.pawn(),
            cap: b.squares[dst],
            promo: None,
            kind: Kind::Normal,
        })
    }

    for dst in west_caps & BACKRANKS {
        let src = dst.offset(-dir.offset());
        push_promos(moves, src, dst, side, b.squares[dst]);
    }

    if let Some(epsq) = b.epsq {
        let candidates = pawn_attacks(!side, epsq) & pawns;
        for src in candidates {
            moves.push(Move {
                src,
                dst: epsq,
                mov: side.pawn(),
                cap: Some(side.other().pawn()),
                promo: None,
                kind: Kind::EnPassant,
            })
        }
    }
}

#[inline]
#[rustfmt::skip]
pub fn is_attacked(b: &Board, sq: Square, by_side: Color) -> bool {
    let pawns = b.bitboards[by_side][Pawn];
    if (pawn_attacks(!by_side, sq) & pawns).any() { return true; }

    let knights = b.bitboards[by_side][Knight];
    if (knight_attacks(sq) & knights).any() { return true; }

    let king = b.bitboards[by_side][King];
    if (king_attacks(sq) & king).any() { return true; }

    let occupied = b.occupied();

    let bishops_queens = b.bitboards[by_side][Bishop] | b.bitboards[by_side][Queen];
    if (bishop_attacks(sq, occupied) & bishops_queens).any() { return true; }

    let rooks_queens = b.bitboards[by_side][Rook] | b.bitboards[by_side][Queen];
    if (rook_attacks(sq, occupied) & rooks_queens).any() { return true; }

    false
}

use core::mem::MaybeUninit;

#[derive(Debug)]
pub struct Array<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    count: usize,
}

impl<T: Copy, const N: usize> Array<T, N> {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: unsafe { MaybeUninit::uninit().assume_init() },
            count: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        debug_assert!(self.count < N);
        self.data[self.count].write(item);
        self.count += 1;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        self.count -= 1;
        unsafe { Some(self.data[self.count].assume_init_read()) }
    }

    #[inline]
    pub fn pop_unchecked(&mut self) -> T {
        debug_assert_ne!(self.count, 0);
        self.count -= 1;
        unsafe { self.data[self.count].assume_init_read() }
    }
}

impl<T, const N: usize> std::ops::Deref for Array<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, self.count) }
    }
}

pub struct IntoIter<T, const N: usize> {
    array: Array<T, N>,
    index: usize,
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.count {
            return None;
        }
        let item = unsafe { self.array.data[self.index].assume_init_read() };
        self.index += 1;
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.count - self.index;
        (len, Some(len))
    }
}

impl<T, const N: usize> IntoIterator for Array<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            array: self,
            index: 0,
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        write!(f, "{}{}", self.src, self.dst)?;
        if let Some(role) = self.promo {
            f.write_char(role.to_char())?;
        }
        Ok(())
    }
}
