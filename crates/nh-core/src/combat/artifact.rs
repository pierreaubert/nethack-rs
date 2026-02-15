//! Artifact runtime system (artifact.c)
//!
//! Tracks artifact creation, checks applicability, computes damage bonuses,
//! handles touch-blast checks, and manages artifact intrinsic properties.
//!
//! The static ARTIFACTS data lives in nh-data; this module provides the
//! runtime logic that operates on it.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use serde::{Deserialize, Serialize};

use super::{Attack, CombatEffect, DamageType};
use crate::monster::Monster;
use crate::object::Object;
use crate::player::{AlignmentType, Property, PropertyFlags, You};
use crate::rng::GameRng;

// ============================================================================
// Artifact data types (shared with nh-data)
// ============================================================================

/// Special property flags for artifacts (from artifact.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ArtifactFlags(u32);

impl ArtifactFlags {
    pub const NONE: Self = Self(0x00000000);
    /// Item is special, bequeathed by gods
    pub const NOGEN: Self = Self(0x00000001);
    /// Item is restricted - can't be named
    pub const RESTR: Self = Self(0x00000002);
    /// Item is self-willed - intelligent
    pub const INTEL: Self = Self(0x00000004);
    /// Item can speak
    pub const SPEAK: Self = Self(0x00000008);
    /// Item helps you search for things
    pub const SEEK: Self = Self(0x00000010);
    /// Item warns you of danger
    pub const WARN: Self = Self(0x00000020);
    /// Item has a special attack (attk)
    pub const ATTK: Self = Self(0x00000040);
    /// Item has a special defence (defn)
    pub const DEFN: Self = Self(0x00000080);
    /// Drains a level from monsters
    pub const DRLI: Self = Self(0x00000100);
    /// Helps searching
    pub const SEARCH: Self = Self(0x00000200);
    /// Beheads monsters
    pub const BEHEAD: Self = Self(0x00000400);
    /// Blocks hallucinations
    pub const HALRES: Self = Self(0x00000800);
    /// ESP (like amulet of ESP)
    pub const ESP: Self = Self(0x00001000);
    /// Stealth
    pub const STLTH: Self = Self(0x00002000);
    /// Regeneration
    pub const REGEN: Self = Self(0x00004000);
    /// Energy Regeneration
    pub const EREGEN: Self = Self(0x00008000);
    /// 1/2 spell damage in combat
    pub const HSPDAM: Self = Self(0x00010000);
    /// 1/2 physical damage in combat
    pub const HPHDAM: Self = Self(0x00020000);
    /// Teleportation Control
    pub const TCTRL: Self = Self(0x00040000);
    /// Increase Luck (like Luckstone)
    pub const LUCK: Self = Self(0x00080000);
    /// Attack bonus on one monster type
    pub const DMONS: Self = Self(0x00100000);
    /// Attack bonus on monsters w/ symbol mtype
    pub const DCLAS: Self = Self(0x00200000);
    /// Attack bonus on monsters w/ mflags1 flag
    pub const DFLAG1: Self = Self(0x00400000);
    /// Attack bonus on monsters w/ mflags2 flag
    pub const DFLAG2: Self = Self(0x00800000);
    /// Attack bonus on non-aligned monsters
    pub const DALIGN: Self = Self(0x01000000);
    /// Attack bonus mask (all DMONS|DCLAS|DFLAG1|DFLAG2|DALIGN)
    pub const DBONUS: Self = Self(0x01F00000);
    /// Gives X-RAY vision to player
    pub const XRAY: Self = Self(0x02000000);
    /// Reflection
    pub const REFLECT: Self = Self(0x04000000);
    /// Protection
    pub const PROTECT: Self = Self(0x08000000);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn minus(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

/// Invocation property types (from artifact.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvokeProperty {
    None,
    Taming,
    Healing,
    EnergyBoost,
    Untrap,
    ChargeObj,
    LevTele,
    CreatePortal,
    Enlightening,
    CreateAmmo,
    Invis,
    Levitation,
    Conflict,
}

/// Alignment for artifacts (None means unaligned)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAlignment {
    None,
    Lawful,
    Neutral,
    Chaotic,
}

impl ArtifactAlignment {
    /// Convert to Option<AlignmentType>
    pub const fn to_alignment_type(self) -> Option<AlignmentType> {
        match self {
            ArtifactAlignment::None => None,
            ArtifactAlignment::Lawful => Some(AlignmentType::Lawful),
            ArtifactAlignment::Neutral => Some(AlignmentType::Neutral),
            ArtifactAlignment::Chaotic => Some(AlignmentType::Chaotic),
        }
    }

    /// Check if matches a player alignment
    pub fn matches_alignment(self, align: AlignmentType) -> bool {
        match self {
            ArtifactAlignment::None => true,
            ArtifactAlignment::Lawful => align == AlignmentType::Lawful,
            ArtifactAlignment::Neutral => align == AlignmentType::Neutral,
            ArtifactAlignment::Chaotic => align == AlignmentType::Chaotic,
        }
    }
}

/// Non-PM sentinel (no specific role/race)
pub const NON_PM: i16 = -1;

/// M2 monster flags for DFLAG2 targeting
pub const M2_ELF: u32 = 0x0010;
pub const M2_ORC: u32 = 0x0020;
pub const M2_DEMON: u32 = 0x0040;
pub const M2_WERE: u32 = 0x0004;
pub const M2_UNDEAD: u32 = 0x0002;
pub const M2_GIANT: u32 = 0x0080;

/// An artifact definition
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Name of the artifact
    pub name: &'static str,
    /// Base object type index (into OBJECTS array)
    pub otyp: i16,
    /// Special effects when wielded/worn
    pub spfx: ArtifactFlags,
    /// Special effects just from carrying
    pub cspfx: ArtifactFlags,
    /// Monster type, symbol, or flag for targeting
    pub mtype: u32,
    /// Special attack when hitting
    pub attk: Attack,
    /// Passive defense effect
    pub defn: Attack,
    /// Effect from carrying
    pub cary: Attack,
    /// Property obtained by invoking artifact
    pub inv_prop: InvokeProperty,
    /// Alignment of bequeathing gods
    pub alignment: ArtifactAlignment,
    /// Character role associated with (NON_PM = any)
    pub role: i16,
    /// Character race associated with (NON_PM = any)
    pub race: i16,
    /// Price when sold to hero
    pub cost: u32,
    /// Color to use if artifact 'glows'
    pub color: u8,
}

// ============================================================================
// Artifact Tracker (artiexist[] equivalent)
// ============================================================================

/// Maximum number of artifacts we can track
pub const MAX_ARTIFACTS: usize = 64;

/// Tracks which artifacts have been created in the current game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactTracker {
    /// Whether artifact at 1-based index has been created
    created: Vec<bool>,
    /// Discovery list (artifact indices that player has identified)
    discovered: Vec<u8>,
    /// Number of artifact gifts bestowed to the player
    pub gift_count: i32,
}

