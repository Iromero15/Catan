// en src/game_logic/victory.rs

use crate::types::*;
use std::collections::HashSet;

// Hacemos públicas las funciones que otros módulos necesitarán
pub fn check_for_winner(board: &Board) -> Option<PlayerType> {
    for player in &board.players {
        if player.victory_points >= 10 {
            println!("¡JUEGO TERMINADO! ¡El ganador es {:?}!", player.id);
            return Some(player.id);
        }
    }
    None
}

pub fn update_largest_army(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    
    let player_index = board.players.iter().position(|p| p.id == player_id).unwrap();
    let knights_played = board.players[player_index].knights_played;

    if knights_played >= 3 && knights_played > board.largest_army_size {
        
        if board.largest_army == Some(player_id) {
            board.largest_army_size = knights_played; 
            return None;
        }

        println!("¡{:?} reclama el Mayor Ejército con {} caballeros!", player_id, knights_played);

        if let Some(old_holder_id) = board.largest_army {
            let old_holder = board.players.iter_mut().find(|p| p.id == old_holder_id).unwrap();
            old_holder.victory_points -= 2;
            println!("- {:?} pierde 2 VP.", old_holder.id);
        }

        let new_holder = &mut board.players[player_index];
        new_holder.victory_points += 2;
        println!("- {:?} gana 2 VP.", new_holder.id);

        board.largest_army = Some(player_id);
        board.largest_army_size = knights_played;

        return check_for_winner(board);
    }
    
    None
}

// Esta función es una auxiliar privada, no necesita `pub`
fn find_path_length(
    board: &Board,
    player_id: PlayerType,
    current_vertex: VertexId,
    visited_edges: &mut HashSet<EdgeId>
) -> u8 {
    
    let mut max_len = 0;

    for &edge_id in &board.vertices[current_vertex].adjacent_edges {
        
        if board.edges[edge_id].owner == Some(player_id) && !visited_edges.contains(&edge_id) {
            
            visited_edges.insert(edge_id); 

            let (v1, v2) = board.edges[edge_id].vertices;
            let next_vertex = if v1 == current_vertex { v2 } else { v1 };

            let is_blocked = board.vertices[next_vertex].owner.is_some() &&
                             board.vertices[next_vertex].owner != Some(player_id);

            let current_len = if is_blocked {
                1
            } else {
                1 + find_path_length(board, player_id, next_vertex, visited_edges)
            };

            if current_len > max_len {
                max_len = current_len;
            }
            
            visited_edges.remove(&edge_id);
        }
    }
    
    max_len
}

// Esta también es una auxiliar privada
fn calculate_player_longest_road(board: &Board, player_id: PlayerType) -> u8 {
    let mut max_road = 0;
    let mut visited_edges: HashSet<EdgeId> = HashSet::new();

    for v_id in 0..board.vertices.len() {
        if board.vertices[v_id].owner == Some(player_id) || 
           board.vertices[v_id].owner.is_none() {
            
            let len = find_path_length(board, player_id, v_id, &mut visited_edges);
            if len > max_road {
                max_road = len;
            }
        }
    }
    max_road
}

pub fn update_longest_road(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    
    let current_longest = calculate_player_longest_road(board, player_id);

    if current_longest >= 5 && current_longest > board.longest_road_size {
        
        if board.longest_road == Some(player_id) {
            board.longest_road_size = current_longest;
            return None;
        }

        println!("¡{:?} reclama el Camino Más Largo con {} segmentos!", player_id, current_longest);

        if let Some(old_holder_id) = board.longest_road {
            let old_holder = board.players.iter_mut().find(|p| p.id == old_holder_id).unwrap();
            old_holder.victory_points -= 2;
            println!("- {:?} pierde 2 VP.", old_holder.id);
        }

        let new_holder = board.players.iter_mut().find(|p| p.id == player_id).unwrap();
        new_holder.victory_points += 2;
        println!("- {:?} gana 2 VP.", new_holder.id);

        board.longest_road = Some(player_id);
        board.longest_road_size = current_longest;
        
        return check_for_winner(board);
    }
    
    None
}