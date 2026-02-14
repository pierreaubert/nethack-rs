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

use crate::action::ActionResult;
use crate::dungeon::CellType;
use crate::gameloop::GameState;
use crate::player::{AlignmentType, HungerState, Property};

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
    // Major troubles (positive, ordered by severity)
    Stoned,
    Strangled,
    Sick,
    Starving,
    CriticalHP,
    Lycanthrope,
    StuckInWall,
    // Minor troubles (less severe)
    Hungry,
    Poisoned,
    Blind,
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
                | Trouble::Strangled
                | Trouble::Sick
                | Trouble::Starving
                | Trouble::CriticalHP
                | Trouble::Lycanthrope
                | Trouble::StuckInWall
        )
    }
}

/// Detect the player's worst trouble (C: in_trouble)
///
/// Returns None if the player has no notable troubles.
/// Major troubles are returned first (in priority order), then minor ones.
pub fn in_trouble(state: &GameState) -> Option<Trouble> {
    // Major troubles (ordered by severity)
    if state.player.stoning > 0 {
        return Some(Trouble::Stoned);
    }
    if state.player.strangled > 0 {
        return Some(Trouble::Strangled);
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
    if stuck_in_wall(state) {
        return Some(Trouble::StuckInWall);
    }

    // Minor troubles
    if matches!(state.player.hunger_state, HungerState::Hungry) {
        return Some(Trouble::Hungry);
    }
    if state.player.blinded_timeout > 0 {
        return Some(Trouble::Blind);
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
        Trouble::Strangled => {
            state.player.strangled = 0;
            state.message("You can breathe again.");
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
        Trouble::StuckInWall => {
            // Teleport to safety
            let (nx, ny) = crate::action::teleport::safe_teleds_pub(state);
            state.player.pos.x = nx;
            state.player.pos.y = ny;
            state.message("Your surroundings change.");
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
fn angry_gods(state: &mut GameState) {
    let god = state.player.alignment.typ.default_god();
    let anger = state.player.god_anger;

    // Calculate punishment severity
    let max_anger = if anger > 0 {
        3 * anger + state.player.luck.abs() as i32
    } else {
        state.player.luck.abs() as i32
    };

    if max_anger >= 3 && state.rng.rn2(max_anger as u32) >= 3 {
        god_zaps_you(state);
    } else if max_anger >= 2 && state.rng.rn2(max_anger as u32 + 1) >= 2 {
        // Summon hostile monster
        state.message(format!("{} sends a minion against you!", god));
        // TODO: actually summon a minion when makemon is wired
    } else {
        // Mild punishment: lose luck, increase anger
        state.message(format!("{} is displeased.", god));
        state.player.luck = (state.player.luck - 1).max(-10);
    }

    // Increase anger for future prayers
    state.player.god_anger += 1;
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

/// Sacrifice a corpse at an altar (C: dosacrifice)
pub fn do_sacrifice(state: &mut GameState, corpse_letter: char) -> ActionResult {
    // Check if on an altar
    if !is_on_altar(state) {
        state.message("You are not standing on an altar.");
        return ActionResult::NoTime;
    }

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

    let corpse_type = state.inventory[corpse_idx].object_type;
    let altar_align = altar_alignment_at(state).unwrap_or(state.player.alignment.typ);

    // Remove the corpse from inventory
    state.inventory.remove(corpse_idx);

    // Calculate sacrifice value
    let value = sacrifice_value(corpse_type, altar_align, state.player.alignment.typ);

    if altar_align == state.player.alignment.typ {
        // Coaligned sacrifice
        if value > 0 {
            state.player.alignment.increase(value);
            state.message("Your offering is consumed in a flash of light!");

            // Check for gift/crowning eligibility
            if state.player.alignment.record >= PIOUS {
                // Chance of crowning or artifact gift
                if state.rng.rn2(10) == 0 {
                    crown_player(state);
                }
            }
        } else {
            state.message("Your offering is consumed, but nothing seems to happen.");
        }
    } else {
        // Wrong-alignment sacrifice — anger
        state.message("Your sacrifice is consumed in a burst of flame!");
        state.message("You sense the anger of your god!");
        state.player.alignment.decrease(3);
        state.player.god_anger += 1;
    }

    ActionResult::Success
}

/// Calculate sacrifice value based on corpse type and altar alignment
fn sacrifice_value(corpse_type: i16, altar_align: AlignmentType, player_align: AlignmentType) -> i32 {
    // Base value from monster type (level-dependent in C)
    // For now, use a simple estimate
    let base = (corpse_type as i32 / 10).max(1);

    if altar_align == player_align {
        base
    } else {
        -base
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
            angry_gods(state);
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
        let old_anger = state.player.god_anger;
        angry_gods(&mut state);
        assert!(state.player.god_anger > old_anger);
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
        // Add a food item
        let mut corpse = crate::object::Object::default();
        corpse.class = crate::object::ObjectClass::Food;
        corpse.inv_letter = 'a';
        corpse.object_type = 10;
        state.inventory.push(corpse);

        let old_record = state.player.alignment.record;
        let result = do_sacrifice(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.alignment.record >= old_record);
        // Corpse should be consumed
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
}
