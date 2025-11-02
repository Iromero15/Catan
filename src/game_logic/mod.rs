// en src/game_logic/mod.rs

// 1. Declara los nuevos módulos de archivos
mod building;
mod economy;
mod victory;

// 2. Exporta (hace públicas) las funciones que `main.rs`
//    o `development_cards.rs` necesitarán.

// Desde `building.rs`
pub use building::{
    place_house, 
    place_city, 
    place_road,
    is_road_adjacent_to_vertex,
    is_road_connectable
};

// Desde `economy.rs`
pub use economy::{
    give_materials_on_roll, 
    give_starting_resources, 
    trade_with_bank, 
    buy_development_card, 
    place_robber,
    get_players_adjacent_to_tile // <-- ¡AÑADE ESTA LÍNEA!
};

// Desde `victory.rs`
pub use victory::{
    check_for_winner, 
    update_largest_army
};