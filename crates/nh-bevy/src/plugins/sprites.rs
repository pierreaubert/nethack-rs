//! Sprite asset loading and path mapping for billboard rendering
//!
//! Loads all PNG sprites from `assets/items/` at startup and provides
//! lookup by monster name, player role, and object class/name.

use std::collections::HashMap;

use bevy::prelude::*;
use nh_core::monster::MonsterSize;
use nh_core::object::ObjectClass;
use nh_core::player::Role;

use crate::resources::AssetRegistryResource;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_sprite_assets);
    }
}

/// Resource holding all loaded sprite image handles and a shared billboard quad mesh.
#[derive(Resource)]
pub struct SpriteAssets {
    /// Monster sprites keyed by lowercase underscore name (e.g. "acid_blob")
    pub monsters: HashMap<String, Handle<Image>>,
    /// Player role sprites keyed by lowercase role name (e.g. "wizard")
    pub player_roles: HashMap<String, Handle<Image>>,
    /// Object sprites keyed by relative path (e.g. "items/items/weapon/long_sword.png")
    pub objects: HashMap<String, Handle<Image>>,
    /// Generic class fallback sprites (e.g. "weapon" → weapon_generic.png handle)
    pub generics: HashMap<String, Handle<Image>>,
    /// Shared 1×1 quad mesh for all billboards
    pub billboard_mesh: Handle<Mesh>,
}

/// Convert a monster name to its sprite asset path.
/// e.g. "acid blob" → "items/monsters/acid_blob.png"
pub fn monster_sprite_path(name: &str) -> String {
    let filename = name.to_lowercase().replace([' ', '-'], "_");
    format!("items/monsters/{filename}.png")
}

/// Convert a player role to its sprite asset path.
pub fn player_sprite_path(role: Role) -> String {
    let role_name = match role {
        Role::Archeologist => "archeologist",
        Role::Barbarian => "barbarian",
        Role::Caveman => "caveman",
        Role::Healer => "healer",
        Role::Knight => "knight",
        Role::Monk => "monk",
        Role::Priest => "priest",
        Role::Ranger => "ranger",
        Role::Rogue => "rogue",
        Role::Samurai => "samurai",
        Role::Tourist => "tourist",
        Role::Valkyrie => "valkyrie",
        Role::Wizard => "wizard",
    };
    format!("items/player/{role_name}.png")
}

/// Convert an object class to its generic fallback sprite path.
pub fn object_class_generic_path(class: ObjectClass) -> Option<String> {
    let name = match class {
        ObjectClass::Weapon => "weapon",
        ObjectClass::Armor => "armor",
        ObjectClass::Ring => "ring",
        ObjectClass::Amulet => "amulet",
        ObjectClass::Tool => "tool",
        ObjectClass::Food => "food",
        ObjectClass::Potion => "potion",
        ObjectClass::Scroll => "scroll",
        ObjectClass::Spellbook => "spellbook",
        ObjectClass::Wand => "wand",
        ObjectClass::Coin => "gold",
        ObjectClass::Gem => "gem",
        ObjectClass::Rock => "rock",
        ObjectClass::Ball => "ball",
        ObjectClass::Chain => "chain",
        ObjectClass::Venom => "venom",
        ObjectClass::Random | ObjectClass::IllObj => return None,
    };
    // Generic sprites are at items/<class>_generic.png (except Coin which is gold.png)
    if class == ObjectClass::Coin {
        Some("items/gold.png".to_string())
    } else {
        Some(format!("items/{name}_generic.png"))
    }
}

/// Get billboard quad scale for a monster based on its size.
pub fn monster_size_scale(size: MonsterSize) -> f32 {
    match size {
        MonsterSize::Tiny => 0.5,
        MonsterSize::Small => 0.65,
        MonsterSize::Medium => 0.8,
        MonsterSize::Large => 1.0,
        MonsterSize::Huge => 1.3,
        MonsterSize::Gigantic => 1.6,
    }
}

