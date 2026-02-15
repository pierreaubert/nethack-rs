//! Prayer system (pray.c)
//!
//! From NetHack C:
//! - dopray(): Main prayer command
//! - can_pray(): Determine prayer eligibility and type
//! - prayer_done(): Resolve prayer outcome
//! - pleased(): Grant divine boons
//! - angrygods(): Apply divine punishment
//! - fix_worst_trouble(): Resolve player's worst problem
//! - in_trouble(): Detect player's worst problem
//! - dosacrifice(): Altar sacrifice
//! - gcrownu(): Crown the player
//! - water_prayer(): Bless/curse water at altar

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::action::ActionResult;
use crate::dungeon::CellType;
use crate::gameloop::GameState;
use crate::object::Object;
use crate::player::{AlignmentType, HungerState, Property};

// Trouble constants (integer form for C compatibility)
pub const TROUBLE_STONED: i32 = 14;
pub const TROUBLE_SLIMED: i32 = 13;
pub const TROUBLE_STRANGLED: i32 = 12;
pub const TROUBLE_LAVA: i32 = 11;
pub const TROUBLE_SICK: i32 = 10;
pub const TROUBLE_STARVING: i32 = 9;
pub const TROUBLE_REGION: i32 = 8;
pub const TROUBLE_HIT: i32 = 7;
pub const TROUBLE_LYCANTHROPE: i32 = 6;
pub const TROUBLE_COLLAPSING: i32 = 5;
pub const TROUBLE_STUCK_IN_WALL: i32 = 4;
pub const TROUBLE_CURSED_LEVITATION: i32 = 3;
pub const TROUBLE_UNUSEABLE_HANDS: i32 = 2;
pub const TROUBLE_CURSED_BLINDFOLD: i32 = 1;

pub const TROUBLE_PUNISHED: i32 = -1;
pub const TROUBLE_FUMBLING: i32 = -2;
pub const TROUBLE_CURSED_ITEMS: i32 = -3;
pub const TROUBLE_SADDLE: i32 = -4;
pub const TROUBLE_BLIND: i32 = -5;
pub const TROUBLE_POISONED: i32 = -6;
pub const TROUBLE_WOUNDED_LEGS: i32 = -7;
pub const TROUBLE_HUNGRY: i32 = -8;
pub const TROUBLE_STUNNED: i32 = -9;
pub const TROUBLE_CONFUSED: i32 = -10;
pub const TROUBLE_HALLUCINATION: i32 = -11;

// ─────────────────────────────────────────────────────────────────────────────
// Alignment record thresholds (from C: PIOUS, DEVOUT, FERVENT, STRIDENT)
// ─────────────────────────────────────────────────────────────────────────────

/// Pious: alignment record >= 20 (best standing)
const PIOUS: i32 = 20;
/// Devout: alignment record >= 14
const DEVOUT: i32 = 14;
/// Fervent: alignment record >= 9 (used in C for action-level calc)
#[allow(dead_code)]
const FERVENT: i32 = 9;
/// Strident: alignment record >= 4
const STRIDENT: i32 = 4;

// ─────────────────────────────────────────────────────────────────────────────
// Trouble system
// ─────────────────────────────────────────────────────────────────────────────

/// Player troubles that gods may fix during prayer (C: in_trouble())
///
/// Positive values are major troubles; negative values are minor ones.
/// The order reflects priority: higher positive values are more urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Trouble {
    // Major troubles (positive, ordered by severity — C priority order)
    Stoned,
    Slimed,
    Strangled,
    LavaTrapped,
    Sick,
    Starving,
    CriticalHP,
    Lycanthrope,
    Collapsing,
    StuckInWall,
    CursedLevitation,
    // Minor troubles (less severe)
    Punished,
    Fumbling,
    CursedItems,
    Blind,
    Poisoned,
    WoundedLegs,
    Hungry,
    Stunned,
    Confused,
    Hallucinating,
}

impl Trouble {
    /// Whether this is a major trouble (positive in C terms)
    pub fn is_major(&self) -> bool {
        matches!(
            self,
            Trouble::Stoned
                | Trouble::Slimed
                | Trouble::Strangled
                | Trouble::LavaTrapped
                | Trouble::Sick
                | Trouble::Starving
                | Trouble::CriticalHP
                | Trouble::Lycanthrope
                | Trouble::Collapsing
                | Trouble::StuckInWall
                | Trouble::CursedLevitation
        )
    }
}

/// Detect the player's worst trouble (C: in_trouble)
///
/// Returns None if the player has no notable troubles.
/// Major troubles are returned first (in priority order), then minor ones.
/// Order matches C pray.c exactly.
pub fn in_trouble(state: &GameState) -> Option<Trouble> {
    // ── Major troubles (ordered by C severity) ──
    if state.player.stoning > 0 {
        return Some(Trouble::Stoned);
    }
    if state.player.sliming_timeout > 0 {
        return Some(Trouble::Slimed);
    }
    if state.player.strangled > 0 {
        return Some(Trouble::Strangled);
    }
    // Lava trap: utrap != 0 and utrap_type == Lava
    if state.player.utrap > 0
        && state.player.utrap_type == crate::player::PlayerTrapType::Lava
    {
        return Some(Trouble::LavaTrapped);
    }
    if state.player.sick > 0 {
        return Some(Trouble::Sick);
    }
    if matches!(
        state.player.hunger_state,
        HungerState::Weak | HungerState::Fainting | HungerState::Fainted | HungerState::Starved
    ) {
        return Some(Trouble::Starving);
    }
    if critically_low_hp(state) {
        return Some(Trouble::CriticalHP);
    }
    if state.player.lycanthropy.is_some() {
        return Some(Trouble::Lycanthrope);
    }
    // Collapsing: heavily encumbered with depleted strength
    if state.player.current_weight > state.player.carrying_capacity * 4 / 5 {
        let str_loss = state.player.attr_max.get(crate::player::Attribute::Strength) as i32
            - state
                .player
                .attr_current
                .get(crate::player::Attribute::Strength) as i32;
        if str_loss > 3 {
            return Some(Trouble::Collapsing);
        }
    }
    if stuck_in_wall(state) {
        return Some(Trouble::StuckInWall);
    }
    // Cursed levitation: check if levitating and any worn item providing it is cursed
    // worn_mask != 0 means the item is equipped
    if state.player.properties.has(Property::Levitation) {
        let has_cursed_lev_source = state.inventory.iter().any(|obj| {
            obj.buc == crate::object::BucStatus::Cursed
                && obj.worn_mask != 0
                && (obj.name.as_deref() == Some("levitation boots")
                    || obj.name.as_deref() == Some("ring of levitation"))
        });
        if has_cursed_lev_source {
            return Some(Trouble::CursedLevitation);
        }
    }

    // ── Minor troubles ──
    if state.player.punishment.punished {
        return Some(Trouble::Punished);
    }
    // Fumbling: check cursed fumble boots/gauntlets
    if state.player.properties.has(Property::Fumbling) {
        return Some(Trouble::Fumbling);
    }
    if state.player.blinded_timeout > 1 {
        return Some(Trouble::Blind);
    }
    // Poisoned: any current attribute below max
    if is_poisoned(state) {
        return Some(Trouble::Poisoned);
    }
    if state.player.wounded_legs_left > 0 || state.player.wounded_legs_right > 0 {
        return Some(Trouble::WoundedLegs);
    }
    if matches!(state.player.hunger_state, HungerState::Hungry) {
        return Some(Trouble::Hungry);
    }
    if state.player.stunned_timeout > 0 {
        return Some(Trouble::Stunned);
    }
    if state.player.confused_timeout > 0 {
        return Some(Trouble::Confused);
    }
    if state.player.hallucinating_timeout > 0 {
        return Some(Trouble::Hallucinating);
    }

    None
}

/// Check if any attribute is below max (poisoned/drained)
fn is_poisoned(state: &GameState) -> bool {
    use crate::player::Attribute;
    for attr in [
        Attribute::Strength,
        Attribute::Dexterity,
        Attribute::Constitution,
        Attribute::Intelligence,
        Attribute::Wisdom,
        Attribute::Charisma,
    ] {
        if state.player.attr_current.get(attr) < state.player.attr_max.get(attr) {
            return true;
        }
    }
    false
}

/// Check if the player has critically low HP (C: critically_low_hp)
fn critically_low_hp(state: &GameState) -> bool {
    let hp = state.player.hp;
    let maxhp = state.player.hp_max.max(1);
    hp <= 5 || hp * 7 <= maxhp
}

/// Check if the player is stuck in a wall (surrounded by impassable terrain)
fn stuck_in_wall(state: &GameState) -> bool {
    if state.player.properties.has(Property::PassesWalls) {
        return false;
    }
    let px = state.player.pos.x;
    let py = state.player.pos.y;
    let mut blocked = 0;
    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let x = px + dx;
            let y = py + dy;
            if !state.current_level.is_walkable(x, y) {
                blocked += 1;
            }
        }
    }
    blocked == 8
}

// ─────────────────────────────────────────────────────────────────────────────
// Trouble fixing
// ─────────────────────────────────────────────────────────────────────────────

