//! Player intrinsic and extrinsic properties
//!
//! Properties are abilities/resistances that can be intrinsic (permanent)
//! or extrinsic (from worn items).

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

/// Property types (from prop.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumIter)]
#[repr(u8)]
pub enum Property {
    // Movement properties
    Speed = 0,
    VeryFast = 1,
    Levitation = 2,
    Flying = 3,
    Swimming = 4,
    MagicBreathing = 5,
    PassesWalls = 6,
    Jumping = 7,
    WaterWalking = 8,

    // Resistances
    FireResistance = 10,
    ColdResistance = 11,
    SleepResistance = 12,
    DisintResistance = 13,
    ShockResistance = 14,
    PoisonResistance = 15,
    AcidResistance = 16,
    StoneResistance = 17,
    DrainResistance = 18,
    SickResistance = 19,

    // Vision
    SeeInvisible = 20,
    Telepathy = 21,
    Infravision = 22,
    Xray = 23,
    Searching = 24,
    Clairvoyant = 25,
    Warning = 26,
    WarnOfMon = 27,

    // Stealth
    Stealth = 30,
    Invisibility = 31,
    Displaced = 32,
    Aggravate = 33,
    Conflict = 34,

    // Protection
    Protection = 40,
    ProtFromShapechangers = 41,
    FreeAction = 42,
    Reflection = 43,
    MagicResistance = 44,
    HalfSpellDamage = 45,
    HalfPhysDamage = 46,
    Regeneration = 47,
    EnergyRegeneration = 48,

    // Misc
    Teleportation = 50,
    TeleportControl = 51,
    Polymorph = 52,
    PolyControl = 53,
    Unchanging = 54,
    Fumbling = 55,
    WoundedLegs = 56,
    Sleepy = 57,
    Hunger = 58,
    SlowDigestion = 59,
    SustainAbility = 60,
    LifeSaving = 61,
    Concentration = 62,
    StoneSkin = 63,
    Silenced = 64,
}

impl Property {
    pub const LAST: Property = Property::Silenced;

    /// Check if this is a resistance property
    pub const fn is_resistance(&self) -> bool {
        (*self as u8) >= 10 && (*self as u8) <= 19
    }

    /// Check if this is a vision property
    pub const fn is_vision(&self) -> bool {
        (*self as u8) >= 20 && (*self as u8) <= 27
    }
}

bitflags! {
    /// Flags for property sources
    #[derive(Debug, Clone, Copy, Default)]
    pub struct PropertyFlags: u32 {
        /// From intrinsic (permanent)
        const INTRINSIC = 0x0001;
        /// Blocked by worn item
        const BLOCKED = 0x0002;
        /// From timeout (temporary)
        const TIMEOUT = 0x0004;

        // Extrinsic sources (from equipment)
        const FROM_HELM = 0x0010;
        const FROM_ARMOR = 0x0020;
        const FROM_CLOAK = 0x0040;
        const FROM_GLOVES = 0x0080;
        const FROM_BOOTS = 0x0100;
        const FROM_SHIELD = 0x0200;
        const FROM_WEAPON = 0x0400;
        const FROM_RING_L = 0x0800;
        const FROM_RING_R = 0x1000;
        const FROM_AMULET = 0x2000;
        const FROM_ARTIFACT = 0x4000;

        /// Any extrinsic source
        const EXTRINSIC = Self::FROM_HELM.bits()
            | Self::FROM_ARMOR.bits()
            | Self::FROM_CLOAK.bits()
            | Self::FROM_GLOVES.bits()
            | Self::FROM_BOOTS.bits()
            | Self::FROM_SHIELD.bits()
            | Self::FROM_WEAPON.bits()
            | Self::FROM_RING_L.bits()
            | Self::FROM_RING_R.bits()
            | Self::FROM_AMULET.bits()
            | Self::FROM_ARTIFACT.bits();
    }
}

// Manual serde impl for PropertyFlags
impl Serialize for PropertyFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PropertyFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Ok(PropertyFlags::from_bits_truncate(bits))
    }
}

/// Property state array for tracking all properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertySet {
    props: Vec<PropertyFlags>,
    timeouts: Vec<u32>,
}

impl Default for PropertySet {
    fn default() -> Self {
        let size = Property::LAST as usize + 1;
        Self {
            props: vec![PropertyFlags::empty(); size],
            timeouts: vec![0; size],
        }
    }
}

