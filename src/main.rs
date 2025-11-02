// en src/main.rs

// Declaración de todos tus módulos
mod types;
mod setup;
mod game_logic;
mod development_cards;
mod terminal_game;

// Usar lo que necesitas
use crate::setup::setup_board;
use crate::terminal_game::start_game;

fn main() {
    // 1. Crea el tablero
    let mut board = setup_board();
    
    // 2. Inicia el juego
    start_game(&mut board);
}