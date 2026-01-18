//! Combat system
//!
//! Implements player-vs-monster, monster-vs-player, and monster-vs-monster combat.

mod attack_type;
mod damage_type;
mod mhitm;
mod mhitu;
mod uhitm;

pub use attack_type::AttackType;
pub use damage_type::DamageType;
pub use mhitu::{
    mattacku, monster_attack_player, MonsterAttackResult,
    hit_message, miss_message, wild_miss_message, damage_effect_message, resistance_message,
    try_escape_grab, apply_grab_damage,
};
pub use uhitm::player_attack_monster;

use crate::NATTK;
use serde::{Deserialize, Serialize};

/// A single attack definition (from struct attk in monattk.h)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attack {
    /// How the attack is delivered
    pub attack_type: AttackType,
    /// What kind of damage is dealt
    pub damage_type: DamageType,
    /// Number of damage dice
    pub dice_num: u8,
    /// Sides per damage die
    pub dice_sides: u8,
}

impl Attack {
    /// Create a new attack
    pub const fn new(
        attack_type: AttackType,
        damage_type: DamageType,
        dice_num: u8,
        dice_sides: u8,
    ) -> Self {
        Self {
            attack_type,
            damage_type,
            dice_num,
            dice_sides,
        }
    }

    /// Check if this is a valid/active attack
    pub const fn is_active(&self) -> bool {
        !matches!(self.attack_type, AttackType::None)
    }

    /// Get the average damage for this attack
    pub fn average_damage(&self) -> f32 {
        if self.dice_sides == 0 {
            return 0.0;
        }
        self.dice_num as f32 * (self.dice_sides as f32 + 1.0) / 2.0
    }
}

/// Attack set for a monster (6 attacks max)
pub type AttackSet = [Attack; NATTK];

/// Create an empty attack set
pub const fn empty_attacks() -> AttackSet {
    [Attack::new(AttackType::None, DamageType::Physical, 0, 0); NATTK]
}

/// Result of a combat action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatResult {
    /// Whether the attack connected
    pub hit: bool,
    /// Whether the defender died
    pub defender_died: bool,
    /// Whether the attacker died (e.g., from cockatrice corpse)
    pub attacker_died: bool,
    /// Damage dealt (before resistances)
    pub damage: i32,
    /// Special effect triggered
    pub special_effect: Option<CombatEffect>,
}

impl CombatResult {
    pub const MISS: Self = Self {
        hit: false,
        defender_died: false,
        attacker_died: false,
        damage: 0,
        special_effect: None,
    };

    pub const fn hit(damage: i32) -> Self {
        Self {
            hit: true,
            defender_died: false,
            attacker_died: false,
            damage,
            special_effect: None,
        }
    }
}

/// Special effects that can occur during combat
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatEffect {
    Poisoned,
    Paralyzed,
    Slowed,
    Stunned,
    Confused,
    Blinded,
    Drained,
    Diseased,
    Petrifying,
    Teleported,
    ItemStolen,
    GoldStolen,
    Engulfed,
    Grabbed,
    ItemDestroyed,
    ArmorCorroded,
}
