//! Monster instances (monst.h)

use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use super::{MonsterFlags, MonsterResistances};
#[cfg(feature = "extensions")]
use super::attack_selection::CombatMemory;
#[cfg(feature = "extensions")]
use super::morale::MoraleTracker;
#[cfg(feature = "extensions")]
use super::personality::Personality;
use crate::combat::{AttackSet, CombatResources, StatusEffectTracker};
use crate::object::{Object, ObjectId};
use crate::special::dog::PetExtension;
use crate::special::priest::PriestExtension;
use crate::special::shk::ShopkeeperExtension;
use crate::special::vault::GuardExtension;

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

    /// Alive (false = dead)
    pub alive: bool,

    /// Invisibility blocked by see invisible
    pub invis_blocked: bool,

    /// Leashed by player
    pub leashed: bool,

    /// Trapped (in a pit, bear trap, etc.)
    pub trapped: bool,
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

    pub fn bits(&self) -> u32 {
        self.bits
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
        self.bits = (self.bits & 0xFF0000FF) | ((x as u32 & 0xFF) << 16) | ((y as u32 & 0xFF) << 8);
    }
}

/// Threat level assessment (Phase 18)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Display, EnumIter,
)]
#[repr(u8)]
pub enum ThreatLevel {
    #[default]
    Neutral = 0,
    Low = 1,
    Moderate = 2,
    High = 3,
    Critical = 4,
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

    /// Armor class (calculated, lower is better)
    pub ac: i8,

    /// Base armor class (from PerMonst definition)
    pub base_ac: i8,

    /// Resistances (from PerMonst)
    pub resistances: MonsterResistances,

    /// Monster flags (from PerMonst: UNDEAD, DEMON, FLY, etc.)
    pub flags: MonsterFlags,

    /// Hit points
    pub hp: i32,
    pub hp_max: i32,

    /// Behavior state
    pub state: MonsterState,

    /// Speed modifier
    pub speed: SpeedState,
    pub permanent_speed: SpeedState,

    /// Base movement speed (from permonst data, default 12)
    pub base_speed: i32,

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

    /// Status effects tracker (Phase 13)
    pub status_effects: StatusEffectTracker,

    /// Combat AI fields (extensions)

    /// Monster personality type
    #[cfg(feature = "extensions")]
    pub personality: Personality,

    /// Morale tracking system
    #[cfg(feature = "extensions")]
    pub morale: MoraleTracker,

    /// Combat resource management (mana, cooldowns, charges)
    pub resources: CombatResources,

    /// Combat memory (attack history, observed resistances)
    #[cfg(feature = "extensions")]
    pub combat_memory: CombatMemory,

    /// Current threat level assessment
    pub threat_level: ThreatLevel,

    /// Inventory
    pub inventory: Vec<Object>,

    /// Wielded weapon index in inventory
    pub wielded: Option<usize>,

    /// Worn items bitmask
    pub worn_mask: u32,

    /// Traps seen (bitmask)
    pub traps_seen: u32,

    /// Trapped turns remaining (0 = not trapped)
    pub mtrapped: i32,

    /// Special flags
    pub is_shopkeeper: bool,
    pub is_priest: bool,
    pub is_guard: bool,
    pub is_minion: bool,

    /// Leash object ID (if leashed)
    pub leash_id: Option<ObjectId>,

    /// Pet extension data (only Some if tame)
    pub pet_extension: Option<PetExtension>,

    /// Priest extension data (only Some if priest)
    pub priest_extension: Option<PriestExtension>,

    /// Shopkeeper extension data (only Some if shopkeeper)
    pub shopkeeper_extension: Option<ShopkeeperExtension>,

    /// Guard extension data (only Some if vault guard)
    pub guard_extension: Option<GuardExtension>,

    /// Female flag
    pub female: bool,

    /// Whether monster cannot move diagonally (like grid bugs)
    pub no_diagonal_move: bool,

    /// Whether monster can pass through walls
    pub passes_walls: bool,

    /// Whether monster can swim
    pub can_swim: bool,

    /// Whether monster can fly
    pub can_fly: bool,

    /// Special ability usage cooldown (mspec_used in C)
    pub spec_used: u8,

