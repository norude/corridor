use super::super::{Axis, Board, Direction, PlayerColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fail {
    PathObstructed,
    NoSecondary,
    InvalidSecondary,
}
impl Board {
    pub(super) fn move_pawn_unchecked(
        &mut self,
        pawn: PlayerColor,
        (xo, yo): (usize, usize),
        (x, y): (usize, usize),
    ) {
        self.squares[yo][xo] = None;
        self.squares[y][x] = Some(pawn);
        match pawn {
            PlayerColor::White => self.white_pawn = (x, y),
            PlayerColor::Black => self.black_pawn = (x, y),
        };
    }
    pub(super) fn unmove_pawn_unchecked(
        &mut self,
        pawn: PlayerColor,
        (xo, yo): (usize, usize),
        (x, y): (usize, usize),
    ) {
        self.squares[y][x] = None;
        self.squares[yo][xo] = Some(pawn);
        match pawn {
            PlayerColor::White => self.white_pawn = (xo, yo),
            PlayerColor::Black => self.black_pawn = (xo, yo),
        };
    }
    pub fn pawn_move_destination(
        &self,
        player: PlayerColor,
        dir: Direction,
        second_dir: Option<Direction>,
    ) -> Result<(usize, usize), Fail> {
        use Fail::{InvalidSecondary, NoSecondary, PathObstructed};
        let (x, y) = self.pawn_pos(player);

        if self.is_obstructed((x, y), dir) {
            return Err(PathObstructed);
        }

        let (x1, y1) = dir.offset((x, y));
        if self.squares[y1][x1].is_none() {
            return Ok((x1, y1));
        }

        if !self.is_obstructed((x1, y1), dir) {
            let (x2, y2) = dir.offset((x1, y1));
            return Ok((x2, y2));
        }

        let Some(sec_dir) = second_dir else {
            return Err(NoSecondary);
        };

        if dir.are_parallel(sec_dir) {
            return Err(InvalidSecondary);
        };

        if self.is_obstructed((x1, y1), sec_dir) {
            return Err(PathObstructed);
        }
        Ok(sec_dir.offset((x1, y1)))
    }

    pub fn is_obstructed(&self, (x, y): (usize, usize), dir: Direction) -> bool {
        match dir {
            Direction::Left => {
                x == 0
                    || y < 8 && self.fences[y][x - 1] == Some(Axis::Vertical)
                    || y > 0 && self.fences[y - 1][x - 1] == Some(Axis::Vertical)
            }
            Direction::Right => {
                x == 8
                    || y < 8 && self.fences[y][x] == Some(Axis::Vertical)
                    || y > 0 && self.fences[y - 1][x] == Some(Axis::Vertical)
            }
            Direction::Down => {
                y == 8
                    || x < 8 && self.fences[y][x] == Some(Axis::Horizontal)
                    || x > 0 && self.fences[y][x - 1] == Some(Axis::Horizontal)
            }
            Direction::Up => {
                y == 0
                    || x < 8 && self.fences[y - 1][x] == Some(Axis::Horizontal)
                    || x > 0 && self.fences[y - 1][x - 1] == Some(Axis::Horizontal)
            }
        }
    }
}
