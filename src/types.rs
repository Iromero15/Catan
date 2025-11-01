
pub type VertexId = usize;
pub type TileId = usize;
pub type EdgeId = usize;
pub type PlayerID = usize;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerType {
    Player1,
    Player2,
    Player3,
    Player4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {

    Setup { anchor_vertex: VertexId },

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DevelopmentCard {
    Knight,
    RoadBuilding,
    YearOfPlenty,
    Monopoly,
    VictoryPoint,
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
    pub has_robber: bool,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub owner: Option<PlayerType>, 
    pub vertices: (VertexId, VertexId),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerType,
    pub resources: HashMap<MaterialType, u8>,
    pub settlement_quantity: u8,
    pub city_quantity: u8,
    pub road_quantity: u8,
    pub power_ups: Vec<PowerUp>, 
    pub victory_points: u8,
    pub dev_cards: Vec<DevelopmentCard>,
}

#[derive(Debug, Clone)]
pub struct Board {
    pub vertices: Vec<Vertex>,
    pub tiles: Vec<Tile>,
    pub edges: Vec<Edge>,
    pub players: Vec<Player>,
    pub development_cards: Vec<DevelopmentCard>,
}
impl Player {
    pub fn new(id: PlayerType) -> Self {
        Player {
            id,
            power_ups: Vec::new(),
            settlement_quantity: 5,
            city_quantity: 4, 
            resources: HashMap::new(), 
            road_quantity: 15,
            victory_points: 0,
            dev_cards: Vec::new(),
        }
    }
}