fn load_sprite_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Option<Res<AssetRegistryResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mut monsters = HashMap::new();
    let mut player_roles = HashMap::new();
    let mut objects = HashMap::new();
    let mut generics = HashMap::new();

    // Load all monster sprites from the MONSTERS data table
    let monsters_data = nh_core::data::monsters::MONSTERS;
    for permonst in monsters_data {
        let path = monster_sprite_path(permonst.name);
        let key = permonst.name.to_lowercase().replace([' ', '-'], "_");
        let handle: Handle<Image> = asset_server.load(&path);
        monsters.insert(key, handle);
    }

    // Load player role sprites
    let roles = [
        Role::Archeologist,
        Role::Barbarian,
        Role::Caveman,
        Role::Healer,
        Role::Knight,
        Role::Monk,
        Role::Priest,
        Role::Ranger,
        Role::Rogue,
        Role::Samurai,
        Role::Tourist,
        Role::Valkyrie,
        Role::Wizard,
    ];
    for role in roles {
        let path = player_sprite_path(role);
        let key = path
            .strip_prefix("items/player/")
            .unwrap()
            .strip_suffix(".png")
            .unwrap()
            .to_string();
        let handle: Handle<Image> = asset_server.load(&path);
        player_roles.insert(key, handle);
    }

    // Load generic class fallback sprites
    let all_classes = [
        ObjectClass::Weapon,
        ObjectClass::Armor,
        ObjectClass::Ring,
        ObjectClass::Amulet,
        ObjectClass::Tool,
        ObjectClass::Food,
        ObjectClass::Potion,
        ObjectClass::Scroll,
        ObjectClass::Spellbook,
        ObjectClass::Wand,
        ObjectClass::Coin,
        ObjectClass::Gem,
        ObjectClass::Rock,
        ObjectClass::Ball,
        ObjectClass::Chain,
        ObjectClass::Venom,
    ];
    for class in all_classes {
        if let Some(path) = object_class_generic_path(class) {
            let key = format!("{class:?}").to_lowercase();
            let handle: Handle<Image> = asset_server.load(&path);
            generics.insert(key, handle);
        }
    }

    // Load specific object sprites from the asset registry mapping
    if let Some(reg) = &registry {
        // Walk through all objects to pre-load their specific sprites
        let objects_data = nh_core::data::objects::OBJECTS;
        for (idx, obj_def) in objects_data.iter().enumerate() {
            // Try to build a temporary object to query the registry
            let obj = nh_core::object::Object::new(
                nh_core::object::ObjectId(idx as u32),
                0,
                obj_def.class,
            );
            if let Some(path) = reg.0.get_sprite_path(&obj) {
                if !path.is_empty() && !objects.contains_key(path) {
                    let owned_path = path.to_string();
                    let handle: Handle<Image> = asset_server.load(owned_path);
                    objects.insert(path.to_string(), handle);
                }
            }
        }
    }

    // Create shared billboard quad mesh (1×1, centered, facing +Z)
    let billboard_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    commands.insert_resource(SpriteAssets {
        monsters,
        player_roles,
        objects,
        generics,
        billboard_mesh,
    });

    info!(
        "Loaded sprite assets: {} monsters, {} roles, {} generics",
        monsters_data.len(),
        roles.len(),
        all_classes.len(),
    );
}

/// Look up an object's sprite, trying the asset registry first, then class generic.
/// Returns the Handle<Image> if found.
pub fn lookup_object_sprite(
    obj: &nh_core::object::Object,
    sprite_assets: &SpriteAssets,
    registry: Option<&AssetRegistryResource>,
    asset_server: &AssetServer,
) -> Option<Handle<Image>> {
    // 1. Try the asset registry for a specific sprite path
    if let Some(reg) = registry
        && let Ok(icon) = reg.0.get_icon(obj)
        && !icon.bevy_sprite.is_empty()
    {
        return Some(
            sprite_assets
                .objects
                .get(&icon.bevy_sprite)
                .cloned()
                .unwrap_or_else(|| asset_server.load(&icon.bevy_sprite)),
        );
    }

    // 2. Fall back to class generic
    let class_key = format!("{:?}", obj.class).to_lowercase();
    sprite_assets.generics.get(&class_key).cloned()
}
