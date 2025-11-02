// src/terminal_game.rs

use crate::types::*;
use crate::game_logic::*;
use crate::development_cards::*;
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};
use rand::rng; // <-- en tu warning decía "Renamed to `rng`"
use rand::Rng; // lo seguimos usando para random_range

// -----------------------------------------------------------------------------
// INICIO DEL JUEGO
// -----------------------------------------------------------------------------

pub fn start_game(board: &mut Board) {
    println!("¡Bienvenido a Catan en Consola!");
    let player_ids = setup_players(board);
    if player_ids.is_empty() {
        return;
    }

    // mostramos tablero inicial
    print_visual_board(board);

    // fase de fundación (ida y vuelta)
    run_setup_phase(board, &player_ids);

    // fase normal
    run_main_loop(board, &player_ids);
}

// -----------------------------------------------------------------------------
// SETUP DE JUGADORES
// -----------------------------------------------------------------------------

fn setup_players(board: &mut Board) -> Vec<PlayerType> {
    let mut player_ids = Vec::new();
    loop {
        let num_str = read_line_prompt("¿Cuántos jugadores (2-4)?");
        match num_str.parse::<u8>() {
            Ok(num) if (2..=4).contains(&num) => {
                for i in 0..num {
                    let next_id = match i {
                        0 => PlayerType::Player1,
                        1 => PlayerType::Player2,
                        2 => PlayerType::Player3,
                        _ => PlayerType::Player4,
                    };
                    board.players.push(Player::new(next_id));
                    player_ids.push(next_id);
                    println!("Jugador {:?} añadido.", next_id);
                }
                break;
            }
            _ => println!("Número no válido. Introduce un número entre 2 y 4."),
        }
    }
    player_ids
}

// -----------------------------------------------------------------------------
// FASE DE FUNDACIÓN
// -----------------------------------------------------------------------------

fn run_setup_phase(board: &mut Board, player_ids: &[PlayerType]) {
    // turno 1 (orden normal)
    println!("\n--- FASE DE FUNDACIÓN (TURNO 1) ---");
    let mut _first_houses: HashMap<PlayerType, usize> = HashMap::new();

    for &player_id in player_ids {
        let house_pos = run_single_setup_turn(board, player_id);
        // en el primer turno NO se dan recursos
        _first_houses.insert(player_id, house_pos);
    }

    // turno 2 (orden inverso)
    println!("\n--- FASE DE FUNDACIÓN (TURNO 2 - Inverso) ---");
    for &player_id in player_ids.iter().rev() {
        let house_pos = run_single_setup_turn(board, player_id);
        // en el segundo turno SÍ se dan recursos
        give_starting_resources(board, player_id, house_pos);
    }
}

/// Ejecuta el turno de fundación de **un** jugador:
/// 1. muestra estado
/// 2. pide un vértice válido
/// 3. pide un camino adyacente
/// Devuelve la posición del asentamiento
fn run_single_setup_turn(board: &mut Board, player_id: PlayerType) -> usize {
    print_player_status(board, player_id);
    println!("Coloca tu asentamiento y camino.");

    // 1) asentamiento
    let house_pos = loop {
        print_visual_board(board);
        let pos = read_u8("Vértice (##) para el asentamiento:");
        match place_house(board, player_id, pos as usize, true) {
            Ok(_) => break pos as usize,
            Err(msg) => {
                println!("{}", msg);
                continue;
            }
        }
    };

    // 2) camino
    loop {
        print_visual_board(board);
        let phase = TurnPhase::Setup { anchor_vertex: house_pos };

        // mostramos dónde puede construir
        print_buildable_roads(board, player_id, phase);

        let edge_pos = read_u8("Borde (##) para el camino (adyacente):");
        match place_road(board, player_id, edge_pos as usize, phase) {
            Ok(_) => break,
            Err(msg) => {
                println!("{}", msg);
                continue;
            }
        }
    }

    house_pos
}

// -----------------------------------------------------------------------------
// BUCLE PRINCIPAL
// -----------------------------------------------------------------------------

