mod setup;           // tu setup.rs
mod types;
mod game_logic;
mod development_cards;
mod visual_game;     // el que te dejo abajo

use bevy::prelude::*;
use visual_game::VisualGamePlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Catan en Bevy".to_string(),
                    resolution: (1280., 720.).into(),
                    ..Default::default()
                }),
                ..Default::default()
            })
        )
        .add_plugins(VisualGamePlugin)
        .run();
}
