#![allow(unused)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use input_macro::input;
mod game;
use crate::game::{Axis, Board, Direction, MovePawnFail, PlayerColor};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Move {
    MovePlayer(Direction, Option<Direction>),
    PlaceFence(Axis, usize, usize),
}
fn game_loop() {
    use Axis::{Horizontal, Vertical};
    use Direction::{Down, Left, Right, Up};
    use Move::{MovePlayer, PlaceFence};
    use PlayerColor::{Black, White};

    let mut turn = White;
    let mut board = Board::default();
    loop {
        println!("{board}");

        loop {
            let str_move = input!("Type in {:?} player's move:", turn);
            let mut chars_move = str_move.chars();
            let Some(first) = chars_move.next() else {
                println!("You didn't type anything");
                continue;
            };
            let Some(the_move) = (match first.to_lowercase().next().unwrap_or(' ') {
                dir @ ('w' | 'a' | 's' | 'd') => 'parse_pawn_move: {
                    let second_dir = {
                        match chars_move.next() {
                            Some('w') => Some(Up),
                            Some('a') => Some(Left),
                            Some('s') => Some(Down),
                            Some('d') => Some(Right),
                            None => None,
                            Some(_) => break 'parse_pawn_move None,
                        }
                    };
                    match dir {
                        'w' => Some(MovePlayer(Up, second_dir)),
                        'a' => Some(MovePlayer(Left, second_dir)),
                        's' => Some(MovePlayer(Down, second_dir)),
                        'd' => Some(MovePlayer(Right, second_dir)),
                        _ => unreachable!(),
                    }
                }
                axis @ ('-' | 'h' | '|' | 'v') => 'parse_fence_move: {
                    let (Some(x_c), Some(y_c)) = (chars_move.next(), chars_move.next()) else {
                        break 'parse_fence_move None;
                    };
                    let (x, y) = (x_c as usize - 'a' as usize, y_c as usize - '1' as usize);
                    if !(0..9).contains(&x) || !(0..9).contains(&y) {
                        break 'parse_fence_move None;
                    }
                    Some(PlaceFence(
                        match axis {
                            '-' | 'h' => Horizontal,
                            '|' | 'v' => Vertical,
                            _ => unreachable!(),
                        },
                        x,
                        y,
                    ))
                }
                _ => None,
            }) else {
                println!("You didn't type a valid move string");
                continue;
            };

            println!("{the_move:?}");

            match the_move {
                MovePlayer(d, sd) => {
                    if let Err(err) = board.move_pawn(turn, d, sd) {
                        println!(
                            "Couldn't make the move, because {}",
                            match err {
                                MovePawnFail::PathObstructed => "the choosen path was obstructed",
                                MovePawnFail::NoSecondaryDirection =>
                                    "secondary direction was required but not provided",
                            }
                        );
                        continue;
                    }
                }
                PlaceFence(axis, x, y) => {
                    if let Err(err) = board.add_fence(axis, (x, y)) {
                        println!(
                            "Couldn't add the fence there, because it would {}",
                            match err {
                                game::AddFencelFail::Collides => "collide with another fence",
                                game::AddFencelFail::NoPathRemaining =>
                                    "leave no path for the other pawn",
                            }
                        );
                        continue;
                    }
                }
            };
            break;
        }
        turn = match turn {
            White => Black,
            Black => White,
        }
    }
}

fn main() {
    game_loop();
    // let mut b = Board::default();
    // b.fences[5][4] = Some(fence::Horizontal);
    // println!("{b}");
}
