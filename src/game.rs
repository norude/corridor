pub mod board;
use core::fmt::Display;

pub use board::{fence_move, pawn_move, Board, LegalMove, MoveMakeFail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerColor {
    White,
    Black,
}
impl Display for PlayerColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::White => write!(f, "white"),
            Self::Black => write!(f, "black"),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
impl Direction {
    pub const fn offset(self, (x, y): (usize, usize)) -> (usize, usize) {
        match self {
            Self::Left => (x - 1, y),
            Self::Right => (x + 1, y),
            Self::Down => (x, y + 1),
            Self::Up => (x, y - 1),
        }
    }

    pub const fn are_parallel(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Left | Self::Right, Self::Left | Self::Right)
                | (Self::Up | Self::Down, Self::Up | Self::Down)
        )
    }

    pub const fn perpendiculars(self) -> [Self; 2] {
        match self {
            Self::Down | Self::Up => [Self::Left, Self::Right],
            Self::Left | Self::Right => [Self::Down, Self::Up],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    MovePlayer(Direction, Option<Direction>),
    PlaceFence(Axis, (usize, usize)),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryIntoMoveError {
    UnrecognizedChar,
    UnexpectedEndOfString,
}
impl TryFrom<String> for Move {
    type Error = TryIntoMoveError;

    fn try_from(value: String) -> Result<Self, TryIntoMoveError> {
        use Axis::{Horizontal, Vertical};
        use Direction::{Down, Left, Right, Up};
        use Move::{MovePlayer, PlaceFence};
        use TryIntoMoveError::{UnexpectedEndOfString, UnrecognizedChar};
        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return Err(UnexpectedEndOfString);
        };
        let parsed = match first.to_lowercase().next().ok_or(UnrecognizedChar)? {
            dir @ ('w' | 'a' | 's' | 'd') => {
                let second_dir = {
                    match chars.next() {
                        Some('w') => Some(Up),
                        Some('a') => Some(Left),
                        Some('s') => Some(Down),
                        Some('d') => Some(Right),
                        None => None,
                        Some(_) => return Err(UnrecognizedChar),
                    }
                };
                match dir {
                    'w' => Ok(MovePlayer(Up, second_dir)),
                    'a' => Ok(MovePlayer(Left, second_dir)),
                    's' => Ok(MovePlayer(Down, second_dir)),
                    'd' => Ok(MovePlayer(Right, second_dir)),
                    _ => unreachable!(),
                }
            }
            axis @ ('-' | 'h' | '|' | 'v') => {
                let (Some(x_c), Some(y_c)) = (chars.next(), chars.next()) else {
                    return Err(UnexpectedEndOfString);
                };
                let (x, y) = (x_c as usize - 'a' as usize, y_c as usize - '1' as usize);
                if !(0..9).contains(&x) || !(0..9).contains(&y) {
                    return Err(UnrecognizedChar);
                }
                Ok(PlaceFence(
                    match axis {
                        '-' | 'h' => Horizontal,
                        '|' | 'v' => Vertical,
                        _ => unreachable!(),
                    },
                    (x, y),
                ))
            }
            _ => Err(UnrecognizedChar),
        };
        if chars.peekable().peek().is_some() {
            return Err(UnrecognizedChar);
        }

        parsed
    }
}
