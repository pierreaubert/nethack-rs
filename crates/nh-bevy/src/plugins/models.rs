//! 3D Model spawning system for Player and Monsters
//!
//! Replaces 2D billboards with 3D primitives based on race/category.

use bevy::prelude::*;
use nh_core::monster::Monster;

use crate::components::{MapPosition, MonsterMarker, PlayerMarker};

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, _app: &mut App) {
        // We will likely call these spawners from entities.rs, so maybe just providing the logic here is enough.
        // Or we can register a system that "upgrades" billboards to models?
        // Better: refactor entities.rs to use these helpers.
    }
}

/// Helpers to create meshes for different creature types
pub struct ModelBuilder<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<StandardMaterial>,
}

impl<'a> ModelBuilder<'a> {
    pub fn new(meshes: &'a mut Assets<Mesh>, materials: &'a mut Assets<StandardMaterial>) -> Self {
        Self { meshes, materials }
    }

    /// Spawn a humanoid player model with head, torso, arms and legs
    pub fn spawn_player(
        &mut self,
        commands: &mut Commands,
        player: &nh_core::player::You,
        transform: Transform,
    ) -> Entity {
        use nh_core::player::Race;

        // Get race-specific colors and scale
        let (skin_color, clothes_color, height_scale) = match player.race {
            Race::Human => (
                Color::srgb(0.87, 0.72, 0.53), // Skin tone
                Color::srgb(0.2, 0.2, 0.8),    // Blue clothes
                1.0,
            ),
            Race::Elf => (
                Color::srgb(0.95, 0.87, 0.73), // Pale skin
                Color::srgb(0.2, 0.6, 0.3),    // Green clothes
                1.15,                          // Taller
            ),
            Race::Dwarf => (
                Color::srgb(0.82, 0.64, 0.48), // Tanned skin
                Color::srgb(0.6, 0.3, 0.1),    // Brown clothes
                0.75,                          // Shorter
            ),
            Race::Gnome => (
                Color::srgb(0.9, 0.75, 0.6), // Light skin
                Color::srgb(0.7, 0.5, 0.1),  // Yellow-brown clothes
                0.65,                        // Short
            ),
            Race::Orc => (
                Color::srgb(0.4, 0.5, 0.35), // Greenish skin
                Color::srgb(0.3, 0.25, 0.2), // Dark clothes
                1.05,                        // Slightly larger
            ),
        };

        let skin_material = self.materials.add(StandardMaterial {
            base_color: skin_color,
            perceptual_roughness: 0.8,
            ..default()
        });

        let clothes_material = self.materials.add(StandardMaterial {
            base_color: clothes_color,
            perceptual_roughness: 0.6,
            ..default()
        });

        // Create meshes for body parts
        let head_mesh = self.meshes.add(Sphere::new(0.12).mesh().ico(2).unwrap());
        let torso_mesh = self.meshes.add(Capsule3d::new(0.1, 0.25));
        let arm_mesh = self.meshes.add(Capsule3d::new(0.04, 0.18));
        let leg_mesh = self.meshes.add(Capsule3d::new(0.05, 0.2));

        let map_pos = MapPosition {
            x: player.pos.x,
            y: player.pos.y,
        };

        // Spawn parent entity (invisible root at feet level)
        let parent = commands
            .spawn((
                PlayerMarker,
                map_pos,
                Transform::from_translation(transform.translation)
                    .with_scale(Vec3::splat(height_scale)),
                Visibility::Inherited,
            ))
            .id();

        // Torso (center of the body)
        let torso = commands
            .spawn((
                Mesh3d(torso_mesh),
                MeshMaterial3d(clothes_material.clone()),
                Transform::from_xyz(0.0, 0.45, 0.0),
            ))
            .id();

        // Head (on top of torso)
        let head = commands
            .spawn((
                Mesh3d(head_mesh),
                MeshMaterial3d(skin_material.clone()),
                Transform::from_xyz(0.0, 0.72, 0.0),
            ))
            .id();

        // Left arm
        let left_arm = commands
            .spawn((
                Mesh3d(arm_mesh.clone()),
                MeshMaterial3d(skin_material.clone()),
                Transform::from_xyz(-0.18, 0.45, 0.0).with_rotation(Quat::from_rotation_z(0.15)), // Slight angle outward
            ))
            .id();

        // Right arm
        let right_arm = commands
            .spawn((
                Mesh3d(arm_mesh),
                MeshMaterial3d(skin_material.clone()),
                Transform::from_xyz(0.18, 0.45, 0.0).with_rotation(Quat::from_rotation_z(-0.15)),
            ))
            .id();

        // Left leg
        let left_leg = commands
            .spawn((
                Mesh3d(leg_mesh.clone()),
                MeshMaterial3d(clothes_material.clone()),
                Transform::from_xyz(-0.07, 0.15, 0.0),
            ))
            .id();

        // Right leg
        let right_leg = commands
            .spawn((
                Mesh3d(leg_mesh),
                MeshMaterial3d(clothes_material),
                Transform::from_xyz(0.07, 0.15, 0.0),
            ))
            .id();

        // Parent all body parts to the root
        commands
            .entity(parent)
            .add_children(&[torso, head, left_arm, right_arm, left_leg, right_leg]);

        parent
    }