impl PropertySet {
    /// Check if player has a property (from any source)
    pub fn has(&self, prop: Property) -> bool {
        let flags = self.props[prop as usize];
        if flags.contains(PropertyFlags::BLOCKED) {
            return false;
        }
        flags.intersects(
            PropertyFlags::INTRINSIC | PropertyFlags::EXTRINSIC | PropertyFlags::TIMEOUT,
        )
    }

    /// Check if player has intrinsic property
    pub fn has_intrinsic(&self, prop: Property) -> bool {
        self.props[prop as usize].contains(PropertyFlags::INTRINSIC)
    }

    /// Check if player has extrinsic property
    pub fn has_extrinsic(&self, prop: Property) -> bool {
        self.props[prop as usize].intersects(PropertyFlags::EXTRINSIC)
    }

    /// Grant intrinsic property
    pub fn grant_intrinsic(&mut self, prop: Property) {
        self.props[prop as usize].insert(PropertyFlags::INTRINSIC);
    }

    /// Remove intrinsic property
    pub fn remove_intrinsic(&mut self, prop: Property) {
        self.props[prop as usize].remove(PropertyFlags::INTRINSIC);
    }

    /// Grant extrinsic property from a source
    pub fn grant_extrinsic(&mut self, prop: Property, source: PropertyFlags) {
        self.props[prop as usize].insert(source);
    }

    /// Remove extrinsic property from a source
    pub fn remove_extrinsic(&mut self, prop: Property, source: PropertyFlags) {
        self.props[prop as usize].remove(source);
    }

    /// Revoke intrinsic property (alias for remove_intrinsic)
    pub fn revoke_intrinsic(&mut self, prop: Property) {
        self.remove_intrinsic(prop);
    }

    /// Revoke all extrinsic sources of a property
    pub fn revoke_extrinsic(&mut self, prop: Property) {
        self.props[prop as usize].remove(PropertyFlags::EXTRINSIC);
    }

    /// Grant extrinsic property (from generic item source)
    pub fn grant_extrinsic_simple(&mut self, prop: Property) {
        self.props[prop as usize].insert(PropertyFlags::FROM_ARTIFACT);
    }

    /// Set property timeout
    pub fn set_timeout(&mut self, prop: Property, turns: u32) {
        self.timeouts[prop as usize] = turns;
        if turns > 0 {
            self.props[prop as usize].insert(PropertyFlags::TIMEOUT);
        } else {
            self.props[prop as usize].remove(PropertyFlags::TIMEOUT);
        }
    }

    /// Decrement all timeouts by 1
    pub fn tick_timeouts(&mut self) {
        for (i, timeout) in self.timeouts.iter_mut().enumerate() {
            if *timeout > 0 {
                *timeout -= 1;
                if *timeout == 0 {
                    self.props[i].remove(PropertyFlags::TIMEOUT);
                }
            }
        }
    }

    /// Get timeout remaining for a property
    pub fn timeout(&self, prop: Property) -> u32 {
        self.timeouts[prop as usize]
    }

    /// Block a property
    pub fn block(&mut self, prop: Property) {
        self.props[prop as usize].insert(PropertyFlags::BLOCKED);
    }

    /// Unblock a property
    pub fn unblock(&mut self, prop: Property) {
        self.props[prop as usize].remove(PropertyFlags::BLOCKED);
    }

    // ========================================================================
    // Convenience methods for common property checks
    // ========================================================================

    /// Check if has see invisible
    pub fn has_see_invisible(&self) -> bool {
        self.has(Property::SeeInvisible)
    }

    /// Check if has telepathy
    pub fn has_telepathy(&self) -> bool {
        self.has(Property::Telepathy)
    }

    /// Check if has infravision
    pub fn has_infravision(&self) -> bool {
        self.has(Property::Infravision)
    }

    /// Check if has levitation
    pub fn has_levitation(&self) -> bool {
        self.has(Property::Levitation)
    }

    /// Check if has flying
    pub fn has_flying(&self) -> bool {
        self.has(Property::Flying)
    }

    /// Check if can phase through walls
    pub fn has_phasing(&self) -> bool {
        self.has(Property::PassesWalls)
    }

    /// Check if has fire resistance
    pub fn has_fire_res(&self) -> bool {
        self.has(Property::FireResistance)
    }

    /// Check if has cold resistance
    pub fn has_cold_res(&self) -> bool {
        self.has(Property::ColdResistance)
    }

    /// Check if has sleep resistance
    pub fn has_sleep_res(&self) -> bool {
        self.has(Property::SleepResistance)
    }

    /// Check if has disintegration resistance
    pub fn has_disint_res(&self) -> bool {
        self.has(Property::DisintResistance)
    }