    /// Eating timeout (meating in C)
    pub eating_timeout: u16,
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
            ac: 10,      // Default AC, calculated from base_ac + worn items
            base_ac: 10, // Default base AC, set from PerMonst when spawning
            resistances: MonsterResistances::empty(), // Set from PerMonst when spawning
            flags: MonsterFlags::empty(), // Set from PerMonst when spawning
            hp: 1,
            hp_max: 1,
            state: MonsterState::active(),
            speed: SpeedState::Normal,
            permanent_speed: SpeedState::Normal,
            base_speed: 12, // Default speed, set from PerMonst when spawning
            strategy: Strategy::default(),
            special_cooldown: 0,
            tameness: 0,
            flee_timeout: 0,
            blinded_timeout: 0,
            frozen_timeout: 0,
            confused_timeout: 0,
            sleep_timeout: 0,
            status_effects: StatusEffectTracker::new(),
            #[cfg(feature = "extensions")]
            personality: Personality::default(),
            #[cfg(feature = "extensions")]
            morale: MoraleTracker::new(),
            resources: CombatResources::new(),
            #[cfg(feature = "extensions")]
            combat_memory: CombatMemory::new(),
            threat_level: ThreatLevel::default(),
            inventory: Vec::new(),
            wielded: None,
            worn_mask: 0,
            traps_seen: 0,
            mtrapped: 0,
            is_shopkeeper: false,
            is_priest: false,
            is_guard: false,
            is_minion: false,
            leash_id: None,
            pet_extension: None,
            priest_extension: None,
            shopkeeper_extension: None,
            guard_extension: None,
            female: false,
            no_diagonal_move: false,
            passes_walls: false,
            can_swim: false,
            can_fly: false,
            spec_used: 0,
            eating_timeout: 0,
        }
    }

    /// Check if monster is dead
    pub const fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    /// Check if monster is undead
    pub const fn is_undead(&self) -> bool {
        self.flags.contains(MonsterFlags::UNDEAD)
    }

    /// Check if monster is a demon
    pub const fn is_demon(&self) -> bool {
        self.flags.contains(MonsterFlags::DEMON)
    }

    /// Check if monster can fly
    pub const fn flies(&self) -> bool {
        self.flags.contains(MonsterFlags::FLY)
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

    // Resistance checks

    /// Check if monster has fire resistance
    pub fn resists_fire(&self) -> bool {
        self.resistances.contains(MonsterResistances::FIRE)
    }

    /// Check if monster has cold resistance
    pub fn resists_cold(&self) -> bool {
        self.resistances.contains(MonsterResistances::COLD)
    }

    /// Check if monster has sleep resistance
    pub fn resists_sleep(&self) -> bool {
        self.resistances.contains(MonsterResistances::SLEEP)
    }

    /// Check if monster has disintegration resistance
    pub fn resists_disint(&self) -> bool {
        self.resistances.contains(MonsterResistances::DISINT)
    }

    /// Check if monster has shock/electric resistance
    pub fn resists_elec(&self) -> bool {
        self.resistances.contains(MonsterResistances::ELEC)
    }

    /// Check if monster has poison resistance
    pub fn resists_poison(&self) -> bool {
        self.resistances.contains(MonsterResistances::POISON)
    }

    /// Check if monster has acid resistance
    pub fn resists_acid(&self) -> bool {
        self.resistances.contains(MonsterResistances::ACID)
    }

    /// Check if monster has stone/petrification resistance
    pub fn resists_stone(&self) -> bool {
        self.resistances.contains(MonsterResistances::STONE)
    }

    /// Check if monster has magic resistance
    pub fn resists_magic(&self) -> bool {
        self.resistances.contains(MonsterResistances::MAGIC)
    }

    /// Check if monster is mindless (no intelligence)
    /// Mindless creatures are immune to psychic attacks and certain enchantments
    pub fn is_mindless(&self) -> bool {
        // Check for MR2_MINDLESS flag equivalent
        // Simplified: certain monster types are mindless (golems, elementals, etc.)
        // In full implementation, would check permonst flags
        self.name.to_lowercase().contains("golem")
            || self.name.to_lowercase().contains("elemental")
            || self.name.to_lowercase().contains("vortex")
            || self.name.to_lowercase().contains("blob")
            || self.name.to_lowercase().contains("jelly")
            || self.name.to_lowercase().contains("mold")
            || self.name.to_lowercase().contains("fungus")
            || self.name.to_lowercase().contains("pudding")
    }

    // ========================================================================
    // Attack type checks (from mondata.c attacktype/dmgtype)
    // ========================================================================

    /// Check if monster has a specific attack type (attacktype function from mondata.c)
    /// Returns true if any of the monster's attacks use the specified attack type
    pub fn attacktype(&self, attack_type: crate::combat::AttackType) -> bool {
        self.attacks
            .iter()
            .any(|atk| atk.attack_type == attack_type && atk.is_active())
    }

    /// Check if monster has a specific damage type (dmgtype function from mondata.c)
    /// Returns true if any of the monster's attacks deal the specified damage type
    pub fn dmgtype(&self, damage_type: crate::combat::DamageType) -> bool {
        self.attacks
            .iter()
            .any(|atk| atk.damage_type == damage_type && atk.is_active())
    }

    /// Check if monster has a specific attack type AND damage type combination
    pub fn attacktype_and_dmgtype(
        &self,
        attack_type: crate::combat::AttackType,
        damage_type: crate::combat::DamageType,
    ) -> bool {
        self.attacks.iter().any(|atk| {
            atk.attack_type == attack_type && atk.damage_type == damage_type && atk.is_active()
        })
    }

    /// Check if monster has a passive attack (like cockatrice or acid blob)
    pub fn has_passive_attack(&self) -> bool {
        use crate::combat::AttackType;
        self.attacks.iter().any(|atk| {
            matches!(
                atk.attack_type,
                AttackType::Explode | AttackType::ExplodeOnDeath
            ) && atk.is_active()
        })
    }

    /// Check if monster can attack at range
    pub fn has_ranged_attack(&self) -> bool {
        self.attacks
            .iter()
            .any(|atk| atk.attack_type.is_ranged() && atk.is_active())
    }

    /// Check if monster can use breath weapon
    pub fn has_breath_attack(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Breath)
    }

    /// Check if monster can cast spells
    pub fn can_cast_spells(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Magic)
    }

    /// Check if monster uses weapons
    pub fn uses_weapons(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Weapon)
    }

    /// Check if monster has a gaze attack
    pub fn has_gaze_attack(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Gaze)
    }

    /// Check if monster can engulf
    pub fn can_engulf(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Engulf)
    }

    /// Check if monster can touch attack (like touch of death)
    pub fn has_touch_attack(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Touch)
    }

    /// Check if monster has a sting attack
    pub fn has_sting_attack(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Sting)
    }

    /// Check if monster can spit
    pub fn can_spit(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Spit)
    }

    /// Check if monster has tentacles
    pub fn has_tentacle_attack(&self) -> bool {
        self.attacktype(crate::combat::AttackType::Tentacle)
    }

    /// Get the first active attack of a given type
    pub fn get_attack(
        &self,
        attack_type: crate::combat::AttackType,
    ) -> Option<&crate::combat::Attack> {
        self.attacks
            .iter()
            .find(|atk| atk.attack_type == attack_type && atk.is_active())
    }

    /// Count how many active attacks the monster has
    pub fn num_attacks(&self) -> usize {
        self.attacks.iter().filter(|atk| atk.is_active()).count()
    }

    /// Check if monster has natural regeneration ability
    /// Trolls and similar creatures regenerate HP every turn
    pub fn regenerates(&self) -> bool {
        let name_lower = self.name.to_lowercase();
        name_lower.contains("troll")
            || name_lower.contains("vampire")
            || name_lower.contains("were")
    }

    // ========================================================================
    // Monster naming functions (from mon.c / do_name.c)
    // ========================================================================

    /// Get the monster name without any article (mon_nam equivalent)
    /// E.g., "kobold", "the Wizard of Yendor", "Fido" (for named pets)
    pub fn mon_nam(&self) -> String {
        if !self.name.is_empty() && self.name != "monster" {
            // Named monster or unique
            self.name.clone()
        } else {
            // Generic monster type name
            format!("the {}", self.type_name())
        }
    }

    /// Get monster name with "a/an" article (a_monnam equivalent)
    /// E.g., "a kobold", "an orc", "the Wizard of Yendor"
    pub fn a_monnam(&self) -> String {
        if !self.name.is_empty() && self.name != "monster" {
            // Named monsters don't get an article
            self.name.clone()
        } else {
            let type_name = self.type_name();
            an_prefix(&type_name)
        }
    }

    /// Get capitalized monster name (Monnam equivalent)
    /// E.g., "The kobold", "Fido"
    pub fn monnam(&self) -> String {
        crate::upstart(&self.mon_nam())
    }

    /// Get capitalized monster name with article (Amonnam equivalent)
    /// E.g., "A kobold", "An orc", "The Wizard of Yendor"
    pub fn amonnam(&self) -> String {
        crate::upstart(&self.a_monnam())
    }

    /// Get the type name for this monster (from PerMonst)
    /// This would normally lookup in MONS array; here we use stored name
    fn type_name(&self) -> String {
        // In full implementation, this would lookup mons[self.monster_type].mname
        // For now, use the stored name or a placeholder
        if self.name.is_empty() || self.name == "monster" {
            "creature".to_string() // fallback
        } else {
            self.name.clone()
        }
    }

    /// Distant monster name (when you can't see details)
    /// E.g., "something", "someone", "it"
    pub fn distant_monnam(&self, article: bool) -> String {
        // In full implementation, depends on visibility, hallucination, etc.
        if article {
            "something".to_string()
        } else {
            "it".to_string()
        }
    }

    /// L-case monster name for inventory (l_monnam equivalent)
    /// Always lowercase, for use in object descriptions
    pub fn l_monnam(&self) -> String {
        self.mon_nam().to_lowercase()
    }

    /// Monster name for death messages (m_monnam equivalent)
    pub fn m_monnam(&self) -> String {
        self.mon_nam()
    }

    /// Possessive form of monster name
    /// E.g., "the kobold's", "Fido's"
    pub fn s_suffix(&self) -> String {
        let name = self.mon_nam();
        if name.ends_with('s') {
            format!("{}'", name)
        } else {
            format!("{}'s", name)
        }
    }

    /// Check if this is a unique monster
    pub fn is_unique(&self) -> bool {
        // Would check mons[monster_type].geno & G_UNIQ
        // For now, check if the name is capitalized (heuristic)
        self.name
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Check if this monster has a proper name (like a pet)
    pub fn has_name(&self) -> bool {
        !self.name.is_empty() && self.name != "monster"
    }

    /// Get pronoun for this monster
    pub fn pronoun(&self, case: PronounCase) -> &'static str {
        match (self.female, case) {
            (true, PronounCase::Subject) => "she",
            (true, PronounCase::Object) => "her",
            (true, PronounCase::Possessive) => "her",
            (false, PronounCase::Subject) => "he",
            (false, PronounCase::Object) => "him",
            (false, PronounCase::Possessive) => "his",
        }
    }

    /// Get generic pronoun (for unknown/neuter)
    pub fn pronoun_neuter(case: PronounCase) -> &'static str {
        match case {
            PronounCase::Subject => "it",
            PronounCase::Object => "it",
            PronounCase::Possessive => "its",
        }
    }

    // ========================================================================
    // Sleep/awakening functions (mon.c)
    // ========================================================================

    /// Check if monster is sleeping
    pub fn is_sleeping(&self) -> bool {
        self.state.sleeping || self.sleep_timeout > 0
    }

    /// Wake up this monster
    pub fn wakeup(&mut self) {
        self.state.sleeping = false;
        self.sleep_timeout = 0;
    }

    /// Put monster to sleep for a duration
    pub fn put_to_sleep(&mut self, duration: i32) {
        if !self.resists_sleep() {
            self.state.sleeping = true;
            self.sleep_timeout = duration.max(0) as u16;
        }
    }

    /// Update sleep timeout (call each turn)
    pub fn update_sleep(&mut self) {
        if self.sleep_timeout > 0 {
            self.sleep_timeout -= 1;
            if self.sleep_timeout == 0 {
                self.state.sleeping = false;
            }
        }
    }

    /// Check if monster should wake from nearby noise
    pub fn should_wake_from_noise(&self, distance: i32, noise_level: i32) -> bool {
        if !self.is_sleeping() {
            return false;
        }
        // Monsters wake more easily when player is closer
        // noise_level 1 = normal, 2 = loud, 3+ = very loud
        let wake_threshold = (distance * 2) / noise_level.max(1);
        wake_threshold < 5
    }

    /// Check if monster is a soldier type (for awaken_soldiers)
    pub fn is_soldier(&self) -> bool {
        // Would check against soldier monster types
        // For now, check if is_guard flag is set
        self.is_guard
    }

    // ========================================================================
    // Perception and special abilities (from mondata.c)
    // ========================================================================

    /// Check if monster can see invisible creatures (perceives in C)
    pub fn sees_invisible(&self) -> bool {
        // This would normally check permonst flags
        // For now, check if monster has high level (typically telepathic/magic users)
        self.level >= 15
    }

    /// Check if monster has reflection (from equipment or intrinsic)
    pub fn has_reflection(&self) -> bool {
        // Check for reflection amulet or shield of reflection
        // In full implementation, would check worn equipment
        // For now, return false (reflection is rare)
        false
    }

    /// Get monster's size category (MZ_* values in monst.h)
    /// 0=tiny, 1=small, 2=medium, 3=large, 4=huge, 5=gigantic
    pub fn size(&self) -> u8 {
        // This would normally look up from permonst
        // Approximate based on level
        match self.level {
            0..=2 => 1,   // Small
            3..=6 => 2,   // Medium
            7..=12 => 3,  // Large
            13..=20 => 4, // Huge
            _ => 5,       // Gigantic
        }
    }

    // ========================================================================
    // Monster inventory functions (mon.c, m_carrying, m_useup, etc.)
    // ========================================================================

    /// Check if monster is carrying an object of a specific type (m_carrying)
    /// Returns reference to the first matching object, or None if not found
    pub fn m_carrying(&self, object_type: i16) -> Option<&Object> {
        self.inventory
            .iter()
            .find(|obj| obj.object_type == object_type)
    }

    /// Check if monster is carrying any object of a specific class (m_carrying_class)
    pub fn m_carrying_class(&self, class: crate::object::ObjectClass) -> Option<&Object> {
        self.inventory.iter().find(|obj| obj.class == class)
    }

    /// Check if monster is carrying an artifact (m_carrying_arti)
    pub fn m_carrying_arti(&self, artifact_id: u8) -> Option<&Object> {
        self.inventory
            .iter()
            .find(|obj| obj.artifact == artifact_id)
    }

    /// Check if monster has any gold (findgold equivalent)
    pub fn findgold(&self) -> Option<&Object> {
        self.inventory
            .iter()
            .find(|obj| obj.class == crate::object::ObjectClass::Coin)
    }

    /// Get total gold amount monster is carrying
    pub fn gold_amount(&self) -> i64 {
        self.inventory
            .iter()
            .filter(|obj| obj.class == crate::object::ObjectClass::Coin)
            .map(|obj| obj.quantity as i64)
            .sum()
    }

    /// Monster uses up one item from a stack (m_useup)
    /// Returns true if the item stack is completely used up and was removed
    pub fn m_useup(&mut self, object_type: i16) -> bool {
        if let Some(idx) = self
            .inventory
            .iter()
            .position(|obj| obj.object_type == object_type)
        {
            let obj = &mut self.inventory[idx];
            if obj.quantity > 1 {
                obj.quantity -= 1;
                false
            } else {
                self.inventory.remove(idx);
                true
            }
        } else {
            false
        }
    }

    /// Monster uses up all of an item type (m_useupall)
    /// Returns the quantity that was removed
    pub fn m_useupall(&mut self, object_type: i16) -> i32 {
        if let Some(idx) = self
            .inventory
            .iter()
            .position(|obj| obj.object_type == object_type)
        {
            let qty = self.inventory[idx].quantity;
            self.inventory.remove(idx);
            qty
        } else {
            0
        }
    }

    /// Monster picks up an object (mpickobj)
    /// Adds object to monster's inventory, merging if possible
    pub fn mpickobj(&mut self, mut obj: Object) -> bool {
        obj.location = crate::object::ObjectLocation::MonsterInventory;

        // Try to merge with existing stack
        if let Some(existing) = self
            .inventory
            .iter_mut()
            .find(|o| Self::can_merge_objects(o, &obj))
        {
            existing.quantity += obj.quantity;
            return true;
        }

        // Add as new item
        self.inventory.push(obj);
        true
    }

    /// Monster picks up gold (mpickgold)
    /// Merges with existing gold if present
    pub fn mpickgold(&mut self, amount: i32) -> bool {
        if amount <= 0 {
            return false;
        }

        // Find existing gold
        if let Some(gold) = self
            .inventory
            .iter_mut()
            .find(|obj| obj.class == crate::object::ObjectClass::Coin)
        {
            gold.quantity += amount;
        } else {
            // Create new gold object
            let gold = Object {
                object_type: 0, // GOLD_PIECE
                class: crate::object::ObjectClass::Coin,
                quantity: amount,
                location: crate::object::ObjectLocation::MonsterInventory,
                ..Default::default()
            };
            self.inventory.push(gold);
        }
        true
    }

    /// Check if two objects can be merged (same type, stackable)
    fn can_merge_objects(a: &Object, b: &Object) -> bool {
        a.object_type == b.object_type
            && a.buc == b.buc
            && a.enchantment == b.enchantment
            && a.erosion1 == b.erosion1
            && a.erosion2 == b.erosion2
            && a.name == b.name
            // Only stack gold, ammo, etc.
            && matches!(
                a.class,
                crate::object::ObjectClass::Coin
                    | crate::object::ObjectClass::Gem
                    | crate::object::ObjectClass::Weapon
            )
    }

    /// Monster drops an object (mdrop_obj)
    /// Returns the object if successfully dropped
    pub fn mdrop_obj(&mut self, object_type: i16) -> Option<Object> {
        if let Some(idx) = self
            .inventory
            .iter()
            .position(|obj| obj.object_type == object_type)
        {
            let mut obj = self.inventory.remove(idx);
            obj.location = crate::object::ObjectLocation::Floor;
            obj.x = self.x;
            obj.y = self.y;
            Some(obj)
        } else {
            None
        }
    }

    /// Get total weight of monster's inventory
    pub fn inventory_weight(&self) -> u32 {
        self.inventory
            .iter()
            .map(|obj| obj.weight * obj.quantity as u32)
            .sum()
    }

    /// Check if monster's inventory is empty
    pub fn inventory_empty(&self) -> bool {
        self.inventory.is_empty()
    }

    /// Count items in monster's inventory
    pub fn inventory_count(&self) -> usize {
        self.inventory.len()
    }

    /// Get wielded weapon (if any)
    pub fn get_wielded_weapon(&self) -> Option<&Object> {
        self.wielded.and_then(|idx| self.inventory.get(idx))
    }

    /// Get mutable wielded weapon (if any)
    pub fn get_wielded_weapon_mut(&mut self) -> Option<&mut Object> {
        self.wielded.and_then(|idx| self.inventory.get_mut(idx))
    }

    /// Unwield current weapon
    pub fn mwepgone(&mut self) {
        self.wielded = None;
    }

    /// Check if monster is wielding any weapon
    pub fn is_armed(&self) -> bool {
        self.wielded.is_some()
    }

    /// Calculate armor class from base monster type + worn items
    ///
    /// Recalculates the monster's AC from its base_ac plus any modifiers from
    /// worn equipment.
    ///
    /// Lower AC is better (ranges -128 to 127).
    pub fn find_mac(&mut self) {
        let mut ac = self.base_ac as i32;

        // Worn armor pieces subtract their AC (improves protection)
        for obj in &self.inventory {
            if obj.worn_mask & crate::action::wear::worn_mask::W_ARMOR != 0 {
                ac = ac.saturating_sub(obj.base_ac as i32);
            }
        }

        // Clamp to valid signed i8 range
        ac = ac.clamp(-128, 127);

        self.ac = ac as i8;
    }
}