fn run_main_loop(board: &mut Board, player_ids: &[PlayerType]) {
    'game_loop: loop {
        for &player_id in player_ids {
            // al inicio del turno del jugador
            if let Some(p) = board.players.iter_mut().find(|p| p.id == player_id) {
                p.played_dev_card_this_turn = false;
            }

            print_global_status(board);
            print_player_status(board, player_id);

            read_line_prompt("Presiona Enter para tirar los dados...");
            let roll = roll_dice();
            println!("¡Has sacado un {}!", roll);

            if roll == 7 {
                handle_seven_roll(board, player_id);
            } else {
                give_materials_on_roll(board, roll);
            }

            // ahora el jugador puede hacer acciones
            loop {
                print_player_status(board, player_id);
                println!("Acciones: (c)onstruir, (t)erminar, (i)ntercambiar, (j)ugar carta, (v)er tablero");

                let input = read_line_prompt(">");
                let command = Command::parse(&input);

                match command {
                    Some(Command::Build) => {
                        if let Some(winner) = handle_build_cmd(board, player_id) {
                            print_global_status(board);
                            println!("¡Ganó {:?}!", winner);
                            break 'game_loop;
                        }
                    }
                    Some(Command::EndTurn) => {
                        // pasa al siguiente jugador
                        break;
                    }
                    Some(Command::Trade) => {
                        handle_trade_cmd(board, player_id);
                    }
                    Some(Command::PlayCard) => {
                        if let Some(winner) = handle_play_cmd(board, player_id) {
                            print_global_status(board);
                            println!("¡Ganó {:?}!", winner);
                            break 'game_loop;
                        }
                    }
                    Some(Command::ShowBoard) => {
                        print_visual_board(board);
                    }
                    None => {
                        println!("Comando no reconocido.");
                    }
                }
            }
        }
    }

    println!("¡Fin del juego!");
}

// -----------------------------------------------------------------------------
// COMANDOS
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Build,
    EndTurn,
    Trade,
    PlayCard,
    ShowBoard,
}

impl Command {
    fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "c" | "C" => Some(Command::Build),
            "t" | "T" => Some(Command::EndTurn),
            "i" | "I" => Some(Command::Trade),
            "j" | "J" => Some(Command::PlayCard),
            "v" | "V" => Some(Command::ShowBoard),
            _ => None,
        }
    }
}

// -----------------------------------------------------------------------------
// HANDLERS (los tuyos, acomodados al loop nuevo)
// -----------------------------------------------------------------------------

fn handle_build_cmd(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    print_visual_board(board);
    println!("¿Qué construir? [c]asa, [i]udad, [r]uta, [d]esarrollo, [v]olver");
    let cmd = read_line_prompt("Construir>");

    match cmd.trim() {
        "c" => {
            let pos = read_u8("Vértice (##) para la casa:");
            match place_house(board, player_id, pos as usize, false) {
                Ok(winner) => return winner,
                Err(msg) => println!("{}", msg),
            }
        }
        "i" => {
            let pos = read_u8("Vértice (##) para la ciudad:");
            match place_city(board, player_id, pos as usize) {
                Ok(winner) => return winner,
                Err(msg) => println!("{}", msg),
            }
        }
        "r" => {
            let phase = TurnPhase::Normal;
            print_buildable_roads(board, player_id, phase);

            let pos = read_u8("Borde (##) para la ruta:");
            match place_road(board, player_id, pos as usize, phase) {
                Ok(winner) => return winner,
                Err(msg) => println!("{}", msg),
            }
        }
        "d" => {
            match buy_development_card(board, player_id) {
                Ok(winner) => return winner,
                Err(msg) => println!("{}", msg),
            }
        }
        _ => {} // volver o comando inválido
    }

    None
}

fn handle_trade_cmd(board: &mut Board, player_id: PlayerType) {
    println!("¿Comerciar con quién? [b]anco, [j]ugador");
    let cmd = read_line_prompt("Comercio>");

    if cmd.trim() == "b" {
        println!("Comercio con el Banco.");
        let give = read_material_type("Material a entregar:");
        let get = read_material_type("Material a recibir:");

        if let (Some(mat_give), Some(mat_get)) = (give, get) {
            trade_with_bank(board, player_id, mat_give, mat_get);
        } else {
            println!("Material(es) no válidos. Cancelando.");
        }
    } else {
        println!("Comercio con jugadores (TODO: no implementado).");
    }
}

