use std::fmt::Write;
use std::mem::transmute;
use std::ops::{Index, IndexMut};

use crate::bitboard::Bitboard;
use crate::movegen::{Array, Move};

use Color::*;
use Role::*;

#[derive(Debug)]
pub struct Board {
    pub bitboards: ByPiece<Bitboard>,
    pub occupancy: ByColor<Bitboard>,
    pub squares: BySquare<Option<Piece>>,

    pub side: Color,
    pub epsq: Option<Square>,
    pub castle: u8,

    pub halfmoves_clock: u32,
    pub fullmoves_count: u32,
    pub key: crate::zobrist::Key,

    pub undos: Array<Undo, 2048>,
}

#[derive(Debug, Clone, Copy)]
pub struct Undo {
    pub m: Move,
    pub epsq: Option<Square>,
    pub castle: u8,
    pub halfmoves_clock: u32,
    pub fullmoves_count: u32,
    pub key: crate::zobrist::Key,
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

            let color = if ch & 32 == 0 { White } else { Black };
            let role = Role::from_char(ch as char)?;

            let sq = Square::from_coords(x, y);
            this.bitboards[color][role].insert(sq);
            this.occupancy[color] |= this.bitboards[color][role];
            this.squares[sq] = Some(role.of(color));

            x += 1;
        }

        if x != 8 || y != 0 {
            return None;
        }

        this.side = Color::from_char(*parts.next()?.get(0)? as char)?;

        let castling_rights = parts.next()?;
        if castling_rights.len() > 4 {
            return None;
        }

        if castling_rights[0] == b'-' {
            this.castle = 0;
        } else {
            for &ch in castling_rights {
                match ch {
                    b'K' => this.castle |= WKSC,
                    b'Q' => this.castle |= WQSC,
                    b'k' => this.castle |= BKSC,
                    b'q' => this.castle |= BQSC,
                    _ => return None,
                }
            }
        }

        let epsq = parts.next()?;
        if epsq[0] == b'-' {
            this.epsq = None;
        } else {
            let x = epsq[0] - b'a';
            let y = epsq[1] - b'1';
            this.epsq = Some(Square::from_coords(x, y));
        }

        this.key = crate::zobrist::hash(&this);

        Some(this)
    }

    #[inline(always)]
    pub fn occupied(&self) -> Bitboard {
        self.occupancy[Color::White] | self.occupancy[Color::Black]
    }

    pub fn check(&self) {
        for (color, role) in Piece::ALL {
            for sq in self.bitboards[color][role] {
                assert_eq!(self.squares[sq], Some(Piece::new(color, role)), "{sq}");
                assert!(self.occupancy[color].contains(sq));
            }
        }

        let occupied = self.occupied();
        for sq in (0..64).map(Square::new) {
            if let Some(p) = self.squares[sq] {
                assert!(self.bitboards[p].contains(sq));
                assert!(self.occupancy[p.color()].contains(sq));
                assert!(!self.occupancy[p.color().other()].contains(sq));
                assert!(occupied.contains(sq));
            } else {
                assert!(!occupied.contains(sq))
            }
        }

        assert_eq!(self.bitboards[Color::White][Role::King].count(), 1);
        assert_eq!(self.bitboards[Color::Black][Role::King].count(), 1);
        assert_eq!(self.key, crate::zobrist::hash(self));
    }

    #[inline(always)]
    pub fn generate(&self) -> Array<Move, 256> {
        crate::movegen::generate(self)
    }

    #[inline(always)]
    pub fn make(&mut self, m: Move) -> bool {
        crate::make::make(self, m)
    }

    #[inline(always)]
    pub fn unmake(&mut self) {
        crate::make::unmake(self)
    }

    #[inline(always)]
    pub fn in_check(&self) -> bool {
        let king = self.bitboards[self.side][Role::King].first().unwrap();
        crate::movegen::is_attacked(self, king, !self.side)
    }
}

/// White king side castle
pub const WKSC: u8 = 0b0001;
/// White queen side castle
pub const WQSC: u8 = 0b0010;
/// Black king side castle
pub const BKSC: u8 = 0b0100;
/// Black queen side castle
pub const BQSC: u8 = 0b1000;