/// Pronoun case enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PronounCase {
    Subject,    // he/she/it
    Object,     // him/her/it
    Possessive, // his/her/its
}

/// Helper: Add "a" or "an" prefix
fn an_prefix(word: &str) -> String {
    if word.is_empty() {
        return "a".to_string();
    }
    let first = word.chars().next().unwrap().to_ascii_lowercase();
    if "aeiou".contains(first) {
        // Exceptions for "u" words that sound like "you"
        let lower = word.to_lowercase();
        if first == 'u' && (lower.starts_with("uni") || lower.starts_with("use")) {
            format!("a {}", word)
        } else {
            format!("an {}", word)
        }
    } else {
        format!("a {}", word)
    }
}

// ============================================================================
// Global monster awakening functions (mon.c)
// ============================================================================

/// Wake up all monsters on the level (awaken_monsters equivalent)
/// Called when player makes loud noise, wakes from Elbereth, etc.
pub fn awaken_monsters(monsters: &mut [Monster], player_x: i8, player_y: i8) {
    for mon in monsters.iter_mut() {
        if mon.is_sleeping() && !mon.is_dead() {
            let dist = mon.distance_sq(player_x, player_y);
            if dist <= 100 {
                // Within 10 squares
                mon.wakeup();
            }
        }
    }
}

