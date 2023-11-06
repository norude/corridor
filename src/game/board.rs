mod move_generation;
mod search_and_evaluation;

pub mod fence_move;
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
                        "\x1b[31m┃\x1b[0m"
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
                let square_side_fmt =
                    if idx < 8 && self.fences[row_idx][idx] == Some(Axis::Horizontal) {
                        "\x1b[31m━━━"
                    } else if idx > 0 && self.fences[row_idx][idx - 1] == Some(Axis::Horizontal) {
                        "━━━\x1b[0m"
                    } else {
                        "───"
                    };
                let square_corner_fmt = if idx == 8 {
                    "┤"
                } else {
                    match self.fences[row_idx][idx] {
                        Some(Axis::Vertical) => "\x1b[31m┃\x1b[0m",
                        Some(Axis::Horizontal) => "━",
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

    const fn pawn_pos(&self, player: PlayerColor) -> (usize, usize) {
        match player {
            PlayerColor::White => self.white_pawn,
            PlayerColor::Black => self.black_pawn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMakeFail {
    AddFenceMove(fence_move::Fail),
    PawnMoveFail(pawn_move::Fail),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Lmi {
    // LegalMove inner workings
    MovePlayer((usize, usize), (usize, usize)),
    PlaceFence(Axis, (usize, usize)),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LegalMove(Lmi);

impl Board {
    pub fn make_legal_move(&mut self, r#move: LegalMove, player: PlayerColor) {
        match r#move {
            LegalMove(Lmi::MovePlayer(orig_pos, pos)) => {
                self.move_pawn_unchecked(player, orig_pos, pos)
            }
            LegalMove(Lmi::PlaceFence(axis, pos)) => self.move_fence_unchecked(player, axis, pos),
        }
    }

    pub fn unmake_legal_move(&mut self, r#move: LegalMove, player: PlayerColor) {
        match r#move {
            LegalMove(Lmi::MovePlayer(orig_pos, pos)) => {
                self.unmove_pawn_unchecked(player, orig_pos, pos)
            }
            LegalMove(Lmi::PlaceFence(axis, pos)) => self.unmove_fence_unchecked(player, axis, pos),
        }
    }

    pub fn make_move_legal(
        &self,
        r#move: Move,
        player: PlayerColor,
    ) -> Result<LegalMove, MoveMakeFail> {
        Ok(LegalMove(match r#move {
            Move::MovePlayer(dir, second_dir) => Lmi::MovePlayer(
                self.pawn_pos(player),
                self.pawn_move_destination(player, dir, second_dir)
                    .map_err(MoveMakeFail::PawnMoveFail)?,
            ),
            Move::PlaceFence(axis, pos) => {
                self.is_fence_move_legal(player, axis, pos)
                    .map_err(MoveMakeFail::AddFenceMove)?;
                Lmi::PlaceFence(axis, pos)
            }
        }))
    }
}

const fn is_nicely_send<T: Sized + Send + Sync + Unpin>() {}
#[test]
const fn normal_types() {
    is_nicely_send::<Board>();
}
