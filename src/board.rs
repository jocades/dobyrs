use std::fmt::Write;
use std::mem::transmute;
use std::ops::{Index, IndexMut};

use crate::bitboard::Bitboard;
use crate::movegen::{Array, Move};

#[derive(Debug)]
pub struct Board {
    pub bitboards: ByPiece<Bitboard>,
    pub occupancy: ByColor<Bitboard>,
    pub squares: BySquare<Option<Piece>>,

    pub side: Color,
    pub epsq: Option<Square>,
    pub castle: (bool, bool, bool, bool),

    pub history: Array<Undo, 2048>,
}

#[derive(Debug)]
pub struct Undo {
    pub m: Move,
    pub epsq: Option<Square>,
}

impl Default for Board {
    fn default() -> Self {
        let bitboards = [
            [
                // white
                Bitboard(0xff00),
                Bitboard(0x42),
                Bitboard(0x24),
                Bitboard(0x81),
                Bitboard(0x8),
                Bitboard(0x10),
            ],
            [
                //black
                Bitboard(0xff000000000000),
                Bitboard(0x4200000000000000),
                Bitboard(0x2400000000000000),
                Bitboard(0x8100000000000000),
                Bitboard(0x800000000000000),
                Bitboard(0x1000000000000000),
            ],
        ];

        todo!()
    }
}

impl Board {
    pub fn from_fen(fen: &str) -> Option<Self> {
        let mut this: Self = unsafe { std::mem::zeroed() };

        let (mut y, mut x) = (7, 0);
        let mut parts = fen.as_bytes().split(|ch| *ch == b' ');

        let pos = parts.next()?;
        for &ch in pos {
            if ch == b'/' {
                if x < 7 {
                    return None;
                }
                x = 0;
                y -= 1;
                continue;
            }

            if (b'1'..=b'8').contains(&ch) {
                x += ch - b'0';
                continue;
            }

            let role = match ch.to_ascii_lowercase() {
                b'k' => Role::King,
                b'q' => Role::Queen,
                b'b' => Role::Bishop,
                b'n' => Role::Knight,
                b'r' => Role::Rook,
                b'p' => Role::Pawn,
                _ => return None,
            };

            let color = if ch.is_ascii_lowercase() {
                Color::Black
            } else {
                Color::White
            };

            let sq = Square::new(y as u32 * 8 + x as u32);
            this.bitboards[color][role] |= 1 << sq as u8;
            this.occupancy[color] |= this.bitboards[color][role];
            this.squares[sq] = Some(Piece::new(color, role));

            x += 1;
        }

        if x != 8 || y != 0 {
            return None;
        }

        this.side = match parts.next()?.get(0) {
            Some(b'w') => Color::White,
            Some(b'b') => Color::Black,
            _ => return None,
        };

        this.epsq = None;

        this.castle = (true, true, true, true);

        Some(this)
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in (0..8).rev() {
            write!(f, "{} ", y + 1)?;
            for x in 0..8 {
                let sq = Square::from_coords(x, y);
                let mut piece = None;
                for color in Color::ALL {
                    for role in Role::ALL {
                        if self.bitboards[color][role].contains(sq) {
                            piece = Some(Piece::new(color, role));
                            break;
                        }
                    }
                    if piece.is_some() {
                        break;
                    }
                }

                f.write_char(piece.map(Piece::char).unwrap_or('·'))?;
                // f.write_char(self.squares[sq].map(Piece::char).unwrap_or('·'))?;
                f.write_char(if x < 7 { ' ' } else { '\n' })?;
            }
        }
        f.write_str("  ")?;
        (0..8).for_each(|x| _ = write!(f, "{} ", char::from(b'a' + x)));
        writeln!(f, "\n\nside: {:?}", self.side)?;
        writeln!(f, "epsq: {:?}", self.epsq)?;
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    #[inline(always)]
    pub const fn new(index: u32) -> Self {
        debug_assert!(index < 64);
        unsafe { transmute(index as u8) }
    }

    #[inline(always)]
    pub const fn from_coords(x: u8, y: u8) -> Self {
        let index = y * 8 + x;
        debug_assert!(index < 64);
        unsafe { transmute(index) }
    }

    #[inline(always)]
    pub const fn offset(self, delta: i8) -> Square {
        debug_assert!(-64 < delta && delta < 64);
        unsafe { transmute((self as u8).wrapping_add_signed(delta)) }
    }
}

#[derive(Debug)]
pub struct BySquare<T>([T; 64]);

impl<T> Index<Square> for BySquare<T> {
    type Output = T;

