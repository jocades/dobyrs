use crate::{
    attacks::pawn_attacks,
    board::{Board, Color, Piece, Role::Pawn, Square},
};

pub type Key = u64;

const SEED: u64 = 0x1806_2011_d0b1_5eed;

pub struct ZobristKeys {
    piece: [[[Key; 64]; 6]; 2],
    castle: [Key; 16],
    en_passant: [Key; 64],
    side: Key,
}

static KEYS: ZobristKeys = {
    let mut rng = SplitMix64(SEED);

    let mut piece = [[[0; 64]; 6]; 2];
    let mut color = 0;
    while color < 2 {
        let mut role = 0;
        while role < 6 {
            let mut sq = 0;
            while sq < 64 {
                piece[color][role][sq] = rng.next();
                sq += 1;
            }
            role += 1;
        }
        color += 1;
    }

    let mut castle = [0; 16];
    let mut i = 0;
    while i < castle.len() {
        castle[i] = rng.next();
        i += 1;
    }

    let mut en_passant = [0; 64];
    i = 0;
    while i < en_passant.len() {
        en_passant[i] = rng.next();
        i += 1;
    }

    ZobristKeys {
        piece,
        castle,
        en_passant,
        side: rng.next(),
    }
};

#[inline(always)]
pub fn piece(piece: Piece, sq: Square) -> Key {
    KEYS.piece[piece.color() as usize][piece.role() as usize][sq as usize]
}

#[inline(always)]
pub fn castle(rights: u8) -> Key {
    debug_assert!(rights < 16);
    KEYS.castle[rights as usize]
}

#[inline(always)]
pub fn en_passant(sq: Square) -> Key {
    KEYS.en_passant[sq as usize]
}

/// Returns the en-passant component only when the side to move has a pawn
/// that attacks the target. An irrelevant FEN en-passant square therefore
/// does not make an otherwise identical chess position hash differently.
#[inline]
pub fn en_passant_component(board: &Board) -> Key {
    let Some(sq) = board.epsq else {
        return 0;
    };

    let candidates = pawn_attacks(!board.side, sq) & board.bitboards[board.side][Pawn];
    if candidates.any() { en_passant(sq) } else { 0 }
}

#[inline(always)]
pub fn side() -> Key {
    KEYS.side
}

/// Computes a key from scratch. Use this when constructing a position and in
/// assertions/tests; normal move making should update `Board::key` with XORs.
pub fn hash(board: &Board) -> Key {
    let mut key = castle(board.castle);

    for (color, role) in Piece::ALL {
        for sq in board.bitboards[color][role] {
            key ^= piece(role.of(color), sq);
        }
    }

    if board.side == Color::Black {
        key ^= side();
    }

    key ^= en_passant_component(board);

    key
}

/// A small, deterministic generator with good bit diffusion for producing
/// Zobrist constants. It is not used for cryptography.
struct SplitMix64(u64);

impl SplitMix64 {
    const fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(board: &mut Board, src: Square, dst: Square) {
        let mv = board
            .generate()
            .into_iter()
            .find(|mv| mv.src == src && mv.dst == dst)
            .unwrap();
        assert!(board.make(mv));
    }

    fn check_moves(fen: &str) {
        let mut board = Board::from_fen(fen).unwrap();
        let original = board.key;
        assert_eq!(original, hash(&board));

        for mv in board.generate() {
            if !board.make(mv) {
                assert_eq!(board.key, original);
                continue;
            }

            assert_eq!(board.key, hash(&board));
            board.unmake();
            assert_eq!(board.key, original);
            assert_eq!(board.key, hash(&board));
        }
    }

    #[test]
    fn incremental_key_handles_special_moves() {
        check_moves("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        check_moves("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        check_moves("4k3/P7/8/8/8/8/8/4K3 w - - 0 1");
    }

    #[test]
    fn key_contains_only_position_identity() {
        let base = Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 b - - 0 1").unwrap();
        let different_clocks = Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 b - - 73 42").unwrap();
        let irrelevant_ep = Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 b - e3 0 1").unwrap();
        let other_side = Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 w - - 0 1").unwrap();
        let other_rights = Board::from_fen("4k3/8/8/8/4P3/8/8/4K3 b K - 0 1").unwrap();

        assert_eq!(base.key, different_clocks.key);
        assert_eq!(base.key, irrelevant_ep.key);
        assert_ne!(base.key, other_side.key);
        assert_ne!(base.key, other_rights.key);
    }

    #[test]
    fn returning_to_a_position_returns_to_its_key() {
        use Square::{B1, B8, C3, C6};

        let mut board = Board::default();
        board.check();
        let initial = board.key;

        play(&mut board, B1, C3);
        play(&mut board, B8, C6);
        play(&mut board, C3, B1);
        play(&mut board, C6, B8);

        assert_eq!(board.key, initial);
        assert_eq!(board.key, hash(&board));
    }
}
