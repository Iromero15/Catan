use crate::types::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
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
    (MaterialType::Stone, 3), // Asumo que Stone es tu "mineral"
];
// --- CONSTANTE DE COSTO ---
const DEVELOPMENT_CARD_COST: &[(MaterialType, u8)] = &[
    (MaterialType::Sheep, 1),
    (MaterialType::Wheat, 1),
    (MaterialType::Stone, 1), // Asumo que Stone es tu "mineral"
];

/**
 * Comprueba si algún jugador ha alcanzado 10 o más
 * puntos de victoria.
 *
 * Devuelve `Some(PlayerType)` si hay un ganador,
 * si no, devuelve `None`.
 */
pub fn check_for_winner(board: &Board) -> Option<PlayerType> {
    for player in &board.players {
        if player.victory_points >= 10 {
            println!("¡JUEGO TERMINADO! ¡El ganador es {:?}!", player.id);
            return Some(player.id);
        }
    }
    None
}
/**
 * (Privada) Actualiza el estado de "Mayor Ejército".
 * Se llama cada vez que un jugador juega un Caballero.
 * Devuelve un ganador si esta acción provoca una victoria.
 */
pub fn update_largest_army(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    
    let player_index = board.players.iter().position(|p| p.id == player_id).unwrap();
    let knights_played = board.players[player_index].knights_played;

    // Regla 1: Debe tener al menos 3 caballeros.
    // Regla 2: Debe tener más que el poseedor actual.
    if knights_played >= 3 && knights_played > board.largest_army_size {
        
        // Comprobar si el jugador ya lo tiene
        if board.largest_army == Some(player_id) {
            board.largest_army_size = knights_played; // Solo actualiza el tamaño
            return None; // Sin cambio de VP, sin ganador
        }

        println!("¡{:?} reclama el Mayor Ejército con {} caballeros!", player_id, knights_played);

        // Quitar 2 VP al poseedor anterior (si existe)
        if let Some(old_holder_id) = board.largest_army {
            let old_holder = board.players.iter_mut().find(|p| p.id == old_holder_id).unwrap();
            old_holder.victory_points -= 2;
            println!("- {:?} pierde 2 VP.", old_holder.id);
        }

        // Dar 2 VP al nuevo poseedor
        let new_holder = &mut board.players[player_index];
        new_holder.victory_points += 2;
        println!("- {:?} gana 2 VP.", new_holder.id);

        // Actualizar el tablero
        board.largest_army = Some(player_id);
        board.largest_army_size = knights_played;

        // Devolvemos si esto causó una victoria
        return check_for_winner(board);
    }
    
    None // No hubo cambios o no hubo ganador
}
/**
 * Otorga recursos a TODOS los jugadores basado en una tirada de dado.
 *
 * Esta función asume que la tirada NO es un 7.
 * La lógica del juego principal (fuera de esta función)
 * debe manejar el 7 (mover al ladrón) por separado.
 */