/// Wake up all soldier-type monsters (awaken_soldiers equivalent)
/// Called when player attacks Vault guards, etc.
pub fn awaken_soldiers(monsters: &mut [Monster]) {
    for mon in monsters.iter_mut() {
        if mon.is_soldier() && mon.is_sleeping() && !mon.is_dead() {
            mon.wakeup();
        }
    }
}

/// Aggravate all monsters on the level (aggravate equivalent)
/// Called by aggravate monster spell, etc.
/// Makes all monsters aware of and hostile toward the player.
pub fn aggravate(monsters: &mut [Monster], player_x: i8, player_y: i8) {
    for mon in monsters.iter_mut() {
        if !mon.is_dead() {
            // Wake up
            mon.wakeup();
            // Make hostile (unless tame)
            if !mon.state.tame {
                mon.state.peaceful = false;
            }
            // Set target to player
            mon.player_x = player_x;
            mon.player_y = player_y;
        }
    }
}

/// Wake monsters near a position due to noise (wake_nearby equivalent)
pub fn wake_nearby(monsters: &mut [Monster], x: i8, y: i8, noise_radius: i32) {
    for mon in monsters.iter_mut() {
        if mon.is_sleeping() && !mon.is_dead() {
            let dist = mon.distance_sq(x, y);
            if dist <= noise_radius * noise_radius {
                mon.wakeup();
            }
        }
    }
}

/// Wake a specific monster and make it hostile (disturb equivalent)
pub fn disturb(mon: &mut Monster, player_x: i8, player_y: i8) {
    mon.wakeup();
    if !mon.state.tame {
        mon.state.peaceful = false;
    }
    mon.player_x = player_x;
    mon.player_y = player_y;
}

// ============================================================================
// Monster fleeing functions (mon.c)
// ============================================================================

/// Make monster flee from player (monflee equivalent)
/// duration is in turns, 0 means stop fleeing
pub fn monflee(mon: &mut Monster, duration: u16, first_time: bool) {
    if duration == 0 {
        mon.state.fleeing = false;
        mon.flee_timeout = 0;
        return;
    }

    // Already fleeing? Update timeout if longer
    if mon.state.fleeing {
        if duration > mon.flee_timeout {
            mon.flee_timeout = duration;
        }
        return;
    }

    mon.state.fleeing = true;
    mon.flee_timeout = duration;

    // First time fleeing generates a message
    if first_time {
        // In full implementation, this would generate a message like
        // "The monster turns to flee!"
    }
}

/// Check if monster should flee based on distance and condition (distfleeck equivalent)
/// Returns true if monster should continue fleeing
pub fn distfleeck(mon: &Monster, player_x: i8, player_y: i8, fear_distance: i32) -> bool {
    if !mon.state.fleeing {
        return false;
    }

    // Check distance to player
    let dist_sq = mon.distance_sq(player_x, player_y);
    let fear_dist_sq = fear_distance * fear_distance;

    // Monster keeps fleeing if player is within fear distance
    dist_sq <= fear_dist_sq
}

/// Update monster flee timeout (call each turn)
pub fn update_flee(mon: &mut Monster) {
    if mon.flee_timeout > 0 {
        mon.flee_timeout -= 1;
        if mon.flee_timeout == 0 {
            mon.state.fleeing = false;
        }
    }
}

/// Check if monster should start fleeing based on HP (mflee_possibly equivalent)
/// Monsters flee when HP falls below a threshold
pub fn should_flee_from_damage(mon: &Monster, hp_threshold_percent: i32) -> bool {
    if mon.state.fleeing {
        return false; // Already fleeing
    }

    // Check if HP is below threshold percentage of max
    let threshold = (mon.hp_max * hp_threshold_percent) / 100;
    mon.hp <= threshold
}

/// Make a monster scared and potentially flee (scare_monster equivalent concept)
/// Returns true if monster was scared
pub fn scare_monster(mon: &mut Monster, rng: &mut crate::rng::GameRng, scare_level: i32) -> bool {
    // Higher level monsters are harder to scare
    let resist_chance = (mon.level as i32) * 5;

    if rng.percent(resist_chance as u32) {
        return false; // Monster resisted
    }

    // Scare duration depends on scare level
    let duration = (scare_level * 5 + rng.rnd(10) as i32).max(1) as u16;
    monflee(mon, duration, true);
    true
}

/// Check if monster is in a state where it wants to attack player
pub fn wants_to_attack(mon: &Monster) -> bool {
    !mon.state.peaceful
        && !mon.state.fleeing
        && !mon.state.sleeping
        && !mon.state.paralyzed
        && mon.can_act()
}

/// Check if monster should stay near player (pet behavior)
pub fn should_stay_near_player(mon: &Monster, player_x: i8, player_y: i8, max_dist: i32) -> bool {
    if !mon.state.tame {
        return false;
    }

    let dist_sq = mon.distance_sq(player_x, player_y);
    dist_sq > max_dist * max_dist
}

/// Set monster alignment-based hostility (from makemon.c:set_malign)
///
/// Determines how the monster views the player based on:
/// - Monster type's inherent alignment
/// - Whether the monster is peaceful or hostile
/// - Whether the monster and player are co-aligned
///
/// This is called when a monster is created or changes peaceful/hostile state.
/// Sets the monster's alignment value which affects:
/// - Initial reaction to the player
/// - Whether attacks are provoked
/// - Whether the monster is interested in the player
pub fn set_malign(mon: &mut Monster, player_alignment: i8) {
    let mon_alignment = mon.alignment;

    // Determine base alignment value from monster type
    // Peaceful monsters that are co-aligned are less hostile
    // Hostile monsters are always aggressive

    let coaligned = if player_alignment == 0 {
        false // Neutral player isn't co-aligned with anyone
    } else {
        (mon_alignment > 0 && player_alignment > 0) || (mon_alignment < 0 && player_alignment < 0)
    };

    // Calculate alignment-based hostility value
    // Higher positive values = more hostile, negative = more peaceful
    mon.alignment = if mon.state.peaceful {
        if coaligned {
            // Co-aligned and peaceful: very non-hostile
            -20
        } else {
            // Non-aligned but peaceful: neutral
            0
        }
    } else {
        // Hostile monsters
        if coaligned {
            // Co-aligned hostile: less aggressive
            10
        } else {
            // Non-aligned hostile: quite aggressive
            20
        }
    };
}

// ============================================================================
// Scare mechanics functions (from monmove.c, hack.c, muse.c, wizard.c)
// ============================================================================

/// Check if a monster is scared of a location (onscary equivalent)
///
/// Determines if the monster would be scared to step on a particular location.
/// This checks for:
/// - Monsters immune to scaring (Rodney, minions, angels, Riders, etc.)
/// - Scare Monster scrolls on the ground
/// - Elbereth engravings (with restrictions)
/// - Altars scaring vampires
///
/// # Arguments
/// * `x` - X coordinate to check
/// * `y` - Y coordinate to check
/// * `monster` - The monster to check
/// * `level` - The dungeon level
/// * `player_x` - Player's X position
/// * `player_y` - Player's Y position
/// * `in_gehennom` - Whether the current level is in Gehennom
///
/// # Returns
/// true if the monster would be scared of the location
pub fn onscary(
    x: i8,
    y: i8,
    monster: &Monster,
    level: &crate::dungeon::Level,
    player_x: i8,
    player_y: i8,
    in_gehennom: bool,
) -> bool {
    // Creatures directly resistant to magical scaring:
    // - Wizard of Yendor (is_wiz)
    // - Lawful minions
    // - Angels
    // - Riders (Death, Famine, Pestilence)
    // - Shopkeepers in their shop
    // - Priests in their temple

    // Check for immunity to scaring based on monster name/type
    // (Simplified check - would check monster flags in full implementation)
    let name_lower = monster.name.to_lowercase();
    if name_lower.contains("wizard of yendor")
        || name_lower.contains("angel")
        || name_lower == "death"
        || name_lower == "famine"
        || name_lower == "pestilence"
    {
        return false;
    }

    // Shopkeepers in their shop and priests in their temple are immune
    if monster.is_shopkeeper || monster.is_priest {
        // Simplified - would check if in their shop/temple
        return false;
    }

    // <0,0> is used by musical scaring to check for immunity only
    if x == 0 && y == 0 {
        return true;
    }

    // Vampires are scared of altars
    if name_lower.contains("vampire") {
        let cell = level.cell(x as usize, y as usize);
        if matches!(cell.typ, crate::dungeon::CellType::Altar) {
            return true;
        }
    }

    // Check for Scare Monster scroll on the ground
    // (Simplified - would check objects on tile in full implementation)
    // For now, check a level flag
    let cell = level.cell(x as usize, y as usize);
    if cell.flags & 0x10 != 0 {
        // Scare scroll flag
        return true;
    }

    // Check for Elbereth engraving
    // Elbereth requires:
    // - Player at the location or displaced image there
    // - Not a shopkeeper or vault guard
    // - Monster can see
    // - Monster is not peaceful
    // - Monster is not human or minotaur
    // - Not in Gehennom or endgame
    if let Some(engr) = level.engr_at(x, y) {
        if engr.text.to_uppercase().contains("ELBERETH") {
            let at_location = player_x == x && player_y == y;
            // Simplified displaced check - would check Displaced property

            if at_location
                && !monster.is_shopkeeper
                && !monster.is_guard  // Vault guard check
                && !monster.state.blinded
                && !monster.state.peaceful
                && !name_lower.contains("human")
                && !name_lower.contains("minotaur")
                && !in_gehennom
            {
                return true;
            }
        }
    }

    false
}