    pub fn spawn_monster(
        &mut self,
        commands: &mut Commands,
        monster: &Monster,
        monster_def: &nh_core::monster::PerMonst,
        transform: Transform,
    ) -> Entity {
        let symbol = monster_def.symbol;
        let color = nethack_color_to_bevy(monster_def.color);

        let (mesh, offset) = match symbol {
            'd' | 'f' | 'q' => {
                // Quadruped: Horizontal capsule or box
                (self.meshes.add(Cuboid::new(0.6, 0.3, 0.3)), Vec3::Y * 0.15)
            }
            'a' | 'b' | 'e' | 's' | 'v' | 'w' | 'y' => {
                // Small/Floating/Crawler: Sphere
                (
                    self.meshes.add(Sphere::new(0.25).mesh().ico(3).unwrap()),
                    Vec3::Y * 0.25,
                )
            }
            'D' => {
                // Dragon: Large box/structure (placeholder)
                (self.meshes.add(Cuboid::new(0.8, 0.6, 1.2)), Vec3::Y * 0.3)
            }
            _ => {
                // Humanoid/Default: Vertical Capsule
                (self.meshes.add(Capsule3d::new(0.2, 0.4)), Vec3::Y * 0.4)
            }
        };

        commands
            .spawn((
                MonsterMarker {
                    monster_id: monster.id,
                },
                Mesh3d(mesh),
                MeshMaterial3d(self.materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.7,
                    ..default()
                })),
                transform.with_translation(transform.translation + offset),
                Visibility::Inherited,
            ))
            .id()
    }

    /// Spawn a 3D model for a floor object based on its class
    pub fn spawn_object(
        &mut self,
        commands: &mut Commands,
        obj: &nh_core::object::Object,
        transform: Transform,
    ) -> Entity {
        use nh_core::object::ObjectClass;

        let (mesh, material, scale, offset) = match obj.class {
            ObjectClass::Weapon => {
                // Sword: elongated blade with handle
                let blade = self.meshes.add(Cuboid::new(0.08, 0.35, 0.02));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.75, 0.75, 0.85),
                    metallic: 0.9,
                    perceptual_roughness: 0.2,
                    ..default()
                });
                (blade, mat, Vec3::ONE, Vec3::Y * 0.18)
            }
            ObjectClass::Armor => {
                // Chest plate: curved box shape
                let plate = self.meshes.add(Cuboid::new(0.25, 0.2, 0.12));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 0.6),
                    metallic: 0.7,
                    perceptual_roughness: 0.4,
                    ..default()
                });
                (plate, mat, Vec3::ONE, Vec3::Y * 0.1)
            }
            ObjectClass::Ring => {
                // Ring: torus shape
                let ring = self.meshes.add(Torus::new(0.04, 0.08));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.84, 0.0),
                    metallic: 1.0,
                    perceptual_roughness: 0.1,
                    ..default()
                });
                (ring, mat, Vec3::ONE, Vec3::Y * 0.05)
            }
            ObjectClass::Amulet => {
                // Amulet: small gem/sphere
                let gem = self.meshes.add(Sphere::new(0.06).mesh().ico(2).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.65, 0.0),
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    ..default()
                });
                (gem, mat, Vec3::ONE, Vec3::Y * 0.06)
            }
            ObjectClass::Tool => {
                // Tool: small box/block
                let tool = self.meshes.add(Cuboid::new(0.12, 0.08, 0.08));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.55, 0.35, 0.17),
                    perceptual_roughness: 0.8,
                    ..default()
                });
                (tool, mat, Vec3::ONE, Vec3::Y * 0.04)
            }
            ObjectClass::Food => {
                // Food: irregular blob (sphere)
                let food = self.meshes.add(Sphere::new(0.08).mesh().ico(1).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.6, 0.35, 0.15),
                    perceptual_roughness: 0.9,
                    ..default()
                });
                (food, mat, Vec3::new(1.0, 0.7, 1.0), Vec3::Y * 0.06)
            }
            ObjectClass::Potion => {
                // Potion: flask/bottle shape (cylinder)
                let flask = self.meshes.add(Capsule3d::new(0.04, 0.12));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgba(0.8, 0.3, 0.6, 0.8),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 0.1,
                    ..default()
                });
                (flask, mat, Vec3::ONE, Vec3::Y * 0.1)
            }
            ObjectClass::Scroll => {
                // Scroll: rolled cylinder
                let scroll = self.meshes.add(Cylinder::new(0.03, 0.15));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.95, 0.92, 0.82),
                    perceptual_roughness: 0.9,
                    ..default()
                });
                (scroll, mat, Vec3::ONE, Vec3::Y * 0.03)
            }
            ObjectClass::Spellbook => {
                // Spellbook: flat box (book shape)
                let book = self.meshes.add(Cuboid::new(0.12, 0.03, 0.15));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.4, 0.1, 0.6),
                    perceptual_roughness: 0.7,
                    ..default()
                });
                (book, mat, Vec3::ONE, Vec3::Y * 0.02)
            }
            ObjectClass::Wand => {
                // Wand: thin rod
                let wand = self.meshes.add(Cylinder::new(0.015, 0.25));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.7, 0.9),
                    metallic: 0.3,
                    perceptual_roughness: 0.3,
                    ..default()
                });
                (wand, mat, Vec3::ONE, Vec3::Y * 0.02)
            }
            ObjectClass::Coin => {
                // Coin: flat disc
                let coin = self.meshes.add(Cylinder::new(0.06, 0.01));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.84, 0.0),
                    metallic: 1.0,
                    perceptual_roughness: 0.2,
                    ..default()
                });
                (coin, mat, Vec3::ONE, Vec3::Y * 0.01)
            }
            ObjectClass::Gem => {
                // Gem: small faceted sphere
                let gem = self.meshes.add(Sphere::new(0.05).mesh().ico(1).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.9, 0.9, 0.9),
                    alpha_mode: AlphaMode::Blend,
                    metallic: 0.2,
                    perceptual_roughness: 0.05,
                    ..default()
                });
                (gem, mat, Vec3::ONE, Vec3::Y * 0.05)
            }
            ObjectClass::Rock => {
                // Rock: irregular sphere
                let rock = self.meshes.add(Sphere::new(0.07).mesh().ico(1).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.5, 0.5, 0.5),
                    perceptual_roughness: 1.0,
                    ..default()
                });
                (rock, mat, Vec3::new(1.2, 0.8, 1.0), Vec3::Y * 0.05)
            }
            ObjectClass::Ball => {
                // Ball: iron ball (larger sphere)
                let ball = self.meshes.add(Sphere::new(0.12).mesh().ico(2).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.3, 0.3, 0.35),
                    metallic: 0.9,
                    perceptual_roughness: 0.5,
                    ..default()
                });
                (ball, mat, Vec3::ONE, Vec3::Y * 0.12)
            }
            ObjectClass::Chain => {
                // Chain: torus (simplified chain link)
                let chain = self.meshes.add(Torus::new(0.02, 0.06));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(0.6, 0.6, 0.65),
                    metallic: 0.8,
                    perceptual_roughness: 0.4,
                    ..default()
                });
                (chain, mat, Vec3::ONE, Vec3::Y * 0.03)
            }
            ObjectClass::Venom => {
                // Venom: flat puddle
                let puddle = self.meshes.add(Cylinder::new(0.08, 0.005));
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.6, 0.1, 0.7),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 0.1,
                    ..default()
                });
                (puddle, mat, Vec3::ONE, Vec3::Y * 0.003)
            }
            ObjectClass::Random | ObjectClass::IllObj => {
                // Unknown: question mark shape (small sphere)
                let unknown = self.meshes.add(Sphere::new(0.06).mesh().ico(2).unwrap());
                let mat = self.materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.0, 1.0),
                    perceptual_roughness: 0.5,
                    ..default()
                });
                (unknown, mat, Vec3::ONE, Vec3::Y * 0.06)
            }
        };

        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform
                    .with_translation(transform.translation + offset)
                    .with_scale(scale),
                Visibility::Inherited,
            ))
            .id()
    }

    /// Spawn a 3D pile indicator (stack of objects)
    pub fn spawn_pile(
        &mut self,
        commands: &mut Commands,
        count: usize,
        transform: Transform,
    ) -> Entity {
        // Create a small pile of stacked discs
        let disc = self.meshes.add(Cylinder::new(0.1, 0.02));
        let mat = self.materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.65, 0.2),
            metallic: 0.3,
            perceptual_roughness: 0.6,
            ..default()
        });

        let parent = commands
            .spawn((
                Transform::from_translation(transform.translation),
                Visibility::Inherited,
            ))
            .id();

        // Stack 2-4 discs based on pile size
        let stack_count = (count.min(4)).max(2);
        let mut children = Vec::new();

        for i in 0..stack_count {
            let y_offset = i as f32 * 0.025;
            let child = commands
                .spawn((
                    Mesh3d(disc.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0.0, y_offset + 0.01, 0.0),
                ))
                .id();
            children.push(child);
        }

        commands.entity(parent).add_children(&children);
        parent
    }
}

