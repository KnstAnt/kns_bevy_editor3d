// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::{WindowResolution, PresentMode};

use kns_bevy_editor3d::AplicationPlugin;


fn main() {

    std::env::set_var("RUST_BACKTRACE", "full");

    let mut app = App::new();

    app
        .insert_resource(Msaa::default())
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some( Window {
                title: "Editor".to_string(), // ToDo
                resolution: WindowResolution::new(800.,600.),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(AplicationPlugin);

        app.run();        
}