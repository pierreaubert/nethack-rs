//! Ring and amulet magic system
//!
//! Manages enchanted rings and amulets with passive effects, wearing mechanics,
//! and power drain/feedback effects.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::Object;
use crate::player::{Attribute, Property, You};
use crate::rng::GameRng;
use serde::{Deserialize, Serialize};

/// Ring/amulet power types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingPower {
    // Ability rings
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,

    // Protection rings
    Protection,
    FireResistance,
    ColdResistance,
    PoisonResistance,

    // Movement rings
    Speed,
    Levitation,
    Teleportation,
    TeleportControl,

    // Vision rings
    SeeInvisible,
    Infravision,

    // Other rings
    Regeneration,
    Invisibility,
    FreeAction,
    Stealth,
}

impl RingPower {
    /// Get ring name
    pub fn name(&self) -> &'static str {
        match self {
            RingPower::Strength => "ring of strength",
            RingPower::Dexterity => "ring of dexterity",
            RingPower::Constitution => "ring of constitution",
            RingPower::Intelligence => "ring of intelligence",
            RingPower::Wisdom => "ring of wisdom",
            RingPower::Charisma => "ring of charisma",
            RingPower::Protection => "ring of protection",
            RingPower::FireResistance => "ring of fire resistance",
            RingPower::ColdResistance => "ring of cold resistance",
            RingPower::PoisonResistance => "ring of poison resistance",
            RingPower::Speed => "ring of speed",
            RingPower::Levitation => "ring of levitation",
            RingPower::Teleportation => "ring of teleportation",
            RingPower::TeleportControl => "ring of teleport control",
            RingPower::SeeInvisible => "ring of see invisible",
            RingPower::Infravision => "ring of infravision",
            RingPower::Regeneration => "ring of regeneration",
            RingPower::Invisibility => "ring of invisibility",
            RingPower::FreeAction => "ring of free action",
            RingPower::Stealth => "ring of stealth",
        }
    }

    /// Get power drain per turn (energy cost)
    pub fn power_drain(&self) -> i32 {
        match self {
            // Low drain (1-2)
            RingPower::Strength | RingPower::Dexterity | RingPower::Constitution => 1,
            RingPower::SeeInvisible | RingPower::Infravision => 1,

            // Medium drain (2-4)
            RingPower::Protection | RingPower::FireResistance | RingPower::ColdResistance => 2,
            RingPower::Regeneration | RingPower::FreeAction => 2,

            // High drain (4-8)
            RingPower::Speed => 4,
            RingPower::Invisibility => 5,
            RingPower::Levitation => 3,
            RingPower::Teleportation => 6,

            // Other
            RingPower::Intelligence | RingPower::Wisdom | RingPower::Charisma => 2,
            RingPower::TeleportControl => 4,
            RingPower::PoisonResistance => 2,
            RingPower::Stealth => 3,
        }
    }

    /// Check if ring causes power feedback when over-drained
    pub fn causes_feedback(&self) -> bool {
        matches!(
            self,
            RingPower::Speed
                | RingPower::Invisibility
                | RingPower::Teleportation
                | RingPower::Levitation
        )
    }
}

/// Wearable ring with tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WornRing {
    pub power: RingPower,
    pub object_id: u32,
    pub enchantment: i8,
    pub turns_worn: i32,
}

impl WornRing {
    pub fn new(power: RingPower, object_id: u32, enchantment: i8) -> Self {
        Self {
            power,
            object_id,
            enchantment,
            turns_worn: 0,
        }
    }
}

/// Ring wear tracker (max 2 rings worn)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RingWear {
    pub left_hand: Option<WornRing>,
    pub right_hand: Option<WornRing>,
}

impl RingWear {
    pub fn new() -> Self {
        Self {
            left_hand: None,
            right_hand: None,
        }
    }

    /// Wear ring on specific hand
    pub fn wear_ring(&mut self, ring: WornRing, hand: RingHand) -> Result<(), String> {
        match hand {
            RingHand::Left if self.left_hand.is_some() => {
                Err("You already wear a ring on your left hand.".to_string())
            }
            RingHand::Left => {
                self.left_hand = Some(ring);
                Ok(())
            }
            RingHand::Right if self.right_hand.is_some() => {
                Err("You already wear a ring on your right hand.".to_string())
            }
            RingHand::Right => {
                self.right_hand = Some(ring);
                Ok(())
            }
        }
    }