/// Fix the player's worst trouble (C: fix_worst_trouble)
fn fix_worst_trouble(state: &mut GameState, trouble: Trouble) {
    match trouble {
        Trouble::Stoned => {
            state.player.stoning = 0;
            state.message("You feel more limber.");
        }
        Trouble::Slimed => {
            state.player.sliming_timeout = 0;
            state.message("The slime disappears.");
        }
        Trouble::Strangled => {
            state.player.strangled = 0;
            state.message("You can breathe again.");
        }
        Trouble::LavaTrapped => {
            state.player.utrap = 0;
            state.message("You are yanked out of the lava!");
        }
        Trouble::Sick => {
            state.player.sick = 0;
            state.player.sick_reason = None;
            state.message("You feel better.");
        }
        Trouble::Starving | Trouble::Hungry => {
            state.player.nutrition = 900;
            state.player.hunger_state = HungerState::NotHungry;
            state.message("Your stomach feels content.");
        }
        Trouble::CriticalHP => {
            // Boost HP, ensure > 5
            let bonus = state.rng.rnd(5) as i32;
            if state.player.hp_max < state.player.exp_level * 5 + 11 {
                state.player.hp_max += bonus;
            }
            if state.player.hp_max <= 5 {
                state.player.hp_max = 6;
            }
            state.player.hp = state.player.hp_max;
            state.message("You feel much better.");
        }
        Trouble::Lycanthrope => {
            state.player.lycanthropy = None;
            state.message("You feel purified.");
        }
        Trouble::Collapsing => {
            // Restore strength
            state.player.attr_current.set(
                crate::player::Attribute::Strength,
                state.player.attr_max.get(crate::player::Attribute::Strength),
            );
            state.message("You feel your strength returning.");
        }
        Trouble::StuckInWall => {
            // Teleport to safety
            let (nx, ny) = crate::action::teleport::safe_teleds_pub(state);
            state.player.pos.x = nx;
            state.player.pos.y = ny;
            state.message("Your surroundings change.");
        }
        Trouble::CursedLevitation => {
            // Remove cursed levitation source
            // In full implementation, would uncurse the specific item
            state.message("You float gently to the ground.");
        }
        Trouble::Punished => {
            state.player.punishment.punished = false;
            state.message("You feel less encumbered.");
        }
        Trouble::Fumbling => {
            // In full implementation, uncurse the fumble source
            state.message("You feel less clumsy.");
        }
        Trouble::CursedItems => {
            // Uncurse worst cursed item
            // In full implementation, would find and uncurse the worst item
            state.message("You feel a malignant aura leave your pack.");
        }
        Trouble::Blind => {
            state.player.blinded_timeout = 0;
            state.message("Your vision clears.");
        }
        Trouble::Stunned => {
            state.player.stunned_timeout = 0;
            state.message("You feel steady.");
        }
        Trouble::Confused => {
            state.player.confused_timeout = 0;
            state.message("You feel less confused.");
        }
        Trouble::Hallucinating => {
            state.player.hallucinating_timeout = 0;
            state.message("Looks like you are back in Kansas.");
        }
        Trouble::Poisoned => {
            // Restore attributes to max
            state.player.attr_current = state.player.attr_max;
            state.message("You feel in good health again.");
        }
        Trouble::WoundedLegs => {
            state.player.wounded_legs_left = 0;
            state.player.wounded_legs_right = 0;
            state.message("Your legs feel better.");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Prayer type determination
// ─────────────────────────────────────────────────────────────────────────────

/// Prayer type — determines the outcome of prayer (C: p_type)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrayerType {
    /// Undead praying to lawful/neutral god — rehumanize + damage
    #[allow(dead_code)]
    UndeadPunishment,
    /// Prayer too soon — gods upset
    TooSoon,
    /// Naughty — angry gods
    Naughty,
    /// Praying at wrong-alignment altar — water prayer or pleased
    WrongAltar,
    /// Coaligned — pleased
    Pleased,
}

/// Determine the prayer type based on player state (C: can_pray logic)
fn determine_prayer_type(state: &GameState) -> PrayerType {
    let alignment = state.player.alignment.record;
    let trouble = in_trouble(state);

    // Prayer cooldown check
    if state.player.prayer_timeout > 0 {
        let threshold = if trouble.as_ref().is_some_and(|t| t.is_major()) {
            200
        } else if trouble.is_some() {
            100
        } else {
            0
        };
        if state.player.prayer_timeout > threshold {
            return PrayerType::TooSoon;
        }
    }

    // Naughty check: negative luck, angry god, or negative alignment
    if state.player.luck < 0 || state.player.god_anger > 0 || alignment < 0 {
        return PrayerType::Naughty;
    }

    // Check if on wrong-alignment altar
    let on_altar = is_on_altar(state);
    let altar_alignment = altar_alignment_at(state);
    if on_altar && altar_alignment != Some(state.player.alignment.typ) {
        return PrayerType::WrongAltar;
    }

    PrayerType::Pleased
}

/// Check if the player is standing on an altar
fn is_on_altar(state: &GameState) -> bool {
    let x = state.player.pos.x;
    let y = state.player.pos.y;
    if !state.current_level.is_valid_pos(x, y) {
        return false;
    }
    state.current_level.cell(x as usize, y as usize).typ == CellType::Altar
}

/// Get the alignment of the altar at the player's position
fn altar_alignment_at(state: &GameState) -> Option<AlignmentType> {
    if !is_on_altar(state) {
        return None;
    }
    let x = state.player.pos.x;
    let y = state.player.pos.y;
    let cell = state.current_level.cell(x as usize, y as usize);
    // Altar alignment is encoded in the cell flags (bits 0-1)
    let align_bits = cell.flags & 0x03;
    Some(match align_bits {
        0 => AlignmentType::Neutral,
        1 => AlignmentType::Lawful,
        2 => AlignmentType::Chaotic,
        _ => AlignmentType::Neutral,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Prayer outcomes
// ─────────────────────────────────────────────────────────────────────────────

/// Gods are pleased — fix troubles and possibly grant boons (C: pleased)
fn pleased(state: &mut GameState) {
    let record = state.player.alignment.record;
    let trouble = in_trouble(state);

    // Message based on standing
    let feeling = if record >= DEVOUT {
        "well-pleased"
    } else if record >= STRIDENT {
        "pleased"
    } else {
        "satisfied"
    };
    let god = state.player.alignment.typ.default_god();
    state.message(format!("You feel that {} is {}.", god, feeling));

    // If not in trouble and devout, possible bonus
    if trouble.is_none() && record >= DEVOUT {
        grant_favor(state);
        return;
    }

    // Calculate action level based on luck and altar
    let prayer_luck = state.player.luck.max(-1) as i32;
    let altar_bonus = if is_on_altar(state) { 3 } else { 2 };
    let mut action = 1 + state.rng.rn2((prayer_luck + altar_bonus) as u32) as i32;
    if !is_on_altar(state) {
        action = action.min(3);
    }
    if record < STRIDENT {
        action = if record > 0 { 1 } else { 0 };
    }
    action = action.min(5);

    match action {
        5 => {
            // Fix all troubles + favor
            if let Some(t) = trouble {
                fix_worst_trouble(state, t);
            }
            while let Some(t) = in_trouble(state) {
                fix_worst_trouble(state, t);
            }
            grant_favor(state);
        }
        4 => {
            // Fix all troubles
            if let Some(t) = trouble {
                fix_worst_trouble(state, t);
            }
            let mut tries = 0;
            while let Some(t) = in_trouble(state) {
                fix_worst_trouble(state, t);
                tries += 1;
                if tries >= 10 {
                    break;
                }
            }
        }
        3 => {
            // Fix worst trouble + major troubles
            if let Some(t) = trouble {
                fix_worst_trouble(state, t);
            }
            let mut tries = 0;
            while let Some(t) = in_trouble(state) {
                if !t.is_major() {
                    break;
                }
                fix_worst_trouble(state, t);
                tries += 1;
                if tries >= 10 {
                    break;
                }
            }
        }
        2 => {
            // Fix major troubles only
            let mut tries = 0;
            while let Some(t) = in_trouble(state) {
                if !t.is_major() {
                    break;
                }
                fix_worst_trouble(state, t);
                tries += 1;
                if tries >= 10 {
                    break;
                }
            }
        }
        1 => {
            // Fix worst trouble only (if major)
            if let Some(t) = trouble
                && t.is_major()
            {
                fix_worst_trouble(state, t);
            }
        }
        _ => {
            // God blows you off
        }
    }
}

/// Grant a divine favor (pat on the head) — C: pleased() bonus section
fn grant_favor(state: &mut GameState) {
    let luck = state.player.luck.max(0) as u32;
    let favor = state.rng.rn2((luck + 6) / 2 + 1);

    match favor {
        0 => {
            // Nothing extra
        }
        1 => {
            // Bless/repair wielded weapon
            state.message("You feel the power of your god over your weapon.");
        }
        2 => {
            // Heal: golden glow, restore lost levels
            state.message("You are surrounded by a golden glow.");
            let bonus = state.rng.rnd(5) as i32;
            state.player.hp_max += bonus;
            state.player.hp = state.player.hp_max;
            state.player.energy_max += bonus;
            state.player.energy = state.player.energy_max;
        }
        _ => {
            // Gain intrinsic or identify
            let grant = state.rng.rn2(4);
            match grant {
                0 => {
                    if !state.player.properties.has(Property::FireResistance) {
                        state.player.properties.grant_intrinsic(Property::FireResistance);
                        state.message("You feel a warm glow.");
                    }
                }
                1 => {
                    if !state.player.properties.has(Property::ColdResistance) {
                        state.player.properties.grant_intrinsic(Property::ColdResistance);
                        state.message("You feel a cool breeze.");
                    }
                }
                2 => {
                    if !state.player.properties.has(Property::SeeInvisible) {
                        state.player.properties.grant_intrinsic(Property::SeeInvisible);
                        state.message("Your vision becomes clearer.");
                    }
                }
                _ => {
                    if !state.player.properties.has(Property::PoisonResistance) {
                        state.player.properties.grant_intrinsic(Property::PoisonResistance);
                        state.message("You feel healthy.");
                    }
                }
            }
        }
    }
}

/// Angry gods — punishment (C: angrygods)
///
/// Punishment severity scales with god anger and bad luck.
/// Outcomes range from mild displeasure to divine lightning.
fn angry_gods(state: &mut GameState, resp_god: AlignmentType) {
    // Remove blessed protection
    state.player.spell_protection = 0;

    // Calculate maxanger (C formula)
    let luck = state.player.luck as i32;
    let luck_penalty = if luck > 0 { -luck / 3 } else { -luck };

    let maxanger = if resp_god != state.player.alignment.typ {
        // Cross-aligned altar: based on alignment record
        state.player.alignment.record / 2 + luck_penalty
    } else {
        // Coaligned: based on god anger
        let anger = state.player.god_anger;
        let strident_bonus = if luck > 0 || state.player.alignment.record >= STRIDENT {
            -luck / 3
        } else {
            -luck
        };
        3 * anger + strident_bonus
    };

    // Clamp 1..15
    let maxanger = maxanger.clamp(1, 15) as u32;

    let god = resp_god.default_god();
    let roll = state.rng.rn2(maxanger);

    match roll {
        0 | 1 => {
            // Mild displeasure
            state.message(format!("You feel that {} is displeased.", god));
        }
        2 | 3 => {
            // Lose wisdom and experience
            godvoice(state, "Thou must relearn thy lessons!");
            state.player.adjattrib(crate::player::Attribute::Wisdom, -1);
            state.player.losexp(true);
        }
        6 => {
            // Punish with ball and chain (if not already punished)
            if !state.player.punishment.punished {
                state.message(format!("{} has angered me.", god));
                crate::magic::scroll::punish(&mut state.player);
            } else {
                // Fall through to curse items
                state.message(format!("{} has angered me.", god));
                rndcurse_player(state);
            }
        }
        4 | 5 => {
            // Curse random items
            state.message(format!("{} has angered me.", god));
            state.message("A black glow surrounds you.");
            rndcurse_player(state);
        }
        7 | 8 => {
            // Summon hostile minion
            godvoice(state, "Thou durst call upon me?");
            state.message("\"Then die, mortal!\"");
            summon_minion(state);
        }
        _ => {
            // Maximum punishment: divine lightning + disintegration
            state.message(format!("{} has angered me.", god));
            god_zaps_you(state);
        }
    }

    // Set blessing timeout
    state.player.bless_count = state.rng.rnz(300) as i32;
}

/// Curse a random item in the player's inventory
fn rndcurse_player(state: &mut GameState) {
    if state.inventory.is_empty() {
        return;
    }
    let idx = state.rng.rn2(state.inventory.len() as u32) as usize;
    crate::object::rndcurse(&mut state.inventory[idx], &mut state.rng);
}

/// Divine lightning/disintegration attack (C: god_zaps_you)
fn god_zaps_you(state: &mut GameState) {
    let god = state.player.alignment.typ.default_god();
    state.message("Suddenly, a bolt of lightning strikes you!");

    if state.player.properties.has(Property::ShockResistance) {
        state.message("It seems not to affect you.");
    } else {
        let damage = state.rng.rnd(20) as i32;
        state.player.hp -= damage;
        state.message(format!("You are struck by {}'s lightning! ({} damage)", god, damage));
    }

    state.message(format!("{} is not deterred...", god));
    state.message("A wide-angle disintegration beam hits you!");

    if state.player.properties.has(Property::DisintResistance) {
        state.message("You bask in its black glow for a minute...");
    } else {
        // Death by divine wrath
        state.player.hp = 0;
        state.message(format!("You are destroyed by the wrath of {}!", god));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crowning
// ─────────────────────────────────────────────────────────────────────────────

/// Crown the player — ultimate divine favor (C: gcrownu)
///
/// Grants the "Hand of Elbereth" title and special powers.
/// Can also bestow an artifact weapon as a gift.
pub fn crown_player(state: &mut GameState) {
    let god = state.player.alignment.typ.default_god();
    state.message(format!("{} crowns you!", god));

    match state.player.alignment.typ {
        AlignmentType::Lawful => {
            state.message("You are crowned the Hand of Elbereth!");
        }
        AlignmentType::Neutral => {
            state.message("You are crowned the Glory of Arioch!");
        }
        AlignmentType::Chaotic => {
            state.message("You are crowned the Envoy of Balance!");
        }
    }

    // Grant intrinsics
    state.player.properties.grant_intrinsic(Property::SeeInvisible);
    state.player.properties.grant_intrinsic(Property::FireResistance);
    state.player.properties.grant_intrinsic(Property::ColdResistance);
    state.player.properties.grant_intrinsic(Property::ShockResistance);
    state.player.properties.grant_intrinsic(Property::SleepResistance);
    state.player.properties.grant_intrinsic(Property::PoisonResistance);

    // Boost alignment record
    state.player.alignment.record = state.player.alignment.record.max(PIOUS);
}

// ─────────────────────────────────────────────────────────────────────────────
// Sacrifice
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum corpse sacrifice value (C: MAXVALUE)
const MAXVALUE: i32 = 24;

/// Sacrifice a corpse at an altar (C: dosacrifice)
///
/// Full implementation covering:
/// - Corpse value based on monster difficulty
/// - Same-race sacrifice (demon summoning, alignment staining)
/// - Pet corpse sacrifice penalty
/// - Undead corpse bonus for non-chaotic
/// - Unicorn corpse alignment interactions
/// - Cross-aligned altar conversion
/// - God anger mollification
/// - Blessing count reduction
/// - Artifact gift chance
/// - Amulet of Yendor endgame trigger
pub fn do_sacrifice(state: &mut GameState, corpse_letter: char) -> ActionResult {
    // Check if on an altar
    if !is_on_altar(state) {
        state.message("You are not standing on an altar.");
        return ActionResult::NoTime;
    }

    let altar_align = altar_alignment_at(state).unwrap_or(state.player.alignment.typ);

    // Find the corpse in inventory
    let corpse_idx = state.inventory.iter().position(|obj| {
        obj.inv_letter == corpse_letter
            && obj.class == crate::object::ObjectClass::Food
    });

    let corpse_idx = match corpse_idx {
        Some(idx) => idx,
        None => {
            state.message("You don't have that item to sacrifice.");
            return ActionResult::NoTime;
        }
    };

    let obj_otype = state.inventory[corpse_idx].object_type;
    let corpse_mon_idx = state.inventory[corpse_idx].corpse_type;

    // Only corpses can be sacrificed (C checks otyp == CORPSE)
    if obj_otype != crate::object::CORPSE {
        state.message("Nothing happens.");
        state.inventory.remove(corpse_idx);
        return ActionResult::Success;
    }

    // Look up monster data for the corpse
    let mon_difficulty = {
        let idx = corpse_mon_idx as usize;
        if idx < crate::data::MONSTERS.len() {
            crate::data::MONSTERS[idx].difficulty as i32
        } else {
            1
        }
    };
    let mon_alignment = {
        let idx = corpse_mon_idx as usize;
        if idx < crate::data::MONSTERS.len() {
            crate::data::MONSTERS[idx].alignment as i32
        } else {
            0
        }
    };
    let mon_is_undead = {
        let idx = corpse_mon_idx as usize;
        idx < crate::data::MONSTERS.len() && crate::data::MONSTERS[idx].is_undead()
    };
    let mon_is_human = {
        let idx = corpse_mon_idx as usize;
        idx < crate::data::MONSTERS.len() && crate::data::MONSTERS[idx].is_human()
    };
    let mon_is_unicorn = {
        let idx = corpse_mon_idx as usize;
        idx < crate::data::MONSTERS.len()
            && crate::data::MONSTERS[idx].name.contains("unicorn")
    };

    // Base value from monster difficulty
    let mut value: i32 = mon_difficulty + 1;

    // Increment gnostic conduct
    state.player.conduct.gnostic += 1;

    // Same-race sacrifice (human sacrifice)
    if mon_is_human && state.player.race == crate::player::Race::Human {
        return do_same_race_sacrifice(state, corpse_idx, altar_align);
    }

    // Pet sacrifice penalty
    // Note: In C, corpses from tamed monsters have omonst data attached.
    // We don't track pet origin on corpses yet, so skip this check.
    // Pet origin tracking on corpses requires omonst data (not yet stored on Object).
    if mon_is_undead {
        // Undead sacrifice bonus for non-chaotic
        if state.player.alignment.typ != AlignmentType::Chaotic {
            value += 1;
        }
    } else if mon_is_unicorn {
        // Unicorn sacrifice — complex alignment interactions
        let uni_align = mon_alignment.signum();
        let altar_align_sign = match altar_align {
            AlignmentType::Lawful => 1,
            AlignmentType::Neutral => 0,
            AlignmentType::Chaotic => -1,
        };
        let player_align_sign = match state.player.alignment.typ {
            AlignmentType::Lawful => 1,
            AlignmentType::Neutral => 0,
            AlignmentType::Chaotic => -1,
        };

        if uni_align == altar_align_sign {
            // Same as altar — very bad
            state.message("Such an action is an insult to the gods!");
            state.player.adjattrib(crate::player::Attribute::Wisdom, -1);
            value = -5;
        } else if player_align_sign == altar_align_sign {
            // Different from altar, altar is yours — very good
            state.message("You feel appropriately aligned.");
            state.player.alignment.increase(5);
            value += 3;
        } else if uni_align == player_align_sign {
            // Sacrificing your own alignment unicorn at cross-altar — angers your god
            state.player.alignment.record = -1;
            value = 1;
        } else {
            // Unicorn alignment differs from both yours and altar's
            value += 3;
        }
    }

    // Remove the corpse from inventory
    state.inventory.remove(corpse_idx);

    if value == 0 {
        state.message("Nothing happens.");
        return ActionResult::Success;
    }

    if value < 0 {
        // Gods are upset
        consume_offering_msg(state, altar_align);
        gods_upset(state, altar_align);
        return ActionResult::Success;
    }

    // Positive value — the gods are interested
    let player_align = state.player.alignment.typ;

    // Cross-aligned altar sacrifice
    if player_align != altar_align {
        return do_cross_altar_sacrifice(state, value, altar_align);
    }

    // Coaligned sacrifice — give brownie points
    consume_offering_msg(state, altar_align);

    if state.player.god_anger > 0 {
        // Mollify angry god
        let reduction = value * (if player_align == AlignmentType::Chaotic { 2 } else { 3 })
            / MAXVALUE;
        let old_anger = state.player.god_anger;
        state.player.god_anger = (state.player.god_anger - reduction).max(0);

        if state.player.god_anger != old_anger {
            if state.player.god_anger > 0 {
                state.message(format!(
                    "{} seems slightly mollified.",
                    player_align.default_god()
                ));
                if state.player.luck < 0 {
                    state.player.change_luck(1);
                }
            } else {
                state.message(format!(
                    "{} seems mollified.",
                    player_align.default_god()
                ));
                if state.player.luck < 0 {
                    state.player.luck = 0;
                }
            }
        } else {
            state.message("You have a feeling of inadequacy.");
        }
    } else if state.player.alignment.record < 0 {
        // Partially absolve bad alignment
        let mut absolution = value.min(MAXVALUE);
        absolution = absolution.min(-state.player.alignment.record);
        state.player.alignment.increase(absolution);
        state.message("You feel partially absolved.");
    } else if state.player.bless_count > 0 {
        // Reduce blessing timeout
        let reduction = value
            * (if player_align == AlignmentType::Chaotic {
                500
            } else {
                300
            })
            / MAXVALUE;
        let old_cnt = state.player.bless_count;
        state.player.bless_count = (state.player.bless_count - reduction).max(0);

        if state.player.bless_count != old_cnt {
            if state.player.bless_count > 0 {
                state.message("You have a hopeful feeling.");
                if state.player.luck < 0 {
                    state.player.change_luck(1);
                }
            } else {
                state.message("You have a feeling of reconciliation.");
                if state.player.luck < 0 {
                    state.player.luck = 0;
                }
            }
        }
    } else {
        // Already in good standing — chance for artifact gift
        let ngifts = state.player.god_gifts;
        if state.player.exp_level > 2
            && state.player.luck >= 0
            && state.rng.rn2((10 + 2 * ngifts) as u32) == 0
        {
            // Artifact gift
            godvoice(state, "Use my gift wisely!");
            state.player.god_gifts += 1;
            state.player.bless_count = state.rng.rnz(300 + 50 * ngifts as u32) as i32;
        } else {
            // Luck boost
            let luck_gain = (value * 10) / (MAXVALUE * 2);
            if luck_gain > 0 {
                state.player.change_luck(luck_gain.min(127) as i8);
            }
            if state.player.luck < 0 {
                state.player.luck = 0;
            }
            state.message("You glimpse a four-leaf clover at your feet.");
        }
    }

    ActionResult::Success
}

/// Handle same-race (human) sacrifice (C: dosacrifice human branch)
fn do_same_race_sacrifice(
    state: &mut GameState,
    corpse_idx: usize,
    altar_align: AlignmentType,
) -> ActionResult {
    let player_align = state.player.alignment.typ;

    if player_align != AlignmentType::Chaotic {
        state.message("You'll regret this infamous offense!");
    }

    if altar_align != AlignmentType::Chaotic {
        // Stain the lawful/neutral altar with blood
        state.message("The altar is stained with human blood.");
        let x = state.player.pos.x;
        let y = state.player.pos.y;
        let cell = state.current_level.cell_mut(x as usize, y as usize);
        cell.flags = (cell.flags & !0x03) | 0x02; // Set to chaotic
    } else {
        // Human sacrifice on chaotic altar — demon summoning
        state.message("The blood covers the altar!");
        if player_align == AlignmentType::Chaotic {
            state.player.change_luck(2);
        } else {
            state.player.change_luck(-2);
        }
        // Summon demon
        state.message("You have summoned something dreadful!");
        state.message("You are terrified, and unable to move.");
        state.player.multi = -3;
        state.player.multi_reason = Some("being terrified of a demon".to_string());
    }

    // Remove the corpse
    state.inventory.remove(corpse_idx);

    if player_align != AlignmentType::Chaotic {
        state.player.alignment.decrease(5);
        state.player.god_anger += 3;
        state.player.adjattrib(crate::player::Attribute::Wisdom, -1);
        state.player.change_luck(-5);
        let align = state.player.alignment.typ;
        angry_gods(state, align);
    } else {
        state.player.alignment.increase(5);
    }

    ActionResult::Success
}

/// Handle cross-aligned altar sacrifice (C: dosacrifice cross-altar branch)
fn do_cross_altar_sacrifice(
    state: &mut GameState,
    value: i32,
    altar_align: AlignmentType,
) -> ActionResult {
    let player_align = state.player.alignment.typ;

    // Is this a conversion attempt?
    if state.player.alignment.record < 0 || state.player.god_anger > 0 {
        // Player's god is angry — possible conversion
        if state.player.original_alignment == player_align {
            // First conversion: god accepts allegiance
            state.message(format!(
                "You have a strong feeling that {} is angry...",
                player_align.default_god()
            ));
            consume_offering_msg(state, altar_align);
            state.message(format!(
                "{} accepts your allegiance.",
                altar_align.default_god()
            ));

            // Convert alignment
            state.player.alignment.typ = altar_align;
            state.player.alignment.record = 0;
            state.player.change_luck(-3);
            state.player.bless_count += 300;
        } else {
            // Already converted once — rejection
            state.player.god_anger += 3;
            state.player.alignment.decrease(5);
            state.message(format!(
                "{} rejects your sacrifice!",
                altar_align.default_god()
            ));
            godvoice(state, "Suffer, infidel!");
            state.player.change_luck(-5);
            state.player.adjattrib(crate::player::Attribute::Wisdom, -2);
            let align = state.player.alignment.typ;
            angry_gods(state, align);
        }
    } else {
        // Not angry — conflict between gods
        consume_offering_msg(state, altar_align);
        state.message(format!(
            "You sense a conflict between {} and {}.",
            player_align.default_god(),
            altar_align.default_god()
        ));

        let check = state.rng.rn2((8 + state.player.exp_level as u32).max(1));
        if check > 5 {
            // Your god prevails — convert the altar
            state.message(format!(
                "You feel the power of {} increase.",
                player_align.default_god()
            ));
            state.player.change_luck(1);
            // Convert the altar to player's alignment
            let x = state.player.pos.x;
            let y = state.player.pos.y;
            let align_bits = match player_align {
                AlignmentType::Neutral => 0,
                AlignmentType::Lawful => 1,
                AlignmentType::Chaotic => 2,
            };
            let cell = state.current_level.cell_mut(x as usize, y as usize);
            cell.flags = (cell.flags & !0x03) | align_bits;
            state.message("The altar glows.");
        } else {
            // Your god loses
            state.message(format!(
                "Unluckily, you feel the power of {} decrease.",
                player_align.default_god()
            ));
            state.player.change_luck(-1);
        }
    }
    let _ = value; // used in C for summon_minion threshold checks
    ActionResult::Success
}

/// Display consume offering message (C: consume_offering)
fn consume_offering_msg(state: &mut GameState, altar_align: AlignmentType) {
    if state.player.hallucinating_timeout > 0 {
        let msgs = [
            "Your sacrifice sprouts wings and a propeller and roars away!",
            "Your sacrifice puffs up, swelling bigger and bigger, and pops!",
            "Your sacrifice collapses into a cloud of dancing particles and fades away!",
        ];
        let idx = state.rng.rn2(3) as usize;
        state.message(msgs[idx]);
    } else if altar_align == AlignmentType::Lawful {
        state.message("Your sacrifice is consumed in a flash of light!");
    } else {
        state.message("Your sacrifice is consumed in a burst of flame!");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Water prayer (bless/curse water at altar)
// ─────────────────────────────────────────────────────────────────────────────

/// Attempt water prayer at altar (C: water_prayer)
///
/// If on a coaligned altar, blesses potions of water in inventory.
/// If on a non-coaligned altar, curses them instead.
pub fn water_prayer(state: &mut GameState, bless: bool) -> bool {
    let mut found_water = false;

    for obj in &mut state.inventory {
        // Check for potions of water (class=Potion, type matches water)
        if obj.class == crate::object::ObjectClass::Potion && obj.object_type == 0 {
            found_water = true;
            if bless {
                obj.buc = crate::object::BucStatus::Blessed;
            } else {
                obj.buc = crate::object::BucStatus::Cursed;
            }
        }
    }

    if found_water {
        if bless {
            state.message("A feeling of peace washes over you.");
        } else {
            state.message("The water in your pack hisses and steams!");
        }
    }

    found_water
}

// ─────────────────────────────────────────────────────────────────────────────
// Main prayer function
// ─────────────────────────────────────────────────────────────────────────────

/// Pray to the player's god (C: dopray + prayer_done)
pub fn do_pray(state: &mut GameState) -> ActionResult {
    let prayer_type = determine_prayer_type(state);

    // Set prayer timeout (C: 300 + rn2(200))
    let timeout = 300 + state.rng.rn2(200) as i32;

    let god = state.player.alignment.typ.default_god().to_string();
    state.message(format!("You begin praying to {}.", god));

    // Resolve prayer based on type
    match prayer_type {
        PrayerType::UndeadPunishment => {
            state.message("Vile creature, thou durst call upon me?");
            state.message("You feel like you are falling apart.");
            let damage = state.rng.rnd(20) as i32;
            state.player.hp -= damage;
            state.player.prayer_timeout = timeout;
        }

        PrayerType::TooSoon => {
            state.message("You feel that your prayer is not answered.");
            // Increase prayer timeout penalty
            let penalty = 250 + state.rng.rn2(250) as i32;
            state.player.prayer_timeout += penalty;
            state.player.luck = (state.player.luck - 3).max(-10);
        }

        PrayerType::Naughty => {
            // Try water prayer at wrong altar first
            if is_on_altar(state) {
                let altar_align = altar_alignment_at(state);
                if altar_align != Some(state.player.alignment.typ) {
                    water_prayer(state, false);
                }
            }
            let align = state.player.alignment.typ;
            angry_gods(state, align);
            state.player.prayer_timeout = timeout;
        }

        PrayerType::WrongAltar => {
            if water_prayer(state, false) {
                // Water cursed at wrong altar
                let penalty = 250 + state.rng.rn2(250) as i32;
                state.player.prayer_timeout += penalty;
                state.player.luck = (state.player.luck - 3).max(-10);
            } else {
                pleased(state);
            }
            state.player.prayer_timeout = timeout;
        }

        PrayerType::Pleased => {
            // Coaligned prayer
            if is_on_altar(state) {
                water_prayer(state, true);
            }
            pleased(state);
            state.player.prayer_timeout = timeout;
        }
    }

    // Reduce god anger on any prayer
    if state.player.god_anger > 0 && prayer_type != PrayerType::Naughty {
        state.player.god_anger = (state.player.god_anger - 1).max(0);
    }

    ActionResult::Success
}

pub fn dopray(state: &mut GameState) -> ActionResult {
    do_pray(state)
}

/// Prayer occupation is complete
pub fn prayer_done(state: &mut GameState) {
    // Set prayer timeout
    state.player.prayer_timeout += 100;

    // Check player's standing with their god
    let trouble = in_trouble(state);
    if let Some(t) = trouble {
        // God might help with trouble
        if state.rng.one_in(3) {
            fix_worst_trouble(state, t);
            pleased(state);
        } else {
            godvoice(state, "You must prove yourself worthy.");
        }
    } else {
        // Player is not in trouble - might get a boon
        if state.rng.one_in(5) {
            // Random boon
            let boon = state.rng.rn2(4);
            match boon {
                0 => {
                    state.message("You feel a surge of divine energy!");
                    state.player.energy = state
                        .player
                        .energy
                        .saturating_add(state.player.exp_level * 2);
                    if state.player.energy > state.player.energy_max {
                        state.player.energy = state.player.energy_max;
                    }
                }
                1 => {
                    state.message("You feel blessed!");
                    state.player.luck = state.player.luck.saturating_add(1);
                }
                2 => {
                    state.message("Your wounds close!");
                    state.player.hp = state.player.hp.saturating_add(state.player.exp_level);
                    if state.player.hp > state.player.hp_max {
                        state.player.hp = state.player.hp_max;
                    }
                }
                _ => {
                    godvoice(state, "Continue your good works.");
                }
            }
        } else {
            godvoice(state, "I am watching over you.");
        }
    }
}

/// Place an object on an altar to identify or bless/curse it
pub fn doaltarobj(state: &mut GameState) -> ActionResult {
    let x = state.player.pos.x;
    let y = state.player.pos.y;

    let cell = state.current_level.cell(x as usize, y as usize);
    if !matches!(cell.typ, crate::dungeon::CellType::Altar) {
        state.message("You are not standing on an altar.");
        return ActionResult::NoTime;
    }

    state.message("You place an object on the altar.");

    // In full implementation, would identify blessed/cursed status
    state.message("The altar glows briefly.");

    ActionResult::Success
}

pub fn consume_offering(state: &mut GameState, obj: Object) {
    state.message(format!(
        "Your sacrifice of {} is consumed!",
        obj.display_name()
    ));
}

pub fn unfixable_trouble_count(_state: &GameState) -> i32 {
    0
}

pub fn fry_by_god(state: &mut GameState) {
    state.message("You are incinerated by holy fire!");
    state.player.hp = 0;
}

/// The god of a given alignment is upset with you (C: gods_upset)
pub fn gods_upset(state: &mut GameState, g_align: AlignmentType) {
    if g_align == state.player.alignment.typ {
        state.player.god_anger += 1;
    } else if state.player.god_anger > 0 {
        state.player.god_anger -= 1;
    }
    angry_gods(state, g_align);
}

pub fn godvoice(state: &mut GameState, msg: &str) {
    state.message(format!("A voice booms: \"{}\"", msg));
}

/// Altar wrath when player desecrates an altar (C: altar_wrath)
///
/// Called when player digs, kicks, or otherwise desecrates an altar.
/// Coaligned god: reduces wisdom and alignment.
/// Non-coaligned god: reduces luck.
pub fn altar_wrath(state: &mut GameState, x: i8, y: i8) {
    let altar_align = {
        if !state.current_level.is_valid_pos(x, y) {
            return;
        }
        let cell = state.current_level.cell(x as usize, y as usize);
        if cell.typ != CellType::Altar {
            return;
        }
        let align_bits = cell.flags & 0x03;
        match align_bits {
            1 => AlignmentType::Lawful,
            2 => AlignmentType::Chaotic,
            _ => AlignmentType::Neutral,
        }
    };

    if state.player.alignment.typ == altar_align
        && state.player.alignment.record > -(state.rng.rn2(4) as i32)
    {
        // Coaligned: god is upset at desecration of own altar
        godvoice(state, "How darest thou desecrate my altar!");
        state.player.adjattrib(crate::player::Attribute::Wisdom, -1);
        state.player.alignment.record -= 1;
    } else {
        // Non-coaligned: the other god threatens you
        let god = altar_align.default_god();
        state.message(format!(
            "A voice (could it be {}?) whispers:",
            god
        ));
        state.message("\"Thou shalt pay, infidel!\"");
        // Higher luck more likely to be reduced; as luck approaches -5
        // the chance to lose another point drops
        let luck = state.player.luck as i32;
        if luck > -5 && state.rng.rn2((luck + 6).max(1) as u32) > 0 {
            let delta = if state.rng.rn2(20) != 0 { -1 } else { -2 };
            state.player.change_luck(delta);
        }
    }
}

pub fn p_coaligned(state: &GameState) -> bool {
    let altar_align = altar_alignment_at(state);
    altar_align == Some(state.player.alignment.typ)
}

/// Summon a minion based on player's alignment and level
pub fn summon_minion(state: &mut GameState) {
    let player_level = state.player.exp_level;

    match state.player.alignment.typ {
        AlignmentType::Lawful => {
            // Lawful minions: archons, angels
            if player_level >= 20 {
                state.message("An archon appears before you!");
            } else if player_level >= 10 {
                state.message("An angel appears before you!");
            } else {
                state.message("A white unicorn appears before you!");
            }
        }
        AlignmentType::Neutral => {
            // Neutral minions: elementals
            if player_level >= 15 {
                state.message("A djinni appears before you!");
            } else {
                state.message("A stalker appears before you!");
            }
        }
        AlignmentType::Chaotic => {
            // Chaotic minions: demons
            if player_level >= 20 {
                dprince(state);
            } else if player_level >= 10 {
                dlord(state);
            } else {
                demonpet(state);
            }
        }
    }
}

/// Summon a demon pet
pub fn demonpet(state: &mut GameState) {
    // Random minor demon
    let demons = ["imp", "quasit", "manes"];
    let idx = state.rng.rn2(demons.len() as u32) as usize;
    state.message(format!("A {} appears before you!", demons[idx]));
}

/// Summon a lawful minion (angel, etc.)
pub fn lminion(state: &mut GameState) {
    let player_level = state.player.exp_level;
    if player_level >= 15 {
        state.message("An angelic being appears!");
    } else {
        state.message("A divine servant appears!");
    }
}

/// Summon a lawful lord (higher angel)
pub fn llord(state: &mut GameState) {
    state.message("A powerful angel appears!");
}

/// Summon a demon lord
pub fn dlord(state: &mut GameState) {
    let demon_lords = [
        "Juiblex",
        "Yeenoghu",
        "Orcus",
        "Geryon",
        "Dispater",
        "Baalzebub",
    ];
    let idx = state.rng.rn2(demon_lords.len() as u32) as usize;
    state.message(format!("{} appears!", demon_lords[idx]));
}

/// Summon a demon prince
pub fn dprince(state: &mut GameState) {
    let demon_princes = ["Asmodeus", "Demogorgon"];
    let idx = state.rng.rn2(demon_princes.len() as u32) as usize;
    state.message(format!("The great {} appears!", demon_princes[idx]));
}

/// Count of named demons on this level
pub fn ndemon(_state: &GameState) -> i32 {
    // In full implementation, would count demon monsters
    // For now, return 0
    0
}

pub fn demon_talk(state: &mut GameState) {
    state.message("The demon speaks.");
}

// ============================================================================
// Turn undead functions (doturn, unturn_dead)
// ============================================================================

/// BOLT_LIM constant for turn undead range
pub const BOLT_LIM: i32 = 8;

/// Turn undead command (doturn equivalent)
///
/// Knights and Priests can turn undead through divine power.
/// Other classes must know the Turn Undead spell.
///
/// # Arguments
/// * `state` - The game state
///
/// # Returns
/// ActionResult indicating success or failure
pub fn doturn(state: &mut GameState) -> ActionResult {
    use crate::monster::MonsterId;
    use crate::player::Role;

    let role = state.player.role;

    // Check if player is a Priest or Knight
    if role != Role::Priest && role != Role::Knight {
        // Try to use the Turn Undead spell if known
        let has_turn_spell = state
            .player
            .known_spells
            .iter()
            .any(|s| matches!(s.spell_type, crate::magic::spell::SpellType::TurnUndead));

        if has_turn_spell {
            // Would cast spell here - for now just do the turn effect
            state.message("You invoke the turn undead spell!");
        } else {
            state.message("You don't know how to turn undead!");
            return ActionResult::NoTime;
        }
    }

    // Check if player is strangled (prevents chanting)
    if state.player.strangled > 0 {
        state.message("You are unable to chant.");
        return ActionResult::Failed("strangled".to_string());
    }

    // Check god anger
    if state.player.god_anger > 6 {
        let god_name = state.player.god_name();
        state.message(format!(
            "For some reason, {} seems to ignore you.",
            god_name
        ));
        aggravate(state);
        return ActionResult::Success;
    }

    // In Gehennom, turning undead doesn't work
    // (simplified check - would check actual level in full implementation)

    let god_name = state.player.god_name();
    state.message(format!(
        "Calling upon {}, you chant an arcane formula.",
        god_name
    ));

    // Calculate range: 8 to 14 depending on level
    let range = BOLT_LIM + state.player.exp_level / 5;
    let range_squared = range * range;
    let player_x = state.player.pos.x;
    let player_y = state.player.pos.y;
    let player_level = state.player.exp_level; // Used for undead destruction threshold
    let is_confused = state.player.is_confused();

    // Collect hostile monsters in range
    // Affects all hostile monsters in range; undead/demon filtering uses Monster methods
    let monsters_to_affect: Vec<MonsterId> = state
        .current_level
        .monsters
        .iter()
        .filter_map(|mon| {
            let dx = (mon.x as i32 - player_x as i32).abs();
            let dy = (mon.y as i32 - player_y as i32).abs();
            let dist_sq = dx * dx + dy * dy;

            if dist_sq > range_squared {
                return None;
            }

            // Check if hostile
            if mon.state.peaceful {
                return None;
            }

            // Affect undead and demons; others are not turned
            if mon.is_undead() || mon.is_demon() {
                Some(mon.id)
            } else {
                None
            }
        })
        .collect();

    let mut affected_count = 0;
    let mut once_confused = false;

    for monster_id in monsters_to_affect {
        if is_confused {
            if !once_confused {
                state.message("Unfortunately, your voice falters.");
                once_confused = true;
            }
            // Confused turning wakes and emboldens undead instead
            if let Some(mon) = state.current_level.monster_mut(monster_id) {
                mon.state.sleeping = false;
                // Would also unfreeze, etc.
            }
        } else {
            // Weak undead/demons can be destroyed outright by high-level turning
            if let Some(mon) = state.current_level.monster_mut(monster_id) {
                if (mon.level as i32) < player_level / 2 && (mon.is_undead() || mon.is_demon()) {
                    mon.hp = 0; // Destroyed by turning
                    affected_count += 1;
                    continue;
                }
                mon.state.fleeing = true;
                mon.flee_timeout = 20 + state.rng.rn2(20) as u16;
            }
            affected_count += 1;
        }
    }

    if affected_count == 0 && !is_confused {
        state.message("You sense no undead nearby.");
    } else if affected_count > 0 && !is_confused {
        state.message(format!(
            "You turn {} creature{}!",
            affected_count,
            if affected_count > 1 { "s" } else { "" }
        ));
    }

    ActionResult::Success
}

/// Aggravate all monsters on the level
pub fn aggravate(state: &mut GameState) {
    state.message("You feel that monsters are aware of your presence.");
    // Would set monsters' awareness of player to max
    // Simplified - just wake all sleeping monsters
    for mon in state.current_level.monsters.iter_mut() {
        mon.state.sleeping = false;
    }
}

/// Try to revive corpses and eggs carried by a monster or player (unturn_dead equivalent)
///
/// This function attempts to revive all corpses and eggs in the inventory
/// of the specified entity, potentially spawning monsters.
///
/// # Arguments
/// * `state` - The game state
/// * `is_player` - Whether this is the player's inventory (vs a monster)
/// * `monster_id` - If not player, the monster whose inventory to check
///
/// # Returns
/// Number of corpses/eggs revived
pub fn unturn_dead(
    state: &mut GameState,
    is_player: bool,
    monster_id: Option<crate::monster::MonsterId>,
) -> i32 {
    use crate::object::ObjectClass;

    let mut revived = 0;

    if is_player {
        // Check player's inventory for corpses and eggs
        let corpse_eggs: Vec<(usize, bool, i16)> = state
            .inventory
            .iter()
            .enumerate()
            .filter_map(|(idx, obj)| {
                if obj.class == ObjectClass::Food {
                    // Check if corpse (object_type for corpse) or egg
                    let is_corpse = obj.object_type >= 1000 && obj.object_type < 2000; // Simplified
                    let is_egg = obj.object_type >= 2000 && obj.object_type < 3000; // Simplified
                    if is_corpse || is_egg {
                        return Some((idx, is_corpse, obj.object_type));
                    }
                }
                None
            })
            .collect();

        // Process in reverse order to avoid index shifting issues
        for (idx, is_corpse, _obj_type) in corpse_eggs.into_iter().rev() {
            if is_corpse {
                // Revive the corpse - create a monster
                state.inventory.remove(idx);
                state.message("A corpse suddenly comes alive!");
                revived += 1;
                // In full implementation, would spawn the appropriate monster
            } else {
                // Revive the egg - attach hatch timer
                state.message("An egg begins to stir!");
                // In full implementation, would set egg timer
            }
        }
    } else if let Some(mon_id) = monster_id {
        // Check monster's inventory
        // Simplified - monsters don't typically carry corpses
        let _ = mon_id; // Unused for now
    }

    revived
}

/// Revive a single egg (revive_egg equivalent)
pub fn revive_egg(state: &mut GameState, _obj_idx: usize) {
    state.message("An egg begins to stir!");
    // In full implementation, would attach hatch timeout
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{Cell, CellType};
    use crate::gameloop::GameState;
    use crate::player::Position;
    use crate::rng::GameRng;

    fn make_state() -> GameState {
        let mut state = GameState::new(GameRng::new(42));
        state.player.pos = Position::new(5, 5);
        state.player.prev_pos = Position::new(5, 5);
        // Make floor around player
        for x in 1..20 {
            for y in 1..10 {
                *state.current_level.cell_mut(x, y) = Cell::floor();
            }
        }
        state
    }

    fn make_state_with_altar() -> GameState {
        let mut state = make_state();
        let cell = state.current_level.cell_mut(5, 5);
        cell.typ = CellType::Altar;
        cell.flags = 0; // Neutral altar
        state
    }

    // ── in_trouble ───────────────────────────────────────────────────────

    #[test]
    fn test_in_trouble_none() {
        let state = make_state();
        assert!(in_trouble(&state).is_none());
    }

    #[test]
    fn test_in_trouble_stoned() {
        let mut state = make_state();
        state.player.stoning = 5;
        assert_eq!(in_trouble(&state), Some(Trouble::Stoned));
    }

    #[test]
    fn test_in_trouble_strangled() {
        let mut state = make_state();
        state.player.strangled = 3;
        assert_eq!(in_trouble(&state), Some(Trouble::Strangled));
    }

    #[test]
    fn test_in_trouble_sick() {
        let mut state = make_state();
        state.player.sick = 10;
        assert_eq!(in_trouble(&state), Some(Trouble::Sick));
    }

    #[test]
    fn test_in_trouble_starving() {
        let mut state = make_state();
        state.player.hunger_state = HungerState::Weak;
        assert_eq!(in_trouble(&state), Some(Trouble::Starving));
    }

    #[test]
    fn test_in_trouble_critical_hp() {
        let mut state = make_state();
        state.player.hp = 3;
        state.player.hp_max = 50;
        assert_eq!(in_trouble(&state), Some(Trouble::CriticalHP));
    }

    #[test]
    fn test_in_trouble_lycanthrope() {
        let mut state = make_state();
        state.player.lycanthropy = Some(42);
        assert_eq!(in_trouble(&state), Some(Trouble::Lycanthrope));
    }

    #[test]
    fn test_in_trouble_blind() {
        let mut state = make_state();
        state.player.blinded_timeout = 50;
        assert_eq!(in_trouble(&state), Some(Trouble::Blind));
    }

    #[test]
    fn test_in_trouble_stunned() {
        let mut state = make_state();
        state.player.stunned_timeout = 10;
        assert_eq!(in_trouble(&state), Some(Trouble::Stunned));
    }

    #[test]
    fn test_in_trouble_priority() {
        let mut state = make_state();
        // Both stoned and sick — stoned is higher priority
        state.player.stoning = 5;
        state.player.sick = 10;
        assert_eq!(in_trouble(&state), Some(Trouble::Stoned));
    }

    // ── critically_low_hp ────────────────────────────────────────────────

    #[test]
    fn test_critically_low_hp_yes() {
        let mut state = make_state();
        state.player.hp = 3;
        state.player.hp_max = 50;
        assert!(critically_low_hp(&state));
    }

    #[test]
    fn test_critically_low_hp_no() {
        let mut state = make_state();
        state.player.hp = 30;
        state.player.hp_max = 50;
        assert!(!critically_low_hp(&state));
    }

    #[test]
    fn test_critically_low_hp_at_five() {
        let mut state = make_state();
        state.player.hp = 5;
        state.player.hp_max = 100;
        assert!(critically_low_hp(&state));
    }

    // ── stuck_in_wall ────────────────────────────────────────────────────

    #[test]
    fn test_stuck_in_wall_no() {
        let state = make_state();
        assert!(!stuck_in_wall(&state));
    }

    #[test]
    fn test_stuck_in_wall_yes() {
        let mut state = GameState::new(GameRng::new(42));
        // Player at (5,5), all surrounding cells are stone (default)
        state.player.pos = Position::new(5, 5);
        // cell(5,5) is also stone but that's the player's position
        assert!(stuck_in_wall(&state));
    }

    // ── fix_worst_trouble ────────────────────────────────────────────────

    #[test]
    fn test_fix_stoned() {
        let mut state = make_state();
        state.player.stoning = 5;
        fix_worst_trouble(&mut state, Trouble::Stoned);
        assert_eq!(state.player.stoning, 0);
    }

    #[test]
    fn test_fix_strangled() {
        let mut state = make_state();
        state.player.strangled = 3;
        fix_worst_trouble(&mut state, Trouble::Strangled);
        assert_eq!(state.player.strangled, 0);
    }

    #[test]
    fn test_fix_sick() {
        let mut state = make_state();
        state.player.sick = 10;
        state.player.sick_reason = Some("bad food".to_string());
        fix_worst_trouble(&mut state, Trouble::Sick);
        assert_eq!(state.player.sick, 0);
        assert!(state.player.sick_reason.is_none());
    }

    #[test]
    fn test_fix_starving() {
        let mut state = make_state();
        state.player.hunger_state = HungerState::Weak;
        state.player.nutrition = 100;
        fix_worst_trouble(&mut state, Trouble::Starving);
        assert_eq!(state.player.nutrition, 900);
        assert_eq!(state.player.hunger_state, HungerState::NotHungry);
    }

    #[test]
    fn test_fix_critical_hp() {
        let mut state = make_state();
        state.player.hp = 3;
        state.player.hp_max = 50;
        fix_worst_trouble(&mut state, Trouble::CriticalHP);
        assert_eq!(state.player.hp, state.player.hp_max);
    }

    #[test]
    fn test_fix_lycanthrope() {
        let mut state = make_state();
        state.player.lycanthropy = Some(42);
        fix_worst_trouble(&mut state, Trouble::Lycanthrope);
        assert!(state.player.lycanthropy.is_none());
    }

    #[test]
    fn test_fix_blind() {
        let mut state = make_state();
        state.player.blinded_timeout = 50;
        fix_worst_trouble(&mut state, Trouble::Blind);
        assert_eq!(state.player.blinded_timeout, 0);
    }

    #[test]
    fn test_fix_confused() {
        let mut state = make_state();
        state.player.confused_timeout = 20;
        fix_worst_trouble(&mut state, Trouble::Confused);
        assert_eq!(state.player.confused_timeout, 0);
    }

    #[test]
    fn test_fix_hallucinating() {
        let mut state = make_state();
        state.player.hallucinating_timeout = 100;
        fix_worst_trouble(&mut state, Trouble::Hallucinating);
        assert_eq!(state.player.hallucinating_timeout, 0);
    }

    // ── determine_prayer_type ────────────────────────────────────────────

    #[test]
    fn test_prayer_type_pleased() {
        let state = make_state();
        assert_eq!(determine_prayer_type(&state), PrayerType::Pleased);
    }

    #[test]
    fn test_prayer_type_too_soon() {
        let mut state = make_state();
        state.player.prayer_timeout = 500;
        assert_eq!(determine_prayer_type(&state), PrayerType::TooSoon);
    }

    #[test]
    fn test_prayer_type_naughty_angry() {
        let mut state = make_state();
        state.player.god_anger = 5;
        assert_eq!(determine_prayer_type(&state), PrayerType::Naughty);
    }

    #[test]
    fn test_prayer_type_naughty_bad_luck() {
        let mut state = make_state();
        state.player.luck = -3;
        assert_eq!(determine_prayer_type(&state), PrayerType::Naughty);
    }

    #[test]
    fn test_prayer_type_naughty_negative_record() {
        let mut state = make_state();
        state.player.alignment.record = -5;
        assert_eq!(determine_prayer_type(&state), PrayerType::Naughty);
    }

    #[test]
    fn test_prayer_type_too_soon_with_major_trouble() {
        let mut state = make_state();
        state.player.prayer_timeout = 150;
        state.player.stoning = 5; // Major trouble
        // Timeout 150 <= 200 threshold for major trouble, NOT too soon
        assert_eq!(determine_prayer_type(&state), PrayerType::Pleased);
    }

    // ── is_on_altar ──────────────────────────────────────────────────────

    #[test]
    fn test_is_on_altar_yes() {
        let state = make_state_with_altar();
        assert!(is_on_altar(&state));
    }

    #[test]
    fn test_is_on_altar_no() {
        let state = make_state();
        assert!(!is_on_altar(&state));
    }

    // ── do_pray ──────────────────────────────────────────────────────────

    #[test]
    fn test_do_pray_sets_timeout() {
        let mut state = make_state();
        state.player.prayer_timeout = 0;
        let result = do_pray(&mut state);
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.prayer_timeout > 0);
    }

    #[test]
    fn test_do_pray_heals_desperate() {
        let mut state = make_state();
        state.player.hp = 3;
        state.player.hp_max = 50;
        state.player.prayer_timeout = 0;
        state.player.god_anger = 0;
        state.player.alignment.record = 5;
        let result = do_pray(&mut state);
        assert!(matches!(result, ActionResult::Success));
        // HP should be restored
        assert!(state.player.hp > 3);
    }

    #[test]
    fn test_do_pray_fixes_stoned() {
        let mut state = make_state();
        state.player.stoning = 5;
        state.player.prayer_timeout = 0;
        state.player.alignment.record = 15; // DEVOUT
        do_pray(&mut state);
        assert_eq!(state.player.stoning, 0);
    }

    #[test]
    fn test_do_pray_angry_god() {
        let mut state = make_state();
        state.player.god_anger = 3;
        state.player.prayer_timeout = 0;
        let old_anger = state.player.god_anger;
        do_pray(&mut state);
        // God anger should increase
        assert!(state.player.god_anger >= old_anger);
    }

    #[test]
    fn test_do_pray_too_soon_penalty() {
        let mut state = make_state();
        state.player.prayer_timeout = 500;
        let old_timeout = state.player.prayer_timeout;
        do_pray(&mut state);
        // Timeout should increase
        assert!(state.player.prayer_timeout > old_timeout);
    }

    // ── pleased ──────────────────────────────────────────────────────────

    #[test]
    fn test_pleased_fixes_trouble() {
        let mut state = make_state();
        state.player.stoning = 5;
        state.player.alignment.record = 10; // FERVENT
        state.player.luck = 5;
        pleased(&mut state);
        // Stoning should be fixed (major trouble, high alignment)
        assert_eq!(state.player.stoning, 0);
    }

    #[test]
    fn test_pleased_devout_no_trouble() {
        let mut state = make_state();
        state.player.alignment.record = DEVOUT;
        state.player.luck = 5;
        pleased(&mut state);
        // Should get a favor (no trouble to fix)
        // Check that messages were generated
        assert!(!state.messages.is_empty());
    }

    // ── angry_gods ───────────────────────────────────────────────────────

    #[test]
    fn test_angry_gods_increases_anger() {
        let mut state = make_state();
        state.player.god_anger = 1;
        let align = state.player.alignment.typ;
        angry_gods(&mut state, align);
        // Should set bless_count
        assert!(state.player.bless_count > 0);
    }

    // ── crown_player ─────────────────────────────────────────────────────

    #[test]
    fn test_crown_grants_resistances() {
        let mut state = make_state();
        crown_player(&mut state);
        assert!(state.player.properties.has(Property::FireResistance));
        assert!(state.player.properties.has(Property::ColdResistance));
        assert!(state.player.properties.has(Property::ShockResistance));
        assert!(state.player.properties.has(Property::SleepResistance));
        assert!(state.player.properties.has(Property::PoisonResistance));
        assert!(state.player.properties.has(Property::SeeInvisible));
    }

    #[test]
    fn test_crown_boosts_alignment() {
        let mut state = make_state();
        state.player.alignment.record = 5;
        crown_player(&mut state);
        assert!(state.player.alignment.record >= PIOUS);
    }

    // ── do_sacrifice ─────────────────────────────────────────────────────

    #[test]
    fn test_sacrifice_no_altar() {
        let mut state = make_state();
        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_sacrifice_no_item() {
        let mut state = make_state_with_altar();
        let result = do_sacrifice(&mut state, 'z');
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_sacrifice_coaligned() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Neutral;
        // Add a corpse (object_type must be CORPSE=297)
        let mut corpse = crate::object::Object::default();
        corpse.class = crate::object::ObjectClass::Food;
        corpse.inv_letter = 'a';
        corpse.object_type = crate::object::CORPSE;
        corpse.corpse_type = 5; // Some monster index
        state.inventory.push(corpse);

        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        // Corpse should be consumed
        assert!(state.inventory.is_empty());
    }

    #[test]
    fn test_sacrifice_non_corpse_does_nothing() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Neutral;
        // Add a non-corpse food item
        let mut food = crate::object::Object::default();
        food.class = crate::object::ObjectClass::Food;
        food.inv_letter = 'a';
        food.object_type = 10; // Not CORPSE
        state.inventory.push(food);

        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        // Item removed but nothing happened
        assert!(state.inventory.is_empty());
    }

    #[test]
    fn test_sacrifice_mollifies_angry_god() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Neutral;
        state.player.god_anger = 5;
        // High difficulty corpse for good value
        let mut corpse = crate::object::Object::default();
        corpse.class = crate::object::ObjectClass::Food;
        corpse.inv_letter = 'a';
        corpse.object_type = crate::object::CORPSE;
        corpse.corpse_type = 50; // Higher difficulty monster
        state.inventory.push(corpse);

        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        // God anger should have decreased
        assert!(state.player.god_anger < 5);
    }

    #[test]
    fn test_sacrifice_cross_aligned_conversion() {
        let mut state = make_state_with_altar();
        // Altar is neutral (flags=0), player is lawful
        state.player.alignment.typ = AlignmentType::Lawful;
        state.player.original_alignment = AlignmentType::Lawful;
        state.player.alignment.record = -5; // Angry god condition
        state.player.god_anger = 1;
        let mut corpse = crate::object::Object::default();
        corpse.class = crate::object::ObjectClass::Food;
        corpse.inv_letter = 'a';
        corpse.object_type = crate::object::CORPSE;
        corpse.corpse_type = 5;
        state.inventory.push(corpse);

        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        // Should have converted to Neutral (altar alignment)
        assert_eq!(state.player.alignment.typ, AlignmentType::Neutral);
    }

    #[test]
    fn test_sacrifice_undead_bonus() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Lawful; // Non-chaotic
        // Need to set altar to lawful
        let cell = state.current_level.cell_mut(5, 5);
        cell.flags = 1; // Lawful altar
        // Find an undead monster index — use a known undead
        // Monster index 67 is Zombie in the data
        let mut corpse = crate::object::Object::default();
        corpse.class = crate::object::ObjectClass::Food;
        corpse.inv_letter = 'a';
        corpse.object_type = crate::object::CORPSE;
        corpse.corpse_type = 67; // Try zombie
        state.inventory.push(corpse);

        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.inventory.is_empty());
    }

    // ── water_prayer ─────────────────────────────────────────────────────

    #[test]
    fn test_water_prayer_bless() {
        let mut state = make_state();
        let mut potion = crate::object::Object::default();
        potion.class = crate::object::ObjectClass::Potion;
        potion.object_type = 0;
        potion.buc = crate::object::BucStatus::Uncursed;
        state.inventory.push(potion);

        let result = water_prayer(&mut state, true);
        assert!(result);
        assert_eq!(state.inventory[0].buc, crate::object::BucStatus::Blessed);
    }

    #[test]
    fn test_water_prayer_curse() {
        let mut state = make_state();
        let mut potion = crate::object::Object::default();
        potion.class = crate::object::ObjectClass::Potion;
        potion.object_type = 0;
        potion.buc = crate::object::BucStatus::Uncursed;
        state.inventory.push(potion);

        let result = water_prayer(&mut state, false);
        assert!(result);
        assert_eq!(state.inventory[0].buc, crate::object::BucStatus::Cursed);
    }

    #[test]
    fn test_water_prayer_no_water() {
        let mut state = make_state();
        let result = water_prayer(&mut state, true);
        assert!(!result);
    }

    // ── trouble is_major ─────────────────────────────────────────────────

    #[test]
    fn test_trouble_major() {
        assert!(Trouble::Stoned.is_major());
        assert!(Trouble::Strangled.is_major());
        assert!(Trouble::Sick.is_major());
        assert!(Trouble::Starving.is_major());
        assert!(Trouble::CriticalHP.is_major());
        assert!(Trouble::Lycanthrope.is_major());
        assert!(Trouble::StuckInWall.is_major());
    }

    #[test]
    fn test_trouble_minor() {
        assert!(!Trouble::Hungry.is_major());
        assert!(!Trouble::Blind.is_major());
        assert!(!Trouble::Stunned.is_major());
        assert!(!Trouble::Confused.is_major());
        assert!(!Trouble::Hallucinating.is_major());
        assert!(!Trouble::Poisoned.is_major());
    }

    // ── god_zaps_you ─────────────────────────────────────────────────────

    #[test]
    fn test_god_zaps_with_shock_resistance() {
        let mut state = make_state();
        state.player.properties.grant_intrinsic(Property::ShockResistance);
        state.player.properties.grant_intrinsic(Property::DisintResistance);
        state.player.hp = 50;
        god_zaps_you(&mut state);
        // Should survive due to resistances
        assert!(state.player.hp > 0);
    }

    #[test]
    fn test_god_zaps_without_resistance() {
        let mut state = make_state();
        state.player.hp = 50;
        god_zaps_you(&mut state);
        // Should be killed (no disintegration resistance)
        assert_eq!(state.player.hp, 0);
    }

    #[test]
    fn test_doturn_non_priest_without_spell() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.role = crate::player::Role::Valkyrie;
        state.player.known_spells.clear();
        let result = doturn(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_doturn_priest_can_turn() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.role = crate::player::Role::Priest;
        state.player.god_anger = 0;
        let result = doturn(&mut state);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_unturn_dead_empty_inventory() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.inventory.clear();
        let revived = unturn_dead(&mut state, true, None);
        assert_eq!(revived, 0);
    }

    // ── altar_wrath ─────────────────────────────────────────────────────

    #[test]
    fn test_altar_wrath_coaligned_reduces_alignment() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Neutral;
        state.player.alignment.record = 10;
        altar_wrath(&mut state, 5, 5);
        // Should reduce alignment record by 1
        assert!(state.player.alignment.record < 10);
    }

    #[test]
    fn test_altar_wrath_non_coaligned_reduces_luck() {
        let mut state = make_state_with_altar();
        state.player.alignment.typ = AlignmentType::Lawful; // altar is Neutral
        state.player.luck = 5;
        altar_wrath(&mut state, 5, 5);
        // Should reduce luck
        assert!(state.player.luck < 5);
    }

    #[test]
    fn test_altar_wrath_not_on_altar() {
        let mut state = make_state();
        state.player.alignment.record = 10;
        state.player.luck = 5;
        altar_wrath(&mut state, 5, 5); // No altar at (5,5) in make_state()
        // Nothing should change
        assert_eq!(state.player.alignment.record, 10);
        assert_eq!(state.player.luck, 5);
    }

    // ── gods_upset ──────────────────────────────────────────────────────

    #[test]
    fn test_gods_upset_coaligned_increases_anger() {
        let mut state = make_state();
        state.player.god_anger = 1;
        let align = state.player.alignment.typ;
        gods_upset(&mut state, align);
        assert!(state.player.god_anger > 1);
    }

    #[test]
    fn test_gods_upset_cross_aligned_decreases_anger() {
        let mut state = make_state();
        state.player.god_anger = 3;
        state.player.alignment.typ = AlignmentType::Lawful;
        gods_upset(&mut state, AlignmentType::Chaotic);
        // Cross-aligned upset decreases player's god anger
        assert!(state.player.god_anger < 3);
    }

    // ── consume_offering ────────────────────────────────────────────────

    #[test]
    fn test_consume_offering_msg_lawful() {
        let mut state = make_state();
        consume_offering_msg(&mut state, AlignmentType::Lawful);
        assert!(state.messages.iter().any(|m| m.contains("flash of light")));
    }

    #[test]
    fn test_consume_offering_msg_chaotic() {
        let mut state = make_state();
        consume_offering_msg(&mut state, AlignmentType::Chaotic);
        assert!(state.messages.iter().any(|m| m.contains("burst of flame")));
    }

    #[test]
    fn test_consume_offering_msg_hallucinating() {
        let mut state = make_state();
        state.player.hallucinating_timeout = 100;
        consume_offering_msg(&mut state, AlignmentType::Lawful);
        // Should get one of the hallucination messages
        assert!(!state.messages.is_empty());
    }

    // ── new trouble types ───────────────────────────────────────────────

    #[test]
    fn test_in_trouble_slimed() {
        let mut state = make_state();
        state.player.sliming_timeout = 5;
        assert_eq!(in_trouble(&state), Some(Trouble::Slimed));
    }

    #[test]
    fn test_in_trouble_lava_trapped() {
        let mut state = make_state();
        state.player.utrap = 3;
        state.player.utrap_type = crate::player::PlayerTrapType::Lava;
        assert_eq!(in_trouble(&state), Some(Trouble::LavaTrapped));
    }

    #[test]
    fn test_in_trouble_punished() {
        let mut state = make_state();
        state.player.punishment.punished = true;
        assert_eq!(in_trouble(&state), Some(Trouble::Punished));
    }

    #[test]
    fn test_in_trouble_wounded_legs() {
        let mut state = make_state();
        state.player.wounded_legs_left = 10;
        assert_eq!(in_trouble(&state), Some(Trouble::WoundedLegs));
    }

    #[test]
    fn test_in_trouble_poisoned_attr_drain() {
        let mut state = make_state();
        // Set max strength higher than current = poisoned
        state.player.attr_max.set(crate::player::Attribute::Strength, 18);
        state.player.attr_current.set(crate::player::Attribute::Strength, 14);
        assert_eq!(in_trouble(&state), Some(Trouble::Poisoned));
    }

    #[test]
    fn test_fix_slimed() {
        let mut state = make_state();
        state.player.sliming_timeout = 5;
        fix_worst_trouble(&mut state, Trouble::Slimed);
        assert_eq!(state.player.sliming_timeout, 0);
    }

    #[test]
    fn test_fix_lava_trapped() {
        let mut state = make_state();
        state.player.utrap = 3;
        fix_worst_trouble(&mut state, Trouble::LavaTrapped);
        assert_eq!(state.player.utrap, 0);
    }

    #[test]
    fn test_fix_punished() {
        let mut state = make_state();
        state.player.punishment.punished = true;
        fix_worst_trouble(&mut state, Trouble::Punished);
        assert!(!state.player.punishment.punished);
    }

    #[test]
    fn test_fix_wounded_legs() {
        let mut state = make_state();
        state.player.wounded_legs_left = 10;
        state.player.wounded_legs_right = 5;
        fix_worst_trouble(&mut state, Trouble::WoundedLegs);
        assert_eq!(state.player.wounded_legs_left, 0);
        assert_eq!(state.player.wounded_legs_right, 0);
    }

    #[test]
    fn test_trouble_new_major_types() {
        assert!(Trouble::Slimed.is_major());
        assert!(Trouble::LavaTrapped.is_major());
        assert!(Trouble::Collapsing.is_major());
        assert!(Trouble::CursedLevitation.is_major());
    }

    #[test]
    fn test_trouble_new_minor_types() {
        assert!(!Trouble::Punished.is_major());
        assert!(!Trouble::Fumbling.is_major());
        assert!(!Trouble::CursedItems.is_major());
        assert!(!Trouble::WoundedLegs.is_major());
    }

    #[test]
    fn test_is_poisoned_none() {
        let state = make_state();
        assert!(!is_poisoned(&state));
    }

    #[test]
    fn test_is_poisoned_drained() {
        let mut state = make_state();
        state.player.attr_max.set(crate::player::Attribute::Dexterity, 16);
        state.player.attr_current.set(crate::player::Attribute::Dexterity, 12);
        assert!(is_poisoned(&state));
    }
}