#[rustfmt::skip]
const CASTLING_RIGHTS: [u8; 64] = [
    13, 15, 15, 15, 12, 15, 15, 14,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
     7, 15, 15, 15,  3, 15, 15, 11,
];

#[inline(always)]
pub const fn castle_mask(sq: Square) -> u8 {
    CASTLING_RIGHTS[sq as usize]
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for y in (0..8).rev() {
            write!(f, "{} ", y + 1)?;
            for x in 0..8 {
                let sq = Square::from_coords(x, y);
                let mut piece = None;
                for (color, role) in Piece::ALL {
                    if self.bitboards[color][role].contains(sq) {
                        piece = Some(Piece::new(color, role));
                        break;
                    }
                }
                f.write_char(piece.map(Piece::char).unwrap_or('·'))?;
                f.write_char(if x < 7 { ' ' } else { '\n' })?;
            }
        }
        f.write_str("  a b c d e f g h\n\n")?;
        writeln!(f, "side: {:?}", self.side)?;
        writeln!(f, "epsq: {:?}", self.epsq)?;
        writeln!(f, "castle: {:b}", self.castle)?;
        writeln!(f, "key: {:016x}", self.key)?;
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
    pub const fn offset(self, delta: i32) -> Square {
        debug_assert!(-64 < delta && delta < 64);
        Square::new((self as u32).wrapping_add_signed(delta))
    }

    #[inline(always)]
    pub const fn mirror(self) -> Square {
        unsafe { transmute(self as u8 ^ 56) }
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(ALGEBRAIC[*self])
    }
}

#[derive(Debug)]
pub struct BySquare<T>([T; 64]);

impl<T> Index<Square> for BySquare<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, sq: Square) -> &Self::Output {
        &self.0[sq as usize]
    }
}

impl<T> IndexMut<Square> for BySquare<T> {
    #[inline(always)]
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

