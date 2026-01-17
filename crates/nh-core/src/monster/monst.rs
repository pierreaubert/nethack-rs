//! Monster instances (monst.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::combat::AttackSet;
use crate::object::Object;

/// Unique identifier for monster instances
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MonsterId(pub u32);

impl MonsterId {
    pub const NONE: MonsterId = MonsterId(0);

    pub fn next(self) -> Self {
        MonsterId(self.0 + 1)
    }
}

/// Monster speed state
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum SpeedState {
    Slow = 0,
    #[default]
    Normal = 1,
    Fast = 2,
}

/// Monster behavior state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct MonsterState {
    /// Peaceful toward player
    pub peaceful: bool,

    /// Tame (pet)
    pub tame: bool,

    /// Currently sleeping
    pub sleeping: bool,

    /// Fleeing
    pub fleeing: bool,

    /// Confused
    pub confused: bool,

    /// Stunned
    pub stunned: bool,

    /// Blinded
    pub blinded: bool,

    /// Paralyzed/frozen
    pub paralyzed: bool,

    /// Can currently move
    pub can_move: bool,

    /// Invisible
    pub invisible: bool,

    /// Hiding/undetected
    pub hiding: bool,

    /// Cancelled (magic suppressed)
    pub cancelled: bool,

    /// Slowed
    pub slowed: bool,

    /// Hasted
    pub hasted: bool,
}

impl MonsterState {
    /// Create default state (active, hostile)
    pub fn active() -> Self {
        Self {
            can_move: true,
            ..Default::default()
        }
    }

    /// Create peaceful state
    pub fn peaceful() -> Self {
        Self {
            peaceful: true,
            can_move: true,
            ..Default::default()
        }
    }

    /// Create tame state
    pub fn tame() -> Self {
        Self {
            peaceful: true,
            tame: true,
            can_move: true,
            ..Default::default()
        }
    }
}

/// Monster AI strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Strategy {
    bits: u32,
}

impl Strategy {
    pub const NONE: u32 = 0x00000000;
    pub const ARRIVE: u32 = 0x40000000;
    pub const WAIT: u32 = 0x20000000;
    pub const CLOSE: u32 = 0x10000000;
    pub const HEAL: u32 = 0x08000000;
    pub const GROUND: u32 = 0x04000000;
    pub const PLAYER: u32 = 0x01000000;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn wants_player(&self) -> bool {
        self.bits & Self::PLAYER != 0
    }

    pub fn wants_ground(&self) -> bool {
        self.bits & Self::GROUND != 0
    }

    pub fn should_heal(&self) -> bool {
        self.bits & Self::HEAL != 0
    }

    /// Get target x coordinate (encoded in bits 16-23)
    pub fn goal_x(&self) -> i8 {
        ((self.bits >> 16) & 0xFF) as i8
    }

    /// Get target y coordinate (encoded in bits 8-15)
    pub fn goal_y(&self) -> i8 {
        ((self.bits >> 8) & 0xFF) as i8
    }

    /// Set goal coordinates
    pub fn set_goal(&mut self, x: i8, y: i8) {
        self.bits = (self.bits & 0xFF0000FF)
            | ((x as u32 & 0xFF) << 16)
            | ((y as u32 & 0xFF) << 8);
    }
}

/// Monster instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monster {
    /// Unique identifier
    pub id: MonsterId,

    /// Monster type index (into PerMonst array)
    pub monster_type: i16,

    /// Original monster type (for shapeshifters)
    pub original_type: i16,

    /// Monster name (for display)
    pub name: String,

    /// Attacks
    pub attacks: AttackSet,

    /// Position
    pub x: i8,
    pub y: i8,

    /// Believed player position (for AI)
    pub player_x: i8,
    pub player_y: i8,

    /// Movement points
    pub movement: i16,

    /// Adjusted level
    pub level: u8,

    /// Current alignment
    pub alignment: i8,

    /// Hit points
    pub hp: i32,
    pub hp_max: i32,

    /// Behavior state
    pub state: MonsterState,

    /// Speed modifier
    pub speed: SpeedState,
    pub permanent_speed: SpeedState,

    /// AI strategy
    pub strategy: Strategy,

    /// Special ability cooldown
    pub special_cooldown: i32,

    /// Tameness (0 = not tame, higher = more tame)
    pub tameness: i8,

    /// Flee timeout
    pub flee_timeout: u16,

    /// Blinded timeout
    pub blinded_timeout: u16,

    /// Frozen timeout
    pub frozen_timeout: u16,

    /// Confused timeout
    pub confused_timeout: u16,

    /// Sleep timeout
    pub sleep_timeout: u16,

    /// Inventory
    pub inventory: Vec<Object>,

    /// Wielded weapon index in inventory
    pub wielded: Option<usize>,

    /// Worn items bitmask
    pub worn_mask: u32,

    /// Traps seen (bitmask)
    pub traps_seen: u32,

    /// Special flags
    pub is_shopkeeper: bool,
    pub is_priest: bool,
    pub is_guard: bool,
    pub is_minion: bool,

    /// Female flag
    pub female: bool,
}

impl Monster {
    /// Create a new monster of the given type
    pub fn new(id: MonsterId, monster_type: i16, x: i8, y: i8) -> Self {
        Self {
            id,
            monster_type,
            original_type: monster_type,
            name: "monster".to_string(),
            attacks: crate::combat::empty_attacks(),
            x,
            y,
            player_x: 0,
            player_y: 0,
            movement: 0,
            level: 0,
            alignment: 0,
            hp: 1,
            hp_max: 1,
            state: MonsterState::active(),
            speed: SpeedState::Normal,
            permanent_speed: SpeedState::Normal,
            strategy: Strategy::default(),
            special_cooldown: 0,
            tameness: 0,
            flee_timeout: 0,
            blinded_timeout: 0,
            frozen_timeout: 0,
            confused_timeout: 0,
            sleep_timeout: 0,
            inventory: Vec::new(),
            wielded: None,
            worn_mask: 0,
            traps_seen: 0,
            is_shopkeeper: false,
            is_priest: false,
            is_guard: false,
            is_minion: false,
            female: false,
        }
    }

    /// Check if monster is dead
    pub const fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// Check if monster can move this turn
    pub fn can_act(&self) -> bool {
        self.state.can_move
            && !self.state.paralyzed
            && !self.state.sleeping
            && self.frozen_timeout == 0
    }

    /// Check if monster is a pet
    pub const fn is_pet(&self) -> bool {
        self.state.tame
    }

    /// Check if monster is peaceful
    pub const fn is_peaceful(&self) -> bool {
        self.state.peaceful
    }

    /// Check if monster is hostile
    pub fn is_hostile(&self) -> bool {
        !self.state.peaceful
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: i32) {
        self.hp -= damage;
    }

    /// Get distance squared to a position
    pub fn distance_sq(&self, x: i8, y: i8) -> i32 {
        let dx = (self.x - x) as i32;
        let dy = (self.y - y) as i32;
        dx * dx + dy * dy
    }

    /// Check if adjacent to a position
    pub fn is_adjacent(&self, x: i8, y: i8) -> bool {
        let dx = (self.x - x).abs();
        let dy = (self.y - y).abs();
        dx <= 1 && dy <= 1 && (dx > 0 || dy > 0)
    }
}