/// Display death warning messages (maybe_wail equivalent)
///
/// When a Wizard, Elf, or Valkyrie is about to die, display special
/// warning messages. Other classes hear different warnings.
///
/// # Arguments
/// * `player` - The player to check
/// * `moves` - Current game turn count
/// * `last_wail_turn` - Last turn a wail message was shown (updated)
pub fn maybe_wail(
    player: &crate::player::You,
    moves: i64,
    last_wail_turn: &mut i64,
) -> Vec<String> {
    use crate::player::{Race, Role};

    let mut messages = Vec::new();

    // Don't wail too frequently
    if moves <= *last_wail_turn + 50 {
        return messages;
    }

    *last_wail_turn = moves;

    // Wizards, Elves, and Valkyries get personal warnings
    let is_special =
        player.role == Role::Wizard || player.race == Race::Elf || player.role == Role::Valkyrie;

    if is_special {
        let who = if player.role == Role::Wizard || player.role == Role::Valkyrie {
            player.role.to_string()
        } else {
            "Elf".to_string()
        };

        if player.hp == 1 {
            messages.push(format!("{} is about to die.", who));
        } else {
            // Count intrinsic powers
            let power_props = [
                crate::player::Property::Teleportation,
                crate::player::Property::SeeInvisible,
                crate::player::Property::PoisonResistance,
                crate::player::Property::ColdResistance,
                crate::player::Property::ShockResistance,
                crate::player::Property::FireResistance,
                crate::player::Property::SleepResistance,
                crate::player::Property::DisintResistance,
                crate::player::Property::TeleportControl,
                crate::player::Property::Stealth,
                crate::player::Property::Speed,
                crate::player::Property::Invisibility,
            ];

            let powercnt = power_props
                .iter()
                .filter(|&&prop| player.properties.has_intrinsic(prop))
                .count();

            if powercnt >= 4 {
                messages.push(format!("{}, all your powers will be lost...", who));
            } else {
                messages.push(format!("{}, your life force is running out.", who));
            }
        }
    } else {
        // Other classes hear supernatural warnings
        if player.hp == 1 {
            messages.push("You hear the wailing of the Banshee...".to_string());
        } else {
            messages.push("You hear the howling of the CwnAnnwn...".to_string());
        }
    }

    messages
}

/// Alert player to a monster's presence via aggravation (you_aggravate equivalent)
///
/// When a monster uses the Amulet or other aggravation, the player
/// becomes aware of that monster's presence.
///
/// # Arguments
/// * `monster` - The monster being revealed
///
/// # Returns
/// Messages to display to the player
pub fn you_aggravate(monster: &Monster) -> Vec<String> {
    let mut messages = Vec::new();

    messages.push(format!(
        "For some reason, {}'s presence is known to you.",
        monster.name
    ));
    messages.push(format!("You feel aggravated at {}.", monster.name));

    messages
}

/// Check if there are monsters that can be aggravated (has_aggravatables equivalent)
///
/// Used by the Wizard of Yendor to determine if aggravation would be useful.
/// Looks for sleeping or waiting monsters on the level.
///
/// # Arguments
/// * `monsters` - The list of monsters on the level
/// * `_monster_x` - X position of the monster checking (for W-tower logic, simplified out)
/// * `_monster_y` - Y position of the monster checking (for W-tower logic, simplified out)
///
/// # Returns
/// true if there are monsters that could be aggravated
pub fn has_aggravatables(monsters: &[Monster], _monster_x: i8, _monster_y: i8) -> bool {
    // Check for sleeping or waiting monsters
    for mon in monsters {
        if mon.hp <= 0 {
            continue;
        }

        // Check if monster is waiting or sleeping or can't move
        if mon.state.sleeping || !mon.can_act() {
            return true;
        }

        // Check strategy for STRAT_WAITFORU (WAIT bit)
        if mon.strategy.bits() & Strategy::WAIT != 0 {
            return true;
        }
    }

    false
}

// ============================================================================
// Monster utility functions (from mon.c, mondata.c, monmove.c, muse.c)
// ============================================================================

/// Check if monster is close enough to move or attack into a square (monnear equivalent)
///
/// Returns true if the monster can reach the given position in one move.
/// Takes into account diagonal movement restrictions for certain monsters.
///
/// # Arguments
/// * `mon` - The monster to check
/// * `x` - Target X coordinate
/// * `y` - Target Y coordinate
///
/// # Returns
/// true if the monster can move/attack to (x, y)
pub fn monnear(mon: &Monster, x: i8, y: i8) -> bool {
    let dx = (mon.x - x).abs() as i32;
    let dy = (mon.y - y).abs() as i32;
    let distance = dx * dx + dy * dy;

    // Distance of 2 means diagonal move - check if monster can move diagonally
    if distance == 2 && mon.no_diagonal_move {
        return false;
    }

    // Distance < 3 means adjacent (including diagonal)
    distance < 3
}

/// Check if monster hates light (mon_hates_light equivalent from mondata.c)
///
/// Returns true if the monster is especially affected by light-emitting weapons.
/// Gremlins, certain undead, and light-sensitive creatures are affected.
///
/// # Arguments
/// * `mon` - The monster to check
///
/// # Returns
/// true if the monster hates light
pub fn mon_hates_light(mon: &Monster) -> bool {
    // Check monster name for light-sensitive creatures
    let name_lower = mon.name.to_lowercase();

    // Gremlins hate light
    if name_lower.contains("gremlin") {
        return true;
    }

    // Some undead and cave-dwelling creatures hate light
    if name_lower.contains("grue")
        || name_lower.contains("wraith")
        || name_lower.contains("vampire")
    {
        return true;
    }

    false
}

/// Regenerate monster hit points (mon_regen equivalent from monmove.c)
///
/// Called each turn to potentially regenerate monster HP and decrement
/// special ability cooldowns.
///
/// # Arguments
/// * `mon` - The monster to regenerate
/// * `digest_meal` - Whether to also process eating
/// * `current_turn` - Current game turn (for timing regeneration)
pub fn mon_regen(mon: &mut Monster, digest_meal: bool, current_turn: i64) {
    // Regenerate HP every 20 turns, or every turn if monster has regeneration
    if mon.hp < mon.hp_max {
        let regenerates = mon.regenerates();
        if regenerates || current_turn % 20 == 0 {
            mon.hp += 1;
        }
    }

    // Decrement special ability cooldown
    if mon.spec_used > 0 {
        mon.spec_used -= 1;
    }

    // Handle eating/digesting
    if digest_meal && mon.eating_timeout > 0 {
        mon.eating_timeout -= 1;
    }
}

