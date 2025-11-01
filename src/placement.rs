use crate::types::*;
use rand::prelude::*;

/**
 * Mueve el ladrón a una nueva casilla y roba 1 recurso a un
 * jugador adyacente (si es posible).
 */
pub fn place_robber (
    board: &mut Board,
    player_id_type: PlayerType, // El jugador que MUEVE el ladrón
    new_tile_pos: TileId,       // A dónde lo mueve (¡Corregido a TileId!)
    player_to_rob_id: PlayerType // A quién elige robar
) {
    
    // --- 1. Encontrar el ladrón actual ---
    let current_robber_index = match board.tiles.iter().position(|t| t.has_robber) {
        Some(index) => index,
        None => {
            println!("Error crítico: ¡El ladrón no está en el tablero! (Debería inicializarse en el desierto)");
            return;
        }
    };

    // --- 2. Chequeo de Reglas (Solo Lectura) ---

    // Regla 1: No se puede mover al mismo lugar
    if current_robber_index == new_tile_pos {
        println!("No se puede mover: Debes mover el ladrón a una *nueva* casilla.");
        return;
    }

    // Regla 2: El jugador a robar debe estar en la *nueva* casilla
    let adjacent_players = get_players_adjacent_to_tile(board, new_tile_pos);
    if !adjacent_players.contains(&player_to_rob_id) {
        println!("No se puede robar: El jugador {:?} no tiene edificios en la casilla {}.", player_to_rob_id, new_tile_pos);
        return;
    }
    
    // Regla 3: No te puedes robar a ti mismo
    if player_id_type == player_to_rob_id {
        println!("No se puede robar: No puedes robarte a ti mismo.");
        return;
    }
    
    // --- 3. Mover el Ladrón (Modificación 1) ---
    board.tiles[current_robber_index].has_robber = false;
    board.tiles[new_tile_pos].has_robber = true;
    println!("Ladrón movido de la casilla {} a la {}.", current_robber_index, new_tile_pos);

    // --- 4. Ejecutar el Robo (Modificación 2) ---
    
    // Paso A: Encontrar índices para evitar el 'borrow checker'
    let player_moving_index = board.players.iter().position(|p| p.id == player_id_type).unwrap();
    let player_robbed_index = board.players.iter().position(|p| p.id == player_to_rob_id).unwrap();

    // Paso B: Crear una lista de los recursos que el jugador tiene
    let robbable_resources: Vec<MaterialType> = board.players[player_robbed_index]
        .resources
        .iter()
        // Solo incluye materiales de los que tenga 1 o más
        .filter(|(_, &count)| count > 0) 
        .map(|(&material, _)| material) // Obtiene el tipo de material
        .collect();

    // Regla 4: Si no tiene cartas, no se roba nada.
    if robbable_resources.is_empty() {
        println!("¡El jugador {:?} no tiene cartas para robar!", player_to_rob_id);
        return; // No roba nada, pero el movimiento es válido
    }

    // Paso C: Elegir un recurso al azar de la lista
    let &resource_stolen = robbable_resources.choose(&mut rand::thread_rng()).unwrap();
    println!("¡{:?} le roba 1 de {:?} a {:?}!", player_id_type, resource_stolen, player_to_rob_id);

    // Paso D: Transferir el recurso (en dos bloques separados)
    {
        // Préstamo 1: Quitar recurso al robado
        let player_robbed = &mut board.players[player_robbed_index];
        *player_robbed.resources.get_mut(&resource_stolen).unwrap() -= 1;
    }
    {
        // Préstamo 2: Dar recurso al que roba
        let player_moving = &mut board.players[player_moving_index];
        let resource_count = player_moving.resources.entry(resource_stolen).or_insert(0);
        *resource_count += 1;
    }
}
/**
 * Devuelve un Vec con los PlayerType (únicos) de todos
 * los jugadores que tienen edificios en los vértices de una casilla.
 */