    fn index(&self, sq: Square) -> &Self::Output {
        &self.0[sq as usize]
    }
}

impl<T> IndexMut<Square> for BySquare<T> {
    fn index_mut(&mut self, sq: Square) -> &mut Self::Output {
        &mut self.0[sq as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {
    pub const ALL: [Color; 2] = [Color::Black, Color::White];

    #[inline(always)]
    pub fn other(self) -> Color {
        unsafe { transmute(self as u8 ^ 1) }
    }

    #[inline(always)]
    pub fn is_white(self) -> bool {
        self == Color::White
    }

    #[inline(always)]
    pub fn fold<T>(self, white: T, black: T) -> T {
        match self {
            Color::White => white,
            Color::Black => black,
        }
    }
}

impl std::ops::Not for Color {
    type Output = Color;
    #[inline(always)]
    fn not(self) -> Self::Output {
        self.other()
    }
}

#[derive(Debug)]
pub struct ByColor<T>([T; 2]);

impl<T> ByColor<T> {
    pub const fn new(white: T, black: T) -> Self {
        Self([black, white])
    }
}

impl<T> IndexMut<Color> for ByColor<T> {
    fn index_mut(&mut self, color: Color) -> &mut Self::Output {
        &mut self.0[color as usize]
    }
}

impl<T> Index<Color> for ByColor<T> {
    type Output = T;

    fn index(&self, color: Color) -> &Self::Output {
        &self.0[color as usize]
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Role {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Role {
    pub const ALL: [Role; 6] = [
        Role::Pawn,
        Role::Knight,
        Role::Bishop,
        Role::Rook,
        Role::Queen,
        Role::King,
    ];

    pub const fn char(self) -> char {
        match self {
            Role::Pawn => 'p',
            Role::Knight => 'n',
            Role::Bishop => 'b',
            Role::Rook => 'r',
            Role::Queen => 'q',
            Role::King => 'k',
        }
    }
}

#[derive(Debug)]
pub struct ByRole<T>([T; 6]);

impl<T> Index<Role> for ByRole<T> {
    type Output = T;

    fn index(&self, role: Role) -> &Self::Output {
        &self.0[role as usize]
    }
}

impl<T> IndexMut<Role> for ByRole<T> {
    fn index_mut(&mut self, role: Role) -> &mut Self::Output {
        &mut self.0[role as usize]
    }
}

/// 0001 -> color
/// 1110 -> role
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Piece(u8);

impl Piece {
    #[inline(always)]
    pub const fn new(color: Color, role: Role) -> Self {
        Piece(color as u8 | (role as u8) << 1)
    }

    #[inline(always)]
    pub const fn color(self) -> Color {
        unsafe { transmute(self.0 & 1) }
    }

    #[inline(always)]
    pub const fn role(self) -> Role {
        unsafe { transmute(self.0 >> 1) }
    }

    #[inline(always)]
    pub const fn is_white(&self) -> bool {
        self.0 & 1 != 0
    }

    pub const fn char(self) -> char {
        let ch = self.role().char();
        match self.color() {
            Color::Black => ch,
            Color::White => ch.to_ascii_uppercase(),
        }
    }
}

impl std::fmt::Debug for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Piece")
            .field("color", &self.color())
            .field("role", &self.role())
            .finish()
    }
}

#[derive(Debug)]
pub struct ByPiece<T>(pub ByColor<ByRole<T>>);

impl<T> Index<Piece> for ByPiece<T> {
    type Output = T;

    fn index(&self, p: Piece) -> &Self::Output {
        &self.0[p.color()][p.role()]
    }
}

impl<T> IndexMut<Piece> for ByPiece<T> {
    fn index_mut(&mut self, p: Piece) -> &mut Self::Output {
        &mut self.0[p.color()][p.role()]
    }
}

impl<T> Index<Color> for ByPiece<T> {
    type Output = ByRole<T>;

    fn index(&self, color: Color) -> &Self::Output {
        &self.0[color]
    }
}

impl<T> IndexMut<Color> for ByPiece<T> {
    fn index_mut(&mut self, color: Color) -> &mut Self::Output {
        &mut self.0[color]
    }
}

#[rustfmt::skip]
pub const ALGEBRAIC: BySquare<&'static str> = BySquare([
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
]);
