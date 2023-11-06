#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_used,
    clippy::expect_used
)]
#![allow(dead_code)]
mod game;
use game::{fence_move, pawn_move, Board, LegalMove, MoveMakeFail, PlayerColor};

fn get_legal_move_using_players_input(board: &Board, turn: PlayerColor) -> LegalMove {
    loop {
        let try_into = input_macro::input!("Type in {turn} player's move:").try_into();
        let the_move = match try_into {
            Ok(the_move) => the_move,
            Err(err) => {
                match err {
                    game::TryIntoMoveError::UnrecognizedChar => println!(
                        "Couldn't understand the move, because there was an unrecognized character"
                    ),
                    game::TryIntoMoveError::UnexpectedEndOfString => println!(
						"Couldn't understand the move, because more characters were expected to be given"
					),
                }
                continue;
            }
        };
        match board.make_move_legal(the_move, turn) {
            Err(err) => {
                match err {
                    MoveMakeFail::PawnMoveFail(pawn_move::Fail::PathObstructed) => {
                        println!("Couldn't move the pawn, because the chosen path was obstructed")
                    }

                    MoveMakeFail::PawnMoveFail(pawn_move::Fail::NoSecondary) => println!(
						"Couldn't move the pawn, because secondary direction was required but not provided"
					),
                    MoveMakeFail::PawnMoveFail(pawn_move::Fail::InvalidSecondary) => println!(
						"Couldn't move the pawn, because secondary direction was not perpendicular to the primary direction"
					),
                    MoveMakeFail::AddFenceMove(fence_move::Fail::Collides) => println!(
                        "Couldn't add the fence there, because it would collide with another fence"
                    ),
                    MoveMakeFail::AddFenceMove(fence_move::Fail::NoPathRemaining) => println!(
						"Couldn't add the fence there, because it would leave no path for at least one of the pawns"
					),
                    MoveMakeFail::AddFenceMove(fence_move::Fail::NoFencesRemaining) => {
                        println!("Couldn't add the fence, because there are no fences left for you")
                    }
                };
                continue;
            }
            Ok(the_move) => return the_move,
        }
    }
}

fn game_loop() {
    use PlayerColor::{Black, White};
    let mut turn = White;
    let mut board = Board::default();
    loop {
        //'game loop
        println!("{board}");
        board.make_legal_move(get_legal_move_using_players_input(&board, turn), turn);
        if let Some(player) = board.is_game_won() {
            println!("{board}");
            println!("{player:?} player won!");
            break;
        }
        turn = match turn {
            White => Black,
            Black => White,
        };
    }
}

fn main() {
    game_loop();
    // println!("{}",Board::default());
}
const fn fun() -> i32 {
    4
}
