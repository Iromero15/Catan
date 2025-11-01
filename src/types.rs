
pub type VertexId = usize;
pub type TileId = usize;
pub type EdgeId = usize;
pub type PlayerID = usize;

use std::collections::HashMap;

// --- Enums ---
// Añadimos `Debug` para poder imprimir, `Copy` y `Clone` para poder
// pasarlos fácilmente, y `PartialEq` y `Eq` para poder compararlos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerType {
    Player1,
    Player2,
    Player3,
    Player4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    // Turno de fundación (1ro y 2do turno)
    // Debe construir junto al vértice especificado.
    Setup { anchor_vertex: VertexId },
    
    // Turno normal
    // Las reglas de conexión estándar aplican.
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingType {
    Settlement,
    City,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    Wheat2,
    Brick2,
    Stone2,
    Sheep2,
    Wood2,
    Any3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Wheat,
    Brick,
    Stone,
    Sheep,
    Wood,
    Dessert,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub owner: Option<PlayerType>,
    pub building: Option<BuildingType>,
    pub adjacent_tiles: Vec<TileId>, 
    pub adjacent_edges: Vec<EdgeId>,
    pub power_up: Option<PowerUp>,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub material: MaterialType,
    pub number: u8, 
    pub vertices: [VertexId; 6], 
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub owner: Option<PlayerType>, 
    pub vertices: (VertexId, VertexId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerType,
    
    // Un HashMap es la mejor forma de guardar recursos.
    // Ej: { MaterialType::Wood: 2, MaterialType::Brick: 1 }
    pub resources: HashMap<MaterialType, u8>,
    
    // Los contadores de piezas (¡recuerda mantenerlos sincronizados!)
    pub settlement_quantity: u8,
    pub city_quantity: u8,
    pub road_quantity: u8,

    // Los puertos que ha conseguido (generalmente al construir
    // en un vértice con puerto)
    pub power_ups: Vec<PowerUp>, 
    
    // (Opcional, pero útil) Puntos de victoria "visibles"
    // Se calcula con (settlement_quantity * 1) + (city_quantity * 2)
    pub victory_points: u8,

    // (Opcional) Cartas de desarrollo que tiene en la mano
    // pub dev_cards: Vec<DevelopmentCard>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub vertices: Vec<Vertex>,
    pub tiles: Vec<Tile>,
    pub edges: Vec<Edge>,
    pub players: Vec<Player>,
}
impl Player {
    /**
     * Crea una nueva instancia de Jugador con valores iniciales.
     */
    pub fn new(id: PlayerType) -> Self {
        
        // (Nota: Corregí tu tipo `PowerUp` a `Power_up` para que coincida 
        // con tu enum. Si tu enum realmente se llama `PowerUp`, usa ese)
        
        Player {
            id,
            power_ups: Vec::new(),
            settlement_quantity: 5, // 5 asentamientos iniciales
            city_quantity: 4,       // 4 ciudades iniciales
            resources: HashMap::new(), 
            road_quantity: 15,
            victory_points: 0,
        }
    }
}