    /// Remove ring from hand
    pub fn remove_ring(&mut self, hand: RingHand) -> Option<WornRing> {
        match hand {
            RingHand::Left => self.left_hand.take(),
            RingHand::Right => self.right_hand.take(),
        }
    }

    /// Get all worn rings
    pub fn get_worn_rings(&self) -> Vec<&WornRing> {
        let mut rings = Vec::new();
        if let Some(ring) = &self.left_hand {
            rings.push(ring);
        }
        if let Some(ring) = &self.right_hand {
            rings.push(ring);
        }
        rings
    }

    /// Check if ring power is active
    pub fn has_ring_power(&self, power: RingPower) -> bool {
        self.get_worn_rings().iter().any(|r| r.power == power)
    }
}

/// Ring hand location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingHand {
    Left,
    Right,
}

/// Apply ring effects to player
pub fn apply_ring_effects(player: &mut You, rings: &RingWear) {
    for ring in rings.get_worn_rings() {
        match ring.power {
            RingPower::Strength => {
                let current = player.attr_current.get(Attribute::Strength);
                player
                    .attr_current
                    .set(Attribute::Strength, (current + 1).min(25));
            }
            RingPower::Dexterity => {
                let current = player.attr_current.get(Attribute::Dexterity);
                player
                    .attr_current
                    .set(Attribute::Dexterity, (current + 1).min(25));
            }
            RingPower::Constitution => {
                let current = player.attr_current.get(Attribute::Constitution);
                player
                    .attr_current
                    .set(Attribute::Constitution, (current + 1).min(25));
            }
            RingPower::Intelligence => {
                let current = player.attr_current.get(Attribute::Intelligence);
                player
                    .attr_current
                    .set(Attribute::Intelligence, (current + 1).min(25));
            }
            RingPower::Wisdom => {
                let current = player.attr_current.get(Attribute::Wisdom);
                player
                    .attr_current
                    .set(Attribute::Wisdom, (current + 1).min(25));
            }
            RingPower::Charisma => {
                let current = player.attr_current.get(Attribute::Charisma);
                player
                    .attr_current
                    .set(Attribute::Charisma, (current + 1).min(25));
            }
            RingPower::Protection => {
                player.armor_class = (player.armor_class - 1).max(-10);
            }
            RingPower::FireResistance => {
                player.properties.grant_intrinsic(Property::FireResistance);
            }
            RingPower::ColdResistance => {
                player.properties.grant_intrinsic(Property::ColdResistance);
            }
            RingPower::PoisonResistance => {
                player
                    .properties
                    .grant_intrinsic(Property::PoisonResistance);
            }
            RingPower::Speed => {
                player.movement_points = (player.movement_points + 3).min(50);
            }
            RingPower::Levitation => {
                player.properties.grant_intrinsic(Property::Levitation);
            }
            RingPower::Teleportation => {
                player.properties.grant_intrinsic(Property::Teleportation);
            }
            RingPower::TeleportControl => {
                player.properties.grant_intrinsic(Property::TeleportControl);
            }
            RingPower::SeeInvisible => {
                player.properties.grant_intrinsic(Property::SeeInvisible);
            }
            RingPower::Infravision => {
                player.properties.grant_intrinsic(Property::Infravision);
            }
            RingPower::Regeneration => {
                player.properties.grant_intrinsic(Property::Regeneration);
            }
            RingPower::Invisibility => {
                player.properties.grant_intrinsic(Property::Invisibility);
            }
            RingPower::FreeAction => {
                player.properties.grant_intrinsic(Property::FreeAction);
            }
            RingPower::Stealth => {
                player.properties.grant_intrinsic(Property::Stealth);
            }
        }
    }
}

/// Calculate power drain from worn rings
pub fn calculate_ring_drain(rings: &RingWear) -> i32 {
    rings
        .get_worn_rings()
        .iter()
        .map(|r| r.power.power_drain() + (r.enchantment.max(0) as i32))
        .sum()
}

