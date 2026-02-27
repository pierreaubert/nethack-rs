//! NetHack-rs Bevy 3D client

use std::path::{Path, PathBuf};
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy_obj::ObjPlugin;
use nh_bevy::GamePlugin;
use nh_bevy::resources::{AssetRegistryResource, AssetsConfig};
use nh_assets::registry::AssetRegistry;
use nh_assets::mapping::AssetMapping;

/// Find the best candidate for the `assets` directory.
fn find_assets_path() -> PathBuf {
    // 1. Try local "assets" (workspace root)
    let local_assets = Path::new("assets");
    if local_assets.is_dir() && local_assets.join("mapping.json").is_file() {
        return std::fs::canonicalize(local_assets).unwrap_or_else(|_| local_assets.to_path_buf());
    }

    // 2. Try parent of parent (if running from crates/nh-bevy)
    let up_assets = Path::new("../../assets");
    if up_assets.is_dir() && up_assets.join("mapping.json").is_file() {
        return std::fs::canonicalize(up_assets).unwrap_or_else(|_| up_assets.to_path_buf());
    }

    // Fallback to "assets"
    PathBuf::from("assets")
}

fn main() {
    // Early system initialization (C: sys_early_init + decl_init)
    nh_core::world::sys_early_init();
    nh_core::world::decl_init();

    // Find the assets directory
    let base_assets = find_assets_path();
    let mapping_path = base_assets.join("mapping.json");

    println!("Asset Root Detected: {:?}", base_assets);
    println!("Mapping Path: {:?}", mapping_path);

    // Load asset mapping
    let registry = AssetRegistry::load_from_file(&mapping_path).unwrap_or_else(|_| {
        eprintln!("Failed to load mapping from {:?}", mapping_path);
        AssetRegistry::new(AssetMapping::default())
    });

    App::new()
        .insert_resource(AssetsConfig { base_path: base_assets.clone() })
        .insert_resource(AssetRegistryResource(registry))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "NetHack-rs".to_string(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Bevy resolves file_path relative to the working directory.
                    // We point it to the discovered assets directory.
                    file_path: base_assets.to_string_lossy().to_string(),
                    ..default()
                }),
        )
        .add_plugins(ObjPlugin)
        .add_plugins(GamePlugin)
        .run();
}