fn get_players_adjacent_to_tile(board: &Board, tile_id: TileId) -> Vec<PlayerType> {
    let mut players_on_tile = Vec::new();
    let tile = &board.tiles[tile_id];

    // Itera sobre los 6 vértices de la casilla
    for &vertex_id in &tile.vertices {
        
        // Comprueba si el vértice tiene un dueño
        if let Some(owner) = board.vertices[vertex_id].owner {
            
            // Si no hemos agregado a este jugador todavía, lo añadimos.
            if !players_on_tile.contains(&owner) {
                players_on_tile.push(owner);
            }
        }
    }
    players_on_tile
}
/**
 * Intenta colocar un camino (Road) en un borde.
 * Las reglas de conexión cambian según la fase del turno.
 */
pub fn place_road (
    board: &mut Board, 
    player_id_type: PlayerType, 
    edge_position: EdgeId,
    turn_phase: TurnPhase  // <-- ¡Parámetro modificado!
) {
    
    // --- 1. Chequeo de Inventario y Ocupación (Solo Lectura) ---

    // Paso A: Encontrar el ÍNDICE del jugador
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return;
        }
    };

    // Paso B: Chequear el inventario
    if board.players[player_index].road_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más caminos disponibles.", player_id_type);
        return;
    }

    // Paso C: Chequear si el borde está vacío
    if board.edges[edge_position].owner.is_some() {
        println!("No se puede construir: El borde {} ya está ocupado.", edge_position);
        return;
    }

    // --- 2. Chequeo de Conexión (¡LÓGICA MODIFICADA!) ---
    
    let is_connected = match turn_phase {
        
        // Caso A: Turno Normal
        // Usa la función de chequeo de red completa que ya teníamos.
        TurnPhase::Normal => {
            is_road_connectable(board, player_id_type, edge_position)
        }
        
        // Caso B: Turno de Fundación
        // Usa una nueva función que solo comprueba si el borde
        // toca el 'anchor_vertex' (la casa recién puesta).
        TurnPhase::Setup { anchor_vertex } => {
            is_road_adjacent_to_vertex(board, edge_position, anchor_vertex)
        }
    };

    if !is_connected {
        println!("No se puede construir: El camino no está conectado correctamente (Fase: {:?}).", turn_phase);
        return;
    }
    
    // --- 3. Modificar el Tablero ---
    board.edges[edge_position].owner = Some(player_id_type);

    // --- 4. Modificar al Jugador ---
    let player = &mut board.players[player_index];
    player.road_quantity -= 1;

    println!("¡Camino construido con éxito en {} para {:?}!", edge_position, player_id_type);
}

/**
 * Comprueba si un borde (`edge_id`) está tocando
 * un vértice específico (`vertex_id`).
 *
 * (Usado para la regla de fundación)
 */
fn is_road_adjacent_to_vertex(board: &Board, edge_id: EdgeId, vertex_id: VertexId) -> bool {
    let (v1, v2) = board.edges[edge_id].vertices;
    
    // El borde es adyacente si uno de sus dos extremos
    // es el vértice que estamos comprobando.
    v1 == vertex_id || v2 == vertex_id
}

/**
 * Comprueba si un *nuevo* camino en `edge_id` estaría
 * conectado a la red existente del jugador.
 *
 * Un camino es "conectable" si uno de sus dos vértices finales:
 * 1. Es propiedad del jugador (tiene un asentamiento/ciudad).
 * 2. O, ya está tocando OTRO camino propiedad del jugador.
 */
