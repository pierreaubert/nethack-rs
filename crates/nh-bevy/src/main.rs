//! NetHack-rs Bevy 3D client

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use nh_bevy::GamePlugin;
use nh_bevy::resources::AssetRegistryResource;
use nh_assets::registry::AssetRegistry;
use nh_assets::mapping::AssetMapping;

fn main() {
    // Load asset mapping
    let assets_path = "assets/mapping.json";
    let registry = AssetRegistry::load_from_file(assets_path).unwrap_or_else(|_| {
        AssetRegistry::new(AssetMapping::default())
    });

    App::new()
        .insert_resource(AssetRegistryResource(registry))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "NetHack-rs".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Bevy resolves file_path relative to the working directory,
                    // which is the workspace root when using `cargo run`.
                    file_path: "assets".to_string(),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
