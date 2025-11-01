use crate::types::*;
use crate::setup::*;

pub fn start_game(quantity_players: usize) -> (Board, Vec<PlayerType>) {
    let mut board = setup_board();
    let mut players: Vec<PlayerType> = Vec::new();
    let num: usize = 1;

    while num < quantity_players {
        players.push(add_player(&mut board).unwrap());
    }

    (board, players)
}

pub fn first_turn(board: &mut Board, players: Vec<PlayerType>) {
    
}