    /// Check if has shock resistance
    pub fn has_shock_res(&self) -> bool {
        self.has(Property::ShockResistance)
    }

    /// Check if has poison resistance
    pub fn has_poison_res(&self) -> bool {
        self.has(Property::PoisonResistance)
    }

    /// Check if has acid resistance
    pub fn has_acid_res(&self) -> bool {
        self.has(Property::AcidResistance)
    }

    /// Check if has stone (petrification) resistance
    pub fn has_stone_res(&self) -> bool {
        self.has(Property::StoneResistance)
    }

    /// Check if has drain resistance
    pub fn has_drain_res(&self) -> bool {
        self.has(Property::DrainResistance)
    }

    /// Check if has sick resistance
    pub fn has_sick_res(&self) -> bool {
        self.has(Property::SickResistance)
    }

    /// Check if has magic resistance
    pub fn has_magic_res(&self) -> bool {
        self.has(Property::MagicResistance)
    }

    /// Check if has reflection
    pub fn has_reflection(&self) -> bool {
        self.has(Property::Reflection)
    }

    /// Check if has free action
    pub fn has_free_action(&self) -> bool {
        self.has(Property::FreeAction)
    }

    /// Check if has regeneration
    pub fn has_regeneration(&self) -> bool {
        self.has(Property::Regeneration)
    }

    /// Check if has energy regeneration
    pub fn has_energy_regen(&self) -> bool {
        self.has(Property::EnergyRegeneration)
    }

    /// Check if has speed
    pub fn has_speed(&self) -> bool {
        self.has(Property::Speed)
    }

    /// Check if has stealth
    pub fn has_stealth(&self) -> bool {
        self.has(Property::Stealth)
    }

    /// Check if has invisibility
    pub fn has_invisibility(&self) -> bool {
        self.has(Property::Invisibility)
    }

    /// Check if has teleportation
    pub fn has_teleportation(&self) -> bool {
        self.has(Property::Teleportation)
    }

    /// Check if has teleport control
    pub fn has_teleport_control(&self) -> bool {
        self.has(Property::TeleportControl)
    }

    /// Check if has polymorph control
    pub fn has_polymorph_control(&self) -> bool {
        self.has(Property::PolyControl)
    }

    /// Check if has warning
    pub fn has_warning(&self) -> bool {
        self.has(Property::Warning)
    }

    /// Check if has life saving
    pub fn has_life_saving(&self) -> bool {
        self.has(Property::LifeSaving)
    }
}

// =============================================================================
// Translation functions from C (intrinsic_possible, toggle_*, etc.)
// =============================================================================

/// Check if a property is possible to get from a creature type (intrinsic_possible equivalent)
///
/// Returns true if eating/polymorphing into a creature could grant a property.
/// Maps creature type flags (MR_FIRE, MR_COLD, etc.) to property grants.
pub fn intrinsic_possible(property: Property, creature_type: &str) -> bool {
    match property {
        Property::FireResistance => creature_type.contains("fire"),
        Property::ColdResistance => creature_type.contains("cold"),
        Property::ShockResistance => {
            creature_type.contains("shock") || creature_type.contains("electric")
        }
        Property::DisintResistance => creature_type.contains("disint"),
        Property::PoisonResistance => creature_type.contains("poison"),
        Property::SleepResistance => creature_type.contains("sleep"),
        Property::Telepathy => creature_type.contains("telepat") || creature_type.contains("psion"),
        Property::SeeInvisible => creature_type.contains("see_invis"),
        Property::Infravision => creature_type.contains("infravision"),
        Property::Levitation => creature_type.contains("levitating"),
        Property::Speed => creature_type.contains("fast"),
        Property::Stealth => creature_type.contains("stealth"),
        _ => false,
    }
}

/// Check if an ability source determines innate status (is_innate/innately equivalent - stub)
///
/// Determines if an ability comes from character creation rather than learned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilitySource {
    /// Not innate
    None,
    /// From role level 1
    Role,
    /// From race
    Race,
    /// From form/polymorph
    Form,
}

pub fn check_innate_source(
    property: Property,
    player_role: &str,
    player_race: &str,
) -> AbilitySource {
    // Check if property is innate from role or race

    // Race-based abilities
    let race_innate = match player_race.to_lowercase().as_str() {
        "elf" => matches!(property, Property::Infravision | Property::SeeInvisible),
        "dwarf" => matches!(property, Property::Infravision),
        "gnome" => matches!(property, Property::Infravision),
        "orc" => matches!(property, Property::Infravision),
        _ => false, // Humans have no race innates
    };

    if race_innate {
        return AbilitySource::Race;
    }

    // Role-based abilities (level-gated, so just returning for level 1)
    let role_innate = match player_role.to_lowercase().as_str() {
        "healer" => matches!(property, Property::Protection),
        "priest" => matches!(property, Property::Protection),
        "monk" => matches!(property, Property::FreeAction | Property::Protection),
        "wizard" => matches!(property, Property::MagicResistance),
        _ => false,
    };

    if role_innate {
        return AbilitySource::Role;
    }

    AbilitySource::None
}

