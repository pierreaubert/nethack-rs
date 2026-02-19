//! Pre-loads 3D OBJ models and their textures from `assets/models/`.
//!
//! Each model directory has the structure `models/<name>/0/mesh.obj` + `texture.png`.
//! The `ModelAssets` resource maps directory names (e.g. `"long_sword"`) to loaded handles.

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;

pub struct ModelAssetsPlugin;

impl Plugin for ModelAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_model_assets);
    }
}

/// A single loaded 3D model: OBJ mesh + PNG texture.
pub struct ModelEntry {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Image>,
}

/// Resource holding all loaded 3D model handles, keyed by directory name.
#[derive(Resource)]
pub struct ModelAssets {
    pub models: HashMap<String, ModelEntry>,
}

/// Extract the model directory name from a `bevy_sprite` path.
///
/// `"items/weapon/long_sword.png"` → `"long_sword"`
/// `"items/monsters/acid_blob.png"` → `"acid_blob"`
pub fn model_name_from_sprite_path(bevy_sprite: &str) -> &str {
    let filename = bevy_sprite.rsplit('/').next().unwrap_or(bevy_sprite);
    filename.strip_suffix(".png").unwrap_or(filename)
}

fn load_model_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut models = HashMap::new();

    // Scan the models directory on disk.
    // Bevy's AssetPlugin is configured with file_path = "assets", so the workspace-relative
    // path `assets/models/` corresponds to the OS path `<workspace>/assets/models/`.
    // We need to find the workspace root. The binary runs via `cargo run` from the workspace root,
    // so `std::env::current_dir()` gives us that.
    let models_dir = Path::new("assets/models");

    let Ok(entries) = std::fs::read_dir(models_dir) else {
        warn!(
            "Could not read models directory at {:?} — no 3D models will be loaded",
            models_dir
        );
        commands.insert_resource(ModelAssets { models });
        return;
    };

    for entry in entries.flatten() {
        let dir_name = entry.file_name();
        let Some(name) = dir_name.to_str() else {
            continue;
        };

        // Check for the expected sub-structure: <name>/0/mesh.obj + texture.png
        let mesh_path = entry.path().join("0/mesh.obj");
        let texture_path = entry.path().join("0/texture.png");

        if !mesh_path.exists() || !texture_path.exists() {
            continue;
        }

        // Asset paths are relative to the `assets/` root (the AssetPlugin file_path)
        let mesh_asset_path = format!("models/{name}/0/mesh.obj");
        let texture_asset_path = format!("models/{name}/0/texture.png");

        let mesh: Handle<Mesh> = asset_server.load(&mesh_asset_path);
        let texture: Handle<Image> = asset_server.load(&texture_asset_path);

        models.insert(name.to_string(), ModelEntry { mesh, texture });
    }

    info!("Loaded {} 3D model entries from assets/models/", models.len());
    commands.insert_resource(ModelAssets { models });
}
