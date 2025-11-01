mod types;
mod setup;
mod placement;
mod control;
// use std::collections::HashSet;
// use rand::seq::SliceRandom;
// use rand::thread_rng;
use crate::setup::*;
use crate::placement::*;


    


// --- Función Main para probar ---
fn main() {
    let mut board = setup_board();

    let player1 = add_player(&mut board).unwrap(); // Agrega Player1
    let player2 = add_player(&mut board).unwrap(); // Agrega Player2
    let player3 = add_player(&mut board).unwrap(); // Agrega Player3
    let player4 = add_player(&mut board).unwrap(); // Agrega Player4


    place_house(&mut board, player1, 5, true);
    place_house(&mut board, player2, 6, true);
    // println!("¡Tablero de Catan creado!");
    // println!("Total de Vértices: {}", board.vertices.len()); // Debería ser 54
    // println!("Total de Casillas: {}", board.tiles.len());  // Debería ser 19
    // println!("Total de Bordes:   {}", board.edges.len());    // Debería ser 72

    // // Prueba: Muestra los vecinos de la casilla central (Tile 18, "19" en tu mapa)
    // let center_tile = &board.tiles[18];
    // println!("\nCasilla Central (18):");
    // println!("  Material: {:?}", center_tile.material);
    // println!("  Número:   {}", center_tile.number);
    // println!("  Vértices: {:?}", center_tile.vertices); // [47, 48, 49, 50, 51, 52]

    // // Prueba: Muestra los vecinos del vértice 47 (el "48" en tu mapa)
    // let test_vertex = &board.vertices[47];
    // println!("\nVértice 47 ('48' en tu mapa):");
    // println!("  Casillas adyacentes: {:?}", test_vertex.adjacent_tiles); // [0, 11, 12, 18]
    // println!("  Bordes adyacentes:   {:?}", test_vertex.adjacent_edges); // 3 IDs de bordes


}