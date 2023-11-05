use core::fmt::Display;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PawnMoveFail {
    PathObstructed,
    NoSecondaryDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddFenceFail {
    Collides,
    NoPathRemaining,
    NoFencesRemaining,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMakeFail {
    AddFenceFail(AddFenceFail),
    PawnMoveFail(PawnMoveFail),
}

impl Board {
    pub fn make_move(&mut self, r#move: Move, player: PlayerColor) -> Result<(), MoveMakeFail> {
        match r#move {
            Move::MovePlayer(dir, second_dir) => self
                .move_pawn(player, dir, second_dir)
                .map_err(MoveMakeFail::PawnMoveFail),
            Move::PlaceFence(axis, pos) => self
                .move_fence(player, axis, pos)
                .map_err(MoveMakeFail::AddFenceFail),
        }
    }

    pub fn is_move_legal(&mut self, r#move: Move, player: PlayerColor) -> Result<(), MoveMakeFail> {
        match r#move {
            Move::MovePlayer(dir, second_dir) => self
                .pawn_move_destination(player, dir, second_dir)
                .map_err(MoveMakeFail::PawnMoveFail)
                .map(|_| ()),
            Move::PlaceFence(axis, pos) => self
                .is_fence_move_legal(player, axis, pos)
                .map_err(MoveMakeFail::AddFenceFail),
        }
    }

    pub fn move_pawn(
        &mut self,
        pawn: PlayerColor,
        dir: Direction,
        second_dir: Option<Direction>,
    ) -> Result<(), PawnMoveFail> {
        let (x, y) = match pawn {
            PlayerColor::White => self.white_pawn,
            PlayerColor::Black => self.black_pawn,
        };
        let (x1, y1) = self.pawn_move_destination(pawn, dir, second_dir)?;
        assert_eq!(self.squares[y1][x1], None);
        self.squares[y1][x1] = Some(pawn);
        self.squares[y][x] = None;
        match pawn {
            PlayerColor::White => self.white_pawn = (x1, y1),
            PlayerColor::Black => self.black_pawn = (x1, y1),
        };
        Ok(())
    }

    pub fn pawn_move_destination(
        &self,
        pawn: PlayerColor,
        dir: Direction,
        second_dir: Option<Direction>,
    ) -> Result<(usize, usize), PawnMoveFail> {
        use PawnMoveFail::{NoSecondaryDirection, PathObstructed};
        let (x, y) = match pawn {
            PlayerColor::White => self.white_pawn,
            PlayerColor::Black => self.black_pawn,
        };

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
            return Err(NoSecondaryDirection);
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

    pub fn move_fence(
        &mut self,
        player: PlayerColor,
        axis: Axis,
        (x, y): (usize, usize),
    ) -> Result<(), AddFenceFail> {
        self.is_fence_move_legal(player, axis, (x, y))?;
        self.fences[y][x] = Some(axis);
        match player {
            PlayerColor::White => self.fences_left_for_white -= 1,
            PlayerColor::Black => self.fences_left_for_black -= 1,
        };
        Ok(())
    }

    pub fn is_fence_move_legal(
        &mut self,
        player: PlayerColor,
        axis: Axis,
        (x, y): (usize, usize),
    ) -> Result<(), AddFenceFail> {
        use AddFenceFail::{Collides, NoFencesRemaining, NoPathRemaining};
        use Axis::{Horizontal, Vertical};

        if 0 == match player {
            PlayerColor::White => self.fences_left_for_white,
            PlayerColor::Black => self.fences_left_for_black,
        } {
            return Err(NoFencesRemaining);
        }

        if self.fences[y][x].is_some() {
            return Err(Collides);
        }

        match axis {
            Axis::Horizontal => {
                if x > 0 && self.fences[y][x - 1] == Some(Horizontal)
                    || x < 7 && self.fences[y][x + 1] == Some(Horizontal)
                {
                    return Err(Collides);
                }
            }
            Axis::Vertical => {
                if y > 0 && self.fences[y - 1][x] == Some(Vertical)
                    || y < 7 && self.fences[y + 1][x] == Some(Vertical)
                {
                    return Err(Collides);
                }
            }
        }
        self.fences[y][x] = Some(axis);
        if !self.are_pawns_able_to_win() {
            self.fences[y][x] = None;
            return Err(NoPathRemaining);
        }
        self.fences[y][x] = None;
        Ok(())
    }

    pub fn are_pawns_able_to_win(&self) -> bool {
        //dfs for the white pawn
        'white_pawn_dfs: {
            let mut stack = vec![self.white_pawn];
            let mut visited = [[false; 9]; 9];
            visited[self.white_pawn.1][self.white_pawn.0] = true;
            while let Some((x, y)) = stack.pop() {
                if y == 8 {
                    break 'white_pawn_dfs;
                }
                for dir in [
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                ] {
                    if self.is_obstructed((x, y), dir) {
                        continue;
                    }
                    let (x1, y1) = dir.offset((x, y));
                    if visited[y1][x1] {
                        continue;
                    }
                    stack.push((x1, y1));
                    visited[y1][x1] = true;
                }
            }
            return false;
        };
        'black_pawn_dfs: {
            let mut stack = vec![self.black_pawn];
            let mut visited = [[false; 9]; 9];
            visited[self.black_pawn.1][self.black_pawn.0] = true;
            while let Some((x, y)) = stack.pop() {
                if y == 0 {
                    break 'black_pawn_dfs;
                }
                for dir in [
                    Direction::Up,
                    Direction::Left,
                    Direction::Right,
                    Direction::Down,
                ] {
                    if self.is_obstructed((x, y), dir) {
                        continue;
                    }
                    let (x1, y1) = dir.offset((x, y));
                    if visited[y1][x1] {
                        continue;
                    }
                    stack.push((x1, y1));
                    visited[y1][x1] = true;
                }
            }
            return false;
        };
        true
    }

    pub const fn is_game_won(&self) -> Option<PlayerColor> {
        if self.white_pawn.1 == 0 {
            Some(PlayerColor::White)
        } else if self.black_pawn.1 == 8 {
            Some(PlayerColor::Black)
        } else {
            None
        }
    }
}

const fn is_nicely_send<T: Sized + Send + Sync + Unpin>() {}
#[test]
const fn normal_types() {
    is_nicely_send::<Board>();
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
            return Err(UnrecognizedChar)
        }

        parsed
    }
}