/// Convert NetHack color index to Bevy Color
pub(crate) fn nethack_color_to_bevy(color: u8) -> Color {
    match color {
        0 => Color::BLACK,                // CLR_BLACK
        1 => Color::srgb(0.8, 0.0, 0.0),  // CLR_RED
        2 => Color::srgb(0.0, 0.6, 0.0),  // CLR_GREEN
        3 => Color::srgb(0.6, 0.4, 0.2),  // CLR_BROWN
        4 => Color::srgb(0.0, 0.0, 0.8),  // CLR_BLUE
        5 => Color::srgb(0.8, 0.0, 0.8),  // CLR_MAGENTA
        6 => Color::srgb(0.0, 0.8, 0.8),  // CLR_CYAN
        7 => Color::srgb(0.6, 0.6, 0.6),  // CLR_GRAY
        8 => Color::srgb(0.3, 0.3, 0.3),  // CLR_NO_COLOR (dark gray)
        9 => Color::srgb(1.0, 0.5, 0.0),  // CLR_ORANGE
        10 => Color::srgb(0.0, 1.0, 0.0), // CLR_BRIGHT_GREEN
        11 => Color::srgb(1.0, 1.0, 0.0), // CLR_YELLOW
        12 => Color::srgb(0.3, 0.3, 1.0), // CLR_BRIGHT_BLUE
        13 => Color::srgb(1.0, 0.3, 1.0), // CLR_BRIGHT_MAGENTA
        14 => Color::srgb(0.3, 1.0, 1.0), // CLR_BRIGHT_CYAN
        15 => Color::WHITE,               // CLR_WHITE
        _ => Color::WHITE,
    }
}
