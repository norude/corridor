#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(dead_code)]
mod game;

use crate::game::{AddFenceFail, Board, MoveMakeFail, PawnMoveFail, PlayerColor};

fn game_loop() {
    use PlayerColor::{Black, White};
    let mut turn = White;
    let mut board = Board::default();
    loop {
        //'game loop
        println!("{board}");
        //'trying_to_get_a_valid_move loop
        loop {
            let try_into = input_macro::input!("Type in {turn} player's move:").try_into();
            let the_move = match try_into {
                Ok(the_move) => the_move,
                Err(err) => {
                    match err {
                            game::TryIntoMoveError::UnrecognizedChar =>println!(
                                "Couldn't understand the move, because there was an unrecognized character"),
                            game::TryIntoMoveError::UnexpectedEndOfString =>println!(
                                "Couldn't understand the move, because more characters were expected to be given"),
                        }
                    continue;
                }
            };
            if let Err(err) = board.make_move(the_move, turn) {
                match err {
                    MoveMakeFail::PawnMoveFail(PawnMoveFail::PathObstructed) => println!(
                        "Couldn't move the pawn, because the chosen path was obstructed"),
                    MoveMakeFail::PawnMoveFail(PawnMoveFail::NoSecondaryDirection) => println!(
                        "Couldn't move the pawn, because secondary direction was required but not provided"),
                    MoveMakeFail::AddFenceFail(AddFenceFail::Collides) => println!(
                        "Couldn't add the fence there, because it would collide with another fence"),
                    MoveMakeFail::AddFenceFail(AddFenceFail::NoPathRemaining) => println!(
                        "Couldn't add the fence there, because it would leave no path for at least one of the pawns"),
                    MoveMakeFail::AddFenceFail(AddFenceFail::NoFencesRemaining) => println!(
                        "Couldn't add the fence, because there are no fences left for you"
                    )
                };
                continue;
            }
            break;
        }
        turn = match turn {
            White => Black,
            Black => White,
        };
        if let Some(player) = board.is_game_won() {
            println!("{board}");
            println!("{player:?} player won!");
            break;
        }
    }
}

fn main() {
    game_loop();
    // println!("{}",Board::default());
}
