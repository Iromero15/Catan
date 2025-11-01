// en src/game_logic/building.rs

use crate::types::*;
use std::collections::HashMap; // Necesario para los costos
// Importamos las funciones de economía y victoria que necesitamos
use super::economy::{has_resources, spend_resources};
use super::victory::{check_for_winner, update_longest_road};

// --- CONSTANTES DE COSTO ---
const SETTLEMENT_COST: &[(MaterialType, u8)] = &[
    (MaterialType::Brick, 1),
    (MaterialType::Wood, 1),
    (MaterialType::Sheep, 1),
    (MaterialType::Wheat, 1),
];

const ROAD_COST: &[(MaterialType, u8)] = &[
    (MaterialType::Brick, 1),
    (MaterialType::Wood, 1),
];

const CITY_COST: &[(MaterialType, u8)] = &[
    (MaterialType::Wheat, 2),
    (MaterialType::Stone, 3),
];

// --- FUNCIONES AUXILIARES (PRIVADAS) ---

fn has_road_connected(board: &Board, player_id: PlayerType, position: VertexId) -> bool {
    let pos: &Vertex = &board.vertices[position];
    for &edge_id in &pos.adjacent_edges {
        let edge = &board.edges[edge_id];
        if edge.owner == Some(player_id) {
            return true;
        }
    }
    println!("no hay Camino conectado");
    return false;
}

fn can_place_house(board: &Board, position: VertexId) -> bool {
    let pos: &Vertex = &board.vertices[position];
    if pos.owner.is_some() {
        println!("No se puede construir en {}: la casilla ya está ocupada.", position);
        return false;
    }
    for &edge_id in &pos.adjacent_edges {
        let edge = &board.edges[edge_id];
        let (v1, v2) = edge.vertices;
        let neighbor_v_id = if v1 == position { v2 } else { v1 };
        if !check_self_is_empty(board, neighbor_v_id) {
            println!("No se puede construir en {}: el vecino {} está ocupado.", position, neighbor_v_id);
            return false;
        }
    }
    return true;
}

fn check_self_is_empty(board: &Board, position: VertexId) -> bool {
    let pos: &Vertex = &board.vertices[position];
    pos.owner.is_none()
}

fn is_road_adjacent_to_vertex(board: &Board, edge_id: EdgeId, vertex_id: VertexId) -> bool {
    let (v1, v2) = board.edges[edge_id].vertices;
    v1 == vertex_id || v2 == vertex_id
}

fn is_settlement_owned_by(board: &Board, player_id: PlayerType, position: VertexId) -> bool {
    let vertex = &board.vertices[position];
    match (vertex.owner, vertex.building) {
        (Some(owner), Some(BuildingType::Settlement)) => owner == player_id,
        _ => false,
    }
}

// --- FUNCIONES PÚBLICAS ---

pub fn is_road_connectable(board: &Board, player_id: PlayerType, edge_id: EdgeId) -> bool {
    let (v1, v2) = board.edges[edge_id].vertices;
    if board.vertices[v1].owner == Some(player_id) || board.vertices[v2].owner == Some(player_id) {
        return true;
    }
    for &other_edge_id in &board.vertices[v1].adjacent_edges {
        if other_edge_id != edge_id && board.edges[other_edge_id].owner == Some(player_id) {
            return true;
        }
    }
    for &other_edge_id in &board.vertices[v2].adjacent_edges {
        if other_edge_id != edge_id && board.edges[other_edge_id].owner == Some(player_id) {
            return true;
        }
    }
    false
}