impl Default for ArtifactTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ArtifactTracker {
    pub fn new() -> Self {
        Self {
            created: vec![false; MAX_ARTIFACTS],
            discovered: Vec::new(),
            gift_count: 0,
        }
    }

    /// Mark an artifact as created (1-based index)
    pub fn mark_created(&mut self, artifact_idx: u8) {
        if (artifact_idx as usize) < MAX_ARTIFACTS {
            self.created[artifact_idx as usize] = true;
        }
    }

    /// Mark an artifact as not created (e.g., when destroyed)
    pub fn mark_uncreated(&mut self, artifact_idx: u8) {
        if (artifact_idx as usize) < MAX_ARTIFACTS {
            self.created[artifact_idx as usize] = false;
        }
    }

    /// Check if an artifact has been created (1-based index)
    pub fn is_created(&self, artifact_idx: u8) -> bool {
        if (artifact_idx as usize) < MAX_ARTIFACTS {
            self.created[artifact_idx as usize]
        } else {
            false
        }
    }

    /// Count how many artifacts exist in the game
    pub fn count_created(&self) -> usize {
        // Skip index 0 (dummy)
        self.created[1..].iter().filter(|&&x| x).count()
    }

    /// Add an artifact to the discovery list
    pub fn discover(&mut self, artifact_idx: u8) {
        if !self.discovered.contains(&artifact_idx) {
            self.discovered.push(artifact_idx);
        }
    }

    /// Check if an artifact has been discovered
    pub fn is_discovered(&self, artifact_idx: u8) -> bool {
        self.discovered.contains(&artifact_idx)
    }

    /// Get the list of discovered artifacts
    pub fn discoveries(&self) -> &[u8] {
        &self.discovered
    }
}

// ============================================================================
// Artifact lookup
// ============================================================================

/// Look up artifact data from an Object's artifact index
pub fn artifact_for_object<'a>(obj: &Object, artifacts: &'a [Artifact]) -> Option<&'a Artifact> {
    if obj.artifact == 0 {
        return None;
    }
    // artifact field is 1-based index
    artifacts.get(obj.artifact as usize - 1)
}

/// Get artifact by 1-based index
pub fn artifact_by_index(artifacts: &[Artifact], idx: u8) -> Option<&Artifact> {
    if idx == 0 {
        return None;
    }
    artifacts.get(idx as usize - 1)
}

/// Find artifact index (1-based) by name
pub fn artifact_index_by_name(artifacts: &[Artifact], name: &str) -> Option<u8> {
    let name_lower = name.to_lowercase();
    let trimmed = name_lower.strip_prefix("the ").unwrap_or(&name_lower);

    for (i, art) in artifacts.iter().enumerate() {
        let art_name = art.name.to_lowercase();
        let art_trimmed = art_name.strip_prefix("the ").unwrap_or(&art_name);
        if trimmed == art_trimmed {
            return Some((i + 1) as u8); // 1-based
        }
    }
    None
}

/// Check if an object type + name combination corresponds to an existing artifact
pub fn exist_artifact(
    artifacts: &[Artifact],
    tracker: &ArtifactTracker,
    otyp: i16,
    name: &str,
) -> bool {
    for (i, art) in artifacts.iter().enumerate() {
        if art.otyp == otyp && art.name == name {
            return tracker.is_created((i + 1) as u8);
        }
    }
    false
}

// ============================================================================
// Artifact creation
// ============================================================================

/// Try to make an object into an artifact.
///
/// If `alignment` is Some, creates a gift artifact matching that alignment
/// (possibly creating a new object). If None, tries to make the given object
/// into an artifact of matching base type.
///
/// Returns true if the object was turned into an artifact.
pub fn mk_artifact(
    obj: &mut Object,
    alignment: Option<AlignmentType>,
    player_role: i16,
    _player_race: i16,
    artifacts: &[Artifact],
    tracker: &mut ArtifactTracker,
    rng: &mut GameRng,
) -> bool {
    let by_align = alignment.is_some();

    // Gather eligible artifacts
    let mut eligible: Vec<u8> = Vec::new();
    let mut fallback: Vec<u8> = Vec::new();

    for (i, art) in artifacts.iter().enumerate() {
        let idx = (i + 1) as u8;

        // Skip already created
        if tracker.is_created(idx) {
            continue;
        }

        // Skip NOGEN (quest artifacts, etc.)
        if art.spfx.contains(ArtifactFlags::NOGEN) {
            continue;
        }

        if !by_align {
            // Looking for a matching base type
            if art.otyp == obj.object_type {
                eligible.push(idx);
            }
        } else {
            // Looking for an alignment-specific gift
            let align = alignment.unwrap();
            let art_align = art.alignment.to_alignment_type();

            if art_align == Some(align) || art_align.is_none() {
                // Check role match for first choice
                if art.role == player_role {
                    // This is the preferred gift for this role
                    eligible.clear();
                    eligible.push(idx);
                    break;
                }

                if art_align.is_some() || tracker.gift_count > 0 {
                    eligible.push(idx);
                } else if eligible.is_empty() {
                    fallback.push(idx);
                }
            }
        }
    }

    // Fall back if no primary candidates
    if eligible.is_empty() {
        eligible = fallback;
    }

    if eligible.is_empty() {
        return false;
    }

    // Pick one at random
    let chosen_idx = eligible[rng.rn2(eligible.len() as u32) as usize];
    let art = &artifacts[chosen_idx as usize - 1];

    // For alignment-based gifts, the caller should create the base object
    // of the correct type. Here we just stamp the artifact onto the object.
    if by_align && obj.object_type != art.otyp {
        // Object type doesn't match - caller needs to handle this
        // Store the needed type for the caller
        obj.object_type = art.otyp;
    }

    obj.artifact = chosen_idx;
    obj.name = Some(art.name.to_string());
    tracker.mark_created(chosen_idx);

    true
}

// ============================================================================
// Touch / blast check
// ============================================================================

/// Result of touching an artifact
#[derive(Debug, Clone)]
pub struct TouchResult {
    /// Whether the player can hold the artifact
    pub can_hold: bool,
    /// Blast damage dealt (0 if none)
    pub blast_damage: i32,
    /// Message to display
    pub message: Option<String>,
}

