//! 3D Model spawning system for Player and Monsters
//!
//! Replaces 2D billboards with 3D primitives based on race/category.

use bevy::prelude::*;
use nh_core::monster::Monster;

use crate::components::{MonsterMarker, PlayerMarker};

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

    pub fn spawn_player(&mut self, commands: &mut Commands, player: &nh_core::player::You, transform: Transform) -> Entity {
        use nh_core::player::Race;

        let (mesh, color, scale) = match player.race {
            Race::Human => (
                self.meshes.add(Capsule3d::new(0.2, 0.4)),
                Color::srgb(0.2, 0.2, 0.8), // Blue
                Vec3::ONE,
            ),
            Race::Elf => (
                self.meshes.add(Cylinder::new(0.2, 0.8)),
                Color::srgb(0.2, 0.8, 0.2), // Green
                Vec3::new(0.8, 1.1, 0.8), // Tall and slender
            ),
            Race::Dwarf => (
                self.meshes.add(Cuboid::new(0.5, 0.6, 0.3)),
                Color::srgb(0.8, 0.2, 0.2), // Red
                Vec3::new(1.2, 0.8, 1.2), // Stout
            ),
            Race::Gnome => (
                self.meshes.add(Sphere::new(0.25).mesh().ico(3).unwrap()),
                Color::srgb(0.9, 0.9, 0.2), // Yellow
                Vec3::splat(0.9),
            ),
            Race::Orc => (
                self.meshes.add(Capsule3d::new(0.25, 0.3)),
                Color::srgb(0.4, 0.5, 0.4), // Gray-Green
                Vec3::new(1.1, 0.9, 1.1),
            ),
        };
        
        commands.spawn((
            PlayerMarker,
            Mesh3d(mesh),
            MeshMaterial3d(self.materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.5,
                ..default()
            })),
            transform.with_translation(transform.translation + Vec3::Y * 0.4).with_scale(scale),
        )).id()
    }

    pub fn spawn_monster(&mut self, commands: &mut Commands, monster: &Monster, monster_def: &nh_core::monster::PerMonst, transform: Transform) -> Entity {
        let symbol = monster_def.symbol;
        let color = nethack_color_to_bevy(monster_def.color);

        let (mesh, offset) = match symbol {
            'd' | 'f' | 'q' => {
                // Quadruped: Horizontal capsule or box
                (
                    self.meshes.add(Cuboid::new(0.6, 0.3, 0.3)),
                    Vec3::Y * 0.15
                )
            },
            'a' | 'b' | 'e' | 's' | 'v' | 'w' | 'y' => {
                // Small/Floating/Crawler: Sphere
                (
                    self.meshes.add(Sphere::new(0.25).mesh().ico(3).unwrap()),
                    Vec3::Y * 0.25
                )
            },
            'D' => {
                // Dragon: Large box/structure (placeholder)
                (
                    self.meshes.add(Cuboid::new(0.8, 0.6, 1.2)),
                    Vec3::Y * 0.3
                )
            },
            _ => {
                // Humanoid/Default: Vertical Capsule
                (
                    self.meshes.add(Capsule3d::new(0.2, 0.4)),
                    Vec3::Y * 0.4
                )
            }
        };

        commands.spawn((
            MonsterMarker { monster_id: monster.id },
            Mesh3d(mesh),
            MeshMaterial3d(self.materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.7,
                ..default()
            })),
            transform.with_translation(transform.translation + offset),
        )).id()
    }
}

/// Convert NetHack color index to Bevy Color (reused from entities.rs logic)
fn nethack_color_to_bevy(color: u8) -> Color {
    match color {
        0 => Color::BLACK,                   // CLR_BLACK
        1 => Color::srgb(0.8, 0.0, 0.0),     // CLR_RED
        2 => Color::srgb(0.0, 0.6, 0.0),     // CLR_GREEN
        3 => Color::srgb(0.6, 0.4, 0.2),     // CLR_BROWN
        4 => Color::srgb(0.0, 0.0, 0.8),     // CLR_BLUE
        5 => Color::srgb(0.8, 0.0, 0.8),     // CLR_MAGENTA
        6 => Color::srgb(0.0, 0.8, 0.8),     // CLR_CYAN
        7 => Color::srgb(0.6, 0.6, 0.6),     // CLR_GRAY
        8 => Color::srgb(0.3, 0.3, 0.3),     // CLR_NO_COLOR (dark gray)
        9 => Color::srgb(1.0, 0.5, 0.0),     // CLR_ORANGE
        10 => Color::srgb(0.0, 1.0, 0.0),    // CLR_BRIGHT_GREEN
        11 => Color::srgb(1.0, 1.0, 0.0),    // CLR_YELLOW
        12 => Color::srgb(0.3, 0.3, 1.0),    // CLR_BRIGHT_BLUE
        13 => Color::srgb(1.0, 0.3, 1.0),    // CLR_BRIGHT_MAGENTA
        14 => Color::srgb(0.3, 1.0, 1.0),    // CLR_BRIGHT_CYAN
        15 => Color::WHITE,                  // CLR_WHITE
        _ => Color::WHITE,
    }
}