pub fn give_materials_on_roll(board: &mut Board, number_rolled: u8) {
    
    // Paso 1: "Recolectar" todos los pagos pendientes.
    // Usamos un HashMap<PlayerID, HashMap<Material, Cantidad>>
    // (Ej: { Player1: {Wood: 2, Brick: 1}, Player2: {Sheep: 1} })
    let mut payouts: HashMap<PlayerType, HashMap<MaterialType, u8>> = HashMap::new();

    // Iteramos sobre todas las casillas del tablero (inmutable)
    for tile in board.tiles.iter() {
        
        // Si la casilla coincide con la tirada Y NO tiene el ladrón
        if tile.number == number_rolled && !tile.has_robber {
            
            let material = tile.material;
            
            // Itera sobre los 6 vértices de esta casilla productora
            for &vertex_id in &tile.vertices {
                
                // Comprueba si el vértice tiene un dueño y un edificio
                if let Some(owner_id) = board.vertices[vertex_id].owner {
                    if let Some(building) = board.vertices[vertex_id].building {
                        
                        // Determina cuánto pagar (1 por asentamiento, 2 por ciudad)
                        let amount = match building {
                            BuildingType::Settlement => 1,
                            BuildingType::City => 2,
                        };

                        // Añade este pago a nuestra lista de recolección
                        let player_payout = payouts.entry(owner_id).or_insert(HashMap::new());
                        let material_count = player_payout.entry(material).or_insert(0);
                        *material_count += amount;
                    }
                }
            }
        }
    }

    // ---
    // Paso 2: "Aplicar" los pagos a los jugadores (mutable)
    // ---
    if payouts.is_empty() {
        println!("Tirada {}: Ninguna casilla produjo recursos.", number_rolled);
        return;
    }
    
    println!("Tirada {}: ¡Repartiendo recursos!", number_rolled);

    // Iteramos sobre los jugadores del tablero (mutable)
    for player in board.players.iter_mut() {
        
        // ¿Este jugador tiene pagos pendientes en nuestra lista?
        if let Some(gains) = payouts.get(&player.id) {
            
            for (&material, &amount) in gains {
                let resource_count = player.resources.entry(material).or_insert(0);
                *resource_count += amount;
                
                println!("- {:?} recibe {} de {:?}", player.id, amount, material);
            }
        }
    }
}

/**
 * Otorga los recursos iniciales a UN jugador.
 *
 * Esta función se llama para la *segunda* casa que coloca
 * un jugador durante la fundación. Le da 1 recurso por cada
 * casilla adyacente a `settlement_pos`.
 */
pub fn give_starting_resources(
    board: &mut Board, 
    player_id: PlayerType, 
    settlement_pos: VertexId
) {
    
    // Paso 1: "Recolectar" los materiales (inmutable)
    let mut resources_to_gain: Vec<MaterialType> = Vec::new();

    // Obtenemos los 'adjacent_tiles' del vértice donde se colocó la casa
    for &tile_id in &board.vertices[settlement_pos].adjacent_tiles {
        
        let tile = &board.tiles[tile_id];
        
        // No se obtienen recursos del desierto
        if tile.material != MaterialType::Dessert {
            resources_to_gain.push(tile.material);
        }
    }

    // ---
    // Paso 2: "Aplicar" los recursos al jugador (mutable)
    // ---

    // Encontramos al jugador
    let player = match board.players.iter_mut().find(|p| p.id == player_id) {
        Some(p) => p,
        None => {
            println!("Error: No se encontró al jugador {:?} para darle sus recursos.", player_id);
            return;
        }
    };
    
    println!("Dando recursos iniciales a {:?}:", player_id);

    // Añadimos 1 de cada recurso recolectado
    for material in resources_to_gain {
        let resource_count = player.resources.entry(material).or_insert(0);
        *resource_count += 1;
        println!("- 1 de {:?}", material);
    }
}

/**
 * Realiza un intercambio de recursos con el banco (la banca).
 * Devuelve `true` si el intercambio fue exitoso, `false` si no.
 */