fn handle_play_cmd(board: &mut Board, player_id: PlayerType) -> Option<PlayerType> {
    println!("¿Qué carta jugar? [c]aballero, [r]utas, [a]bundancia, [m]onopolio, [v]olver");
    let cmd = read_line_prompt("Jugar>");

    match cmd.trim() {
        "c" => {
            print_visual_board(board);
            println!("Mover al ladrón.");
            let tile_pos = read_u8("Casilla (##) a mover:");
            let target_player = read_player_to_rob(board, tile_pos as usize, player_id);

            if let Some(target) = target_player {
                play_knight_card(board, player_id, tile_pos as usize, target)
            } else {
                println!("Robo cancelado.");
                None
            }
        }
        "r" => {
            print_visual_board(board);
            println!("Jugar 'Construcción de Rutas'.");
            let edge1 = read_u8("Posición de la primera ruta:");
            let edge2 = read_u8("Posición de la segunda ruta:");
            match play_road_building_card(board, player_id, edge1 as usize, edge2 as usize) {
                Ok(winner) => return winner,
                Err(msg) => println!("{}", msg),
            }
            None
        }
        "a" => {
            println!("Jugar 'Año de la Abundancia'.");
            let mat1 = read_material_type("Primer recurso a tomar:");
            let mat2 = read_material_type("Segundo recurso a tomar:");
            if let (Some(m1), Some(m2)) = (mat1, mat2) {
                play_year_of_plenty_card(board, player_id, m1, m2);
            } else {
                println!("Material(es) no válidos. Cancelando.");
            }
            None
        }
        "m" => {
            println!("Jugar 'Monopolio'.");
            let mat = read_material_type("Recurso a monopolizar:");
            if let Some(m) = mat {
                play_monopoly_card(board, player_id, m);
            } else {
                println!("Material no válido. Cancelando.");
            }
            None
        }
        _ => None,
    }
}

fn handle_seven_roll(board: &mut Board, player_id: PlayerType) {
    println!("¡TODOS CON MÁS DE 7 CARTAS DEBEN DESCARTAR LA MITAD!");
    // TODO: Implementar la lógica de descarte forzado.

    print_visual_board(board);
    println!("\n{:?}, debes mover al ladrón.", player_id);
    let tile_pos = read_u8("Casilla (##) a mover:");
    let target_player = read_player_to_rob(board, tile_pos as usize, player_id);

    if let Some(target) = target_player {
        place_robber(board, player_id, tile_pos as usize, target);
    } else {
        println!("Movimiento de ladrón cancelado o inválido.");
    }
}

// -----------------------------------------------------------------------------
// VISTA / PRINTS (los que te faltaban)
// -----------------------------------------------------------------------------

pub fn print_visual_board(board: &Board) {
    const TILE_ROWS: [[usize; 5]; 5] = [
        [ 0,  1,  2, 99, 99],
        [11, 12, 13,  3, 99],
        [10, 17, 18, 14,  4],
        [ 9, 16, 15,  5, 99],
        [ 8,  7,  6, 99, 99],
    ];
    const INDENTS: [usize; 5] = [12, 6, 0, 6, 12];

    // ----- helpers -----

    fn v(board: &Board, id: usize) -> String {
        if id >= board.vertices.len() {
            return "??".to_string();
        }
        match board.vertices[id].owner {
            Some(PlayerType::Player1) => "P1".to_string(),
            Some(PlayerType::Player2) => "P2".to_string(),
            Some(PlayerType::Player3) => "P3".to_string(),
            Some(PlayerType::Player4) => "P4".to_string(),
            None => format!("{:02}", id),
        }
    }

    // todos de 6 chars
    fn res(tile: &Tile) -> String {
        if tile.has_robber {
            return "ROBBER".to_string(); // 6 chars
        }
        match tile.material {
            MaterialType::Wood   => "Wood  ".to_string(), // 6
            MaterialType::Brick  => "Brick ".to_string(), // 6
            MaterialType::Sheep  => "Sheep ".to_string(), // 6
            MaterialType::Wheat  => "Wheat ".to_string(), // 6
            MaterialType::Stone  => "Stone ".to_string(), // 6
            MaterialType::Dessert => "Desert".to_string(), // 6
        }
    }

    fn num(tile: &Tile) -> String {
        if tile.has_robber || tile.material == MaterialType::Dessert {
            "  --  ".to_string()
        } else {
            format!("  {:02}  ", tile.number)
        }
    }

    fn render_tile(board: &Board, tile_id: usize) -> [String; 5] {
        if tile_id == 99 {
            return [
                "                 ".to_string(),
                "                 ".to_string(),
                "                 ".to_string(),
                "                 ".to_string(),
                "                 ".to_string(),
            ];
        }

        let tile = &board.tiles[tile_id];
        let vv = &tile.vertices;
        let get = |i: usize| -> String {
            vv.get(i).map(|&id| v(board, id)).unwrap_or_else(|| "--".to_string())
        };

        let v0 = get(0);
        let v1 = get(1);
        let v2 = get(2);
        let v3 = get(3);
        let v4 = get(4);
        let v5 = get(5);

        let r = res(tile);     // 6 chars
        let n = num(tile);     // 6 chars

        // ancho 17
        let line1 = format!("   / {} {} \\   ", v0, v1);
        let line2 = format!("  / {} {} {}\\ ", v5, r, v2);  // v5 (2) + espacio + r (6) + espacio + v2 (2)
        let line3 = format!("  |   {}  | ", n);             // número centrado
        let line4 = format!("  \\ {}_____{} / ", v4, v3);
        let line5 = format!("    \\  {:02}  /   ", tile_id);

        [line1, line2, line3, line4, line5]
    }

    println!();
    println!("======================== MAPA DEL TABLERO ========================");
    println!("Leyenda: P# = jugador | ## = vértice libre | número = ficha");
    println!();

    for (row_idx, row) in TILE_ROWS.iter().enumerate() {
        let indent = " ".repeat(INDENTS[row_idx]);
        let rendered: Vec<[String; 5]> = row.iter().map(|&id| render_tile(board, id)).collect();

        for line_idx in 0..5 {
            let mut line = String::new();
            line.push_str(&indent);
            for tile_lines in &rendered {
                line.push_str(&tile_lines[line_idx]);
            }
            println!("{}", line);
        }

        println!();
    }

    println!("==================================================================");
    println!();
}



