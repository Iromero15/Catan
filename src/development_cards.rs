// --- En `development_cards.rs` ---

use crate::types::*;
// Importa las funciones de lógica que necesitamos
use crate::game_logic::{place_robber, place_road, update_largest_army}; 

/**
 * Función auxiliar para encontrar y quitar una carta de la
 * mano del jugador. Devuelve `true` si se quitó con éxito.
 */
fn consume_card(player: &mut Player, card: DevelopmentCard) -> bool {
    // Busca la posición de la carta en la mano
    if let Some(position) = player.dev_cards.iter().position(|&c| c == card) {
        // Si la encuentra, la quita
        player.dev_cards.remove(position);
        true
    } else {
        // El jugador no tenía esta carta
        false
    }
}

// --- FUNCIONES PÚBLICAS DE JUEGO ---

/**
 * Juega una carta de Caballero (Knight).
 * Mueve el ladrón y roba a un jugador.
 * Devuelve `true` si se jugó con éxito.
 */
pub fn play_knight_card(
    board: &mut Board,
    player_id: PlayerType,
    new_tile_pos: TileId,
    player_to_rob_id: PlayerType
) -> Option<PlayerType> {
    
    // Paso 1: Encontrar al jugador
    let player_index = board.players.iter().position(|p| p.id == player_id).unwrap();
    
    // Paso 2: Chequear reglas
    if board.players[player_index].played_dev_card_this_turn {
        println!("Error: Ya has jugado una carta de desarrollo este turno.");
        return None;
    }
    
    // Paso 3: Consumir la carta
    if !consume_card(&mut board.players[player_index], DevelopmentCard::Knight) {
        println!("Error: {:?} no tiene una carta de Caballero.", player_id);
        return None;
    }
    
    // Paso 4: Ejecutar la lógica
    let player = &mut board.players[player_index];
    player.played_dev_card_this_turn = true;
    player.knights_played += 1;
    
    println!("¡{:?} ha jugado un Caballero! (Total: {})", player_id, player.knights_played);
    
    // Llamamos a la función de `game_logic` para mover el ladrón
    place_robber(board, player_id, new_tile_pos, player_to_rob_id);
    
    // (Aquí deberías llamar a una función `update_largest_army(board)`)
    
    update_largest_army(board, player_id)
}

/**
 * Juega la carta de Construcción de Caminos (Road Building).
 * Permite al jugador colocar 2 caminos gratis.
 * Devuelve `true` si se jugó con éxito.
 */
pub fn play_road_building_card(
    board: &mut Board,
    player_id: PlayerType,
    edge1_pos: EdgeId,
    edge2_pos: EdgeId
) -> Result<Option<PlayerType>, &'static str> { // <-- 1. TIPO DE RETORNO CORREGIDO

    let player_index = match board.players.iter().position(|p| p.id == player_id) {
        Some(idx) => idx,
        None => return Err("Error: No se encontró al jugador."),
    };

    // --- Chequeos ---
    if board.players[player_index].played_dev_card_this_turn {
        return Err("Error: Ya has jugado una carta de desarrollo este turno.");
    }
    if board.players[player_index].road_quantity < 2 {
        return Err("Error: No tienes suficientes piezas de camino (necesitas 2).");
    }
    if !consume_card(&mut board.players[player_index], DevelopmentCard::RoadBuilding) {
        return Err("Error: No tienes una carta de Construcción de Caminos.");
    }

    // --- Lógica ---
    board.players[player_index].played_dev_card_this_turn = true;
    println!("¡{:?} ha jugado Construcción de Caminos!", player_id);

    // 2. Colocar el primer camino
    // Usamos `?` para propagar el error si `place_road` falla.
    let winner1 = place_road(board, player_id, edge1_pos, TurnPhase::FreeRoad)?;
    if winner1.is_some() {
        return Ok(winner1); // ¡Ganó con el primer camino!
    }
    
    // 3. Colocar el segundo camino
    let winner2 = place_road(board, player_id, edge2_pos, TurnPhase::FreeRoad)?;
    if winner2.is_some() {
        return Ok(winner2); // ¡Ganó con el segundo camino!
    }

    // Si no ganó, devuelve Éxito sin ganador
    Ok(None)
}