/// Check what happens when a creature tries to touch/pick up an artifact.
///
/// Based on touch_artifact() in artifact.c.
/// Returns damage and whether the creature can hold it.
pub fn touch_artifact(
    obj: &Object,
    player: &You,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> TouchResult {
    let art = match artifact_for_object(obj, artifacts) {
        Some(a) => a,
        None => {
            return TouchResult {
                can_hold: true,
                blast_damage: 0,
                message: None,
            }
        }
    };

    let self_willed = art.spfx.contains(ArtifactFlags::INTEL);

    // Check if wrong class (role/race) for self-willed artifacts
    let badclass = self_willed
        && ((art.role != NON_PM && art.role != player.role.role_id())
            || (art.race != NON_PM && art.race != player.race.race_id()));

    // Check if wrong alignment for restricted artifacts
    let badalign = art.spfx.contains(ArtifactFlags::RESTR)
        && art.alignment != ArtifactAlignment::None
        && (!art.alignment.matches_alignment(player.alignment.typ)
            || player.alignment.record < 0);

    // Bane artifacts are bad for matching targets
    let bane_bad = !badalign && bane_applies(art, player);

    let badalign = badalign || bane_bad;

    if ((badclass || badalign) && self_willed) || (badalign && rng.rn2(4) == 0) {
        // Blast the player
        let has_mr = player.properties.has(Property::MagicResistance);
        let dmg_dice: u32 = if has_mr { 2 } else { 4 };
        let dmg_sides: u32 = if self_willed { 10 } else { 4 };
        let damage = rng.dice(dmg_dice, dmg_sides) as i32;

        let msg = format!(
            "You are blasted by {}'s power!",
            obj.name.as_deref().unwrap_or("the artifact")
        );

        // Can pick it up unless totally non-synched
        let can_hold = !(badclass && badalign && self_willed);

        return TouchResult {
            can_hold,
            blast_damage: damage,
            message: Some(msg),
        };
    }

    TouchResult {
        can_hold: true,
        blast_damage: 0,
        message: None,
    }
}

/// Check if an artifact's bane (targeting bonus) applies against the player
fn bane_applies(art: &Artifact, player: &You) -> bool {
    if !art.spfx.intersects(ArtifactFlags::DBONUS) {
        return false;
    }

    if art.spfx.contains(ArtifactFlags::DFLAG2) {
        // Check if player matches the targeted M2 flag
        let race_id = player.race.race_id();
        if art.mtype == M2_ELF && race_id == 101 {
            return true;
        }
        if art.mtype == M2_ORC && race_id == 104 {
            return true;
        }
        if art.mtype == M2_WERE && player.lycanthropy.is_some() {
            return true;
        }
        // Undead/demon don't apply to normal players
    }

    if art.spfx.contains(ArtifactFlags::DALIGN) {
        // Non-aligned weapons hurt those of different alignment
        if let Some(art_align) = art.alignment.to_alignment_type() {
            return player.alignment.typ != art_align;
        }
    }

    false
}

// ============================================================================
// Spec applies / damage bonus
// ============================================================================

/// Check if an artifact's special attacks apply against a monster target.
///
/// Based on spec_applies() in artifact.c.
pub fn spec_applies(art: &Artifact, target: &Monster) -> bool {
    // If no damage bonus flags and no attack flag, just check physical
    if !art.spfx.intersects(ArtifactFlags::DBONUS.union(ArtifactFlags::ATTK)) {
        return art.attk.damage_type == DamageType::Physical;
    }

    if art.spfx.contains(ArtifactFlags::DMONS) {
        return target.monster_type == art.mtype as i16;
    }

    if art.spfx.contains(ArtifactFlags::DCLAS) {
        // mtype is the monster class letter as u32
        // Check if target's class letter matches
        let target_letter = monster_class_letter(target.monster_type);
        return target_letter == art.mtype;
    }

    if art.spfx.contains(ArtifactFlags::DFLAG1) {
        return (target.flags.bits() as u32 & art.mtype) != 0;
    }

    if art.spfx.contains(ArtifactFlags::DFLAG2) {
        // Check M2 flags
        let m2_bits = monster_m2_flags(target);
        return (m2_bits & art.mtype) != 0;
    }

    if art.spfx.contains(ArtifactFlags::DALIGN) {
        // Artifact hurts monsters of different alignment
        if let Some(art_align) = art.alignment.to_alignment_type() {
            let mon_align = AlignmentType::from_value(target.alignment);
            return mon_align != art_align;
        }
        return false;
    }

    if art.spfx.contains(ArtifactFlags::ATTK) {
        // Check if the target resists the attack type
        match art.attk.damage_type {
            DamageType::Fire => return !target.resists_fire(),
            DamageType::Cold => return !target.resists_cold(),
            DamageType::Electric => return !target.resists_elec(),
            DamageType::MagicMissile | DamageType::Stun => {
                // Magic resistance check (simplified: high-level monsters resist)
                return target.level < 10 || target.ac > 0;
            }
            DamageType::DrainStrength => return !target.resists_poison(),
            DamageType::DrainLife => {
                // Undead and demons resist drain life (simplified)
                return !target.is_undead() && !target.is_demon();
            }
            DamageType::Stone => return !target.resists_stone(),
            _ => {}
        }
    }

    false
}

/// Get the M2 flags for a monster (simplified - uses Monster struct flags)
fn monster_m2_flags(target: &Monster) -> u32 {
    use crate::monster::MonsterFlags;
    let mut flags: u32 = 0;
    if target.flags.contains(MonsterFlags::UNDEAD) {
        flags |= M2_UNDEAD;
    }
    if target.flags.contains(MonsterFlags::DEMON) {
        flags |= M2_DEMON;
    }
    // Additional M2 flags based on monster type ranges (simplified)
    // In full implementation these come from PerMonst data
    flags
}

/// Get the monster class letter from type index (simplified)
fn monster_class_letter(monster_type: i16) -> u32 {
    // In full implementation, look up from MONSTERS[monster_type].mlet
    // Simplified ranges based on NetHack monster ordering
    match monster_type {
        200..=209 => 'D' as u32, // Dragons
        210..=219 => 'O' as u32, // Ogres
        220..=229 => 'T' as u32, // Trolls
        _ => 0,
    }
}

/// Calculate artifact special damage bonus against a target.
///
/// Based on spec_dbon() in artifact.c.
/// Returns (bonus_damage, spec_dbon_applies)
pub fn spec_dbon(
    _obj: &Object,
    art: &Artifact,
    target: &Monster,
    base_dmg: i32,
    rng: &mut GameRng,
) -> (i32, bool) {
    // Check for NO_ATTK
    if art.attk.damage_type == DamageType::Physical
        && art.attk.dice_num == 0
        && art.attk.dice_sides == 0
    {
        return (0, false);
    }

    // Grimtooth always applies damage bonus
    let applies = if art.name == "Grimtooth" {
        true
    } else {
        spec_applies(art, target)
    };

    if applies {
        let bonus = if art.attk.dice_sides > 0 {
            rng.dice(art.attk.dice_num as u32, art.attk.dice_sides as u32) as i32
        } else {
            base_dmg.max(1)
        };
        (bonus, true)
    } else {
        (0, false)
    }
}

/// Calculate artifact special attack (to-hit) bonus
pub fn spec_abon(art: &Artifact, target: &Monster, rng: &mut GameRng) -> i32 {
    if art.attk.dice_num > 0 && spec_applies(art, target) {
        rng.dice(1, art.attk.dice_num as u32) as i32
    } else {
        0
    }
}

// ============================================================================
// Artifact hit effects
// ============================================================================

/// Result of an artifact's special hit effects
#[derive(Debug, Clone, Default)]
pub struct ArtifactHitResult {
    /// Whether the artifact produced a visible/notable effect
    pub had_effect: bool,
    /// Extra damage to add
    pub extra_damage: i32,
    /// Whether the defender should die regardless of HP
    pub instant_kill: bool,
    /// Messages to display
    pub messages: Vec<String>,
    /// Special combat effects
    pub effects: Vec<CombatEffect>,
}

/// Damage modifier to ensure kills even through damage reduction
const FATAL_DAMAGE_MODIFIER: i32 = 200;

/// Apply artifact hit effects after a successful melee hit.
///
/// Based on artifact_hit() in artifact.c.
/// Modifies damage and returns hit result with messages and effects.
pub fn artifact_hit(
    obj: &Object,
    target: &Monster,
    dmg: &mut i32,
    dieroll: i32,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> ArtifactHitResult {
    let art = match artifact_for_object(obj, artifacts) {
        Some(a) => a,
        None => return ArtifactHitResult::default(),
    };

    let mut result = ArtifactHitResult::default();

    // Apply spec_dbon damage
    let (bonus, applies) = spec_dbon(obj, art, target, *dmg, rng);
    *dmg += bonus;

    let target_name = &target.name;

    // Elemental attacks: fire, cold, elec, magic missile
    if art.spfx.contains(ArtifactFlags::ATTK) {
        match art.attk.damage_type {
            DamageType::Fire => {
                if applies {
                    result.messages.push(format!(
                        "The fiery blade burns {}!",
                        target_name
                    ));
                } else {
                    result.messages.push(format!(
                        "The fiery blade hits {}.",
                        target_name
                    ));
                }
                result.had_effect = true;
                return result;
            }
            DamageType::Cold => {
                if applies {
                    result.messages.push(format!(
                        "The ice-cold blade freezes {}!",
                        target_name
                    ));
                } else {
                    result.messages.push(format!(
                        "The ice-cold blade hits {}.",
                        target_name
                    ));
                }
                result.had_effect = true;
                return result;
            }
            DamageType::Electric => {
                if applies {
                    result.messages.push(format!(
                        "The massive hammer hits!  Lightning strikes {}!",
                        target_name
                    ));
                } else {
                    result.messages.push(format!(
                        "The massive hammer hits {}.",
                        target_name
                    ));
                }
                result.had_effect = true;
                return result;
            }
            DamageType::MagicMissile => {
                if applies {
                    result.messages.push(format!(
                        "A hail of magic missiles strikes {}!",
                        target_name
                    ));
                }
                result.had_effect = true;
                return result;
            }
            DamageType::Stun => {
                // Magicbane special effects
                if dieroll <= 8 {
                    return magicbane_hit(target, dmg, dieroll, rng);
                }
            }
            _ => {}
        }
    }

    if !applies {
        return result;
    }

    // Vorpal Blade / Tsurugi beheading
    if art.spfx.contains(ArtifactFlags::BEHEAD) {
        if art.name == "The Tsurugi of Muramasa" && dieroll == 1 {
            // Bisection
            if target.hp_max > 50 {
                // Big monster: double damage
                result
                    .messages
                    .push(format!("You slice deeply into {}!", target_name));
                *dmg *= 2;
            } else {
                // Small monster: instant kill
                *dmg = 2 * target.hp + FATAL_DAMAGE_MODIFIER;
                result.messages.push(format!(
                    "The razor-sharp blade cuts {} in half!",
                    target_name
                ));
                result.instant_kill = true;
            }
            result.had_effect = true;
            return result;
        } else if art.name == "Vorpal Blade" && dieroll == 1 {
            // Beheading
            // Check if target has a head (simplified: most monsters do)
            let has_head = !target.flags.contains(crate::monster::MonsterFlags::NOHEAD)
                && !target.flags.contains(crate::monster::MonsterFlags::AMORPHOUS);
            if has_head {
                *dmg = 2 * target.hp + FATAL_DAMAGE_MODIFIER;
                let msg = if rng.rn2(2) == 0 {
                    format!("Vorpal Blade beheads {}!", target_name)
                } else {
                    format!("Vorpal Blade decapitates {}!", target_name)
                };
                result.messages.push(msg);
                result.instant_kill = true;
            } else {
                result.messages.push(format!(
                    "Vorpal Blade slices through {}.",
                    target_name
                ));
            }
            result.had_effect = true;
            return result;
        }
    }

    // Life drain (Stormbringer, Staff of Aesculapius)
    if art.spfx.contains(ArtifactFlags::DRLI) {
        if target.level == 0 {
            *dmg = 2 * target.hp + FATAL_DAMAGE_MODIFIER;
            result.instant_kill = true;
        } else {
            let drain = (target.hp_max / target.level as i32).max(1);
            *dmg += drain;
            result.extra_damage = drain;
        }

        if art.name == "Stormbringer" {
            result.messages.push(format!(
                "The black blade draws the life from {}!",
                target_name
            ));
        } else {
            result.messages.push(format!(
                "{} draws the life from {}!",
                obj.name.as_deref().unwrap_or("The artifact"),
                target_name
            ));
        }
        result.effects.push(CombatEffect::Drained);
        result.had_effect = true;
        return result;
    }

    result
}

/// Magicbane special hit effects
fn magicbane_hit(
    target: &Monster,
    dmg: &mut i32,
    dieroll: i32,
    rng: &mut GameRng,
) -> ArtifactHitResult {
    let mut result = ArtifactHitResult::default();

    // Magicbane has special effects based on die roll
    // Lower rolls = stronger effects
    if dieroll <= 2 {
        // Cancel (strongest)
        result
            .messages
            .push(format!("The magic-Loss blade cancels {}!", target.name));
        result.effects.push(CombatEffect::Confused);
        *dmg += rng.dice(1, 4) as i32;
    } else if dieroll <= 4 {
        // Stun
        result
            .messages
            .push(format!("The magic blade stuns {}!", target.name));
        result.effects.push(CombatEffect::Stunned);
        *dmg += rng.dice(1, 4) as i32;
    } else if dieroll <= 6 {
        // Scare/probe
        result.messages.push(format!(
            "The magic blade scares {}!",
            target.name
        ));
        *dmg += rng.dice(1, 4) as i32;
    } else {
        // Regular extra damage
        *dmg += rng.dice(1, 4) as i32;
    }

    result.had_effect = true;
    result
}

// ============================================================================
// Artifact defense checks
// ============================================================================

/// Check if a wielded artifact defends against a damage type
pub fn defends(damage_type: DamageType, obj: &Object, artifacts: &[Artifact]) -> bool {
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.defn.damage_type == damage_type
    } else {
        false
    }
}

/// Check if a carried artifact defends against a damage type
pub fn defends_when_carried(damage_type: DamageType, obj: &Object, artifacts: &[Artifact]) -> bool {
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.cary.damage_type == damage_type
    } else {
        false
    }
}