pub fn print_buildable_roads(board: &Board, player_id: PlayerType, phase: TurnPhase) {
    println!("\n--- Caminos Disponibles ---");
    let mut found = false;

    for (id, edge) in board.edges.iter().enumerate() {
        if edge.owner.is_some() { continue; }

        let (v1, v2) = edge.vertices;
        let is_buildable = match phase {
            TurnPhase::Normal | TurnPhase::FreeRoad => {
                // estas funciones están en tu game_logic
                crate::game_logic::is_road_connectable(board, player_id, id)
            }
            TurnPhase::Setup { anchor_vertex } => {
                crate::game_logic::is_road_adjacent_to_vertex(board, id, anchor_vertex)
            }
        };

        if is_buildable {
            println!("  - [Camino {}]: Conecta Vértices ({:02}) y ({:02})", id, v1, v2);
            found = true;
        }
    }

    if !found {
        println!("No hay caminos válidos para construir en este momento.");
    }
    println!("----------------------------");
}

pub fn print_player_status(board: &Board, player_id: PlayerType) {
    let player = match board.players.iter().find(|p| p.id == player_id) {
        Some(p) => p,
        None => {
            println!("Error: No se encontró al jugador {:?}", player_id);
            return;
        }
    };
    println!("\n+---------------------------------------+");
    println!("|   TURNO DE: {:?}   |", player_id);
    println!("+---------------------------------------+");
    println!("  Puntos de Victoria: {}", player.victory_points);
    if board.largest_army == Some(player_id) {
        println!("  > (Tiene Mayor Ejército +2 VP)");
    }
    if board.longest_road == Some(player_id) {
        println!("  > (Tiene Camino Más Largo +2 VP)");
    }
    let vp_cards = player.dev_cards.iter().filter(|&&c| c == DevelopmentCard::VictoryPoint).count();
    if vp_cards > 0 {
        println!("  > (Tiene {} Puntos de Victoria en mano)", vp_cards);
    }
    println!("---");
    println!("  Recursos: {}", format_resources(&player.resources));
    println!("---");
    println!("  Piezas Disponibles:");
    println!("    - Asentamientos: {}", player.settlement_quantity);
    println!("    - Ciudades:      {}", player.city_quantity);
    println!("    - Caminos:       {}", player.road_quantity);
    println!("---");
    println!("  Cartas de Desarrollo: {}", format_dev_cards(&player.dev_cards));
    println!("---");
    println!("  Puertos: {}", format_ports(&player.power_ups));
    println!("+---------------------------------------+\n");
}

pub fn print_global_status(board: &Board) {
    println!("\n=========================================");
    println!("==          ESTADO DEL JUEGO         ==");
    println!("=========================================");
    let robber_pos = board.tiles.iter().position(|t| t.has_robber).unwrap_or(99);
    println!("  Ladrón: Casilla {}", robber_pos);
    println!("---");
    match board.largest_army {
        Some(p) => println!("  Mayor Ejército:   {:?} ({} caballeros)", p, board.largest_army_size),
        None => println!("  Mayor Ejército:   Nadie (se necesita > {})", board.largest_army_size),
    }
    match board.longest_road {
        Some(p) => println!("  Camino Más Largo: {:?} ({} segmentos)", p, board.longest_road_size),
        None => println!("  Camino Más Largo: Nadie (se necesita > {})", board.longest_road_size),
    }
    println!("---");
    println!("  Cartas de Desarrollo Restantes: {}", board.development_cards.len());
    println!("---");
    println!("  Resumen de Jugadores:");
    for player in &board.players {
        let resource_total: u8 = player.resources.values().sum();
        println!(
            "    - {:?}: {} VP, {} Recursos, {} Cartas Dev.",
            player.id,
            player.victory_points,
            resource_total,
            player.dev_cards.len()
        );
    }
    println!("=========================================\n");
}

