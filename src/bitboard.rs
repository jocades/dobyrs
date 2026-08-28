use core::fmt::{self, Write};
use core::ops;

use crate::board::Square;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bitboard(pub u64);

impl Bitboard {
    #[inline(always)]
    pub const fn from_square(sq: Square) -> Self {
        Bitboard(1 << sq as u8)
    }

    #[inline(always)]
    pub const fn from_squares<const N: usize>(squares: [Square; N]) -> Self {
        let mut i = 0;
        let mut bits = 0u64;
        while i < N {
            bits |= 1 << squares[i] as u8;
            i += 1;
        }
        Bitboard(bits)
    }

    #[inline(always)]
    pub const fn insert(&mut self, sq: Square) {
        self.0 |= 1 << sq as u8;
    }

    #[inline(always)]
    pub const fn remove(&mut self, sq: Square) {
        self.0 &= !(1 << sq as u8);
    }

    #[inline(always)]
    pub const fn replace(&mut self, src: Square, dst: Square) {
        self.remove(src);
        self.insert(dst);
    }

    #[inline(always)]
    pub const fn contains(self, sq: Square) -> bool {
        self.0 & (1 << sq as u8) != 0
    }

    #[inline(always)]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline(always)]
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub const fn first(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(Square::new(self.0.trailing_zeros()))
        }
    }
}

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(n: u64) -> Self {
        Bitboard(n)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(sq: Square) -> Self {
        Bitboard::from_square(sq)
    }
}

impl<T: Into<Bitboard>> ops::BitAnd<T> for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let Bitboard(rhs) = rhs.into();
        Bitboard(self.0 & rhs)
    }
}

impl<T: Into<Bitboard>> ops::BitAndAssign<T> for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let Bitboard(rhs) = rhs.into();
        self.0 &= rhs;
    }
}

impl<T: Into<Bitboard>> ops::BitOr<T> for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn bitor(self, rhs: T) -> Bitboard {
        let Bitboard(rhs) = rhs.into();
        Bitboard(self.0 | rhs)
    }
}

impl<T: Into<Bitboard>> ops::BitOrAssign<T> for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let Bitboard(rhs) = rhs.into();
        self.0 |= rhs;
    }
}

impl<T: Into<Bitboard>> ops::BitXor<T> for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn bitxor(self, rhs: T) -> Bitboard {
        let Bitboard(rhs) = rhs.into();
        Bitboard(self.0 ^ rhs)
    }
}

impl<T: Into<Bitboard>> ops::BitXorAssign<T> for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let Bitboard(rhs) = rhs.into();
        self.0 ^= rhs;
    }
}

impl ops::Not for Bitboard {
    type Output = Bitboard;
    #[inline(always)]
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

pub struct IntoIter(u64);

impl Iterator for IntoIter {
    type Item = Square;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        let sq = Square::new(self.0.trailing_zeros());
        self.0 &= self.0 - 1;
        Some(sq)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.count_ones() as usize;
        (len, Some(len))
    }
}

impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0)
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in (0..8).rev() {
            write!(f, "{} ", y + 1)?;
            for x in 0..8 {
                let sq = Square::from_coords(x, y);
                f.write_char(if self.contains(sq) { '#' } else { '·' })?;
                f.write_char(if x < 7 { ' ' } else { '\n' })?;
            }
        }
        f.write_str("  a b c d e f g h\n")?;
        writeln!(f)
    }
}