/// Check if an artifact is immune to a type of erosion damage
pub fn arti_immune(obj: &Object, damage_type: DamageType, artifacts: &[Artifact]) -> bool {
    if damage_type == DamageType::Physical {
        return false;
    }
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.attk.damage_type == damage_type
            || art.defn.damage_type == damage_type
            || art.cary.damage_type == damage_type
    } else {
        false
    }
}

/// Check if an artifact reflects
pub fn arti_reflects(obj: &Object, artifacts: &[Artifact]) -> bool {
    if let Some(art) = artifact_for_object(obj, artifacts) {
        // Reflects when wielded/worn
        if obj.is_worn() && art.spfx.contains(ArtifactFlags::REFLECT) {
            return true;
        }
        // Reflects just from carrying
        if art.cspfx.contains(ArtifactFlags::REFLECT) {
            return true;
        }
    }
    false
}

/// Check if an artifact confers luck
pub fn confers_luck(obj: &Object, artifacts: &[Artifact]) -> bool {
    // Luckstones always confer luck (object_type check elsewhere)
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.spfx.contains(ArtifactFlags::LUCK) || art.cspfx.contains(ArtifactFlags::LUCK)
    } else {
        false
    }
}

/// Check if an artifact confers protection
pub fn protects(obj: &Object, being_worn: bool, artifacts: &[Artifact]) -> bool {
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.cspfx.contains(ArtifactFlags::PROTECT)
            || (being_worn && art.spfx.contains(ArtifactFlags::PROTECT))
    } else {
        false
    }
}