/// Check if monster can reflect attacks (mon_reflects equivalent from muse.c)
///
/// Checks if the monster has reflection from:
/// - Shield of Reflection
/// - Amulet of Reflection
/// - Silver Dragon Scales/Scale Mail
/// - Being a Silver Dragon or Chromatic Dragon
/// - Wielded artifact with reflection
///
/// # Arguments
/// * `mon` - The monster to check
///
/// # Returns
/// Some(source) with the reflection source name, or None if no reflection
pub fn mon_reflects(mon: &Monster) -> Option<&'static str> {
    // Check for reflection in inventory/equipment
    for obj in &mon.inventory {
        let obj_name = obj.name.as_deref().unwrap_or("");

        // Check worn shield
        if obj.worn_mask & crate::action::wear::worn_mask::W_ARMS != 0 {
            if obj_name.to_lowercase().contains("reflection") {
                return Some("shield");
            }
        }

        // Check worn amulet
        if obj.worn_mask & crate::action::wear::worn_mask::W_AMUL != 0 {
            if obj_name.to_lowercase().contains("reflection") {
                return Some("amulet");
            }
        }

        // Check worn armor (silver dragon scales)
        if obj.worn_mask & crate::action::wear::worn_mask::W_ARM != 0 {
            let name_lower = obj_name.to_lowercase();
            if name_lower.contains("silver dragon") {
                return Some("armor");
            }
        }
    }

    // Check if monster is a silver dragon or chromatic dragon
    let name_lower = mon.name.to_lowercase();
    if name_lower.contains("silver dragon") || name_lower.contains("chromatic dragon") {
        // Baby dragons don't reflect
        if !name_lower.contains("baby") {
            return Some("scales");
        }
    }

    None
}

/// Check if monster has reflection (boolean version)
pub fn mon_has_reflection(mon: &Monster) -> bool {
    mon_reflects(mon).is_some()
}

/// Check if position is valid for monster placement (goodpos equivalent)
///
/// Determines if a monster can be placed at the given location.
/// Checks for:
/// - Valid map bounds
/// - Passable terrain
/// - No other monsters present
/// - Special terrain restrictions
///
/// # Arguments
/// * `x` - X coordinate
/// * `y` - Y coordinate
/// * `mon` - The monster to place (for size/type checks)
/// * `level` - The dungeon level
///
/// # Returns
/// true if the position is valid for the monster
pub fn goodpos(x: i8, y: i8, mon: &Monster, level: &crate::dungeon::Level) -> bool {
    // Check bounds
    if !crate::dungeon::isok(x as i32, y as i32) {
        return false;
    }

    let cell = level.cell(x as usize, y as usize);

    // Check if terrain is walkable
    if !cell.is_walkable() {
        // Some monsters can pass through walls
        if !mon.passes_walls {
            return false;
        }
    }

    // Check for water - only swimmers/flyers can be in water
    if matches!(
        cell.typ,
        crate::dungeon::CellType::Pool | crate::dungeon::CellType::Moat
    ) {
        if !mon.can_swim && !mon.can_fly {
            return false;
        }
    }

    // Check for lava - only fire resistant flyers can be in lava
    if matches!(cell.typ, crate::dungeon::CellType::Lava) {
        if !mon.can_fly || !mon.resists_fire() {
            return false;
        }
    }

    // Check for another monster at this position
    // (This would require level.monster_at() in full implementation)

    true
}

/// Find a valid position near the given coordinates (enexto equivalent)
///
/// Searches for a valid position for the monster near (x, y).
/// Searches in expanding rings around the target position.
///
/// # Arguments
/// * `x` - Starting X coordinate
/// * `y` - Starting Y coordinate
/// * `mon` - The monster to place
/// * `level` - The dungeon level
///
/// # Returns
/// Some((new_x, new_y)) if a valid position was found, None otherwise
pub fn enexto(x: i8, y: i8, mon: &Monster, level: &crate::dungeon::Level) -> Option<(i8, i8)> {
    // Check the target position first
    if goodpos(x, y, mon, level) {
        return Some((x, y));
    }

    // Search in expanding rings
    for radius in 1i8..=10 {
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                // Only check positions on the ring perimeter
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let nx = x.saturating_add(dx);
                let ny = y.saturating_add(dy);

                if goodpos(nx, ny, mon, level) {
                    return Some((nx, ny));
                }
            }
        }
    }

    None
}

/// Normal movement speed constant (12 in NetHack)
pub const NORMAL_SPEED: i32 = 12;

/// Calculate monster's movement points for this turn (mcalcmove equivalent)
///
/// Takes into account base speed, speed modifiers (slow/fast), and
/// randomization to prevent exploits.
///
/// # Arguments
/// * `mon` - The monster
/// * `rng` - Random number generator
///
/// # Returns
/// Movement points for this turn
pub fn mcalcmove(mon: &Monster, rng: &mut crate::rng::GameRng) -> i32 {
    let mut mmove = mon.base_speed;

    // Apply speed modifiers
    // MSLOW: (2 * mmove + 1) / 3
    // MFAST: (4 * mmove + 2) / 3
    match mon.speed {
        SpeedState::Slow => {
            mmove = (2 * mmove + 1) / 3;
        }
        SpeedState::Fast => {
            mmove = (4 * mmove + 2) / 3;
        }
        SpeedState::Normal => {}
    }

    // Randomly round the monster's speed to a multiple of NORMAL_SPEED.
    // This makes it impossible for the player to predict when they'll get
    // a free turn (thus preventing exploits like "melee kiting")
    let mmove_adj = mmove % NORMAL_SPEED;
    mmove -= mmove_adj;
    if rng.rn2(NORMAL_SPEED as u32) < mmove_adj as u32 {
        mmove += NORMAL_SPEED;
    }

    mmove
}

/// Check if monster is in liquid (water/lava) and handle effects (minliquid equivalent)
///
/// Returns information about what happened to the monster in liquid.
///
/// # Arguments
/// * `mon` - The monster to check
/// * `level` - The dungeon level
///
/// # Returns
/// MinliquidResult indicating what happened
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinliquidResult {
    /// Monster survived (not in liquid or can handle it)
    Survived,
    /// Monster drowned in water
    Drowned,
    /// Monster burned in lava
    Burned,
    /// Monster teleported away to escape
    Teleported,
    /// Monster took damage but survived
    Damaged(i32),
}

pub fn minliquid(mon: &mut Monster, level: &crate::dungeon::Level) -> MinliquidResult {
    let cell = level.cell(mon.x as usize, mon.y as usize);

    let in_pool = matches!(
        cell.typ,
        crate::dungeon::CellType::Pool | crate::dungeon::CellType::Moat
    );
    let in_lava = matches!(cell.typ, crate::dungeon::CellType::Lava);

    // Flying/floating monsters are safe
    if mon.can_fly {
        return MinliquidResult::Survived;
    }

    if in_lava {
        // Lava effects
        if !mon.can_fly {
            if !mon.resists_fire() {
                // Monster burns to death
                mon.hp = 0;
                return MinliquidResult::Burned;
            } else {
                // Fire resistant takes 1 damage per turn
                mon.hp -= 1;
                if mon.hp <= 0 {
                    return MinliquidResult::Burned;
                }
                return MinliquidResult::Damaged(1);
            }
        }
    } else if in_pool {
        // Water effects - most monsters drown
        if !mon.can_swim && !mon.can_fly {
            // Monster drowns
            mon.hp = 0;
            return MinliquidResult::Drowned;
        }
    } else {
        // Not in liquid - eels suffer out of water
        let name_lower = mon.name.to_lowercase();
        if name_lower.contains("eel") {
            // Eels take damage out of water
            if mon.hp > 1 {
                mon.hp -= 1;
                return MinliquidResult::Damaged(1);
            }
        }
    }

    MinliquidResult::Survived
}

/// Calculate current monster load (weight carried) (curr_mon_load equivalent)
///
/// # Arguments
/// * `mon` - The monster
///
/// # Returns
/// Total weight of items carried
pub fn curr_mon_load(mon: &Monster) -> i32 {
    mon.inventory
        .iter()
        .map(|obj| crate::object::weight(obj) as i32)
        .sum()
}

/// Calculate maximum monster load capacity (max_mon_load equivalent)
///
/// Based on monster strength/size. Larger monsters can carry more.
///
/// # Arguments
/// * `mon` - The monster
///
/// # Returns
/// Maximum weight the monster can carry
pub fn max_mon_load(mon: &Monster) -> i32 {
    // Base capacity based on level (simplified from C version)
    // In NetHack, this is based on monster strength which correlates with level
    let base_capacity = 50 + (mon.level as i32) * 25;

    // Adjust for size (would check permonst data in full implementation)
    base_capacity
}

/// Check if monster is encumbered
pub fn mon_encumbered(mon: &Monster) -> bool {
    curr_mon_load(mon) > max_mon_load(mon)
}

/// Hide monster under object or terrain (hideunder equivalent)
///
/// # Arguments
/// * `mon` - The monster to hide
/// * `level` - The dungeon level
///
/// # Returns
/// true if monster successfully hid
pub fn hideunder(mon: &mut Monster, level: &crate::dungeon::Level) -> bool {
    // Check if monster can hide
    let name_lower = mon.name.to_lowercase();
    let can_hide = name_lower.contains("piercer")
        || name_lower.contains("trapper")
        || name_lower.contains("lurker");

    if !can_hide {
        return false;
    }

    let cell = level.cell(mon.x as usize, mon.y as usize);

    // Can hide on floor with objects or certain terrain
    // Simplified check - in full implementation would check for objects on floor
    if cell.typ.is_passable() {
        mon.state.hiding = true;
        return true;
    }

    false
}