/// Translate extrinsic ability flag to damage type (abil_to_adtyp equivalent)
///
/// Maps property flags to attack damage types for resistance checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    Fire = 0,
    Cold = 1,
    Shock = 2,
    Acid = 3,
    Disint = 4,
    Poison = 5,
    Drain = 6,
    Magic = 7,
    None = 255,
}

pub fn property_to_damage_type(property: Property) -> DamageType {
    match property {
        Property::FireResistance => DamageType::Fire,
        Property::ColdResistance => DamageType::Cold,
        Property::ShockResistance => DamageType::Shock,
        Property::AcidResistance => DamageType::Acid,
        Property::DisintResistance => DamageType::Disint,
        Property::PoisonResistance => DamageType::Poison,
        Property::DrainResistance => DamageType::Drain,
        Property::MagicResistance => DamageType::Magic,
        _ => DamageType::None,
    }
}

/// Translate ability flag to special effect code (abil_to_spfx equivalent)
///
/// Maps property flags to artifact/item special effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialEffect {
    None = 0,
    Search = 1,
    HallucinationResist = 2,
    Telepathy = 3,
    Stealth = 4,
    Regeneration = 5,
    TeleportControl = 6,
    Warning = 7,
    EnergyRegeneration = 8,
    HalfSpellDamage = 9,
    HalfPhysDamage = 10,
    Reflection = 11,
}

pub fn property_to_spfx(property: Property) -> SpecialEffect {
    match property {
        Property::Searching => SpecialEffect::Search,
        Property::Telepathy => SpecialEffect::Telepathy,
        Property::Stealth => SpecialEffect::Stealth,
        Property::Regeneration => SpecialEffect::Regeneration,
        Property::TeleportControl => SpecialEffect::TeleportControl,
        Property::Warning => SpecialEffect::Warning,
        Property::EnergyRegeneration => SpecialEffect::EnergyRegeneration,
        Property::HalfSpellDamage => SpecialEffect::HalfSpellDamage,
        Property::HalfPhysDamage => SpecialEffect::HalfPhysDamage,
        Property::Reflection => SpecialEffect::Reflection,
        _ => SpecialEffect::None,
    }
}

/// Toggle stealth property and provide feedback (toggle_stealth equivalent - stub)
///
/// Handles messages when stealth property changes from equipment.
pub fn toggle_stealth(properties: &mut PropertySet, gaining: bool) {
    if gaining {
        properties.grant_intrinsic(Property::Stealth);
        // Message: "You move very quietly." or "You float imperceptibly."
    } else {
        properties.remove_intrinsic(Property::Stealth);
        // Message: "You sure are noisy."
    }
}

/// Toggle displacement property and provide feedback (toggle_displacement equivalent - stub)
///
/// Handles messages when displacement property changes from equipment.
pub fn toggle_displacement(properties: &mut PropertySet, gaining: bool) {
    if gaining {
        properties.grant_intrinsic(Property::Displaced);
        // Message: "You feel that monsters have difficulty pinpointing your location."
    } else {
        properties.remove_intrinsic(Property::Displaced);
        // Message: "You feel that monsters no longer have difficulty pinpointing your location."
    }
}

/// Toggle blindness and update vision (toggle_blindness equivalent - stub)
///
/// Called when blindness state changes. Updates vision and monster detection.
pub fn toggle_blindness(properties: &mut PropertySet, becoming_blind: bool) {
    if becoming_blind {
        properties.grant_intrinsic(Property::Fumbling); // Blindness causes fumbling
    // Message: update status line, recalculate vision
    } else {
        properties.remove_intrinsic(Property::Fumbling);
        // Message: restore learning of unseen inventory
    }
}

/// Start levitation effect (float_up equivalent - stub)
///
/// Called when levitation begins. Provides appropriate feedback based on situation.
pub fn float_up(properties: &mut PropertySet, in_pit: bool, trapped: bool) -> &'static str {
    properties.grant_intrinsic(Property::Levitation);

    if in_pit && !trapped {
        "You float up, out of the pit!"
    } else if trapped {
        "You feel lighter, but you're still stuck."
    } else {
        "You start to float in the air!"
    }
}

