use crate::attacks::{bishop_attacks, king_attacks, knight_attacks, rook_attacks};
use crate::bitboard::Bitboard;
use crate::board::Color;
use crate::board::{
    Board,
    Color::*,
    Piece,
    Role::{self, *},
    Square,
};
use crate::make::is_attacked;

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

const RANK_1: u64 = 0xff;
const RANK_2: u64 = 0xff00;
const RANK_3: u64 = 0xff0000;
const RANK_4: u64 = 0xff000000;
const RANK_5: u64 = 0xff00000000;
const RANK_6: u64 = 0xff0000000000;
const RANK_7: u64 = 0xff000000000000;
const RANK_8: u64 = 0xff00000000000000;

pub fn generate(b: &Board) -> Array<Move, 256> {
    let mut moves = Array::new();

    let not_us = !b.occupancy[b.side]; // empty | enemy
    let occupied = b.occupancy[White] | b.occupancy[Black];

    gen_pawn_moves(b, &mut moves, occupied);

    let knights = b.bitboards[b.side][Knight];
    for src in knights {
        let dsts = knight_attacks(src) & not_us;
        for dst in dsts {
            moves.push(Move {
                src,
                dst,
                mov: Piece::new(b.side, Knight),
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
                mov: Piece::new(b.side, Bishop),
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
                mov: Piece::new(b.side, Rook),
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
                mov: Piece::new(b.side, Queen),
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
            mov: Piece::new(b.side, King),
            cap: b.squares[dst],
            promo: None,
            kind: Kind::Normal,
        });
    }

    if b.side.is_white() {
        if b.castle.0
            && b.squares[Square::F1].is_none()
            && b.squares[Square::G1].is_none()
            && !is_attacked(b, Square::F1, Black)
            && !is_attacked(b, Square::G1, Black)
        {
            moves.push(Move {
                src: Square::E1,
                dst: Square::G1,
                mov: Piece::new(White, King),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }
        if b.castle.1
            && b.squares[Square::D1].is_none()
            && b.squares[Square::C1].is_none()
            && b.squares[Square::B1].is_none()
            && !is_attacked(b, Square::D1, Black)
            && !is_attacked(b, Square::C1, Black)
        {
            moves.push(Move {
                src: Square::E1,
                dst: Square::C1,
                mov: Piece::new(White, King),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }
    } else {
        if b.castle.2
            && b.squares[Square::F8].is_none()
            && b.squares[Square::G8].is_none()
            && !is_attacked(b, Square::F8, White)
            && !is_attacked(b, Square::G8, White)
        {
            moves.push(Move {
                src: Square::E8,
                dst: Square::G8,
                mov: Piece::new(Black, King),
                cap: None,
                promo: None,
                kind: Kind::Castle,
            })
        }
        if b.castle.3
            && b.squares[Square::D8].is_none()
            && b.squares[Square::C8].is_none()
            && b.squares[Square::B8].is_none()
            && !is_attacked(b, Square::D8, White)
            && !is_attacked(b, Square::C8, White)
        {
            moves.push(Move {
                src: Square::E8,
                dst: Square::C8,
                mov: Piece::new(Black, King),
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
                mov: Piece::new(side, Pawn),
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
            mov: Piece::new(b.side, Pawn),
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
            mov: Piece::new(side, Pawn),
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
            mov: Piece::new(side, Pawn),
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
            mov: Piece::new(side, Pawn),
            cap: b.squares[dst],
            promo: None,
            kind: Kind::Normal,
        })
    }

    for dst in west_caps & BACKRANKS {
        let src = dst.offset(-dir.offset());
        push_promos(moves, src, dst, side, b.squares[dst]);
    }
}

#[derive(Copy, Clone)]
pub(crate) enum Direction {
    North,
    South,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = 0x0202020202020202;
const FILE_G: u64 = 0x4040404040404040;
const FILE_H: u64 = 0x8080808080808080;

const BACKRANKS: u64 = RANK_1 | RANK_8;

impl Direction {
    #[inline(always)]
    pub const fn offset(self) -> i8 {
        match self {
            Direction::North => 8,
            Direction::South => -8,
            Direction::NorthWest => 7,
            Direction::SouthWest => -9,
            Direction::NorthEast => 9,
            Direction::SouthEast => -7,
        }
    }

    #[inline(always)]
    pub const fn translate(self, bb: Bitboard) -> Bitboard {
        Bitboard(match self {
            Direction::North => bb.0 << 8,
            Direction::South => bb.0 >> 8,
            Direction::NorthWest => (bb.0 & !FILE_A) << 7,
            Direction::SouthWest => (bb.0 & !FILE_A) >> 9,
            Direction::NorthEast => (bb.0 << 9) & !FILE_A,
            Direction::SouthEast => (bb.0 >> 7) & !FILE_A,
        })
    }
}

use core::mem::MaybeUninit;

#[derive(Debug)]
pub struct Array<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    count: usize,
}

impl<T, const N: usize> Array<T, N> {
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