/// Check if a specific ability is granted by an artifact
pub fn spec_ability(obj: &Object, ability: ArtifactFlags, artifacts: &[Artifact]) -> bool {
    if let Some(art) = artifact_for_object(obj, artifacts) {
        art.spfx.contains(ability)
    } else {
        false
    }
}

// ============================================================================
// Artifact intrinsic property management
// ============================================================================

/// Properties granted by an artifact
#[derive(Debug, Clone, Default)]
pub struct ArtifactPropertyGrants {
    /// Properties to grant from wielding/wearing (spfx)
    pub wielded: Vec<Property>,
    /// Properties to grant from just carrying (cspfx)
    pub carried: Vec<Property>,
    /// Resistance from defn field (wielded)
    pub wielded_resistance: Option<Property>,
    /// Resistance from cary field (carried)
    pub carried_resistance: Option<Property>,
}

/// Get the properties granted by an artifact
pub fn artifact_properties(obj: &Object, artifacts: &[Artifact]) -> ArtifactPropertyGrants {
    let mut grants = ArtifactPropertyGrants::default();

    let art = match artifact_for_object(obj, artifacts) {
        Some(a) => a,
        None => return grants,
    };

    // Properties from spfx (wielded/worn)
    collect_spfx_properties(art.spfx, &mut grants.wielded);

    // Properties from cspfx (carried)
    collect_spfx_properties(art.cspfx, &mut grants.carried);

    // Resistance from defn field (wielded)
    grants.wielded_resistance = damage_type_to_resistance(art.defn.damage_type);

    // Resistance from cary field (carried)
    grants.carried_resistance = damage_type_to_resistance(art.cary.damage_type);

    grants
}

/// Convert artifact spfx flags to player properties
fn collect_spfx_properties(spfx: ArtifactFlags, props: &mut Vec<Property>) {
    if spfx.contains(ArtifactFlags::SEARCH) {
        props.push(Property::Searching);
    }
    if spfx.contains(ArtifactFlags::ESP) {
        props.push(Property::Telepathy);
    }
    if spfx.contains(ArtifactFlags::STLTH) {
        props.push(Property::Stealth);
    }
    if spfx.contains(ArtifactFlags::REGEN) {
        props.push(Property::Regeneration);
    }
    if spfx.contains(ArtifactFlags::EREGEN) {
        props.push(Property::EnergyRegeneration);
    }
    if spfx.contains(ArtifactFlags::TCTRL) {
        props.push(Property::TeleportControl);
    }
    if spfx.contains(ArtifactFlags::WARN) {
        props.push(Property::Warning);
    }
    if spfx.contains(ArtifactFlags::HSPDAM) {
        props.push(Property::HalfSpellDamage);
    }
    if spfx.contains(ArtifactFlags::HPHDAM) {
        props.push(Property::HalfPhysDamage);
    }
    if spfx.contains(ArtifactFlags::XRAY) {
        props.push(Property::Xray);
    }
    if spfx.contains(ArtifactFlags::REFLECT) {
        props.push(Property::Reflection);
    }
    if spfx.contains(ArtifactFlags::PROTECT) {
        props.push(Property::Protection);
    }
}

/// Convert a damage type to its corresponding resistance property
fn damage_type_to_resistance(dt: DamageType) -> Option<Property> {
    match dt {
        DamageType::Fire => Some(Property::FireResistance),
        DamageType::Cold => Some(Property::ColdResistance),
        DamageType::Electric => Some(Property::ShockResistance),
        DamageType::MagicMissile => Some(Property::MagicResistance),
        DamageType::DrainStrength => Some(Property::PoisonResistance),
        DamageType::DrainLife => Some(Property::DrainResistance),
        DamageType::Stone => Some(Property::StoneResistance),
        _ => None,
    }
}

/// Apply or remove artifact intrinsic properties on the player.
///
/// Called when an artifact is wielded/worn/picked up or
/// unwielded/removed/dropped.
///
/// Based on set_artifact_intrinsic() in artifact.c.
pub fn set_artifact_intrinsic(
    obj: &Object,
    on: bool,
    worn_mask: u32,
    player: &mut You,
    artifacts: &[Artifact],
) {
    let art = match artifact_for_object(obj, artifacts) {
        Some(a) => a,
        None => return,
    };

    // W_ART mask indicates "just being carried" (not wielded/worn)
    const W_ART: u32 = 0x10000000;
    let is_carried_only = worn_mask == W_ART;

    // Defense resistance: wielded uses defn, carried uses cary
    let defense_type = if !is_carried_only {
        art.defn.damage_type
    } else {
        art.cary.damage_type
    };

    if let Some(resistance) = damage_type_to_resistance(defense_type) {
        let source = PropertyFlags::FROM_ARTIFACT;
        if on {
            player.properties.grant_extrinsic(resistance, source);
        } else {
            player.properties.remove_extrinsic(resistance, source);
        }
    }

    // Spfx properties: wielded/worn uses spfx, carried uses cspfx
    let spfx = if !is_carried_only {
        art.spfx
    } else {
        art.cspfx
    };

    let mut props = Vec::new();
    collect_spfx_properties(spfx, &mut props);

    let source = PropertyFlags::FROM_ARTIFACT;
    for prop in props {
        if on {
            player.properties.grant_extrinsic(prop, source);
        } else {
            player.properties.remove_extrinsic(prop, source);
        }
    }
}

