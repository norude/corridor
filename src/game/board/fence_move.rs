use super::super::{Axis, Direction, PlayerColor};
use super::{Board, FenceLegality};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fail {
    Collides,
    NoPathRemaining,
    NoFencesRemaining,
}
impl Board {
    pub fn move_fence(
        &mut self,
        player: PlayerColor,
        axis: Axis,
        (x, y): (usize, usize),
    ) -> Result<(), Fail> {
        self.is_fence_move_legal(player, axis, (x, y))?;
        self.move_fence_unchecked(player, axis, (x, y));
        Ok(())
    }

    pub(super) fn move_fence_unchecked(
        &mut self,
        player: PlayerColor,
        axis: Axis,
        (x, y): (usize, usize),
    ) {
        self.fences[y][x] = Some(axis);
        match player {
            PlayerColor::White => self.fences_left_for_white -= 1,
            PlayerColor::Black => self.fences_left_for_black -= 1,
        };

        self.legal_fence_places[y][x] = FenceLegality::None;
        {
            let mut change = |x: usize, y: usize| {
                self.legal_fence_places[y][x] = self.legal_fence_places[y][x].restrict(axis);
            };
            match axis {
                Axis::Horizontal => {
                    if x != 0 {
                        change(x - 1, y);
                    }
                    if x != 7 {
                        change(x + 1, y);
                    }
                }
                Axis::Vertical => {
                    if y != 0 {
                        change(x, y - 1);
                    }
                    if y != 7 {
                        change(x, y + 1);
                    }
                }
            };
        }
    }

    pub fn is_fence_move_legal(
        &self,
        player: PlayerColor,
        axis: Axis,
        (x, y): (usize, usize),
    ) -> Result<(), Fail> {
        use Axis::{Horizontal, Vertical};
        use Fail::{Collides, NoFencesRemaining, NoPathRemaining};

        if 0 == match player {
            PlayerColor::White => self.fences_left_for_white,
            PlayerColor::Black => self.fences_left_for_black,
        } {
            return Err(NoFencesRemaining);
        }

        if let (FenceLegality::None, _)
        | (FenceLegality::Vertical, Horizontal)
        | (FenceLegality::Horizontal, Vertical) = (self.legal_fence_places[y][x], axis)
        {
            return Err(Collides);
        }

        let mut clone = self.clone();
        clone.fences[y][x] = Some(axis);
        let are_pawns_able_to_win = clone.are_pawns_able_to_win();

        if are_pawns_able_to_win {
            Ok(())
        } else {
            Err(NoPathRemaining)
        }
    }

    pub fn are_pawns_able_to_win(&self) -> bool {
        let dfs = |pawn: (usize, usize), yl, dirs| {
            let mut stack = vec![pawn];
            let mut is_on_stack = [[false; 9]; 9];
            is_on_stack[pawn.1][pawn.0] = true;
            while let Some((x, y)) = stack.pop() {
                if y == yl {
                    return true;
                }
                for dir in dirs {
                    if self.is_obstructed((x, y), dir) {
                        continue;
                    }
                    let (x1, y1) = dir.offset((x, y));
                    if is_on_stack[y1][x1] {
                        continue;
                    }
                    stack.push((x1, y1));
                    is_on_stack[y1][x1] = true;
                }
            }
            false
        };
        dfs(
            self.white_pawn,
            0,
            [
                Direction::Down,
                Direction::Left,
                Direction::Right,
                Direction::Up,
            ],
        ) && dfs(
            self.black_pawn,
            8,
            [
                Direction::Down,
                Direction::Left,
                Direction::Right,
                Direction::Up,
            ],
        )
    }
}
