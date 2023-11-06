pub mod fence_move;
mod move_generation;
pub mod pawn_move;
use core::fmt::Display;

use super::{Axis, Move, PlayerColor};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum FenceLegality {
    #[default]
    Any,
    Horizontal,
    Vertical,
    None,
}

impl FenceLegality {
    const fn restrict(self, axis: Axis) -> Self {
        match (self, axis) {
            (Self::Horizontal | Self::Any, Axis::Vertical) => Self::Horizontal,
            (Self::Vertical | Self::Any, Axis::Horizontal) => Self::Vertical,
            _ => Self::None,
        }
    }

    const fn does_allow(self, axis: Axis) -> bool {
        matches!(
            (self, axis),
            (Self::Any, _)
                | (Self::Horizontal, Axis::Horizontal)
                | (Self::Vertical, Axis::Vertical)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    squares: [[Option<PlayerColor>; 9]; 9],
    fences: [[Option<Axis>; 8]; 8],
    legal_fence_places: [[FenceLegality; 8]; 8],
    black_pawn: (usize, usize),
    white_pawn: (usize, usize),
    fences_left_for_white: u32,
    fences_left_for_black: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: {
                let mut board: [[Option<PlayerColor>; 9]; 9] = Default::default();
                board[0][4] = Some(PlayerColor::Black);
                board[8][4] = Some(PlayerColor::White);
                board
            },
            fences: Default::default(),
            legal_fence_places: Default::default(),
            black_pawn: (4, 0),
            white_pawn: (4, 8),
            fences_left_for_black: 10,
            fences_left_for_white: 10,
        }
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "   ╭───┬───┬───┬───┬───┬───┬───┬───┬───╮")?;
        for row_idx in 0..9 {
            write!(f, "   │")?;
            self.squares[row_idx]
                .iter()
                .enumerate()
                .try_for_each(|(idx, sq)| {
                    let square_fmt = sq.as_ref().map_or("   ", |c| match c {
                        PlayerColor::White => " W ",
                        PlayerColor::Black => " B ",
                    });
                    let bar_fmt = if idx != 8
                        && (row_idx < 8 && self.fences[row_idx][idx] == Some(Axis::Vertical)
                            || row_idx > 0 && self.fences[row_idx - 1][idx] == Some(Axis::Vertical))
                    {
                        "┃"
                    } else {
                        "│"
                    };
                    write!(f, "{square_fmt}{bar_fmt}")
                })?;
            writeln!(f)?;
            if row_idx == 8 {
                break;
            }
            write!(f, " {}─┼", row_idx + 1)?;
            (0..9).try_for_each(|idx| {
                let square_side_fmt = if idx < 8
                    && self.fences[row_idx][idx] == Some(Axis::Horizontal)
                    || idx > 0 && self.fences[row_idx][idx - 1] == Some(Axis::Horizontal)
                {
                    "━━━"
                } else {
                    "───"
                };
                let square_corner_fmt = if idx == 8 {
                    "┤"
                } else {
                    match self.fences[row_idx][idx] {
                        Some(Axis::Vertical) => "╂",
                        Some(Axis::Horizontal) => "┿",
                        _ => "┼",
                    }
                };
                write!(f, "{square_side_fmt}{square_corner_fmt}",)
            })?;
            writeln!(f)?;
        }
        writeln!(f, "   ╰───┼───┼───┼───┼───┼───┼───┼───┼───╯")?;
        writeln!(f, "       A   B   C   D   E   F   G   H    ")?;
        writeln!(f)?;
        writeln!(
            f,
            "   White - {:>2} fences │ Black - {:>2} fences",
            self.fences_left_for_white, self.fences_left_for_black
        )?;
        Ok(())
    }
}
impl Board {
    pub const fn is_game_won(&self) -> Option<PlayerColor> {
        match (self.white_pawn.1, self.black_pawn.1) {
            (0, _) => Some(PlayerColor::White),
            (_, 8) => Some(PlayerColor::Black),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMakeFail {
    AddFenceMove(fence_move::Fail),
    PawnMoveFail(pawn_move::Fail),
}
impl Board {
    pub fn make_move(&mut self, r#move: Move, player: PlayerColor) -> Result<(), MoveMakeFail> {
        match r#move {
            Move::MovePlayer(dir, second_dir) => self
                .move_pawn(player, dir, second_dir)
                .map_err(MoveMakeFail::PawnMoveFail),
            Move::PlaceFence(axis, pos) => self
                .move_fence(player, axis, pos)
                .map_err(MoveMakeFail::AddFenceMove),
        }
    }

    pub fn is_move_legal(&self, r#move: Move, player: PlayerColor) -> Result<(), MoveMakeFail> {
        match r#move {
            Move::MovePlayer(dir, second_dir) => self
                .pawn_move_destination(player, dir, second_dir)
                .map_err(MoveMakeFail::PawnMoveFail)
                .map(|_| ()),
            Move::PlaceFence(axis, pos) => self
                .is_fence_move_legal(player, axis, pos)
                .map_err(MoveMakeFail::AddFenceMove),
        }
    }
}

const fn is_nicely_send<T: Sized + Send + Sync + Unpin>() {}
#[test]
const fn normal_types() {
    is_nicely_send::<Board>();
}
