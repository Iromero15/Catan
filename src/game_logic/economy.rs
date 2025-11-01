// en src/game_logic/economy.rs

use crate::types::*;
use rand::prelude::*;
use std::collections::HashMap;

// --- CONSTANTES DE COSTO ---
const DEVELOPMENT_CARD_COST: &[(MaterialType, u8)] = &[
    (MaterialType::Sheep, 1),
    (MaterialType::Wheat, 1),
    (MaterialType::Stone, 1),
];

// --- FUNCIONES AUXILIARES (PRIVADAS) ---

pub fn has_resources(player: &Player, cost: &[(MaterialType, u8)]) -> bool {
    for &(material, required_count) in cost {
        let current_count = player.resources.get(&material).unwrap_or(&0);
        if *current_count < required_count {
            return false;
        }
    }
    true
}

pub fn spend_resources(player: &mut Player, cost: &[(MaterialType, u8)]) {
    for &(material, required_count) in cost {
        let current_count = player.resources.get_mut(&material)
            .expect("Error: Se intentó gastar un recurso que no existía o no era suficiente.");
        *current_count -= required_count;
    }
}

fn get_players_adjacent_to_tile(board: &Board, tile_id: TileId) -> Vec<PlayerType> {
    let mut players_on_tile = Vec::new();
    let tile = &board.tiles[tile_id];

    for &vertex_id in &tile.vertices {
        if let Some(owner) = board.vertices[vertex_id].owner {
            if !players_on_tile.contains(&owner) {
                players_on_tile.push(owner);
            }
        }
    }
    players_on_tile
}

// --- FUNCIONES PÚBLICAS ---

pub fn give_materials_on_roll(board: &mut Board, number_rolled: u8) {
    let mut payouts: HashMap<PlayerType, HashMap<MaterialType, u8>> = HashMap::new();

    for tile in board.tiles.iter() {
        if tile.number == number_rolled && !tile.has_robber {
            let material = tile.material;
            for &vertex_id in &tile.vertices {
                if let Some(owner_id) = board.vertices[vertex_id].owner {
                    if let Some(building) = board.vertices[vertex_id].building {
                        let amount = match building {
                            BuildingType::Settlement => 1,
                            BuildingType::City => 2,
                        };
                        let player_payout = payouts.entry(owner_id).or_insert(HashMap::new());
                        let material_count = player_payout.entry(material).or_insert(0);
                        *material_count += amount;
                    }
                }
            }
        }
    }

    if payouts.is_empty() {
        println!("Tirada {}: Ninguna casilla produjo recursos.", number_rolled);
        return;
    }
    println!("Tirada {}: ¡Repartiendo recursos!", number_rolled);
    for player in board.players.iter_mut() {
        if let Some(gains) = payouts.get(&player.id) {
            for (&material, &amount) in gains {
                let resource_count = player.resources.entry(material).or_insert(0);
                *resource_count += amount;
                println!("- {:?} recibe {} de {:?}", player.id, amount, material);
            }
        }
    }
}

pub fn give_starting_resources(
    board: &mut Board, 
    player_id: PlayerType, 
    settlement_pos: VertexId
) {
    let mut resources_to_gain: Vec<MaterialType> = Vec::new();
    for &tile_id in &board.vertices[settlement_pos].adjacent_tiles {
        let tile = &board.tiles[tile_id];
        if tile.material != MaterialType::Dessert {
            resources_to_gain.push(tile.material);
        }
    }

    let player = match board.players.iter_mut().find(|p| p.id == player_id) {
        Some(p) => p,
        None => {
            println!("Error: No se encontró al jugador {:?} para darle sus recursos.", player_id);
            return;
        }
    };
    
    println!("Dando recursos iniciales a {:?}:", player_id);
    for material in resources_to_gain {
        let resource_count = player.resources.entry(material).or_insert(0);
        *resource_count += 1;
        println!("- 1 de {:?}", material);
    }
}

