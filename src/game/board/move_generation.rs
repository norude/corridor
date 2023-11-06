use crate::game::{Axis, Board, Direction, Move, PlayerColor};

impl Board {
    pub fn legal_moves(&self, player: PlayerColor) -> Vec<Move> {
        let mut moves = vec![];
        {
            let (x, y) = match player {
                PlayerColor::White => self.white_pawn,
                PlayerColor::Black => self.black_pawn,
            };

            for dir in [
                Direction::Left,
                Direction::Right,
                Direction::Up,
                Direction::Down,
            ] {
                if self.is_obstructed((x, y), dir) {
                    continue;
                }
                let (x1, y1) = dir.offset((x, y));
                if !self.is_obstructed((x1, y1), dir) || self.squares[y1][x1].is_none() {
                    moves.push(Move::MovePlayer(dir, None));
                    continue;
                }
                for sec_dir in dir.perpendiculars() {
                    if !self.is_obstructed((x, y), sec_dir) {
                        moves.push(Move::MovePlayer(dir, Some(sec_dir)));
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
                    moves.push(Move::PlaceFence(axis, (x, y)));
                }
            }
        }
        moves
    }
}
