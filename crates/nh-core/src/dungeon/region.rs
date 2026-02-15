//! Region effects system (region.c)
//!
//! Regions are persistent area effects on the level — gas clouds,
//! stinking clouds, areas of silence, etc.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};

/// Maximum number of active regions per level
pub const MAX_REGIONS: usize = 32;

/// Region effect types (matches C NHRegion types)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionType {
    /// Stinking cloud (from scroll or spell)
    StinkingCloud,
    /// Gas cloud (from trap or monster)
    GasCloud,
    /// Fog cloud (obscures vision)
    FogCloud,
    /// Silence (prevents spellcasting)
    Silence,
    /// Force field (blocks movement)
    ForceField,
}

impl RegionType {
    pub const fn name(&self) -> &'static str {
        match self {
            RegionType::StinkingCloud => "a cloud of noxious gas",
            RegionType::GasCloud => "a cloud of gas",
            RegionType::FogCloud => "a cloud of fog",
            RegionType::Silence => "an area of silence",
            RegionType::ForceField => "a force field",
        }
    }

    /// Whether this region blocks vision
    pub const fn blocks_vision(&self) -> bool {
        matches!(self, RegionType::StinkingCloud | RegionType::GasCloud | RegionType::FogCloud)
    }

    /// Whether this region blocks movement
    pub const fn blocks_movement(&self) -> bool {
        matches!(self, RegionType::ForceField)
    }

    /// Whether this region damages entities inside it
    pub const fn is_damaging(&self) -> bool {
        matches!(self, RegionType::StinkingCloud | RegionType::GasCloud)
    }
}

/// A persistent region effect on the level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    /// Region type
    pub region_type: RegionType,
    /// Bounding box: top-left (x1, y1)
    pub x1: i8,
    pub y1: i8,
    /// Bounding box: bottom-right (x2, y2)
    pub x2: i8,
    pub y2: i8,
    /// Turns remaining before the region dissipates (0 = permanent)
    pub turns_remaining: u32,
    /// Damage per turn for damaging regions
    pub damage: i32,
    /// Whether the region is visible to the player
    pub visible: bool,
    /// Whether the player created this region
    pub player_created: bool,
}

impl Region {
    /// Create a new region
    pub fn new(
        region_type: RegionType,
        x1: i8,
        y1: i8,
        x2: i8,
        y2: i8,
        duration: u32,
    ) -> Self {
        let damage = match region_type {
            RegionType::StinkingCloud => 2,
            RegionType::GasCloud => 4,
            _ => 0,
        };

        Self {
            region_type,
            x1: x1.min(x2),
            y1: y1.min(y2),
            x2: x1.max(x2),
            y2: y1.max(y2),
            turns_remaining: duration,
            damage,
            visible: true,
            player_created: false,
        }
    }

    /// Check if a position is inside this region
    pub fn contains(&self, x: i8, y: i8) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    /// Tick the region — reduce duration. Returns true if still active.
    pub fn tick(&mut self) -> bool {
        if self.turns_remaining == 0 {
            return true; // Permanent
        }
        self.turns_remaining = self.turns_remaining.saturating_sub(1);
        self.turns_remaining > 0
    }

    /// Get the area of this region in tiles
    pub fn area(&self) -> i32 {
        ((self.x2 - self.x1 + 1) as i32) * ((self.y2 - self.y1 + 1) as i32)
    }
}

/// Create a stinking cloud at a position with a given radius.
///
/// Matches C create_gas_cloud from region.c.
pub fn create_gas_cloud(
    center_x: i8,
    center_y: i8,
    radius: i8,
    duration: u32,
    cloud_type: RegionType,
) -> Region {
    Region::new(
        cloud_type,
        center_x - radius,
        center_y - radius,
        center_x + radius,
        center_y + radius,
        duration,
    )
}

/// Process all regions on a level — tick timers, apply damage.
///
/// Returns a list of (region_index, message) for regions that affected the player.
pub fn process_regions(
    regions: &mut Vec<Region>,
    player_x: i8,
    player_y: i8,
) -> Vec<(usize, String, i32)> {
    let mut effects = Vec::new();

    // Tick all regions and collect effects
    let mut i = 0;
    while i < regions.len() {
        if !regions[i].tick() {
            regions.remove(i);
            continue;
        }

        if regions[i].contains(player_x, player_y) && regions[i].region_type.is_damaging() {
            effects.push((
                i,
                format!("You are engulfed in {}!", regions[i].region_type.name()),
                regions[i].damage,
            ));
        }
        i += 1;
    }

    effects
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_contains() {
        let region = Region::new(RegionType::StinkingCloud, 5, 5, 10, 10, 15);
        assert!(region.contains(7, 7));
        assert!(region.contains(5, 5)); // Edge
        assert!(region.contains(10, 10)); // Edge
        assert!(!region.contains(4, 7)); // Outside
        assert!(!region.contains(11, 7)); // Outside
    }

    #[test]
    fn test_region_tick_temporary() {
        let mut region = Region::new(RegionType::GasCloud, 5, 5, 10, 10, 3);
        assert!(region.tick()); // 2 remaining
        assert!(region.tick()); // 1 remaining
        assert!(!region.tick()); // 0 remaining — expired
    }

    #[test]
    fn test_region_tick_permanent() {
        let mut region = Region::new(RegionType::Silence, 5, 5, 10, 10, 0);
        assert!(region.tick()); // Permanent — always active
        assert!(region.tick());
    }

    #[test]
    fn test_region_area() {
        let region = Region::new(RegionType::FogCloud, 5, 5, 7, 7, 10);
        assert_eq!(region.area(), 9); // 3x3
    }

    #[test]
    fn test_create_gas_cloud() {
        let cloud = create_gas_cloud(10, 10, 2, 15, RegionType::StinkingCloud);
        assert!(cloud.contains(10, 10)); // Center
        assert!(cloud.contains(8, 8)); // Edge
        assert!(!cloud.contains(7, 7)); // Outside
    }

    #[test]
    fn test_process_regions_damage() {
        let mut regions = vec![
            Region::new(RegionType::StinkingCloud, 5, 5, 10, 10, 5),
        ];
        let effects = process_regions(&mut regions, 7, 7);
        assert_eq!(effects.len(), 1);
        assert!(effects[0].2 > 0); // Should deal damage
    }

    #[test]
    fn test_process_regions_outside() {
        let mut regions = vec![
            Region::new(RegionType::StinkingCloud, 5, 5, 10, 10, 5),
        ];
        let effects = process_regions(&mut regions, 20, 20);
        assert!(effects.is_empty());
    }

    #[test]
    fn test_process_regions_expiry() {
        let mut regions = vec![
            Region::new(RegionType::GasCloud, 5, 5, 10, 10, 1),
        ];
        process_regions(&mut regions, 0, 0);
        assert!(regions.is_empty()); // Should have expired
    }

    #[test]
    fn test_region_type_properties() {
        assert!(RegionType::StinkingCloud.blocks_vision());
        assert!(RegionType::StinkingCloud.is_damaging());
        assert!(!RegionType::Silence.blocks_vision());
        assert!(!RegionType::Silence.is_damaging());
        assert!(RegionType::ForceField.blocks_movement());
    }
}