pub fn trade_with_bank(
    board: &mut Board,
    player_id: PlayerType,
    material_to_give: MaterialType,
    material_to_get: MaterialType
) -> bool {
    if material_to_give == material_to_get {
        println!("Error de intercambio: No puedes intercambiar un material por sí mismo.");
        return false;
    }
    if material_to_give == MaterialType::Dessert || material_to_get == MaterialType::Dessert {
        println!("Error de intercambio: No se puede comerciar con el Desierto.");
        return false;
    }

    let player_index = match board.players.iter().position(|p| p.id == player_id) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id);
            return false;
        }
    };

    let player = &board.players[player_index];
    let mut required_to_give = 4;

    let has_specific_port = 
        (material_to_give == MaterialType::Wheat && player.power_ups.contains(&PowerUp::Wheat2)) ||
        (material_to_give == MaterialType::Brick && player.power_ups.contains(&PowerUp::Brick2)) ||
        (material_to_give == MaterialType::Stone && player.power_ups.contains(&PowerUp::Stone2)) ||
        (material_to_give == MaterialType::Sheep && player.power_ups.contains(&PowerUp::Sheep2)) ||
        (material_to_give == MaterialType::Wood  && player.power_ups.contains(&PowerUp::Wood2));
    
    if has_specific_port {
        required_to_give = 2;
    } else if player.power_ups.contains(&PowerUp::Any3) {
        required_to_give = 3;
    }

    let current_resource_count = player.resources.get(&material_to_give).unwrap_or(&0);
    if *current_resource_count < required_to_give {
        println!(
            "Error de intercambio: {:?} necesita {} de {:?} pero solo tiene {}.",
            player_id, required_to_give, material_to_give, *current_resource_count
        );
        return false;
    }

    let player = &mut board.players[player_index];
    let give_count = player.resources.get_mut(&material_to_give).unwrap();
    *give_count -= required_to_give;
    let get_count = player.resources.entry(material_to_get).or_insert(0);
    *get_count += 1;

    println!(
        "¡Intercambio exitoso! {:?} entregó {} de {:?} y recibió 1 de {:?}.",
        player_id, required_to_give, material_to_give, material_to_get
    );
    true
}

pub fn buy_development_card(
    board: &mut Board, 
    player_id_type: PlayerType
) -> Option<DevelopmentCard> {
    
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    if board.development_cards.is_empty() {
        println!("No se puede comprar: ¡El mazo de cartas de desarrollo está vacío!");
        return None;
    }

    if !has_resources(&board.players[player_index], DEVELOPMENT_CARD_COST) {
        println!("No se puede comprar: {:?} no tiene los recursos necesarios.", player_id_type);
        return None;
    }

    let card_drawn = board.development_cards.pop().unwrap();
    println!("¡{:?} ha comprado una carta de desarrollo!", player_id_type);

    let player = &mut board.players[player_index];
    spend_resources(player, DEVELOPMENT_CARD_COST);
    player.dev_cards.push(card_drawn);

    if card_drawn == DevelopmentCard::VictoryPoint {
        println!("¡La carta era un Punto de Victoria!");
        player.victory_points += 1;
        // ¡Importante! Necesitamos llamar a la función de victoria.
        // La haremos pública desde `victory.rs` y la importaremos aquí.
        // Por ahora, lo dejamos así, pero `main.rs` deberá llamarla.
        // O mejor, `buy_development_card` debe devolver Option<PlayerType>
        // y llamar a `check_for_winner` (lo haremos en el siguiente paso).
    }

    Some(card_drawn)
}

pub fn place_robber (
    board: &mut Board,
    player_id_type: PlayerType,
    new_tile_pos: TileId,
    player_to_rob_id: PlayerType
) {
    let current_robber_index = match board.tiles.iter().position(|t| t.has_robber) {
        Some(index) => index,
        None => {
            println!("Error crítico: ¡El ladrón no está en el tablero!");
            return;
        }
    };

    if current_robber_index == new_tile_pos {
        println!("No se puede mover: Debes mover el ladrón a una *nueva* casilla.");
        return;
    }

    let adjacent_players = get_players_adjacent_to_tile(board, new_tile_pos);
    if !adjacent_players.contains(&player_to_rob_id) {
        println!("No se puede robar: El jugador {:?} no tiene edificios en la casilla {}.", player_to_rob_id, new_tile_pos);
        return;
    }
    
    if player_id_type == player_to_rob_id {
        println!("No se puede robar: No puedes robarte a ti mismo.");
        return;
    }
    
    board.tiles[current_robber_index].has_robber = false;
    board.tiles[new_tile_pos].has_robber = true;
    println!("Ladrón movido de la casilla {} a la {}.", current_robber_index, new_tile_pos);

    let player_moving_index = board.players.iter().position(|p| p.id == player_id_type).unwrap();
    let player_robbed_index = board.players.iter().position(|p| p.id == player_to_rob_id).unwrap();

    let robbable_resources: Vec<MaterialType> = board.players[player_robbed_index]
        .resources
        .iter()
        .filter(|(_, &count)| count > 0) 
        .map(|(&material, _)| material)
        .collect();

    if robbable_resources.is_empty() {
        println!("¡El jugador {:?} no tiene cartas para robar!", player_to_rob_id);
        return;
    }

    let &resource_stolen = robbable_resources.choose(&mut rand::thread_rng()).unwrap();
    println!("¡{:?} le roba 1 de {:?} a {:?}!", player_id_type, resource_stolen, player_to_rob_id);

    {
        let player_robbed = &mut board.players[player_robbed_index];
        *player_robbed.resources.get_mut(&resource_stolen).unwrap() -= 1;
    }
    {
        let player_moving = &mut board.players[player_moving_index];
        let resource_count = player_moving.resources.entry(resource_stolen).or_insert(0);
        *resource_count += 1;
    }
}