use super::{LegalMove, Lmi};
use crate::game::{Axis, Board, Direction, PlayerColor};
impl Board {
    pub fn legal_moves(&self, player: PlayerColor) -> Vec<LegalMove> {
        let mut moves = vec![];
        {
            let player_pos = self.pawn_pos(player);

            for dir in [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ] {
                if self.is_obstructed(player_pos, dir) {
                    continue;
                }
                let (x1, y1) = dir.offset(player_pos);
                if self.squares[y1][x1].is_none() {
                    moves.push(LegalMove(Lmi::MovePlayer(player_pos, (x1, y1))));
                    continue;
                }
                if !self.is_obstructed((x1, y1), dir) {
                    moves.push(LegalMove(Lmi::MovePlayer(player_pos, dir.offset((x1, y1)))));
                    continue;
                }
                for sec_dir in dir.perpendiculars() {
                    if !self.is_obstructed(player_pos, sec_dir) {
                        moves.push(LegalMove(Lmi::MovePlayer(
                            player_pos,
                            sec_dir.offset((x1, y1)),
                        )));
                    }
                }
            }
        }
        for (y, row) in self.legal_fence_places.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                for axis in [Axis::Horizontal, Axis::Vertical]
                    .into_iter()
                    .filter(|&axis| {
                        item.does_allow(axis)
                            && self.is_fence_move_legal(player, axis, (x, y)).is_ok()
                    })
                {
                    moves.push(LegalMove(Lmi::PlaceFence(axis, (x, y))));
                }
            }
        }
        moves
    }
}
