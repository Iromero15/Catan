use std::collections::HashSet;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::types::*;



/// Construye un tablero de Catan estándar, 100% conectado.
pub fn setup_board() -> Board {
    let mut vertices = Vec::new();
    let mut tiles = Vec::new();
    let mut edges = Vec::new();
    let players = Vec::new();

    // --- Paso 1: Instanciar todos los vértices ---
    // Creamos 54 vértices vacíos, basados en tu mapa (1-54).
    for _ in 0..54 {
        vertices.push(Vertex {
            owner: None,
            building: None,
            adjacent_tiles: Vec::new(), // Se llenará después
            adjacent_edges: Vec::new(), // Se llenará después
            power_up: None,
        });
    }

    // --- Paso 1.5: Asignar Puertos (Power_ups) ---
    // Define qué vértices tienen qué puertos.
    // (Estos IDs de vértice son solo un ejemplo, 
    // debes usar los que correspondan a tu mapa)
    let port_map: &[(VertexId, PowerUp)] = &[
        (0,  PowerUp::Wheat2),
        (1,  PowerUp::Wheat2),
        (4,  PowerUp::Stone2),
        (5,  PowerUp::Stone2),
        (7, PowerUp::Any3),
        (8, PowerUp::Any3),
        (10, PowerUp::Sheep2),
        (11, PowerUp::Sheep2),
        (14, PowerUp::Any3),
        (15, PowerUp::Any3),
        (17, PowerUp::Any3),
        (18, PowerUp::Any3),
        (20, PowerUp::Brick2),
        (21, PowerUp::Brick2),
        (24, PowerUp::Wood2),
        (25, PowerUp::Wood2),
        (27, PowerUp::Any3),
        (28, PowerUp::Any3),
    ];

    // Itera sobre el mapa y asigna cada puerto
    for &(v_id, port_type) in port_map {
        // Usa la sintaxis corregida con `Some`
        vertices[v_id].power_up = Some(port_type);
    }

    // --- Paso 2: Instanciar las 19 casillas (Tiles) ---
    // Creamos 19 casillas "dummy" que llenaremos ahora.
    for _ in 0..19 {
        tiles.push(Tile {
            material: MaterialType::Dessert, // Temporal
            number: 0,                       // Temporal
            vertices: [0; 6],                // Temporal
            has_robber: false,
        });
    }

    // --- Paso 3: Mapear Casillas (Tiles) a Vértices ---
    // Esta es la transcripción manual de tu imagen.
    // Usamos índices 0-based (tu "1" es 0, tu "48" es 47).
    // (Índice de Casilla, [IDs de Vértices en orden])
    let tile_to_vertices_map: &[(TileId, [VertexId; 6])] = &[
        // Casillas 0-11 (Tu 1-12)
        (0,  [0,  1,  31, 30, 47, 29]), // Tile 1
        (1,  [1,  2,  3,  4,  32, 31]), // Tile 2
        (2,  [4,  5,  6,  34, 33, 32]), // Tile 3
        (3,  [6,  7,  8,  9,  35, 34]), // Tile 4
        (4,  [35, 9,  10, 11, 37, 36]), // Tile 5
        (5,  [37, 11, 12, 13, 14, 38]), // Tile 6
        (6,  [39, 38, 14, 17, 16, 40]), // Tile 7
        (7,  [41, 40, 16, 17, 18, 19]), // Tile 8
        (8,  [43, 42, 41, 19, 20, 21]), // Tile 9
        (9,  [24, 44, 43, 21, 22, 23]), // Tile 10
        (10, [26, 46, 45, 44, 24, 25]), // Tile 11
        (11, [28, 29, 47, 46, 26, 27]), // Tile 12 <-- Este estaba MUY mal

        // Casillas 12-17 (Tu 13-18)
        (12, [47, 30, 48, 53, 45, 46]), // Tile 13
        (13, [31, 32, 33, 49, 48, 30]), // Tile 14
        (14, [33, 34, 35, 36, 50, 49]), // Tile 15
        (15, [50, 36, 37, 38, 39, 51]), // Tile 16
        (16, [52, 51, 39, 40, 41, 42]), // Tile 17 <-- Este también
        (17, [45, 53, 52, 42, 43, 44]), // Tile 18

        // Casilla 18 (Tu 19, central)
        (18, [48, 49, 50, 51, 52, 53]), // Tile 19
    ];

    // Ahora, poblamos las casillas con sus vértices
    // Y, al mismo tiempo, poblamos los vértices con sus casillas (enlace bidireccional)
    for (tile_id, vertex_ids) in tile_to_vertices_map.iter() {
        // 1. Asigna los vértices a la casilla
        tiles[*tile_id].vertices = *vertex_ids;

        // 2. Asigna esta casilla a cada uno de sus vértices
        for &v_id in vertex_ids {
            vertices[v_id].adjacent_tiles.push(*tile_id);
        }
    }

    // --- Paso 4: Descubrir y crear todos los Bordes (Edges) ---
    // No definimos los 72 bordes manualmente. Los *descubrimos*
    // usando las casillas que ya definimos.
    // Usamos un HashSet para evitar crear bordes duplicados.
    let mut edge_set: HashSet<(VertexId, VertexId)> = HashSet::new();

    for tile in tiles.iter() {
        for i in 0..6 {
            let v1_id = tile.vertices[i];
            let v2_id = tile.vertices[(i + 1) % 6]; // El siguiente vértice, volviendo al inicio

            // Creamos una tupla "canónica" (el ID más bajo primero)
            // para que (1, 2) y (2, 1) se traten como el mismo borde.
            let edge = if v1_id < v2_id {
                (v1_id, v2_id)
            } else {
                (v2_id, v1_id)
            };
            
            edge_set.insert(edge);
        }
    }

    // Ahora `edge_set` contiene los 72 bordes únicos.
    // Los convertimos en `struct Edge` y conectamos todo.
    for (v1_id, v2_id) in edge_set.into_iter() {
        let edge_id = edges.len(); // El ID del nuevo borde es su índice
        
        // 1. Creamos el borde
        edges.push(Edge {
            owner: None,
            vertices: (v1_id, v2_id),
        });

        // 2. Asignamos este borde a sus dos vértices
        vertices[v1_id].adjacent_edges.push(edge_id);
        vertices[v2_id].adjacent_edges.push(edge_id);
    }

    // --- Paso 5: Asignar Materiales y Números (Layout Estándar) ---
    // Usamos el orden estándar, que corresponde a nuestros TileId 0-18
    
    // `Copy` y `Clone` en el enum nos permite hacer esto:
    let mut tile_materials = [
        MaterialType::Stone, MaterialType::Sheep, MaterialType::Wood,  // 0-2
        MaterialType::Brick, MaterialType::Wheat, MaterialType::Sheep, // 3-5
        MaterialType::Brick, MaterialType::Wheat, MaterialType::Wood,  // 6-8
        MaterialType::Stone, MaterialType::Wood,  MaterialType::Stone, // 9-11
        MaterialType::Wheat, MaterialType::Sheep, MaterialType::Brick, // 12-14
        MaterialType::Wheat, MaterialType::Sheep, MaterialType::Wood,  // 15-17
        MaterialType::Dessert,                                         // 18
    ];
    
    let tile_numbers = [
        5, 2, 6, 10, 9, 4, 3, 8, 11, 5, 8, 4, 11, 12, 9, 6, 3, 10,
    ]; // Total: 18 fichas de número
    
    let mut development_cards = [
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Knight, DevelopmentCard::Knight,
        DevelopmentCard::Monopoly, DevelopmentCard::Monopoly,
        DevelopmentCard::RoadBuilding, DevelopmentCard::RoadBuilding,
        DevelopmentCard::YearOfPlenty, DevelopmentCard::YearOfPlenty,
        DevelopmentCard::VictoryPoint, DevelopmentCard::VictoryPoint, 
        DevelopmentCard::VictoryPoint, DevelopmentCard::VictoryPoint, 
        DevelopmentCard::VictoryPoint, DevelopmentCard::VictoryPoint, 
        DevelopmentCard::VictoryPoint,
    ];

    let mut rng = rand::thread_rng();
    tile_materials.shuffle(&mut rng);
    development_cards.shuffle(&mut rng);

    let mut number_index = 0;

    for i in 0..19 {
        let material = tile_materials[i];
        tiles[i].material = material;

        if material == MaterialType::Dessert {
            tiles[i].number = 0;
            tiles[i].has_robber = true;
        } else {
            // Si no es desierto, le toca el siguiente número de la lista
            tiles[i].number = tile_numbers[number_index];
            number_index += 1; // Avanzamos al siguiente número
        }
    }

    // --- ¡Listo! ---
    // Devolvemos el tablero completamente instanciado y conectado.
    Board { 
    vertices, 
    tiles, 
    edges, 
    players, 
    development_cards: development_cards.to_vec(),
    largest_army: None,
    largest_army_size: 2, // Se necesita > 2 (o sea, 3) para reclamarlo
    longest_road: None,
    longest_road_size: 4, // Se necesita > 4 (o sea, 5) para reclamarlo
    }
}

pub fn add_player(board: &mut Board) -> Option<PlayerType> {
    
    let current_player_count = board.players.len();

    // --- 1. Chequeo de límite de jugadores ---
    if current_player_count >= 4 {
        println!("Error: No se pueden agregar más jugadores. El juego está lleno.");
        return None;
    }

    // --- 2. Determina el ID del próximo jugador ---
    let next_player_id = match current_player_count {
        0 => PlayerType::Player1,
        1 => PlayerType::Player2,
        2 => PlayerType::Player3,
        3 => PlayerType::Player4,
        // Esto no debería pasar gracias al chequeo de arriba,
        // pero `unreachable!` es una buena práctica.
        _ => unreachable!("La lógica de conteo de jugadores falló."),
    };

    // --- 3. Crea el nuevo jugador usando el constructor ---
    let new_player = Player::new(next_player_id);
    println!("Jugador {:?} agregado al juego.", new_player.id);

    // --- 4. Agrega el jugador al tablero ---
    board.players.push(new_player);

    // --- 5. Devuelve el ID del jugador agregado ---
    return Some(next_player_id);
}