fn is_road_connectable(board: &Board, player_id: PlayerType, edge_id: EdgeId) -> bool {
    
    // Obtenemos los dos vértices que define este borde
    let (v1, v2) = board.edges[edge_id].vertices;

    // --- Chequeo 1: ¿El jugador posee uno de los vértices? ---
    // (Esto cubre el caso de construir junto a un asentamiento)
    if board.vertices[v1].owner == Some(player_id) {
        return true;
    }
    if board.vertices[v2].owner == Some(player_id) {
        return true;
    }

    // --- Chequeo 2: ¿Alguno de los vértices toca OTRO camino del jugador? ---
    
    // Comprobar todos los caminos que salen de v1
    for &other_edge_id in &board.vertices[v1].adjacent_edges {
        // (Nos saltamos el borde que estamos intentando construir)
        if other_edge_id == edge_id {
            continue;
        }
        
        if board.edges[other_edge_id].owner == Some(player_id) {
            return true; // v1 está conectado a otro camino
        }
    }

    // Comprobar todos los caminos que salen de v2
    for &other_edge_id in &board.vertices[v2].adjacent_edges {
        if other_edge_id == edge_id {
            continue;
        }
        
        if board.edges[other_edge_id].owner == Some(player_id) {
            return true; // v2 está conectado a otro camino
        }
    }

    // Si ninguna condición se cumple, no es conectable
    false
}

/**
 * Intenta reemplazar un Asentamiento existente por una Ciudad.
 */
pub fn place_city (board: &mut Board, player_id_type: PlayerType, position: VertexId) {
    
    // --- 1. Chequeo de Propiedad ---
    // ¿Tiene este jugador un ASENTAMIENTO en esta posición?
    if !is_settlement_owned_by(board, player_id_type, position) {
        println!("No se puede construir: {:?} no posee un asentamiento en {}.", player_id_type, position);
        return;
    }

    // --- 2. Chequeo de Inventario (Evitando el Borrow Checker) ---

    // Paso A: Encontrar el ÍNDICE del jugador (solo lectura)
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return;
        }
    };

    // Paso B: Chequear el inventario usando el índice (solo lectura)
    if board.players[player_index].city_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más ciudades disponibles.", player_id_type);
        return;
    }

    // --- 3. Modificar el Tablero ---
    // (Inicia el primer préstamo mutable)
    board.vertices[position].building = Some(BuildingType::City);
    // (Termina el primer préstamo mutable)

    // --- 4. Modificar al Jugador ---
    // (Inicia el segundo préstamo mutable, ¡totalmente legal!)
    let player = &mut board.players[player_index];
    
    player.city_quantity -= 1;       // Gasta una pieza de ciudad
    player.settlement_quantity += 1; // Recupera la pieza de asentamiento
    player.victory_points += 1;      // Gana 1 VP (de 1 a 2)

    println!("¡Ciudad construida con éxito en {} para {:?}!", position, player_id_type);
    println!("A {:?} le quedan {} ciudades y tiene {} puntos.", player.id, player.city_quantity, player.victory_points);
}
/**
 * Comprueba si un vértice contiene un Asentamiento (Settlement)
 * que pertenece al jugador especificado.
 */
fn is_settlement_owned_by(board: &Board, player_id: PlayerType, position: VertexId) -> bool {
    
    // Obtenemos el vértice de forma inmutable
    let vertex = &board.vertices[position];

    // Usamos 'match' para comprobar limpiamente ambas condiciones
    match (vertex.owner, vertex.building) {
        
        // El caso de éxito:
        // 1. El dueño (`owner`) es `Some(player)`
        // 2. El edificio (`building`) es `Some(BuildingType::Settlement)`
        (Some(owner), Some(BuildingType::Settlement)) => {
            // Si ambas son verdad, comprobamos si el dueño es el que buscamos
            owner == player_id
        }
        
        // Cualquier otro caso (está vacío, ya es una ciudad, o es de otro jugador)
        _ => false,
    }
}