fn format_resources(resources: &HashMap<MaterialType, u8>) -> String {
    let order = [
        MaterialType::Wood, MaterialType::Brick, MaterialType::Sheep,
        MaterialType::Wheat, MaterialType::Stone,
    ];
    let mut parts = Vec::new();
    for material in order {
        let count = *resources.get(&material).unwrap_or(&0);
        if count > 0 {
            parts.push(format!("{:?}: {}", material, count));
        }
    }
    if parts.is_empty() { "Ninguno".to_string() } else { parts.join(", ") }
}

fn format_dev_cards(cards: &Vec<DevelopmentCard>) -> String {
    if cards.is_empty() { return "Ninguna".to_string(); }
    let mut counts: HashMap<DevelopmentCard, u8> = HashMap::new();
    for card in cards {
        *counts.entry(*card).or_insert(0) += 1;
    }
    let mut parts = Vec::new();
    for (card, count) in counts {
        parts.push(format!("{:?}: {}", card, count));
    }
    parts.join(", ")
}

fn format_ports(ports: &Vec<PowerUp>) -> String {
    if ports.is_empty() { return "Ninguna".to_string(); }
    let parts: Vec<String> = ports.iter().map(|p| format!("{:?}", p)).collect();
    parts.join(", ")
}

// -----------------------------------------------------------------------------
// INPUT HELPERS
// -----------------------------------------------------------------------------

fn read_line_prompt(prompt: &str) -> String {
    print!("{} ", prompt);
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Error al leer la línea");
    input.trim().to_string()
}

fn read_u8(prompt: &str) -> u8 {
    loop {
        let input_str = read_line_prompt(prompt);
        match input_str.parse::<u8>() {
            Ok(num) => return num,
            Err(_) => println!("Entrada inválida. Introduce un número."),
        }
    }
}

fn roll_dice() -> u8 {
    // según tu warning, estas son las nuevas APIs
    let mut r = rng();
    let die1: u8 = r.random_range(1..=6);
    let die2: u8 = r.random_range(1..=6);
    die1 + die2
}

fn read_material_type(prompt: &str) -> Option<MaterialType> {
    let input = read_line_prompt(prompt);
    match input.to_lowercase().as_str() {
        "m" | "madera" | "wood" => Some(MaterialType::Wood),
        "l" | "ladrillo" | "brick" => Some(MaterialType::Brick),
        "o" | "oveja" | "sheep" => Some(MaterialType::Sheep),
        "t" | "trigo" | "wheat" => Some(MaterialType::Wheat),
        "p" | "piedra" | "stone" => Some(MaterialType::Stone),
        _ => {
            println!("Material no reconocido. (madera, ladrillo, oveja, trigo, piedra)");
            None
        }
    }
}

// -----------------------------------------------------------------------------
// ROBO DE JUGADOR (te faltaba en el scope del refactor)
// -----------------------------------------------------------------------------

fn read_player_to_rob(board: &Board, tile_id: usize, self_id: PlayerType) -> Option<PlayerType> {
    // esta función la tenías en tu versión anterior
    use crate::game_logic::get_players_adjacent_to_tile;

    let adjacent_players = get_players_adjacent_to_tile(board, tile_id);

    let robbable_players: Vec<PlayerType> = adjacent_players
        .into_iter()
        .filter(|&p| p != self_id)
        .collect();

    if robbable_players.is_empty() {
        println!("No hay jugadores a quienes robar en esa casilla.");
        return None;
    }

    println!("Jugadores disponibles para robar:");
    for (i, player) in robbable_players.iter().enumerate() {
        println!("[{}] {:?}", i, player);
    }

    loop {
        let choice_str = read_line_prompt("Elige un número de jugador:");
        match choice_str.parse::<usize>() {
            Ok(index) if index < robbable_players.len() => {
                return Some(robbable_players[index]);
            }
            _ => println!("Número inválido."),
        }
    }
}