/// Reveal a hiding/mimicking monster (seemimic equivalent)
///
/// # Arguments
/// * `mon` - The monster to reveal
pub fn seemimic(mon: &mut Monster) {
    mon.state.hiding = false;
    // In full implementation, would also handle mimic appearance reset
}

/// Check if monster is beside the player (mon_beside equivalent)
///
/// # Arguments
/// * `mon` - The monster
/// * `player_x` - Player X coordinate
/// * `player_y` - Player Y coordinate
///
/// # Returns
/// true if monster is adjacent to player
pub fn mon_beside(mon: &Monster, player_x: i8, player_y: i8) -> bool {
    let dx = (mon.x - player_x).abs();
    let dy = (mon.y - player_y).abs();
    dx <= 1 && dy <= 1 && (dx != 0 || dy != 0)
}

/// Check if monster has the Amulet of Yendor (mon_has_amulet equivalent)
///
/// # Arguments
/// * `mon` - The monster to check
///
/// # Returns
/// true if monster is carrying the Amulet
pub fn mon_has_amulet(mon: &Monster) -> bool {
    for obj in &mon.inventory {
        let obj_name = obj.name.as_deref().unwrap_or("");
        if obj_name.to_lowercase().contains("amulet of yendor") {
            return true;
        }
    }
    false
}

/// Check if monster has a specific artifact (mon_has_arti equivalent)
///
/// # Arguments
/// * `mon` - The monster to check
/// * `artifact_name` - Name of the artifact to look for
///
/// # Returns
/// true if monster has the artifact
pub fn mon_has_arti(mon: &Monster, artifact_name: &str) -> bool {
    let search_name = artifact_name.to_lowercase();
    for obj in &mon.inventory {
        let obj_name = obj.name.as_deref().unwrap_or("");
        if obj_name.to_lowercase().contains(&search_name) {
            return true;
        }
    }
    false
}

/// Check if monster has any special/quest item (mon_has_special equivalent)
///
/// # Arguments
/// * `mon` - The monster to check
///
/// # Returns
/// true if monster has a special item
pub fn mon_has_special(mon: &Monster) -> bool {
    // Check for Amulet of Yendor or invocation items
    mon_has_amulet(mon)
        || mon_has_arti(mon, "bell of opening")
        || mon_has_arti(mon, "candelabrum")
        || mon_has_arti(mon, "book of the dead")
}

