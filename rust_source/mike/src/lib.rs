use wasm_bindgen::prelude::*;
use bevy::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn main() {
    log("Test.");
    App::new().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            canvas: Some("#game".into()),
            ..default()
        }),
        ..default()
    })).add_systems(Startup, start).run();
    log("End.");
}

fn start() {
    log("start");
}