pub fn place_road (
    board: &mut Board, 
    player_id_type: PlayerType, 
    edge_position: EdgeId,
    turn_phase: TurnPhase
) -> Option<PlayerType>{
    
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    if board.players[player_index].road_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más caminos disponibles.", player_id_type);
        return None;
    }
    if board.edges[edge_position].owner.is_some() {
        println!("No se puede construir: El borde {} ya está ocupado.", edge_position);
        return None;
    }

    if let TurnPhase::Normal = turn_phase {
        if !has_resources(&board.players[player_index], ROAD_COST) {
            println!("No se puede construir: {:?} no tiene los recursos necesarios.", player_id_type);
            return None;
        }
    }

    let is_connected = match turn_phase {
        TurnPhase::Normal | TurnPhase::FreeRoad => {
            is_road_connectable(board, player_id_type, edge_position)
        }
        TurnPhase::Setup { anchor_vertex } => {
            is_road_adjacent_to_vertex(board, edge_position, anchor_vertex)
        }
    };

    if !is_connected {
        println!("No se puede construir: El camino no está conectado correctamente (Fase: {:?}).", turn_phase);
        return None;
    }
    
    board.edges[edge_position].owner = Some(player_id_type);

    let player = &mut board.players[player_index];
    player.road_quantity -= 1; // <-- Bug corregido, solo se resta una vez

    if let TurnPhase::Normal = turn_phase {
        spend_resources(player, ROAD_COST);
        println!("¡Camino construido con éxito en {}! (Recursos gastados)", edge_position);
    } else {
        println!("¡Camino construido con éxito en {}! (Sin costo)", edge_position);
    }
    
    println!("A {:?} le quedan {} caminos.", player.id, player.road_quantity);
    update_longest_road(board, player_id_type)
}

pub fn place_city (board: &mut Board, player_id_type: PlayerType, position: VertexId) -> Option<PlayerType>{
    if !is_settlement_owned_by(board, player_id_type, position) {
        println!("No se puede construir: {:?} no posee un asentamiento en {}.", player_id_type, position);
        return None;
    }

    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    if board.players[player_index].city_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más ciudades disponibles.", player_id_type);
        return None;
    }
    if !has_resources(&board.players[player_index], CITY_COST) {
        println!("No se puede construir: {:?} no tiene los recursos necesarios.", player_id_type);
        return None;
    }

    board.vertices[position].building = Some(BuildingType::City);
    let player = &mut board.players[player_index];
    player.city_quantity -= 1;
    player.settlement_quantity += 1;
    player.victory_points += 1;

    spend_resources(player, CITY_COST);

    println!("¡Ciudad construida con éxito en {} para {:?}!", position, player_id_type);
    println!("A {:?} le quedan {} ciudades y tiene {} puntos.", player.id, player.city_quantity, player.victory_points);
    check_for_winner(board)
}

pub fn place_house (board: &mut Board, player_id_type: PlayerType, position: VertexId, is_first_turn: bool) -> Option<PlayerType>{
    if !can_place_house(board, position) {
        return None;
    }
    
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };
    
    if is_first_turn {
        // Turno de fundación
    } else {
        if !has_road_connected(board, player_id_type, position) {
            println!("No se puede construir en {}: No tienes un camino conectado.", position);
            return None;
        }
        if !has_resources(&board.players[player_index], SETTLEMENT_COST) {
            println!("No se puede construir: {:?} no tiene los recursos necesarios.", player_id_type);
            return None;
        }
    }
    
    if board.players[player_index].settlement_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más asentamientos disponibles.", player_id_type);
        return None;
    }

    board.vertices[position].owner = Some(player_id_type);
    board.vertices[position].building = Some(BuildingType::Settlement{});
    
    let acquired_power_up = board.vertices[position].power_up;

    let player = &mut board.players[player_index];
    player.settlement_quantity -= 1;
    player.victory_points += 1;

    if let Some(power_up) = acquired_power_up {
        if !player.power_ups.contains(&power_up) {
            player.power_ups.push(power_up);
            println!("¡{:?} ha conseguido un nuevo puerto: {:?}!", player.id, power_up);
        }
    }

    if !is_first_turn {
        spend_resources(player, SETTLEMENT_COST);
        println!("¡Casa ubicada con éxito en {}! (Recursos gastados)", position);
    } else {
        println!("¡Casa ubicada con éxito en {}! (Turno de fundación)", position);
    }
    
    println!("A {:?} le quedan {} asentamientos y tiene {} puntos.", player.id, player.settlement_quantity, player.victory_points);
    check_for_winner(board)
}