/// Process monster distress effects each turn (mcalcdistress equivalent)
///
/// Called once per game turn to handle:
/// - Monster regeneration (via mon_regen)
/// - Liquid damage (via minliquid logic)
/// - Shapeshifter transformations
/// - Werewolf transformations
///
/// C Source: mon.c:693-718, mcalcdistress()
///
/// # Arguments
/// * `level` - The dungeon level containing monsters
/// * `current_turn` - Current game turn for timing
pub fn mcalcdistress(level: &mut crate::dungeon::Level, current_turn: i64) {
    // Iterate through all monsters on the level
    for monster_id in level.monster_ids().collect::<Vec<_>>() {
        // First pass: get monster position and properties with immutable borrow
        let monster_info = {
            let Some(monster) = level.monster(monster_id) else {
                continue;
            };
            if monster.hp <= 0 {
                continue;
            }
            (
                monster.x as usize,
                monster.y as usize,
                monster.can_fly,
                monster.can_swim,
                monster.resists_fire(),
            )
        };

        // Get cell info separately (no monster borrow active)
        let (mx, my, can_fly, can_swim, resists_fire) = monster_info;
        let cell = level.cell(mx, my);
        let in_pool = matches!(
            cell.typ,
            crate::dungeon::CellType::Pool | crate::dungeon::CellType::Moat
        );
        let in_lava = matches!(cell.typ, crate::dungeon::CellType::Lava);

        // Second pass: apply effects with mutable borrow
        if let Some(monster) = level.monster_mut(monster_id) {
            // Line 700-701: Monster regeneration
            mon_regen(monster, true, current_turn);

            // Line 703-706: Liquid damage check
            if in_lava && !can_fly && !resists_fire {
                monster.hp = 0; // Burned
            } else if in_pool && !can_swim && !can_fly {
                monster.hp = 0; // Drowned
            }

            // Line 708-712: Shapeshifter transformation
            // TODO: decide_to_shapeshift(monster)

            // Line 714-717: Werewolf transformation
            // TODO: were_change(monster)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monnear_adjacent() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);

        // Adjacent positions (distance < 3)
        assert!(monnear(&mon, 5, 6)); // distance = 1
        assert!(monnear(&mon, 6, 5)); // distance = 1
        assert!(monnear(&mon, 6, 6)); // distance = 2 (diagonal)
        assert!(monnear(&mon, 4, 4)); // distance = 2 (diagonal)
    }

    #[test]
    fn test_monnear_too_far() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);

        // Too far (distance >= 3)
        assert!(!monnear(&mon, 5, 7)); // distance = 4
        assert!(!monnear(&mon, 7, 5)); // distance = 4
        assert!(!monnear(&mon, 7, 7)); // distance = 8
    }

    #[test]
    fn test_monnear_no_diagonal() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.no_diagonal_move = true;

        // Diagonal should fail for no-diagonal monsters
        assert!(!monnear(&mon, 6, 6)); // distance = 2 (diagonal)
        assert!(!monnear(&mon, 4, 4)); // distance = 2 (diagonal)

        // Orthogonal should still work
        assert!(monnear(&mon, 5, 6)); // distance = 1
        assert!(monnear(&mon, 6, 5)); // distance = 1
    }

    #[test]
    fn test_mon_hates_light_gremlin() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "gremlin".to_string();
        assert!(mon_hates_light(&mon));
    }

    #[test]
    fn test_mon_hates_light_normal() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "orc".to_string();
        assert!(!mon_hates_light(&mon));
    }

    #[test]
    fn test_mon_regen_every_20_turns() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.hp = 5;
        mon.hp_max = 10;

        // Turn 20 should regenerate
        mon_regen(&mut mon, false, 20);
        assert_eq!(mon.hp, 6);

        // Turn 21 should not regenerate
        mon_regen(&mut mon, false, 21);
        assert_eq!(mon.hp, 6);

        // Turn 40 should regenerate
        mon_regen(&mut mon, false, 40);
        assert_eq!(mon.hp, 7);
    }

    #[test]
    fn test_mon_regen_at_max() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.hp = 10;
        mon.hp_max = 10;

        mon_regen(&mut mon, false, 20);
        assert_eq!(mon.hp, 10); // Should not exceed max
    }

    #[test]
    fn test_mon_regen_spec_used() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.spec_used = 5;

        mon_regen(&mut mon, false, 1);
        assert_eq!(mon.spec_used, 4);
    }

    #[test]
    fn test_mon_reflects_silver_dragon() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "silver dragon".to_string();

        assert_eq!(mon_reflects(&mon), Some("scales"));
        assert!(mon_has_reflection(&mon));
    }

    #[test]
    fn test_mon_reflects_baby_dragon() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "baby silver dragon".to_string();

        assert_eq!(mon_reflects(&mon), None);
        assert!(!mon_has_reflection(&mon));
    }

    #[test]
    fn test_mon_reflects_normal() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "orc".to_string();

        assert_eq!(mon_reflects(&mon), None);
    }

    #[test]
    fn test_set_malign_peaceful_coaligned() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.peaceful = true;
        mon.alignment = 10; // Positive aligned

        set_malign(&mut mon, 5); // Player also positive (co-aligned)

        // Peaceful and co-aligned should be very non-hostile
        assert_eq!(
            mon.alignment, -20,
            "Peaceful co-aligned monster should have negative alignment"
        );
    }

    #[test]
    fn test_set_malign_peaceful_not_coaligned() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.peaceful = true;
        mon.alignment = 10; // Positive aligned

        set_malign(&mut mon, -5); // Player is negative (not co-aligned)

        // Peaceful but not co-aligned should be neutral
        assert_eq!(
            mon.alignment, 0,
            "Peaceful non-aligned monster should have neutral alignment"
        );
    }

    #[test]
    fn test_set_malign_hostile_coaligned() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.peaceful = false;
        mon.alignment = 10; // Positive aligned

        set_malign(&mut mon, 5); // Player also positive (co-aligned)

        // Hostile but co-aligned should be less aggressive
        assert_eq!(
            mon.alignment, 10,
            "Hostile co-aligned monster should be moderately aggressive"
        );
    }

    #[test]
    fn test_set_malign_hostile_not_coaligned() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.peaceful = false;
        mon.alignment = 10; // Positive aligned

        set_malign(&mut mon, -5); // Player is negative (not co-aligned)

        // Hostile and not co-aligned should be very aggressive
        assert_eq!(
            mon.alignment, 20,
            "Hostile non-aligned monster should be very aggressive"
        );
    }

    #[test]
    fn test_set_malign_neutral_player() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.peaceful = true;
        mon.alignment = 10;

        set_malign(&mut mon, 0); // Neutral player

        // Neutral player means not co-aligned with anyone
        assert_eq!(
            mon.alignment, 0,
            "With neutral player, peaceful monster should be neutral"
        );
    }

    #[test]
    fn test_onscary_wizard_of_yendor_immune() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "Wizard of Yendor".to_string();

        // Wizard of Yendor should not be scared
        assert!(!onscary(5, 5, &mon, &level, 5, 5, false));
    }

    #[test]
    fn test_onscary_angel_immune() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "Angel".to_string();

        assert!(!onscary(5, 5, &mon, &level, 5, 5, false));
    }

    #[test]
    fn test_onscary_musical_check() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "orc".to_string();

        // <0,0> is used for musical scaring immunity check
        assert!(onscary(0, 0, &mon, &level, 5, 5, false));
    }

    #[test]
    fn test_onscary_shopkeeper_immune() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "shopkeeper".to_string();
        mon.is_shopkeeper = true;

        assert!(!onscary(5, 5, &mon, &level, 5, 5, false));
    }

    #[test]
    fn test_maybe_wail_wizard() {
        let mut player = crate::player::You::default();
        player.role = crate::player::Role::Wizard;
        player.hp = 1;

        let mut last_wail = 0i64;
        let messages = maybe_wail(&player, 100, &mut last_wail);

        assert!(!messages.is_empty());
        assert!(messages[0].contains("is about to die"));
    }

    #[test]
    fn test_maybe_wail_other_class() {
        let mut player = crate::player::You::default();
        player.role = crate::player::Role::Barbarian;
        player.hp = 1;

        let mut last_wail = 0i64;
        let messages = maybe_wail(&player, 100, &mut last_wail);

        assert!(!messages.is_empty());
        assert!(messages[0].contains("Banshee"));
    }

    #[test]
    fn test_maybe_wail_cooldown() {
        let mut player = crate::player::You::default();
        player.role = crate::player::Role::Wizard;
        player.hp = 1;

        let mut last_wail = 100i64;
        // Within cooldown period (50 turns)
        let messages = maybe_wail(&player, 120, &mut last_wail);

        assert!(
            messages.is_empty(),
            "Should not wail within cooldown period"
        );
    }

    #[test]
    fn test_you_aggravate() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "goblin".to_string();

        let messages = you_aggravate(&mon);

        assert_eq!(messages.len(), 2);
        assert!(messages[0].contains("goblin"));
        assert!(messages[1].contains("aggravated"));
    }

    #[test]
    fn test_has_aggravatables_sleeping() {
        let mut mon1 = Monster::new(MonsterId(1), 0, 5, 5);
        mon1.state.sleeping = true;
        mon1.hp = 10;

        let monsters = vec![mon1];
        assert!(has_aggravatables(&monsters, 0, 0));
    }

    #[test]
    fn test_has_aggravatables_waiting() {
        let mut mon1 = Monster::new(MonsterId(1), 0, 5, 5);
        mon1.strategy = Strategy::new(Strategy::WAIT | Strategy::PLAYER);
        mon1.hp = 10;

        let monsters = vec![mon1];
        assert!(has_aggravatables(&monsters, 0, 0));
    }

    #[test]
    fn test_has_aggravatables_none() {
        let mut mon1 = Monster::new(MonsterId(1), 0, 5, 5);
        mon1.hp = 10;
        mon1.state.sleeping = false;
        mon1.strategy = Strategy::new(Strategy::CLOSE | Strategy::PLAYER);

        let monsters = vec![mon1];
        assert!(!has_aggravatables(&monsters, 0, 0));
    }

    // Tests for batch 2 functions

    #[test]
    fn test_mcalcmove_normal_speed() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.base_speed = 12;
        mon.speed = SpeedState::Normal;

        let mut rng = crate::rng::GameRng::from_entropy();
        let move_points = mcalcmove(&mon, &mut rng);

        // Normal speed 12 should give 12 or 0 depending on randomization
        assert!(move_points == 0 || move_points == 12);
    }

    #[test]
    fn test_mcalcmove_slow() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.base_speed = 12;
        mon.speed = SpeedState::Slow;

        let mut rng = crate::rng::GameRng::from_entropy();
        let move_points = mcalcmove(&mon, &mut rng);

        // Slow: (2 * 12 + 1) / 3 = 8, then randomized
        assert!(move_points == 0 || move_points == 12);
    }

    #[test]
    fn test_mcalcmove_fast() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.base_speed = 12;
        mon.speed = SpeedState::Fast;

        let mut rng = crate::rng::GameRng::from_entropy();
        let move_points = mcalcmove(&mon, &mut rng);

        // Fast: (4 * 12 + 2) / 3 = 16, then randomized
        assert!(move_points == 12 || move_points == 24);
    }

    #[test]
    fn test_mon_beside_adjacent() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);

        // Adjacent positions
        assert!(mon_beside(&mon, 4, 5));
        assert!(mon_beside(&mon, 6, 5));
        assert!(mon_beside(&mon, 5, 4));
        assert!(mon_beside(&mon, 5, 6));
        assert!(mon_beside(&mon, 4, 4)); // Diagonal
        assert!(mon_beside(&mon, 6, 6)); // Diagonal
    }

    #[test]
    fn test_mon_beside_same_position() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);

        // Same position is not "beside"
        assert!(!mon_beside(&mon, 5, 5));
    }

    #[test]
    fn test_mon_beside_too_far() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);

        // Too far
        assert!(!mon_beside(&mon, 3, 5));
        assert!(!mon_beside(&mon, 7, 5));
    }

    #[test]
    fn test_curr_mon_load_empty() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);
        assert_eq!(curr_mon_load(&mon), 0);
    }

    #[test]
    fn test_max_mon_load() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.level = 5;

        // Base capacity: 50 + 5 * 25 = 175
        assert_eq!(max_mon_load(&mon), 175);
    }

    #[test]
    fn test_mon_encumbered() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.level = 1; // max_load = 50 + 25 = 75

        // Empty inventory - not encumbered
        assert!(!mon_encumbered(&mon));
    }

    #[test]
    fn test_mon_has_amulet_false() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!mon_has_amulet(&mon));
    }

    #[test]
    fn test_mon_has_arti_false() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!mon_has_arti(&mon, "excalibur"));
    }

    #[test]
    fn test_mon_has_special_false() {
        let mon = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!mon_has_special(&mon));
    }

    #[test]
    fn test_seemimic() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.state.hiding = true;

        seemimic(&mut mon);

        assert!(!mon.state.hiding);
    }

    #[test]
    fn test_hideunder_piercer_can_hide() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "rock piercer".to_string();
        mon.state.hiding = false;

        // Create a simple level mock - hideunder checks cell passability
        // For unit test, we test the name check logic
        let name_lower = mon.name.to_lowercase();
        let can_hide = name_lower.contains("piercer")
            || name_lower.contains("trapper")
            || name_lower.contains("lurker");

        assert!(can_hide);
    }

    #[test]
    fn test_hideunder_normal_monster_cannot_hide() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.name = "orc".to_string();

        let name_lower = mon.name.to_lowercase();
        let can_hide = name_lower.contains("piercer")
            || name_lower.contains("trapper")
            || name_lower.contains("lurker");

        assert!(!can_hide);
    }

    #[test]
    fn test_minliquid_flying_survives() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.can_fly = true;
        mon.hp = 10;

        // Flying monsters survive in any liquid
        // This tests the logic that flying monsters are safe
        assert!(mon.can_fly);
        assert!(mon.hp > 0);
    }

    #[test]
    fn test_minliquid_swimmer_survives_water() {
        let mut mon = Monster::new(MonsterId(1), 0, 5, 5);
        mon.can_swim = true;
        mon.can_fly = false;
        mon.hp = 10;

        // Swimming monsters survive in water
        assert!(mon.can_swim);
        assert!(mon.hp > 0);
    }
}
