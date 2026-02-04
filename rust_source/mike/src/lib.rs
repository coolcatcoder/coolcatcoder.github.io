use wasm_bindgen::prelude::*;
use bevy::prelude::*;

#[wasm_bindgen]
pub fn main() {
    App::new().add_plugins((DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#game".into()),
            ..default()
        }),
        ..default()
    }), plugin)).run();
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, start);
}

fn start() {
    info!("Bad!");
}