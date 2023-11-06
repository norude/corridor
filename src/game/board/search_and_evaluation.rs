use rand::seq::SliceRandom;

use super::{Board, LegalMove};
use crate::game::PlayerColor;
impl Board {
    pub fn find_best_move(&self, player: PlayerColor) -> LegalMove {
        self.legal_moves(player)
            .choose(&mut rand::thread_rng())
            .copied()
            .expect("A player should always be able to make a move")
    }
}
