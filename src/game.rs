use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerColor {
    White,
    Black,
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
}

impl PlayerColor {
    const fn repr(&self) -> &str {
        match self {
            Self::White => " W ",
            Self::Black => " B ",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    squares: [[Option<PlayerColor>; 9]; 9],
    fences: [[Option<Axis>; 8]; 8],
    black_pawn: (usize, usize),
    white_pawn: (usize, usize),
}

impl Default for Board {
    fn default() -> Self {
        Self {
            squares: {
                let mut a: [[Option<PlayerColor>; 9]; 9] = Default::default();
                a[0][4] = Some(PlayerColor::Black);
                a[8][4] = Some(PlayerColor::White);
                a
            },
            fences: Default::default(),
            black_pawn: (4, 0),
            white_pawn: (4, 8),
        }
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╭───┬───┬───┬───┬───┬───┬───┬───┬───╮")?;
        for row_idx in 0..9 {
            write!(f, "│")?;
            self.squares[row_idx]
                .iter()
                .enumerate()
                .try_for_each(|(idx, sq)| {
                    write!(
                        f,
                        "{}{}",
                        sq.as_ref().map_or("   ", PlayerColor::repr),
                        if idx == 8 {
                            "│"
                        } else if row_idx < 8 && self.fences[row_idx][idx] == Some(Axis::Vertical)
                            || row_idx > 0 && self.fences[row_idx - 1][idx] == Some(Axis::Vertical)
                        {
                            "┃"
                        } else {
                            "│"
                        }
                    )
                });
            writeln!(f)?;
            if row_idx == 8 {
                break;
            }
            write!(f, "├")?;
            (0..9).try_for_each(|idx| {
                write!(
                    f,
                    "{}{}",
                    if idx < 8 && self.fences[row_idx][idx] == Some(Axis::Horizontal)
                        || idx > 0 && self.fences[row_idx][idx - 1] == Some(Axis::Horizontal)
                    {
                        "━━━"
                    } else {
                        "───"
                    },
                    if idx == 8 {
                        "│"
                    } else {
                        match self.fences[row_idx][idx] {
                            Some(Axis::Vertical) => "╂",
                            Some(Axis::Horizontal) => "┿",
                            _ => "┼",
                        }
                    }
                )
            });
            writeln!(f)?;
        }
        writeln!(f, "╰───┴───┴───┴───┴───┴───┴───┴───┴───╯")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovePawnFail {
    PathObstructed,
    NoSecondaryDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddFencelFail {
    Collides,
    NoPathRemaining,
}

impl Board {
    pub fn move_pawn(
        &mut self,
        pawn: PlayerColor,
        dir: Direction,
        second_dir: Option<Direction>,
    ) -> Result<(), MovePawnFail> {
        use MovePawnFail::{NoSecondaryDirection, PathObstructed};
        let (x, y) = match pawn {
            PlayerColor::White => self.white_pawn,
            PlayerColor::Black => self.black_pawn,
        };

        let move_with_pawn_no_checks = |s: &mut Self, x1: usize, y1: usize| {
            assert_eq!(s.squares[y1][x1], None);
            s.squares[y1][x1] = Some(pawn);
            s.squares[y][x] = None;
            match pawn {
                PlayerColor::White => s.white_pawn = (x1, y1),
                PlayerColor::Black => s.black_pawn = (x1, y1),
            };
            Ok::<(), MovePawnFail>(())
        };

        if self.is_obstructed((x, y), dir) {
            return Err(PathObstructed);
        }

        let (x1, y1) = dir.offset((x, y));
        if self.squares[y1][x1].is_none() {
            return move_with_pawn_no_checks(self, x1, y1);
        }

        if !self.is_obstructed((x1, y1), dir) {
            let (x2, y2) = dir.offset((x1, y1));
            return move_with_pawn_no_checks(self, x2, y2);
        }

        let Some(sec_dir) = second_dir else {
            return Err(NoSecondaryDirection);
        };

        if self.is_obstructed((x1, y1), sec_dir) {
            return Err(PathObstructed);
        }
        let (x2, y2) = sec_dir.offset((x1, y1));
        move_with_pawn_no_checks(self, x2, y2)
    }

    pub fn is_obstructed(&self, (x, y): (usize, usize), dir: Direction) -> bool {
        match dir {
            Direction::Left => {
                x == 0
                    || y < 8 && self.fences[y][x - 1] == Some(Axis::Vertical)
                    || y > 0 && self.fences[y - 1][x - 1] == Some(Axis::Vertical)
            }
            Direction::Right => {
                x >= 8
                    || y < 8 && self.fences[y][x] == Some(Axis::Vertical)
                    || y > 0 && self.fences[y - 1][x] == Some(Axis::Vertical)
            }
            Direction::Down => {
                y >= 8
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

    pub fn add_fence(&mut self, axis: Axis, (x, y): (usize, usize)) -> Result<(), AddFencelFail> {
        use AddFencelFail::{Collides, NoPathRemaining};
        use Axis::{Horizontal, Vertical};
        if self.fences[y][x].is_some() {
            return Err(Collides);
        }

        match axis {
            Axis::Horizontal => {
                if x > 0 && self.fences[y][x - 1] == Some(Horizontal)
                    || x < 9 && self.fences[y][x + 1] == Some(Horizontal)
                {
                    return Err(Collides);
                }
            }
            Axis::Vertical => {
                if y > 0 && self.fences[y - 1][x] == Some(Vertical)
                    || y < 9 && self.fences[y + 1][x] == Some(Vertical)
                {
                    return Err(Collides);
                }
            }
        }

        todo!()
        //TODOOOOO:check if there is a path

        self.fences[y][x] = Some(axis);

        Ok(())
    }
}

const fn is_normal<T: Sized + Send + Sync + Unpin>() {}
#[test]
const fn normal_types() {
    is_normal::<Board>();
}