pub fn trade_with_bank(
    board: &mut Board,
    player_id: PlayerType,
    material_to_give: MaterialType,
    material_to_get: MaterialType
) -> bool {
    
    // --- 1. Chequeos Preliminares ---
    if material_to_give == material_to_get {
        println!("Error de intercambio: No puedes intercambiar un material por sí mismo.");
        return false;
    }
    
    // El desierto no es un recurso comerciable
    if material_to_give == MaterialType::Dessert || material_to_get == MaterialType::Dessert {
        println!("Error de intercambio: No se puede comerciar con el Desierto.");
        return false;
    }

    // --- 2. Encontrar al Jugador y Determinar la Tasa ---

    // Paso A: Encontrar el ÍNDICE del jugador
    let player_index = match board.players.iter().position(|p| p.id == player_id) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id);
            return false;
        }
    };

    let player = &board.players[player_index]; // Referencia inmutable por ahora
    let mut required_to_give = 4; // Tasa por defecto 4:1

    // Paso B: Chequear si tiene un puerto 2:1 específico
    let has_specific_port = 
        (material_to_give == MaterialType::Wheat && player.power_ups.contains(&PowerUp::Wheat2)) ||
        (material_to_give == MaterialType::Brick && player.power_ups.contains(&PowerUp::Brick2)) ||
        (material_to_give == MaterialType::Stone && player.power_ups.contains(&PowerUp::Stone2)) ||
        (material_to_give == MaterialType::Sheep && player.power_ups.contains(&PowerUp::Sheep2)) ||
        (material_to_give == MaterialType::Wood  && player.power_ups.contains(&PowerUp::Wood2));
    
    if has_specific_port {
        required_to_give = 2;
    // Paso C: Si no, chequear si tiene un puerto genérico 3:1
    } else if player.power_ups.contains(&PowerUp::Any3) {
        required_to_give = 3;
    }
    // Si no, `required_to_give` se mantiene en 4.

    // --- 3. Chequear si el Jugador Tiene Recursos Suficientes ---
    
    let current_resource_count = player.resources.get(&material_to_give).unwrap_or(&0);
    
    if *current_resource_count < required_to_give {
        println!(
            "Error de intercambio: {:?} necesita {} de {:?} pero solo tiene {}.",
            player_id, required_to_give, material_to_give, *current_resource_count
        );
        return false;
    }

    // --- 4. Ejecutar el Intercambio ---
    
    // ¡Todos los chequeos pasaron! Ahora obtenemos al jugador mutable.
    let player = &mut board.players[player_index];

    // 1. Quitar el material pagado
    let give_count = player.resources.get_mut(&material_to_give).unwrap();
    *give_count -= required_to_give;

    // 2. Añadir el material recibido
    let get_count = player.resources.entry(material_to_get).or_insert(0);
    *get_count += 1;

    println!(
        "¡Intercambio exitoso! {:?} entregó {} de {:?} y recibió 1 de {:?}.",
        player_id, required_to_give, material_to_give, material_to_get
    );
    true
}
/**
 * Intenta comprar una carta de desarrollo para un jugador.
 * Devuelve `Some(DevelopmentCard)` si la compra fue exitosa.
 * Devuelve `None` si el jugador no tenía recursos o si el mazo está vacío.
 */
pub fn buy_development_card(
    board: &mut Board, 
    player_id_type: PlayerType
) -> Option<DevelopmentCard> {
    
    // --- 1. Chequeos (Solo Lectura) ---

    // Paso A: Encontrar el ÍNDICE del jugador
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    // Paso B: Chequear si el mazo tiene cartas
    if board.development_cards.is_empty() {
        println!("No se puede comprar: ¡El mazo de cartas de desarrollo está vacío!");
        return None;
    }

    // Paso C: Chequear si el jugador tiene los recursos
    if !has_resources(&board.players[player_index], DEVELOPMENT_CARD_COST) {
        println!("No se puede comprar: {:?} no tiene los recursos necesarios (1 oveja, 1 trigo, 1 mineral/piedra).", player_id_type);
        return None;
    }

    // --- 2. Ejecutar la Compra (Modificaciones) ---
    // (Lo hacemos en dos pasos para evitar el 'borrow checker')

    // Paso A: Sacar la carta del mazo (Préstamo mutable 1)
    // Usamos .pop() para tomar la carta de arriba (la última del Vec)
    // Sabemos que no está vacío por el chequeo de arriba, así que .unwrap() es seguro.
    let card_drawn = board.development_cards.pop().unwrap();
    println!("¡{:?} ha comprado una carta de desarrollo!", player_id_type);

    // Paso B: Modificar al jugador (Préstamo mutable 2)
    let player = &mut board.players[player_index];
    
    // 1. Gastar recursos
    spend_resources(player, DEVELOPMENT_CARD_COST);
    
    // 2. Añadir la carta a la mano del jugador
    player.dev_cards.push(card_drawn);

    // (Opcional: si es un Punto de Victoria, se podría
    // registrar de inmediato, aunque las reglas dicen
    // que se mantienen en secreto)
    if card_drawn == DevelopmentCard::VictoryPoint {
        println!("¡La carta era un Punto de Victoria!");
        player.victory_points += 1; // (Decide si quieres que sea automático)
        check_for_winner(board);
    }

    // Devolvemos la carta comprada para que el juego sepa qué pasó
    Some(card_drawn)
}
/**
 * Comprueba si un jugador tiene suficientes recursos,
 * sin modificar nada.
 */
