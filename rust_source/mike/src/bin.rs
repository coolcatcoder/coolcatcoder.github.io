use bevy::prelude::*;

fn main() {
    App::new().add_plugins((DefaultPlugins, mike::plugin)).run();
}