/// End levitation effect (float_down equivalent - stub)
///
/// Called when levitation ends. Returns true if still levitating (via other source).
pub fn float_down(properties: &mut PropertySet, check_flight: bool) -> bool {
    properties.remove_intrinsic(Property::Levitation);

    if check_flight && properties.has_flying() {
        return true; // Still flying, not fully down
    }

    // Check if still have levitation from another source
    properties.has(Property::Levitation)
}

/// Check levitation vs flight priority (float_vs_flight equivalent - stub)
///
/// Determines which property takes precedence when both are available.
/// Levitation overrides flying unless trapped on ground.
pub fn levitation_vs_flight(properties: &PropertySet, trapped_on_ground: bool) -> bool {
    if trapped_on_ground {
        // Trapped on ground overrides flying
        properties.has(Property::Levitation)
    } else {
        // Normal case: levitation takes priority over flying
        properties.has(Property::Levitation) || properties.has(Property::Flying)
    }
}

/// Check if floating above something (floating_above equivalent - stub)
///
/// Provides feedback message when player is floating.
pub fn floating_above(properties: &PropertySet, surface: &str) -> String {
    if properties.has(Property::Levitation) {
        format!("You are floating high above the {}.", surface)
    } else if properties.has(Property::Flying) {
        format!("You are flying above the {}.", surface)
    } else {
        format!("You are above the {}.", surface)
    }
}

/// Generate levitation timeout dialogue (levitation_dialogue equivalent - stub)
///
/// Returns periodic reminder messages when levitation is about to time out.
pub fn levitation_dialogue(
    _properties: &PropertySet,
    remaining_turns: u32,
) -> Option<&'static str> {
    let messages = [
        "You feel yourself gradually being pulled downward.",
        "You're descending slowly.",
        "You're floating lower.",
        "Your levitation is fading.",
    ];

    if remaining_turns == 0 {
        return None;
    }

    let idx = (messages.len() - 1).min(remaining_turns as usize / 2);
    Some(messages[messages.len() - idx - 1])
}

/// Generate phasing timeout dialogue (phaze_dialogue equivalent - stub)
///
/// Returns periodic reminder messages when wall phasing is about to time out.
pub fn phasing_dialogue(remaining_turns: u32) -> Option<&'static str> {
    let messages = [
        "You feel solid again.",
        "Your body feels less ethereal.",
        "The walls feel more solid.",
    ];

    if remaining_turns == 0 {
        return None;
    }

    let idx = (messages.len() - 1).min(remaining_turns as usize / 2);
    Some(messages[messages.len() - idx - 1])
}

/// Update monster intrinsics when wearing/removing items (update_mon_intrinsics equivalent - stub)
///
/// Handles property changes for monsters wearing/removing equipment.
/// This would be called when a monster equips an artifact or special item.
pub fn update_monster_properties(
    mon_properties: &mut PropertySet,
    item_property: Property,
    equipping: bool,
) {
    // Update monster's intrinsic/extrinsic properties based on item worn/removed
    // Most properties don't affect monsters, but some key ones do:

    match item_property {
        // Invisibility affects monster visibility and combat
        Property::Invisibility => {
            if equipping {
                mon_properties.grant_extrinsic_simple(Property::Invisibility);
            } else {
                mon_properties.revoke_extrinsic(Property::Invisibility);
            }
        }

        // Speed/very fast affects movement
        Property::Speed | Property::VeryFast => {
            if equipping {
                mon_properties.grant_extrinsic_simple(item_property);
            } else {
                mon_properties.revoke_extrinsic(item_property);
            }
        }

        // Protection affects AC (monsters get protection too)
        Property::Protection => {
            if equipping {
                mon_properties.grant_extrinsic_simple(Property::Protection);
            } else {
                mon_properties.revoke_extrinsic(Property::Protection);
            }
        }

        // Flying/levitation
        Property::Flying | Property::Levitation => {
            if equipping {
                mon_properties.grant_extrinsic_simple(item_property);
            } else {
                mon_properties.revoke_extrinsic(item_property);
            }
        }

        // Displacement/reflection - can affect combat
        Property::Displaced | Property::Reflection => {
            if equipping {
                mon_properties.grant_extrinsic_simple(item_property);
            } else {
                mon_properties.revoke_extrinsic(item_property);
            }
        }

        // Most resistances don't apply to monsters from items
        _ => {
            // Monsters don't gain intrinsic resistances from items in standard NetHack
            // This is simplified; full implementation would check more carefully
        }
    }
}
