//! NetHack-rs Bevy 3D client

use bevy::prelude::*;
use bevy::asset::AssetPlugin;
use nh_bevy::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "NetHack-rs".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                // Look for assets in crates/nh-bevy/assets when running from workspace root
                file_path: "crates/nh-bevy".to_string(),
                ..default()
            })
        )
        .add_plugins(GamePlugin)
        .run();
}