/**
 * Juega la carta de Año de la Abundancia (Year of Plenty).
 * El jugador toma 2 recursos cualesquiera del banco.
 * Devuelve `true` si se jugó con éxito.
 */
pub fn play_year_of_plenty_card(
    board: &mut Board,
    player_id: PlayerType,
    material1: MaterialType,
    material2: MaterialType
) -> bool {
    
    // Paso 1: Encontrar al jugador
    let player_index = board.players.iter().position(|p| p.id == player_id).unwrap();

    // Paso 2: Chequear reglas
    if board.players[player_index].played_dev_card_this_turn {
        println!("Error: Ya has jugado una carta de desarrollo este turno.");
        return false;
    }

    // Paso 3: Consumir la carta
    if !consume_card(&mut board.players[player_index], DevelopmentCard::YearOfPlenty) {
        println!("Error: {:?} no tiene una carta de Año de la Abundancia.", player_id);
        return false;
    }

    // Paso 4: Ejecutar la lógica
    let player = &mut board.players[player_index];
    player.played_dev_card_this_turn = true;

    *player.resources.entry(material1).or_insert(0) += 1;
    *player.resources.entry(material2).or_insert(0) += 1;
    
    println!("¡{:?} ha jugado Año de la Abundancia! Recibe 1 de {:?} y 1 de {:?}.", player_id, material1, material2);
    
    true
}

/**
 * Juega la carta de Monopolio (Monopoly).
 * El jugador roba todas las cartas de un recurso de todos los demás jugadores.
 * Devuelve `true` si se jugó con éxito.
 */
pub fn play_monopoly_card(
    board: &mut Board,
    player_id: PlayerType,
    material: MaterialType
) -> bool {

    // Paso 1: Encontrar el ÍNDICE del jugador que juega
    let player_playing_index = board.players.iter().position(|p| p.id == player_id).unwrap();

    // Paso 2: Chequear reglas
    if board.players[player_playing_index].played_dev_card_this_turn {
        println!("Error: Ya has jugado una carta de desarrollo este turno.");
        return false;
    }

    // Paso 3: Consumir la carta
    if !consume_card(&mut board.players[player_playing_index], DevelopmentCard::Monopoly) {
        println!("Error: {:?} no tiene una carta de Monopolio.", player_id);
        return false;
    }
    
    // Paso 4: Ejecutar la lógica
    board.players[player_playing_index].played_dev_card_this_turn = true;
    println!("¡{:?} ha jugado Monopolio sobre {:?}!", player_id, material);
    
    let mut total_stolen = 0;

    // Iteramos por los índices para evitar el borrow checker
    for i in 0..board.players.len() {
        
        // No robarse a sí mismo
        if i == player_playing_index {
            continue;
        }

        // Usamos un bloque para que el préstamo mutable `&mut other_player`
        // termine rápido.
        let stolen_amount = {
            let other_player = &mut board.players[i];
            
            // Tomamos la cantidad que tiene
            let amount = *other_player.resources.get(&material).unwrap_or(&0);
            
            // Si tiene, se lo quitamos
            if amount > 0 {
                *other_player.resources.get_mut(&material).unwrap() = 0;
                println!("- Roba {} de {:?}.", amount, other_player.id);
            }
            amount
        };
        
        total_stolen += stolen_amount;
    }
    
    // Damos el total al jugador que jugó la carta
    let player_playing = &mut board.players[player_playing_index];
    *player_playing.resources.entry(material).or_insert(0) += total_stolen;
    
    println!("¡En total, {:?} robó {} de {:?}!", player_id, total_stolen, material);
    
    true
}