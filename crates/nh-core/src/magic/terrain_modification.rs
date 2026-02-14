//! Terrain modification system - Spells create or modify terrain
//!
//! Allows spells to temporarily or permanently modify level terrain.

use serde::{Deserialize, Serialize};

/// Types of terrain modifications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainSpellEffect {
    CreateWall,
    CreatePit,
    CreateLava,
    CreateIce,
    CreateWeb,
    DestroyWall,
    RaiseTerrain,
    LowerTerrain,
}

impl TerrainSpellEffect {
    pub const fn name(&self) -> &'static str {
        match self {
            TerrainSpellEffect::CreateWall => "Create Wall",
            TerrainSpellEffect::CreatePit => "Create Pit",
            TerrainSpellEffect::CreateLava => "Create Lava",
            TerrainSpellEffect::CreateIce => "Create Ice",
            TerrainSpellEffect::CreateWeb => "Create Web",
            TerrainSpellEffect::DestroyWall => "Destroy Wall",
            TerrainSpellEffect::RaiseTerrain => "Raise Terrain",
            TerrainSpellEffect::LowerTerrain => "Lower Terrain",
        }
    }
}

/// Temporary terrain modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryTerrain {
    pub x: i8,
    pub y: i8,
    pub effect: TerrainSpellEffect,
    pub duration: u32,
    pub creator_level: u32,
}

impl TemporaryTerrain {
    pub fn new(x: i8, y: i8, effect: TerrainSpellEffect, duration: u32) -> Self {
        Self {
            x,
            y,
            effect,
            duration,
            creator_level: 0,
        }
    }

    pub fn tick(&mut self) {
        if self.duration > 0 {
            self.duration -= 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.duration > 0
    }
}

/// Tracker for all terrain modifications on a level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TerrainModificationTracker {
    pub modifications: Vec<TemporaryTerrain>,
}

impl TerrainModificationTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_modification(&mut self, mod_: TemporaryTerrain) {
        self.modifications.push(mod_);
    }

    pub fn tick_all(&mut self) {
        for mod_ in &mut self.modifications {
            mod_.tick();
        }
        self.modifications.retain(|m| m.is_active());
    }

    pub fn get_at_position(&self, x: i8, y: i8) -> Option<&TemporaryTerrain> {
        self.modifications.iter().find(|m| m.x == x && m.y == y)
    }

    pub fn clear(&mut self) {
        self.modifications.clear();
    }
}

pub fn modify_terrain(effect: TerrainSpellEffect, x: i8, y: i8, duration: u32) -> TemporaryTerrain {
    TemporaryTerrain::new(x, y, effect, duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporary_terrain() {
        let mut terrain = TemporaryTerrain::new(5, 10, TerrainSpellEffect::CreateWall, 5);
        assert!(terrain.is_active());
        for _ in 0..5 {
            terrain.tick();
        }
        assert!(!terrain.is_active());
    }

    #[test]
    fn test_terrain_tracker() {
        let mut tracker = TerrainModificationTracker::new();
        let terrain = TemporaryTerrain::new(5, 10, TerrainSpellEffect::CreatePit, 3);
        tracker.add_modification(terrain);
        assert_eq!(tracker.modifications.len(), 1);
        tracker.tick_all();
        assert_eq!(tracker.modifications.len(), 1);
        for _ in 0..2 {
            tracker.tick_all();
        }
        assert_eq!(tracker.modifications.len(), 0);
    }
}