// Renombré 'player_id' a 'player_id_type' para mayor claridad
pub fn place_house (board: &mut Board, player_id_type: PlayerType, position: VertexId, is_first_turn: bool) {
    
    // --- 1. Chequeo de Regla de Distancia ---
    if !can_place_house(board, position) {
        return;
    }

    // --- 2. Chequeo de Regla de Conexión (Turno Normal) ---
    if !is_first_turn {
        if !has_road_connected(board, player_id_type, position) {
            println!("No se puede construir en {}: No tienes un camino conectado a este vértice.", position);
            return;
        }
    }
    
    // --- 3. Chequeo de Inventario (¡MODIFICADO!) ---
    
    // *Paso A: Encontrar el ÍNDICE del jugador*
    // `position()` solo necesita un préstamo INMUTABLE (&board), por lo que no hay conflicto.
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return;
        }
    };

    // *Paso B: Chequear el inventario (todavía inmutable)*
    if board.players[player_index].settlement_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más asentamientos disponibles.", player_id_type);
        return;
    }

    // --- 4. Colocar la Casa (En el Tablero) ---
    // *Aquí comienza el PRIMER préstamo mutable.*
    // Afecta a `board.vertices`.
    board.vertices[position].owner = Some(player_id_type);
    board.vertices[position].building = Some(BuildingType::Settlement{});
    // *El préstamo mutable a `board.vertices` TERMINA aquí.*

    
    // --- 5. Actualizar al Jugador ---
    // *Aquí comienza el SEGUNDO préstamo mutable.*
    // Afecta a `board.players`.
    // Como el préstamo anterior ya terminó, ¡esto es 100% legal!
    let player = &mut board.players[player_index];
    player.settlement_quantity -= 1;
    player.victory_points += 1;

    println!("¡Casa ubicada con éxito en {} para {:?}!", position, player_id_type);
    println!("A {:?} le quedan {} asentamientos y tiene {} puntos.", player.id, player.settlement_quantity, player.victory_points);
}
fn has_road_connected(board: &Board, player_id: PlayerType, position: VertexId) -> bool {
    let pos: &Vertex = &board.vertices[position];

    for &edge_id in &pos.adjacent_edges {
        
        // Obtén el borde actual
        let edge = &board.edges[edge_id];
        if edge.owner == Some(player_id) {
            return true;
        }
    }
    println!("no hay Camino conectado");
    return false;
}

// 1. Acepta una referencia `&Board` para no "robarse" el tablero.
fn can_place_house(board: &Board, position: VertexId) -> bool {
    
    let pos: &Vertex = &board.vertices[position];

    

    // 1. Revisa si el vértice *actual* está ocupado.
    if pos.owner.is_some() { // .is_some() es más claro aquí que '!' .is_none()
        println!("No se puede construir en {}: la casilla ya está ocupada.", position);
        return false;
    }

    // 2. Si está vacío, revisa los *vecinos*.
    // println!("Vértice {} está vacío. Revisando vecinos...", position);

    // Itera sobre los IDs de los bordes adyacentes
    for &edge_id in &pos.adjacent_edges {
        
        // Obtén el borde actual
        let edge = &board.edges[edge_id];
        let (v1, v2) = edge.vertices; // p.ej., (5, 4)

        // Identifica cuál de los dos es el *vecino*
        // (El que NO es nuestra 'position' actual)
        let neighbor_v_id = if v1 == position {
            v2 // v1 es nuestra posición, así que v2 es el vecino
        } else {
            v1 // v2 debe ser nuestra posición, así que v1 es el vecino
        };

        // Ahora, usa la función helper para revisar si el *vecino* está ocupado
        // ¡Usamos '!' (not) para invertir la lógica!
        if !check_self_is_empty(board, neighbor_v_id) {
            
            // Si el vecino NO está vacío...
            println!("No se puede construir en {}: el vecino {} está ocupado.", position, neighbor_v_id);
            return false; // ... la regla falla.
        }
        // Si el vecino está vacío, el bucle continúa con el siguiente borde.
    }
    
    // Si salimos del bucle, significa que la casilla está vacía
    // Y *todos* sus vecinos también están vacíos.
    return true;
}

fn check_self_is_empty(board: &Board, position: VertexId) -> bool {
    let pos: &Vertex = &board.vertices[position];
    pos.owner.is_none()
}