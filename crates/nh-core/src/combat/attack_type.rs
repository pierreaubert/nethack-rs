//! Attack types from monattk.h
//!
//! These define HOW an attack is delivered.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Attack type - how the attack is delivered (AT_* from monattk.h)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum AttackType {
    /// No attack (AT_NONE)
    #[default]
    None = 0,

    /// Claw attack (AT_CLAW)
    Claw = 1,

    /// Bite attack (AT_BITE)
    Bite = 2,

    /// Kick attack (AT_KICK)
    Kick = 3,

    /// Head butt attack (AT_BUTT)
    Butt = 4,

    /// Touch attack (AT_TUCH)
    Touch = 5,

    /// Sting attack (AT_STNG)
    Sting = 6,

    /// Crushing hug (AT_HUGS)
    Hug = 7,

    // 8-9 unused
    /// Spit attack (AT_SPIT)
    Spit = 10,

    /// Engulf attack (AT_ENGL)
    Engulf = 11,

    /// Breath weapon (AT_BREA)
    Breath = 12,

    /// Explode on contact (AT_EXPL)
    Explode = 13,

    /// Explode when killed (AT_BOOM)
    ExplodeOnDeath = 14,

    /// Gaze attack (AT_GAZE)
    Gaze = 15,

    /// Tentacle attack (AT_TENT)
    Tentacle = 16,

    // 17-253 unused
    /// Weapon attack (AT_WEAP)
    Weapon = 254,

    /// Magic spell attack (AT_MAGC)
    Magic = 255,
}

impl AttackType {
    /// Check if this is a ranged attack type
    pub const fn is_ranged(&self) -> bool {
        matches!(
            self,
            AttackType::Spit | AttackType::Breath | AttackType::Gaze | AttackType::Magic
        )
    }

    /// Check if this is a melee attack type
    pub const fn is_melee(&self) -> bool {
        matches!(
            self,
            AttackType::Claw
                | AttackType::Bite
                | AttackType::Kick
                | AttackType::Butt
                | AttackType::Touch
                | AttackType::Sting
                | AttackType::Hug
                | AttackType::Tentacle
                | AttackType::Weapon
        )
    }

    /// Check if this attack requires adjacency
    pub const fn requires_adjacency(&self) -> bool {
        !self.is_ranged() && !matches!(self, AttackType::Engulf)
    }

    /// Check if this is a passive attack (triggers when attacked)
    pub const fn is_passive(&self) -> bool {
        matches!(self, AttackType::Explode | AttackType::ExplodeOnDeath)
    }
}