/// Check for power feedback (danger when over-drained)
pub fn check_power_feedback(player: &You, rings: &RingWear, rng: &mut GameRng) -> Option<String> {
    let drain = calculate_ring_drain(rings);

    // Feedback chance increases with energy deficit
    let energy_deficit = (drain as i32 - player.energy).max(0);
    let feedback_chance = (energy_deficit / 5).min(50);

    if rng.percent(feedback_chance as u32) {
        // Find which ring caused it
        for ring in rings.get_worn_rings() {
            if ring.power.causes_feedback() {
                return Some(format!("Your {} overheats!", ring.power.name()));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_power_drain() {
        assert_eq!(RingPower::Strength.power_drain(), 1);
        assert_eq!(RingPower::Speed.power_drain(), 4);
        assert_eq!(RingPower::Invisibility.power_drain(), 5);
    }

    #[test]
    fn test_ring_wear_left_hand() {
        let mut wear = RingWear::new();
        let ring = WornRing::new(RingPower::Strength, 1, 0);

        let result = wear.wear_ring(ring, RingHand::Left);
        assert!(result.is_ok());
        assert!(wear.left_hand.is_some());
    }

    #[test]
    fn test_ring_wear_both_hands() {
        let mut wear = RingWear::new();
        let ring1 = WornRing::new(RingPower::Strength, 1, 0);
        let ring2 = WornRing::new(RingPower::Dexterity, 2, 0);

        wear.wear_ring(ring1, RingHand::Left).unwrap();
        wear.wear_ring(ring2, RingHand::Right).unwrap();

        assert!(wear.left_hand.is_some());
        assert!(wear.right_hand.is_some());
    }

    #[test]
    fn test_ring_wear_duplicate_hand() {
        let mut wear = RingWear::new();
        let ring1 = WornRing::new(RingPower::Strength, 1, 0);
        let ring2 = WornRing::new(RingPower::Dexterity, 2, 0);

        wear.wear_ring(ring1, RingHand::Left).unwrap();
        let result = wear.wear_ring(ring2, RingHand::Left);

        assert!(result.is_err());
    }

    #[test]
    fn test_remove_ring() {
        let mut wear = RingWear::new();
        let ring = WornRing::new(RingPower::Strength, 1, 0);

        wear.wear_ring(ring, RingHand::Left).unwrap();
        let removed = wear.remove_ring(RingHand::Left);

        assert!(removed.is_some());
        assert!(wear.left_hand.is_none());
    }

    #[test]
    fn test_has_ring_power() {
        let mut wear = RingWear::new();
        let ring = WornRing::new(RingPower::FireResistance, 1, 0);

        wear.wear_ring(ring, RingHand::Left).unwrap();
        assert!(wear.has_ring_power(RingPower::FireResistance));
        assert!(!wear.has_ring_power(RingPower::Speed));
    }

    #[test]
    fn test_get_worn_rings() {
        let mut wear = RingWear::new();
        let ring1 = WornRing::new(RingPower::Strength, 1, 0);
        let ring2 = WornRing::new(RingPower::Dexterity, 2, 0);

        wear.wear_ring(ring1, RingHand::Left).unwrap();
        wear.wear_ring(ring2, RingHand::Right).unwrap();

        let rings = wear.get_worn_rings();
        assert_eq!(rings.len(), 2);
    }

    #[test]
    fn test_calculate_ring_drain() {
        let mut wear = RingWear::new();
        let ring = WornRing::new(RingPower::Speed, 1, 2); // base 4 + 2 enchantment

        wear.wear_ring(ring, RingHand::Left).unwrap();
        let drain = calculate_ring_drain(&wear);

        assert_eq!(drain, 6);
    }

    #[test]
    fn test_calculate_ring_drain_multiple() {
        let mut wear = RingWear::new();
        let ring1 = WornRing::new(RingPower::Strength, 1, 0); // drain 1
        let ring2 = WornRing::new(RingPower::Speed, 2, 0); // drain 4

        wear.wear_ring(ring1, RingHand::Left).unwrap();
        wear.wear_ring(ring2, RingHand::Right).unwrap();

        let drain = calculate_ring_drain(&wear);
        assert_eq!(drain, 5);
    }

    #[test]
    fn test_apply_ring_effects_strength() {
        let mut player = You::default();
        let initial_str = player.attr_current.get(Attribute::Strength);

        let mut wear = RingWear::new();
        let ring = WornRing::new(RingPower::Strength, 1, 0);
        wear.wear_ring(ring, RingHand::Left).unwrap();

        apply_ring_effects(&mut player, &wear);

        let new_str = player.attr_current.get(Attribute::Strength);
        assert!(new_str > initial_str);
    }

    #[test]
    fn test_ring_power_name() {
        assert_eq!(RingPower::Strength.name(), "ring of strength");
        assert_eq!(RingPower::Speed.name(), "ring of speed");
    }

    #[test]
    fn test_ring_causes_feedback() {
        assert!(!RingPower::Strength.causes_feedback());
        assert!(RingPower::Speed.causes_feedback());
        assert!(RingPower::Invisibility.causes_feedback());
    }
}