// ============================================================================
// Artifact invocation
// ============================================================================

/// Result of invoking an artifact
#[derive(Debug, Clone)]
pub struct InvokeResult {
    pub success: bool,
    pub messages: Vec<String>,
    pub healing: i32,
    pub energy_boost: i32,
}

/// Invoke an artifact's special power.
///
/// Based on arti_invoke() in artifact.c.
pub fn arti_invoke(
    obj: &Object,
    player: &mut You,
    artifacts: &[Artifact],
    rng: &mut GameRng,
) -> InvokeResult {
    let mut result = InvokeResult {
        success: false,
        messages: Vec::new(),
        healing: 0,
        energy_boost: 0,
    };

    let art = match artifact_for_object(obj, artifacts) {
        Some(a) => a,
        None => {
            result.messages.push("Nothing happens.".to_string());
            return result;
        }
    };

    match art.inv_prop {
        InvokeProperty::None => {
            result.messages.push("Nothing happens.".to_string());
        }
        InvokeProperty::Healing => {
            let heal_amount = player.hp_max - player.hp;
            if heal_amount > 0 {
                player.hp = player.hp_max;
                result.healing = heal_amount;
                result.messages.push("You feel much better.".to_string());
                result.success = true;
            } else {
                result.messages.push("You feel quite well already.".to_string());
            }
            // Cure blindness, sickness, etc.
            player.sick = 0;
            player.sick_reason = None;
            player.blinded_timeout = 0;
        }
        InvokeProperty::EnergyBoost => {
            let boost = rng.dice(4, 6) as i32 + player.exp_level;
            player.energy = (player.energy + boost).min(player.energy_max * 2);
            result.energy_boost = boost;
            result
                .messages
                .push("You feel a surge of magical energy!".to_string());
            result.success = true;
        }
        InvokeProperty::Taming => {
            result
                .messages
                .push("You feel a wave of calming influence go out.".to_string());
            result.success = true;
        }
        InvokeProperty::Untrap => {
            if player.utrap > 0 {
                player.utrap = 0;
                player.utrap_type = crate::player::PlayerTrapType::None;
                result.messages.push("You are freed!".to_string());
                result.success = true;
            } else {
                result
                    .messages
                    .push("You feel like a master thief.".to_string());
                result.success = true;
            }
        }
        InvokeProperty::ChargeObj => {
            result
                .messages
                .push("You feel a surge of power.".to_string());
            result.success = true;
        }
        InvokeProperty::LevTele => {
            result
                .messages
                .push("You feel a wrenching sensation.".to_string());
            result.success = true;
        }
        InvokeProperty::CreatePortal => {
            result
                .messages
                .push("You feel the magic portal forming.".to_string());
            result.success = true;
        }
        InvokeProperty::Enlightening => {
            result
                .messages
                .push("You feel self-knowledgeable.".to_string());
            result.success = true;
        }
        InvokeProperty::CreateAmmo => {
            result
                .messages
                .push("A shower of arrows appears!".to_string());
            result.success = true;
        }
        InvokeProperty::Invis => {
            if !player.properties.has(Property::Invisibility) {
                player.properties.set_timeout(Property::Invisibility, 100);
                result.messages.push("You vanish!".to_string());
            } else {
                result.messages.push("You feel quite invisible already.".to_string());
            }
            result.success = true;
        }
        InvokeProperty::Levitation => {
            if !player.properties.has(Property::Levitation) {
                player.properties.set_timeout(Property::Levitation, 100);
                result.messages.push("You float up!".to_string());
            } else {
                result.messages.push("You are already levitating.".to_string());
            }
            result.success = true;
        }
        InvokeProperty::Conflict => {
            if !player.properties.has(Property::Conflict) {
                player.properties.set_timeout(Property::Conflict, 50);
                result
                    .messages
                    .push("You feel like a rabble-rouser.".to_string());
            } else {
                result
                    .messages
                    .push("You feel conflicted already.".to_string());
            }
            result.success = true;
        }
    }

    result
}

// ============================================================================
// Naming restriction check
// ============================================================================