fn has_resources(player: &Player, cost: &[(MaterialType, u8)]) -> bool {
    for &(material, required_count) in cost {
        // Usa .get() y .unwrap_or(&0) para manejar el caso
        // en que el jugador tenga 0 de ese recurso (sin clave).
        let current_count = player.resources.get(&material).unwrap_or(&0);
        if *current_count < required_count {
            return false; // No tiene suficiente
        }
    }
    true // Tiene todo lo necesario
}

/**
 * Gasta los recursos del jugador.
 * IMPORTANTE: ¡Llama a `has_resources` *primero* para evitar un pánico!
 */
fn spend_resources(player: &mut Player, cost: &[(MaterialType, u8)]) {
    for &(material, required_count) in cost {
        // Aquí sí usamos .get_mut() y .unwrap() porque asumimos
        // que `has_resources` ya confirmó que la cantidad es suficiente.
        let current_count = player.resources.get_mut(&material)
            .expect("Error: Se intentó gastar un recurso que no existía o no era suficiente.");
        *current_count -= required_count;
    }
}
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
    turn_phase: TurnPhase
) -> Option<PlayerType>{
    
    // --- 1. Chequeo de Inventario, Ocupación y Recursos ---

    // Paso A: Encontrar el ÍNDICE del jugador
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    // Paso B: Chequear el inventario
    if board.players[player_index].road_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más caminos disponibles.", player_id_type);
        return None;
    }

    // Paso C: Chequear si el borde está vacío
    if board.edges[edge_position].owner.is_some() {
        println!("No se puede construir: El borde {} ya está ocupado.", edge_position);
        return None;
    }

    // Paso D: Chequear Recursos (¡NUEVO!)
    if let TurnPhase::Normal = turn_phase { // <-- SÓLO chequear en Turno Normal
        if !has_resources(&board.players[player_index], ROAD_COST) {
            println!("No se puede construir: {:?} no tiene los recursos necesarios.", player_id_type);
            return None;
        }
    }

    // --- 2. Chequeo de Conexión ---
    let is_connected = match turn_phase {
        
        TurnPhase::Normal | TurnPhase::FreeRoad => { // <-- MODIFICADO
            // En turno normal o con carta, usar la conexión de red
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
    
    // --- 3. Modificar el Tablero ---
    board.edges[edge_position].owner = Some(player_id_type);

    // --- 4. Modificar al Jugador ---
    // ¡Gastar recursos si es turno normal! (¡NUEVO!)
   let player = &mut board.players[player_index];
    player.road_quantity -= 1;

    // ¡Gastar recursos si es turno normal!
    if let TurnPhase::Normal = turn_phase { // <-- MODIFICADO
        spend_resources(player, ROAD_COST);
        println!("¡Camino construido con éxito en {}! (Recursos gastados)", edge_position);
    } else {
        // Esto ahora cubre Setup Y FreeRoad
        println!("¡Camino construido con éxito en {}! (Sin costo)", edge_position);
    }
    
    println!("A {:?} le quedan {} caminos.", player.id, player.road_quantity);
    update_longest_road(board, player_id_type)
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
pub fn is_road_connectable(board: &Board, player_id: PlayerType, edge_id: EdgeId) -> bool {
    
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
pub fn place_city (board: &mut Board, player_id_type: PlayerType, position: VertexId) -> Option<PlayerType>{
    
    // --- 1. Chequeo de Propiedad ---
    if !is_settlement_owned_by(board, player_id_type, position) {
        println!("No se puede construir: {:?} no posee un asentamiento en {}.", player_id_type, position);
        return None;
    }

    // --- 2. Chequeo de Inventario y Recursos ---

    // Paso A: Encontrar el ÍNDICE del jugador
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };

    // Paso B: Chequear el inventario de piezas
    if board.players[player_index].city_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más ciudades disponibles.", player_id_type);
        return None;
    }

    // Paso C: Chequear Recursos (¡NUEVO!)
    if !has_resources(&board.players[player_index], CITY_COST) {
        println!("No se puede construir: {:?} no tiene los recursos necesarios (3 mineral/piedra, 2 trigo).", player_id_type);
        return None;
    }

    // --- 3. Modificar el Tablero ---
    board.vertices[position].building = Some(BuildingType::City);

    // --- 4. Modificar al Jugador ---
    let player = &mut board.players[player_index];
    
    player.city_quantity -= 1;       // Gasta una pieza de ciudad
    player.settlement_quantity += 1; // Recupera la pieza de asentamiento
    player.victory_points += 1;      // Gana 1 VP (de 1 a 2)

    // ¡Gastar recursos! (¡NUEVO!)
    spend_resources(player, CITY_COST);

    println!("¡Ciudad construida con éxito en {} para {:?}!", position, player_id_type);
    println!("A {:?} le quedan {} ciudades y tiene {} puntos.", player.id, player.city_quantity, player.victory_points);
    check_for_winner(board)
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
pub fn place_house (board: &mut Board, player_id_type: PlayerType, position: VertexId, is_first_turn: bool) -> Option<PlayerType>{
    
    // --- 1. Chequeo de Regla de Distancia ---
    if !can_place_house(board, position) {
        return None;
    }
    
    // --- 2. Encontrar el ÍNDICE del jugador ---
    // (Movido hacia arriba, es necesario para todos los chequeos)
    let player_index = match board.players.iter().position(|p| p.id == player_id_type) {
        Some(index) => index,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id_type);
            return None;
        }
    };
    
    // --- 3. Chequeos Específicos del Turno ---
    if is_first_turn {
        // Es el primer turno. No se requieren recursos ni camino.
    } else {
        // Es un turno normal.
        
        // Chequeo A: ¿Tiene camino conectado?
        if !has_road_connected(board, player_id_type, position) {
            println!("No se puede construir en {}: No tienes un camino conectado a este vértice.", position);
            return None;
        }
        
        // Chequeo B: ¿Tiene los recursos? (¡NUEVO!)
        if !has_resources(&board.players[player_index], SETTLEMENT_COST) {
            println!("No se puede construir: {:?} no tiene los recursos necesarios (1 ladrillo, 1 madera, 1 oveja, 1 trigo).", player_id_type);
            return None;
        }
    }
    
    // --- 4. Chequeo de Inventario ---
    if board.players[player_index].settlement_quantity == 0 {
        println!("No se puede construir: {:?} no tiene más asentamientos disponibles.", player_id_type);
        return None;
    }

    // --- 5. Colocar la Casa (En el Tablero) ---
    board.vertices[position].owner = Some(player_id_type);
    board.vertices[position].building = Some(BuildingType::Settlement{});
    
    let acquired_power_up = board.vertices[position].power_up;

    // --- 6. Actualizar al Jugador ---
    let player = &mut board.players[player_index];
    player.settlement_quantity -= 1;
    player.victory_points += 1;

    if let Some(power_up) = acquired_power_up {
        // Evita añadir el mismo puerto varias veces
        if !player.power_ups.contains(&power_up) {
            player.power_ups.push(power_up);
            println!("¡{:?} ha conseguido un nuevo puerto: {:?}!", player.id, power_up);
        }
    }

    // ¡Gastar recursos si no es el primer turno! (¡NUEVO!)
    if !is_first_turn {
        spend_resources(player, SETTLEMENT_COST);
        println!("¡Casa ubicada con éxito en {} para {:?}! (Recursos gastados)", position, player_id_type);
    } else {
        println!("¡Casa ubicada con éxito en {} para {:?}! (Turno de fundación)", position, player_id_type);
    }
    
    println!("A {:?} le quedan {} asentamientos y tiene {} puntos.", player.id, player.settlement_quantity, player.victory_points);
    check_for_winner(board)
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
/**
 * (Privada) Recorre un camino para encontrar la longitud máxima
 * desde un vértice inicial.
 */
fn find_path_length(
    board: &Board,
    player_id: PlayerType,
    current_vertex: VertexId,
    visited_edges: &mut HashSet<EdgeId>
) -> u8 {
    
    let mut max_len = 0;

    // Itera por todos los bordes que tocan el vértice actual
    for &edge_id in &board.vertices[current_vertex].adjacent_edges {
        
        // Si el borde es del jugador Y no lo hemos visitado
        if board.edges[edge_id].owner == Some(player_id) && !visited_edges.contains(&edge_id) {
            
            visited_edges.insert(edge_id); // Marcar como visitado

            // Obtener el *otro* vértice del borde
            let (v1, v2) = board.edges[edge_id].vertices;
            let next_vertex = if v1 == current_vertex { v2 } else { v1 };

            // ¡Regla clave! ¿El camino está bloqueado por otro jugador?
            let is_blocked = board.vertices[next_vertex].owner.is_some() &&
                             board.vertices[next_vertex].owner != Some(player_id);

            let current_len = if is_blocked {
                1 // El camino termina aquí, longitud 1
            } else {
                // Continuar recursivamente
                1 + find_path_length(board, player_id, next_vertex, visited_edges)
            };

            if current_len > max_len {
                max_len = current_len;
            }
            
            visited_edges.remove(&edge_id); // Desmarcar (backtracking)
        }
    }
    
    max_len
}

/**
 * (Privada) Calcula la longitud real del camino más largo
 * de un jugador.
 */
fn calculate_player_longest_road(board: &Board, player_id: PlayerType) -> u8 {
    let mut max_road = 0;
    let mut visited_edges: HashSet<EdgeId> = HashSet::new();

    // Iteramos por todos los vértices del tablero
    for v_id in 0..board.vertices.len() {
        // Solo iniciamos la búsqueda desde un vértice que
        // pertenezca al jugador o esté vacío
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

/**
 * (Pública) Actualiza el estado de "Camino Más Largo".
 * Se llama cada vez que un jugador construye un camino.
 * Devuelve un ganador si esta acción provoca una victoria.
 */
pub fn update_longest_road(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    
    let current_longest = calculate_player_longest_road(board, player_id);

    // Regla 1: Debe ser al menos 5.
    // Regla 2: Debe ser más largo que el poseedor actual.
    if current_longest >= 5 && current_longest > board.longest_road_size {
        
        // Comprobar si el jugador ya lo tiene
        if board.longest_road == Some(player_id) {
            board.longest_road_size = current_longest; // Solo actualiza
            return None; // Sin cambio de VP
        }

        println!("¡{:?} reclama el Camino Más Largo con {} segmentos!", player_id, current_longest);

        // Quitar 2 VP al poseedor anterior
        if let Some(old_holder_id) = board.longest_road {
            let old_holder = board.players.iter_mut().find(|p| p.id == old_holder_id).unwrap();
            old_holder.victory_points -= 2;
            println!("- {:?} pierde 2 VP.", old_holder.id);
        }

        // Dar 2 VP al nuevo poseedor
        let new_holder = board.players.iter_mut().find(|p| p.id == player_id).unwrap();
        new_holder.victory_points += 2;
        println!("- {:?} gana 2 VP.", new_holder.id);

        // Actualizar el tablero
        board.longest_road = Some(player_id);
        board.longest_road_size = current_longest;
        
        return check_for_winner(board);
    }
    
    None // No hubo cambios o no hubo ganador
}