    pub const fn from_char(ch: char) -> Option<Color> {
        match ch {
            'w' => Some(Color::White),
            'b' => Some(Color::Black),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn other(self) -> Color {
        unsafe { transmute(self as u8 ^ 1) }
    }

    #[inline(always)]
    pub const fn is_white(self) -> bool {
        self as u8 & 1 != 0
    }

    #[inline(always)]
    pub fn fold<T>(self, white: T, black: T) -> T {
        match self {
            Color::White => white,
            Color::Black => black,
        }
    }

    #[inline(always)]
    pub const fn pawn(self) -> Piece {
        Piece::new(self, Role::Pawn)
    }

    #[inline(always)]
    pub const fn knight(self) -> Piece {
        Piece::new(self, Role::Knight)
    }

    #[inline(always)]
    pub const fn bishop(self) -> Piece {
        Piece::new(self, Role::Bishop)
    }

    #[inline(always)]
    pub const fn rook(self) -> Piece {
        Piece::new(self, Role::Rook)
    }

    #[inline(always)]
    pub const fn queen(self) -> Piece {
        Piece::new(self, Role::Queen)
    }

    #[inline(always)]
    pub const fn king(self) -> Piece {
        Piece::new(self, Role::King)
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

impl<T> Index<Color> for ByColor<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, color: Color) -> &Self::Output {
        &self.0[color as usize]
    }
}

impl<T> IndexMut<Color> for ByColor<T> {
    #[inline(always)]
    fn index_mut(&mut self, color: Color) -> &mut Self::Output {
        &mut self.0[color as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub const fn to_char(self) -> char {
        match self {
            Role::Pawn => 'p',
            Role::Knight => 'n',
            Role::Bishop => 'b',
            Role::Rook => 'r',
            Role::Queen => 'q',
            Role::King => 'k',
        }
    }

    pub const fn from_char(ch: char) -> Option<Role> {
        match ch {
            'P' | 'p' => Some(Role::Pawn),
            'N' | 'n' => Some(Role::Knight),
            'B' | 'b' => Some(Role::Bishop),
            'R' | 'r' => Some(Role::Rook),
            'Q' | 'q' => Some(Role::Queen),
            'K' | 'k' => Some(Role::King),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn of(self, color: Color) -> Piece {
        Piece::new(color, self)
    }
}

#[derive(Debug)]
pub struct ByRole<T>([T; 6]);

impl<T> Index<Role> for ByRole<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, role: Role) -> &Self::Output {
        &self.0[role as usize]
    }
}

impl<T> IndexMut<Role> for ByRole<T> {
    #[inline(always)]
    fn index_mut(&mut self, role: Role) -> &mut Self::Output {
        &mut self.0[role as usize]
    }
}

/// 0001 -> color
/// 1110 -> role
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Piece(u8);

impl Piece {
    #[inline(always)]
    pub const fn new(color: Color, role: Role) -> Self {
        Piece(color as u8 | (role as u8) << 1)
    }

    pub const fn from_char(ch: char) -> Option<Self> {
        let Some(role) = Role::from_char(ch) else {
            return None;
        };
        Some(role.of(if ch as u8 & 32 == 0 { White } else { Black }))
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
        let ch = self.role().to_char();
        match self.color() {
            Color::Black => ch,
            Color::White => ch.to_ascii_uppercase(),
        }
    }

    pub const ALL: [(Color, Role); 12] = [
        (Color::White, Role::Pawn),
        (Color::White, Role::Knight),
        (Color::White, Role::Bishop),
        (Color::White, Role::Rook),
        (Color::White, Role::Queen),
        (Color::White, Role::King),
        (Color::Black, Role::Pawn),
        (Color::Black, Role::Knight),
        (Color::Black, Role::Bishop),
        (Color::Black, Role::Rook),
        (Color::Black, Role::Queen),
        (Color::Black, Role::King),
    ];
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
    #[inline(always)]
    fn index(&self, p: Piece) -> &Self::Output {
        &self.0[p.color()][p.role()]
    }
}

impl<T> IndexMut<Piece> for ByPiece<T> {
    #[inline(always)]
    fn index_mut(&mut self, p: Piece) -> &mut Self::Output {
        &mut self.0[p.color()][p.role()]
    }
}

impl<T> Index<Color> for ByPiece<T> {
    type Output = ByRole<T>;
    #[inline(always)]
    fn index(&self, color: Color) -> &Self::Output {
        &self.0[color]
    }
}

impl<T> IndexMut<Color> for ByPiece<T> {
    #[inline(always)]
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

impl Default for Board {
    fn default() -> Self {
        let bitboards = ByColor([
            ByRole([
                Bitboard(0xff000000000000),
                Bitboard(0x4200000000000000),
                Bitboard(0x2400000000000000),
                Bitboard(0x8100000000000000),
                Bitboard(0x800000000000000),
                Bitboard(0x1000000000000000),
            ]),
            ByRole([
                Bitboard(0xff00),
                Bitboard(0x42),
                Bitboard(0x24),
                Bitboard(0x81),
                Bitboard(0x8),
                Bitboard(0x10),
            ]),
        ]);

        let occupancy = [Bitboard(0xffff000000000000), Bitboard(0xffff)];

        #[rustfmt::skip]
        let squares = [
            Some(White.rook()), Some(White.knight()), Some(White.bishop()), Some(White.queen()), Some(White.king()), Some(White.bishop()), Some(White.knight()), Some(White.rook()),
            Some(White.pawn()), Some(White.pawn()),   Some(White.pawn()),   Some(White.pawn()),  Some(White.pawn()), Some(White.pawn()),   Some(White.pawn()),   Some(White.pawn()),
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            Some(Black.pawn()), Some(Black.pawn()),   Some(Black.pawn()),   Some(Black.pawn()),  Some(Black.pawn()), Some(Black.pawn()),   Some(Black.pawn()),   Some(Black.pawn()),
            Some(Black.rook()), Some(Black.knight()), Some(Black.bishop()), Some(Black.queen()), Some(Black.king()), Some(Black.bishop()), Some(Black.knight()), Some(Black.rook()),
        ];

        let mut board = Self {
            bitboards: ByPiece(bitboards),
            occupancy: ByColor(occupancy),
            squares: BySquare(squares),
            side: Color::White,
            epsq: None,
            castle: WKSC | WQSC | BKSC | BQSC,
            halfmoves_clock: 0,
            fullmoves_count: 1,
            key: 0,
            undos: Array::new(),
        };
        board.key = crate::zobrist::hash(&board);
        board
    }
}
