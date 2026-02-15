//! Monster system
//!
//! Contains monster templates (permonst) and instances (monst).

pub mod ai;
pub mod casting;
pub mod lifecycle;
pub mod makemon;
pub mod item_usage;
mod monst;
mod permonst;
pub mod tactics;
pub mod throw;
pub mod worm;

// Extensions: Combat AI & Monster Tactics (Rust-only, no C equivalent)
#[cfg(feature = "extensions")]
pub mod attack_selection;
#[cfg(feature = "extensions")]
pub mod combat_hooks;
#[cfg(feature = "extensions")]
pub mod morale;
#[cfg(feature = "extensions")]
pub mod personality;
#[cfg(feature = "extensions")]
pub mod tactical_ai;

pub use crate::combat::CombatResources;
pub use ai::{AiAction, process_monster_ai};
#[cfg(feature = "extensions")]
pub use attack_selection::{AbilityType, AttackOption, CombatMemory, Precondition, ResourceCost};
pub use casting::{
    CastResult, CasterSnapshot, ClericSpell, MageSpell, buzzmu, castmu,
    choose_clerical_spell, choose_magic_spell, cleric_spell_would_be_useless,
    is_undirected_cleric_spell, is_undirected_mage_spell, mage_spell_would_be_useless,
};
pub use item_usage::{HornEffect, monster_zap_wand, mplayhorn, mzapwand};
pub use monst::{
    MinliquidResult,
    Monster,
    MonsterId,
    MonsterState,
    // Movement and load functions
    NORMAL_SPEED,
    PronounCase,
    SpeedState,
    Strategy,
    ThreatLevel,
    aggravate,
    awaken_monsters,
    awaken_soldiers,
    curr_mon_load,
    distfleeck,
    disturb,
    enexto,
    goodpos,
    has_aggravatables,
    // Hiding functions
    hideunder,
    max_mon_load,
    maybe_wail,
    // Distress processing
    mcalcdistress,
    mcalcmove,
    minliquid,
    // Position and inventory checks
    mon_beside,
    mon_encumbered,
    mon_has_amulet,
    mon_has_arti,
    mon_has_reflection,
    mon_has_special,
    mon_hates_light,
    mon_reflects,
    mon_regen,
    monflee,
    // Monster utility functions
    monnear,
    // Scare mechanics
    onscary,
    scare_monster,
    seemimic,
    set_malign,
    should_flee_from_damage,
    should_stay_near_player,
    update_flee,
    wake_nearby,
    wants_to_attack,
    you_aggravate,
};
#[cfg(feature = "extensions")]
pub use morale::{MoraleEvent, MoraleTracker, RetreatReason};
pub use permonst::{
    MonsterClass, MonsterFlags, MonsterResistances, MonsterSize, MonsterSound, PerMonst,
    big_little_match, big_to_little, def_char_to_monclass, genus, green_mon, is_home_elemental,
    little_to_big, name_to_mon, name_to_monclass, propagate, same_race, validspecmon, validvamp,
};
#[cfg(feature = "extensions")]
pub use personality::{Personality, PersonalityProfile, assign_personality};
pub use tactics::{
    Intelligence, SpecialAbility, TacticalAction, WeaponCheck, determine_tactics, getmattk,
    mon_wield_item, monmightthrowwep, monster_intelligence, mwelded, possibly_unwield, select_hwep,
    select_rwep, setmnotwielded,
};

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::Object;

/// Reference to a monster instance
pub type MonsterRef = MonsterId;

// ============================================================================
// PLACEHOLDER FUNCTIONS (for systems not yet fully implemented)
// ============================================================================

/// Placeholder: Set monster invisible (full implementation pending)
///
/// Currently just sets the invisible flag. Full implementation would require:
/// - Updating visibility calculations for all observers
/// - Adjusting AI behavior for invisible monsters
/// - Handling magic that reveals invisible creatures
///
/// # Note
/// This is a simplified placeholder pending full visibility system implementation (Phase TBD)
pub fn mon_set_minvis(monster: &mut Monster) {
    monster.state.invisible = true;
    // Full visibility recalculation and observer notification pending visibility system
}

/// Placeholder: Adjust monster speed (full implementation pending)
///
/// Currently modifies the speed field directly. Full implementation would require:
/// - Distinguishing between temporary and permanent speed changes
/// - Tracking speed modification sources (item, spell, etc.)
/// - Applying speed modifiers to action costs
///
/// # Arguments
/// * `monster` - The monster to adjust
/// * `delta` - Amount to adjust speed by (positive = faster, negative = slower)
/// * `_item` - The item causing the change (for full implementation)
///
/// # Note
/// This is a simplified placeholder pending full speed system implementation (Phase TBD)
pub fn mon_adjust_speed(monster: &mut Monster, delta: i8, _item: Option<&Object>) {
    // Convert SpeedState to numeric value, apply delta, convert back
    let current = monster.speed as i8;
    let new_val = (current + delta).clamp(0, 2);
    monster.speed = match new_val {
        0 => SpeedState::Slow,
        1 => SpeedState::Normal,
        _ => SpeedState::Fast,
    };
    // Full speed system with temporary/permanent source tracking pending
}

/// Placeholder: Polymorph monster into new form (full implementation pending)
///
/// Currently returns false (fails). Full implementation would require:
/// - Selecting a compatible monster type to transform into
/// - Updating monster stats and abilities
/// - Handling visual transformations and messages
/// - Preserving important state like AI and morale
///
/// # Arguments
/// * `monster` - The monster to polymorph
/// * `_new_type` - Optional specific type to polymorph into
///
/// # Returns
/// true if successful, false if failed (currently always false)
///
/// # Note
/// This is a stub implementation. Full polymorph system is Phase TBD priority
pub fn newcham(monster: &mut Monster, _new_type: Option<&crate::monster::PerMonst>) -> bool {
    // Conservative: fail until full polymorph system with type selection is implemented
    false
}

/// Placeholder: Drop boulder on target location
///
/// Currently does nothing. Full implementation would require:
/// - Creating a boulder object at the location
/// - Applying damage to creatures at that location
/// - Handling terrain destruction
///
/// # Arguments
/// * `_x` - Target X coordinate
/// * `_y` - Target Y coordinate
/// * `_level` - Level to place boulder on
/// * `_confused` - Whether the caster is confused (affects accuracy)
///
/// # Note
/// This is a stub implementation pending object system integration (Phase TBD)
pub fn drop_boulder_on_target(_x: i8, _y: i8, _level: &mut crate::dungeon::Level, _confused: bool) {
    // Boulder spawning deferred: requires object system integration for ROCK class creation
}