/// Check if a name is restricted for an object type.
/// Returns true if the name matches a restricted artifact.
pub fn restrict_name(obj: &Object, name: &str, artifacts: &[Artifact]) -> bool {
    if name.is_empty() {
        return false;
    }

    let name_lower = name.to_lowercase();
    let name_trimmed = name_lower.strip_prefix("the ").unwrap_or(&name_lower);

    for art in artifacts {
        if art.otyp != obj.object_type {
            continue;
        }
        let art_name = art.name.to_lowercase();
        let art_trimmed = art_name.strip_prefix("the ").unwrap_or(&art_name);
        if name_trimmed == art_trimmed {
            return art
                .spfx
                .intersects(ArtifactFlags::NOGEN.union(ArtifactFlags::RESTR))
                || obj.quantity > 1;
        }
    }

    false
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{AttackType, DamageType};
    use crate::monster::{Monster, MonsterId, MonsterResistances};

    /// Helper: create Excalibur-like artifact for testing
    fn test_excalibur() -> Artifact {
        Artifact {
            name: "Excalibur",
            otyp: 10, // Long sword
            spfx: ArtifactFlags::NOGEN
                .union(ArtifactFlags::RESTR)
                .union(ArtifactFlags::SEEK)
                .union(ArtifactFlags::DEFN)
                .union(ArtifactFlags::INTEL)
                .union(ArtifactFlags::SEARCH),
            cspfx: ArtifactFlags::NONE,
            mtype: 0,
            attk: Attack::new(AttackType::None, DamageType::Physical, 5, 10),
            defn: Attack::new(AttackType::None, DamageType::DrainLife, 0, 0),
            cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            inv_prop: InvokeProperty::None,
            alignment: ArtifactAlignment::Lawful,
            role: 4, // PM_KNIGHT
            race: NON_PM,
            cost: 4000,
            color: 0,
        }
    }

    /// Helper: create Stormbringer-like artifact
    fn test_stormbringer() -> Artifact {
        Artifact {
            name: "Stormbringer",
            otyp: 20, // Runesword
            spfx: ArtifactFlags::RESTR
                .union(ArtifactFlags::ATTK)
                .union(ArtifactFlags::DEFN)
                .union(ArtifactFlags::INTEL)
                .union(ArtifactFlags::DRLI),
            cspfx: ArtifactFlags::NONE,
            mtype: 0,
            attk: Attack::new(AttackType::None, DamageType::DrainLife, 5, 2),
            defn: Attack::new(AttackType::None, DamageType::DrainLife, 0, 0),
            cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            inv_prop: InvokeProperty::None,
            alignment: ArtifactAlignment::Chaotic,
            role: NON_PM,
            race: NON_PM,
            cost: 8000,
            color: 0,
        }
    }

    /// Helper: create Frost Brand-like artifact
    fn test_frost_brand() -> Artifact {
        Artifact {
            name: "Frost Brand",
            otyp: 10,
            spfx: ArtifactFlags::RESTR
                .union(ArtifactFlags::ATTK)
                .union(ArtifactFlags::DEFN),
            cspfx: ArtifactFlags::NONE,
            mtype: 0,
            attk: Attack::new(AttackType::None, DamageType::Cold, 5, 0),
            defn: Attack::new(AttackType::None, DamageType::Cold, 0, 0),
            cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            inv_prop: InvokeProperty::None,
            alignment: ArtifactAlignment::None,
            role: NON_PM,
            race: NON_PM,
            cost: 3000,
            color: 0,
        }
    }

    /// Helper: create a test monster
    fn test_monster() -> Monster {
        let mut m = Monster::new(MonsterId(1), 100, 5, 5);
        m.name = "orc".to_string();
        m.hp = 20;
        m.hp_max = 20;
        m.level = 3;
        m
    }

    fn test_artifacts() -> Vec<Artifact> {
        vec![test_excalibur(), test_stormbringer(), test_frost_brand()]
    }

    #[test]
    fn test_artifact_tracker_basic() {
        let mut tracker = ArtifactTracker::new();
        assert!(!tracker.is_created(1));
        assert_eq!(tracker.count_created(), 0);

        tracker.mark_created(1);
        assert!(tracker.is_created(1));
        assert!(!tracker.is_created(2));
        assert_eq!(tracker.count_created(), 1);

        tracker.mark_uncreated(1);
        assert!(!tracker.is_created(1));
        assert_eq!(tracker.count_created(), 0);
    }

    #[test]
    fn test_artifact_discovery() {
        let mut tracker = ArtifactTracker::new();
        assert!(!tracker.is_discovered(1));

        tracker.discover(1);
        assert!(tracker.is_discovered(1));
        assert!(!tracker.is_discovered(2));

        // Duplicate discover is idempotent
        tracker.discover(1);
        assert_eq!(tracker.discoveries().len(), 1);
    }

    #[test]
    fn test_artifact_for_object() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 0;
        assert!(artifact_for_object(&obj, &artifacts).is_none());

        obj.artifact = 1; // Excalibur (1-based)
        let art = artifact_for_object(&obj, &artifacts).unwrap();
        assert_eq!(art.name, "Excalibur");

        obj.artifact = 2; // Stormbringer
        let art = artifact_for_object(&obj, &artifacts).unwrap();
        assert_eq!(art.name, "Stormbringer");
    }

    #[test]
    fn test_artifact_index_by_name() {
        let artifacts = test_artifacts();

        assert_eq!(artifact_index_by_name(&artifacts, "Excalibur"), Some(1));
        assert_eq!(artifact_index_by_name(&artifacts, "Stormbringer"), Some(2));
        assert_eq!(artifact_index_by_name(&artifacts, "Frost Brand"), Some(3));
        assert_eq!(artifact_index_by_name(&artifacts, "Nonexistent"), None);

        // Case insensitive
        assert_eq!(artifact_index_by_name(&artifacts, "excalibur"), Some(1));
    }

    #[test]
    fn test_defends() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 1; // Excalibur - defends vs DrainLife
        assert!(defends(DamageType::DrainLife, &obj, &artifacts));
        assert!(!defends(DamageType::Fire, &obj, &artifacts));

        obj.artifact = 3; // Frost Brand - defends vs Cold
        assert!(defends(DamageType::Cold, &obj, &artifacts));
        assert!(!defends(DamageType::Fire, &obj, &artifacts));
    }

    #[test]
    fn test_arti_immune() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 3; // Frost Brand
        // Immune to cold (from attk and defn)
        assert!(arti_immune(&obj, DamageType::Cold, &artifacts));
        // Not immune to fire
        assert!(!arti_immune(&obj, DamageType::Fire, &artifacts));
        // Nothing immune to physical
        assert!(!arti_immune(&obj, DamageType::Physical, &artifacts));
    }

    #[test]
    fn test_spec_dbon_frost_brand() {
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 3; // Frost Brand

        // Monster without cold resistance should take bonus damage
        let target = test_monster();
        let (bonus, applies) = spec_dbon(&obj, &artifacts[2], &target, 5, &mut rng);
        assert!(applies);
        assert!(bonus >= 1); // dice_sides is 0, so bonus = max(base_dmg, 1) = 5
    }

    #[test]
    fn test_spec_dbon_vs_resistant() {
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 3; // Frost Brand - cold attack

        // Monster WITH cold resistance should NOT take bonus damage
        let mut target = test_monster();
        target.resistances = MonsterResistances::COLD;
        let (bonus, applies) = spec_dbon(&obj, &artifacts[2], &target, 5, &mut rng);
        assert!(!applies);
        assert_eq!(bonus, 0);
    }

    #[test]
    fn test_mk_artifact_by_type() {
        let artifacts = test_artifacts();
        let mut tracker = ArtifactTracker::new();
        let mut rng = GameRng::new(42);

        // Create an object matching Frost Brand's base type
        let mut obj = Object::default();
        obj.object_type = 10; // Long sword (matches Excalibur and Frost Brand)

        // Excalibur is NOGEN so only Frost Brand should be eligible
        let result = mk_artifact(&mut obj, None, NON_PM, NON_PM, &artifacts, &mut tracker, &mut rng);
        assert!(result);
        assert_eq!(obj.artifact, 3); // Frost Brand
        assert!(tracker.is_created(3));
    }

    #[test]
    fn test_mk_artifact_by_alignment() {
        let artifacts = test_artifacts();
        let mut tracker = ArtifactTracker::new();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();

        // Request a chaotic artifact
        let result = mk_artifact(
            &mut obj,
            Some(AlignmentType::Chaotic),
            NON_PM,
            NON_PM,
            &artifacts,
            &mut tracker,
            &mut rng,
        );
        assert!(result);
        assert_eq!(obj.artifact, 2); // Stormbringer (only non-NOGEN chaotic)
    }

    #[test]
    fn test_mk_artifact_none_available() {
        let artifacts = test_artifacts();
        let mut tracker = ArtifactTracker::new();
        let mut rng = GameRng::new(42);

        // Mark all non-NOGEN artifacts as created
        tracker.mark_created(2); // Stormbringer
        tracker.mark_created(3); // Frost Brand

        let mut obj = Object::default();
        obj.object_type = 10;

        // No more artifacts available
        let result = mk_artifact(&mut obj, None, NON_PM, NON_PM, &artifacts, &mut tracker, &mut rng);
        assert!(!result);
        assert_eq!(obj.artifact, 0); // Unchanged
    }

    #[test]
    fn test_artifact_properties() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 1; // Excalibur - SEARCH in spfx

        let grants = artifact_properties(&obj, &artifacts);
        assert!(grants.wielded.contains(&Property::Searching));
        assert_eq!(
            grants.wielded_resistance,
            Some(Property::DrainResistance)
        );
    }

    #[test]
    fn test_set_artifact_intrinsic() {
        let artifacts = test_artifacts();

        let mut player = You::default();
        let mut obj = Object::default();
        obj.artifact = 1; // Excalibur

        // Equip: should grant Searching and drain resistance
        set_artifact_intrinsic(&obj, true, 0x8000, &mut player, &artifacts);
        assert!(player.properties.has(Property::Searching));
        assert!(player.properties.has(Property::DrainResistance));

        // Unequip: should remove
        set_artifact_intrinsic(&obj, false, 0x8000, &mut player, &artifacts);
        assert!(!player.properties.has(Property::Searching));
        assert!(!player.properties.has(Property::DrainResistance));
    }

    #[test]
    fn test_restrict_name() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.object_type = 10; // Long sword

        // Can't name a long sword "Excalibur" (NOGEN + RESTR)
        assert!(restrict_name(&obj, "Excalibur", &artifacts));

        // Can't name a long sword "Frost Brand" (RESTR)
        assert!(restrict_name(&obj, "Frost Brand", &artifacts));

        // Different base type is fine
        obj.object_type = 5; // Not a long sword
        assert!(!restrict_name(&obj, "Excalibur", &artifacts));

        // Empty name is fine
        assert!(!restrict_name(&obj, "", &artifacts));
    }

    #[test]
    fn test_artifact_hit_frost_brand() {
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 3; // Frost Brand
        obj.name = Some("Frost Brand".to_string());

        let target = test_monster();
        let mut dmg = 8;

        let result = artifact_hit(&obj, &target, &mut dmg, 10, &artifacts, &mut rng);
        assert!(result.had_effect);
        assert!(dmg > 8); // Should have added bonus damage
        assert!(!result.messages.is_empty());
    }

    #[test]
    fn test_artifact_hit_stormbringer_drain() {
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 2; // Stormbringer
        obj.name = Some("Stormbringer".to_string());

        let mut target = test_monster();
        target.level = 5;
        target.hp = 30;
        target.hp_max = 30;
        let mut dmg = 8;

        let result = artifact_hit(&obj, &target, &mut dmg, 10, &artifacts, &mut rng);
        assert!(result.had_effect);
        assert!(result.effects.contains(&CombatEffect::Drained));
        assert!(result.messages.iter().any(|m| m.contains("black blade")));
    }

    #[test]
    fn test_confers_luck() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 0;
        assert!(!confers_luck(&obj, &artifacts));

        // Excalibur doesn't confer luck
        obj.artifact = 1;
        assert!(!confers_luck(&obj, &artifacts));
    }

    #[test]
    fn test_arti_reflects() {
        let artifacts = test_artifacts();

        let mut obj = Object::default();
        obj.artifact = 1; // Excalibur - no reflect
        assert!(!arti_reflects(&obj, &artifacts));
    }

    #[test]
    fn test_invoke_healing() {
        let artifacts = vec![Artifact {
            name: "Staff of Aesculapius",
            otyp: 30,
            spfx: ArtifactFlags::NOGEN
                .union(ArtifactFlags::RESTR)
                .union(ArtifactFlags::ATTK)
                .union(ArtifactFlags::INTEL)
                .union(ArtifactFlags::DRLI)
                .union(ArtifactFlags::REGEN),
            cspfx: ArtifactFlags::NONE,
            mtype: 0,
            attk: Attack::new(AttackType::None, DamageType::DrainLife, 0, 0),
            defn: Attack::new(AttackType::None, DamageType::DrainLife, 0, 0),
            cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            inv_prop: InvokeProperty::Healing,
            alignment: ArtifactAlignment::Neutral,
            role: 3, // Healer
            race: NON_PM,
            cost: 5000,
            color: 0,
        }];
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 1;

        let mut player = You::default();
        player.hp = 5;
        player.hp_max = 20;
        player.sick = 10;

        let result = arti_invoke(&obj, &mut player, &artifacts, &mut rng);
        assert!(result.success);
        assert_eq!(player.hp, 20); // Fully healed
        assert_eq!(player.sick, 0); // Cured
        assert_eq!(result.healing, 15);
    }

    #[test]
    fn test_invoke_energy_boost() {
        let artifacts = vec![Artifact {
            name: "Test Energy",
            otyp: 30,
            spfx: ArtifactFlags::NOGEN,
            cspfx: ArtifactFlags::NONE,
            mtype: 0,
            attk: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            defn: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            cary: Attack::new(AttackType::None, DamageType::Physical, 0, 0),
            inv_prop: InvokeProperty::EnergyBoost,
            alignment: ArtifactAlignment::None,
            role: NON_PM,
            race: NON_PM,
            cost: 1000,
            color: 0,
        }];
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 1;

        let mut player = You::default();
        player.energy = 5;
        player.energy_max = 50;
        player.exp_level = 10;

        let result = arti_invoke(&obj, &mut player, &artifacts, &mut rng);
        assert!(result.success);
        assert!(player.energy > 5);
        assert!(result.energy_boost > 0);
    }

    #[test]
    fn test_touch_artifact_alignment_mismatch() {
        let artifacts = test_artifacts(); // Excalibur is Lawful
        let _rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 1; // Excalibur
        obj.name = Some("Excalibur".to_string());

        let mut player = You::default();
        player.alignment.typ = AlignmentType::Chaotic;
        player.alignment.record = 10;

        // Try multiple times - it's random (1/4 chance of blast for non-INTEL)
        let mut was_blasted = false;
        for seed in 0..100 {
            let mut rng = GameRng::new(seed);
            let result = touch_artifact(&obj, &player, &artifacts, &mut rng);
            if result.blast_damage > 0 {
                was_blasted = true;
                assert!(result.message.is_some());
                break;
            }
        }
        // Excalibur is INTEL + RESTR, chaotic player with role mismatch
        // Should be blasted at some point
        assert!(was_blasted, "Expected to be blasted by misaligned artifact");
    }

    #[test]
    fn test_touch_artifact_correct_alignment() {
        let artifacts = test_artifacts();
        let mut rng = GameRng::new(42);

        let mut obj = Object::default();
        obj.artifact = 3; // Frost Brand - ArtifactAlignment::None
        obj.name = Some("Frost Brand".to_string());

        let mut player = You::default();
        player.alignment.typ = AlignmentType::Neutral;
        player.alignment.record = 10;

        let result = touch_artifact(&obj, &player, &artifacts, &mut rng);
        assert!(result.can_hold);
        assert_eq!(result.blast_damage, 0);
    }
}
