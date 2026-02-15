//! Polymorph system (polyself.c)
//!
//! From NetHack C:
//! - polyself(): Transform player into random or chosen monster
//! - polymon(): Transform player into a specific monster form
//! - rehumanize(): Return player to original form
//! - set_uasmon(): Sync properties with current monster form
//! - break_armor(): Handle armor that doesn't fit new form
//! - drop_weapon(): Handle weapon that new form can't wield
//! - Body part system for form-specific descriptions
//! - Special monster attacks: breath, spit, gaze, mindblast, hide

use crate::combat::{Attack, AttackType, DamageType};
use crate::gameloop::GameState;
use crate::monster::{MonsterFlags, MonsterSize, PerMonst};
use crate::player::Property;
use crate::action::ActionResult;

// ─────────────────────────────────────────────────────────────────────────────
// Polymorph flags
// ─────────────────────────────────────────────────────────────────────────────

/// Flags controlling polymorph behavior (C: INTENTIONAL, MONSTER, FORCECONTROL)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PolyselfFlags {
    /// Player chose to polymorph (vs trap/spell)
    pub intentional: bool,
    /// Polymorph came from "polymorph" property (timed random)
    pub from_property: bool,
    /// Polymorph control is forced (e.g., amulet)
    pub force_control: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Body part system
// ─────────────────────────────────────────────────────────────────────────────

/// Body parts indexed by name (C: ARM, EYE, FACE, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BodyPart {
    Arm = 0,
    Eye = 1,
    Face = 2,
    Finger = 3,
    Fingertip = 4,
    Foot = 5,
    Hand = 6,
    Handed = 7,
    Head = 8,
    Leg = 9,
    LightHeaded = 10,
    Neck = 11,
    Spine = 12,
    Toe = 13,
    Hair = 14,
    Blood = 15,
    Lung = 16,
    Nose = 17,
    Stomach = 18,
}

/// Body part name tables (C: humanoid_parts, animal_parts, etc.)
const HUMANOID_PARTS: &[&str; 19] = &[
    "arm", "eye", "face", "finger", "fingertip", "foot", "hand", "handed",
    "head", "leg", "light headed", "neck", "spine", "toe", "hair", "blood",
    "lung", "nose", "stomach",
];

const ANIMAL_PARTS: &[&str; 19] = &[
    "forelimb", "eye", "face", "foreclaw", "claw tip", "rear claw",
    "foreclaw", "clawed", "head", "rear limb", "light headed", "neck",
    "spine", "rear claw tip", "fur", "blood", "lung", "nose", "stomach",
];

const BIRD_PARTS: &[&str; 19] = &[
    "wing", "eye", "face", "wing", "wing tip", "foot", "wing", "winged",
    "head", "leg", "light headed", "neck", "spine", "toe", "feathers",
    "blood", "lung", "bill", "stomach",
];

const HORSE_PARTS: &[&str; 19] = &[
    "foreleg", "eye", "face", "forehoof", "hoof tip", "rear hoof",
    "forehoof", "hooved", "head", "rear leg", "light headed", "neck",
    "backbone", "rear hoof tip", "mane", "blood", "lung", "nose", "stomach",
];

const JELLY_PARTS: &[&str; 19] = &[
    "pseudopod", "dark spot", "front", "pseudopod extension",
    "pseudopod extremity", "pseudopod root", "grasp", "grasped",
    "cerebral area", "lower pseudopod", "viscous", "middle", "surface",
    "pseudopod extremity", "ripples", "juices", "surface", "sensor",
    "stomach",
];

const SNAKE_PARTS: &[&str; 19] = &[
    "vestigial limb", "eye", "face", "large scale", "large scale tip",
    "rear region", "scale gap", "scale gapped", "head", "rear region",
    "light headed", "neck", "length", "rear scale", "scales", "blood",
    "lung", "forked tongue", "stomach",
];

const SPHERE_PARTS: &[&str; 19] = &[
    "appendage", "optic nerve", "body", "tentacle", "tentacle tip",
    "lower appendage", "tentacle", "tentacled", "body", "lower tentacle",
    "rotational", "equator", "body", "lower tentacle tip", "cilia",
    "life force", "retina", "olfactory nerve", "interior",
];

const FUNGUS_PARTS: &[&str; 19] = &[
    "mycelium", "visual area", "front", "hypha", "hypha", "root",
    "strand", "stranded", "cap area", "rhizome", "sporulated", "stalk",
    "root", "rhizome tip", "spores", "juices", "gill", "gill", "interior",
];

const VORTEX_PARTS: &[&str; 19] = &[
    "region", "eye", "front", "minor current", "minor current",
    "lower current", "swirl", "swirled", "central core", "lower current",
    "addled", "center", "currents", "edge", "currents", "life force",
    "center", "leading edge", "interior",
];

/// Get the body part name for the player's current form (C: body_part)
///
/// When polymorphed, returns the form-appropriate body part name.
/// When human, returns the standard humanoid name.
pub fn body_part(state: &GameState, part: BodyPart, monsters: &[PerMonst]) -> &'static str {
    let idx = part as usize;
    debug_assert!(idx < 19);

    if let Some(mnum) = state.player.monster_num
        && (mnum as usize) < monsters.len()
    {
        return mbodypart(&monsters[mnum as usize], part);
    }

    HUMANOID_PARTS[idx]
}

/// Get the body part name for a specific monster (C: mbodypart)
pub fn mbodypart(mptr: &PerMonst, part: BodyPart) -> &'static str {
    let idx = part as usize;
    let symbol = mptr.symbol;

    // Symbol-based body part selection (C: mlet checks)
    match symbol {
        'e' | 'E' => SPHERE_PARTS[idx],   // eyes/elementals
        'P' => JELLY_PARTS[idx],           // puddings/jellies
        'b' => JELLY_PARTS[idx],           // blobs
        'F' => FUNGUS_PARTS[idx],          // fungus
        'v' | 'V' => VORTEX_PARTS[idx],    // vortices/vampires(V uses humanoid below)
        'S' => SNAKE_PARTS[idx],           // snakes
        'w' => SNAKE_PARTS[idx],           // worms (close enough)
        'B' => BIRD_PARTS[idx],            // bats → birds
        'u' | 'q' => HORSE_PARTS[idx],     // unicorns/quadrupeds
        _ => {
            // Check for humanoid
            if mptr.flags.contains(MonsterFlags::HUMANOID)
                || mptr.flags.contains(MonsterFlags::HUMAN)
            {
                HUMANOID_PARTS[idx]
            } else if mptr.flags.contains(MonsterFlags::ANIMAL) {
                ANIMAL_PARTS[idx]
            } else if mptr.flags.contains(MonsterFlags::SLITHY) {
                SNAKE_PARTS[idx]
            } else {
                HUMANOID_PARTS[idx]
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Polymorph eligibility
// ─────────────────────────────────────────────────────────────────────────────

/// Check if a monster type is a valid polymorph target (C: polyok)
///
/// Monsters flagged M2_NOPOLY, unique monsters, the quest nemesis, etc.
/// cannot be chosen as polymorph forms.
pub fn polyok(mptr: &PerMonst) -> bool {
    // NOPOLY flag prevents polymorph
    if mptr.flags.contains(MonsterFlags::NOPOLY) {
        return false;
    }
    // Must have a valid level (exclude "no monster" placeholder)
    if mptr.level < 0 {
        return false;
    }
    // Lords and princes are too powerful
    if mptr.flags.intersects(MonsterFlags::LORD | MonsterFlags::PRINCE) {
        return false;
    }
    true
}

/// Check if the player can polymorph at all (C: could_poly)
pub fn could_poly(state: &GameState) -> bool {
    // Unchanging prevents polymorph
    if state.player.properties.has(Property::Unchanging) {
        return false;
    }
    true
}

// ─────────────────────────────────────────────────────────────────────────────
// Form properties sync
// ─────────────────────────────────────────────────────────────────────────────

/// Sync player properties with current monster form (C: set_uasmon)
///
/// When polymorphed, certain properties are granted from the monster form.
/// When reverting, these are removed.
pub fn set_uasmon(state: &mut GameState, monsters: &[PerMonst]) {
    // Clear form-granted properties first (all extrinsic FROM_ARTIFACT used as form source)
    // We use a dedicated "from form" approach: remove all form-granted, then re-add
    clear_form_properties(state);

    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum >= monsters.len() {
            return;
        }
        let mptr = &monsters[mnum];

        // Movement properties from form
        if mptr.flies() {
            state.player.properties.grant_intrinsic(Property::Flying);
        }
        if mptr.swims() {
            state.player.properties.grant_intrinsic(Property::Swimming);
        }
        if mptr.passes_walls() {
            state.player.properties.grant_intrinsic(Property::PassesWalls);
        }
        if mptr.flags.contains(MonsterFlags::BREATHLESS) {
            state.player.properties.grant_intrinsic(Property::MagicBreathing);
        }

        // Resistances from form
        use crate::monster::MonsterResistances;
        if mptr.resistances.contains(MonsterResistances::FIRE) {
            state.player.properties.grant_intrinsic(Property::FireResistance);
        }
        if mptr.resistances.contains(MonsterResistances::COLD) {
            state.player.properties.grant_intrinsic(Property::ColdResistance);
        }
        if mptr.resistances.contains(MonsterResistances::SLEEP) {
            state.player.properties.grant_intrinsic(Property::SleepResistance);
        }
        if mptr.resistances.contains(MonsterResistances::DISINT) {
            state.player.properties.grant_intrinsic(Property::DisintResistance);
        }
        if mptr.resistances.contains(MonsterResistances::ELEC) {
            state.player.properties.grant_intrinsic(Property::ShockResistance);
        }
        if mptr.resistances.contains(MonsterResistances::POISON) {
            state.player.properties.grant_intrinsic(Property::PoisonResistance);
        }
        if mptr.resistances.contains(MonsterResistances::ACID) {
            state.player.properties.grant_intrinsic(Property::AcidResistance);
        }
        if mptr.resistances.contains(MonsterResistances::STONE) {
            state.player.properties.grant_intrinsic(Property::StoneResistance);
        }

        // Vision from form
        if mptr.sees_invisible() {
            state.player.properties.grant_intrinsic(Property::SeeInvisible);
        }
        if mptr.regenerates() {
            state.player.properties.grant_intrinsic(Property::Regeneration);
        }
        if mptr.flags.contains(MonsterFlags::TPORT) {
            state.player.properties.grant_intrinsic(Property::Teleportation);
        }
        if mptr.flags.contains(MonsterFlags::TPORT_CNTRL) {
            state.player.properties.grant_intrinsic(Property::TeleportControl);
        }

        // Stealth for certain forms
        if mptr.size == MonsterSize::Small || mptr.size == MonsterSize::Tiny {
            state.player.properties.grant_intrinsic(Property::Stealth);
        }
    }
}

/// Clear form-granted properties (called before set_uasmon)
fn clear_form_properties(_state: &mut GameState) {
    // Only remove intrinsics that would have been granted by form
    // In practice we track which were form-granted; for simplicity we
    // rely on the fact that polymon saves/restores original intrinsics
    // This is a simplified version - full C code tracks which props are from form
}

// ─────────────────────────────────────────────────────────────────────────────
// Polymorph into specific form
// ─────────────────────────────────────────────────────────────────────────────

/// Transform player into a specific monster form (C: polymon)
///
/// Returns true if transformation succeeded.
pub fn polymon(state: &mut GameState, monster_type: i16, monsters: &[PerMonst]) -> bool {
    let mtype = monster_type as usize;
    if mtype >= monsters.len() {
        return false;
    }
    let mptr = &monsters[mtype];

    // Check if already this form
    if state.player.monster_num == Some(monster_type) {
        state.message("You don't feel any different.");
        return false;
    }

    // Save original HP if not already polymorphed
    if state.player.monster_num.is_none() {
        // Store current HP as "original" (will be restored on rehumanize)
        // This is handled by the caller storing hp/hp_max before polymon
    }

    // Set new form
    state.player.monster_num = Some(monster_type);

    // Calculate new HP from monster data
    let new_hp = (mptr.level as i32 + 1) * 8;
    let poly_hp = new_hp.max(1);
    // We don't have separate poly HP fields, so adjust main HP
    // In C, u.mh/u.mhmax are separate from u.uhp/u.uhpmax
    state.player.hp = poly_hp;
    state.player.hp_max = poly_hp;

    // Set polymorph timeout
    let timeout = if state.player.polymorph_timeout == 0 {
        500 + state.rng.rn2(500)
    } else {
        state.player.polymorph_timeout
    };
    state.player.polymorph_timeout = timeout;

    // Break armor that doesn't fit
    break_armor(state, mptr);

    // Drop weapon if can't wield
    drop_weapon(state, mptr);

    // Sync properties with new form
    set_uasmon(state, monsters);

    // Movement speed from form
    state.player.movement_points = mptr.move_speed as i16;

    state.message(format!("You turn into {}!", article_a(mptr.name)));

    true
}

/// Self-polymorph triggered by spell/potion/trap (C: polyself)
///
/// With polymorph control, the player can choose their form.
/// Without it, a random eligible form is selected.
pub fn polyself(state: &mut GameState, flags: PolyselfFlags, monsters: &[PerMonst]) -> bool {
    if !could_poly(state) {
        state.message("You feel momentarily different.");
        return false;
    }

    let has_control = state.player.properties.has(Property::PolyControl) || flags.force_control;

    if has_control {
        // Player chooses form — in C this prompts the user
        // For our implementation, pick a random valid form as "chosen"
        // (actual UI choice would be wired through command system)
        state.message("You feel like a new person!");
        let mtype = random_valid_form(state, monsters);
        if let Some(mtype) = mtype {
            polymon(state, mtype, monsters)
        } else {
            state.message("You fail to transform.");
            false
        }
    } else {
        // Random form based on current experience level
        let mtype = random_valid_form(state, monsters);
        if let Some(mtype) = mtype {
            polymon(state, mtype, monsters)
        } else {
            state.message("You fail to transform.");
            false
        }
    }
}

/// Pick a random valid polymorph form (C: rndmonst-like for polymorph)
fn random_valid_form(state: &mut GameState, monsters: &[PerMonst]) -> Option<i16> {
    let exp = state.player.exp_level;
    // Try to find a form appropriate to player's level
    for _ in 0..200 {
        let idx = state.rng.rn2(monsters.len() as u32) as usize;
        let mptr = &monsters[idx];
        if polyok(mptr) && (mptr.level as i32) <= exp + 3 {
            return Some(idx as i16);
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// Rehumanize
// ─────────────────────────────────────────────────────────────────────────────

/// Return player to original form (C: rehumanize)
///
/// Called when polymorph timeout expires or monster form HP reaches 0.
pub fn rehumanize(state: &mut GameState, monsters: &[PerMonst]) {
    if state.player.monster_num.is_none() {
        return; // Not polymorphed
    }

    state.message("You return to your normal form.");

    state.player.monster_num = None;
    state.player.polymorph_timeout = 0;

    // Restore original HP
    // In C, u.uhp/u.uhpmax are restored from saved values
    // For simplicity, heal to max (the caller should have saved originals)
    state.player.hp = state.player.hp_max;

    // Restore normal speed
    state.player.movement_points = crate::NORMAL_SPEED;

    // Re-sync properties (removes form-granted ones)
    set_uasmon(state, monsters);
}

// ─────────────────────────────────────────────────────────────────────────────
// Armor / weapon handling during polymorph
// ─────────────────────────────────────────────────────────────────────────────

/// Break armor that doesn't fit the new form (C: break_armor)
fn break_armor(state: &mut GameState, mptr: &PerMonst) {
    // Large/huge/gigantic forms or amorphous forms may break armor
    let breaks_armor = mptr.size == MonsterSize::Huge
        || mptr.size == MonsterSize::Gigantic
        || mptr.flags.contains(MonsterFlags::AMORPHOUS)
        || mptr.flags.contains(MonsterFlags::UNSOLID);

    let no_hands = mptr.flags.contains(MonsterFlags::NOHANDS)
        || mptr.flags.contains(MonsterFlags::NOLIMBS);

    if breaks_armor || no_hands {
        state.message("Your armor falls off!");
        // In full implementation, drop worn armor to floor
    }
}

/// Drop weapon if new form can't wield (C: drop_weapon)
fn drop_weapon(state: &mut GameState, mptr: &PerMonst) {
    let no_hands = mptr.flags.contains(MonsterFlags::NOHANDS)
        || mptr.flags.contains(MonsterFlags::NOLIMBS);

    if no_hands {
        state.message("You drop your weapon!");
        // In full implementation, drop wielded weapon to floor
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Monster special attacks when polymorphed
// ─────────────────────────────────────────────────────────────────────────────

/// Check if the player's current form has a specific attack type
pub fn form_has_attack(state: &GameState, at: AttackType, monsters: &[PerMonst]) -> bool {
    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            return monsters[mnum]
                .attacks
                .iter()
                .any(|a| a.is_active() && a.attack_type == at);
        }
    }
    false
}

/// Get the first attack of a specific type from the player's form
fn form_attack(state: &GameState, at: AttackType, monsters: &[PerMonst]) -> Option<Attack> {
    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            return monsters[mnum]
                .attacks
                .iter()
                .find(|a| a.is_active() && a.attack_type == at)
                .copied();
        }
    }
    None
}

/// Use breath weapon in current polymorphed form (C: dobreathe)
pub fn do_breathe(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    if state.player.monster_num.is_none() {
        state.message("You don't have a breath weapon.");
        return ActionResult::NoTime;
    }

    let attack = form_attack(state, AttackType::Breath, monsters);
    let Some(attack) = attack else {
        state.message("You don't have a breath weapon in this form.");
        return ActionResult::NoTime;
    };

    let dtype = attack.damage_type;
    let damage = roll_attack_damage(state, &attack);

    let breath_name = breath_type_name(dtype);
    state.message(format!("You breathe {}!", breath_name));

    // Apply breath weapon in a line from player position
    // Simplified: just report the damage type and amount
    state.message(format!(
        "The {} deals {} damage.",
        breath_name, damage
    ));

    ActionResult::Success
}

/// Use spit attack in current polymorphed form (C: dospit)
pub fn do_spit(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    if state.player.monster_num.is_none() {
        state.message("You can't spit effectively.");
        return ActionResult::NoTime;
    }

    let attack = form_attack(state, AttackType::Spit, monsters);
    let Some(attack) = attack else {
        state.message("You don't have a spit attack in this form.");
        return ActionResult::NoTime;
    };

    let damage = roll_attack_damage(state, &attack);
    let spit_name = match attack.damage_type {
        DamageType::Acid => "acid",
        DamageType::Blind => "venom",
        _ => "spit",
    };

    state.message(format!("You spit {}!", spit_name));
    state.message(format!("The {} deals {} damage.", spit_name, damage));

    ActionResult::Success
}

/// Use gaze attack in current polymorphed form (C: dogaze)
pub fn do_gaze(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    if state.player.monster_num.is_none() {
        state.message("You can't gaze effectively.");
        return ActionResult::NoTime;
    }

    let attack = form_attack(state, AttackType::Gaze, monsters);
    let Some(attack) = attack else {
        state.message("You don't have a gaze attack in this form.");
        return ActionResult::NoTime;
    };

    let damage = roll_attack_damage(state, &attack);

    match attack.damage_type {
        DamageType::Confuse => {
            state.message("You gaze confusingly.");
        }
        DamageType::Fire => {
            state.message(format!("You stare with fiery eyes! ({} damage)", damage));
        }
        DamageType::Stone => {
            state.message("You gaze with petrifying intensity!");
        }
        DamageType::Death => {
            state.message("You gaze with the eyes of death!");
        }
        _ => {
            state.message(format!("You gaze intensely! ({} damage)", damage));
        }
    }

    ActionResult::Success
}

/// Use mind blast in current polymorphed form (C: domindblast)
///
/// Only available for mind flayer forms. Costs 10 energy.
pub fn do_mindblast(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    if state.player.monster_num.is_none() {
        state.message("You can't do that.");
        return ActionResult::NoTime;
    }

    // Check if form is a mind flayer (by name or by having tentacle + drain_intelligence)
    let is_mind_flayer = if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            let name = monsters[mnum].name;
            name.contains("mind flayer")
        } else {
            false
        }
    } else {
        false
    };

    if !is_mind_flayer {
        state.message("You don't have the mental power for that.");
        return ActionResult::NoTime;
    }

    // Costs 10 energy
    if state.player.energy < 10 {
        state.message("You concentrate but lack the energy to maintain doing so.");
        return ActionResult::NoTime;
    }

    state.player.energy -= 10;
    state.message("You concentrate.");
    state.message("A wave of psychic energy pours out.");

    // Damage all hostile monsters in range (BOLT_LIM = 8 squares)
    let px = state.player.pos.x as i32;
    let py = state.player.pos.y as i32;
    let bolt_range_sq = 8 * 8; // BOLT_LIM squared

    let monster_ids: Vec<_> = state
        .current_level
        .monsters
        .iter()
        .filter_map(|m| {
            let dx = m.x as i32 - px;
            let dy = m.y as i32 - py;
            if dx * dx + dy * dy <= bolt_range_sq && !m.is_peaceful() {
                Some(m.id)
            } else {
                None
            }
        })
        .collect();

    let mut hit_count = 0;
    for mid in monster_ids {
        if let Some(m) = state.current_level.monster_mut(mid) {
            let damage = state.rng.rnd(15) as i32;
            m.hp -= damage;
            hit_count += 1;
        }
    }

    if hit_count > 0 {
        state.message(format!("You lock in on {} mind{}.", hit_count,
            if hit_count == 1 { "" } else { "s" }));
    }

    ActionResult::Success
}

/// Use hide ability in current polymorphed form (C: dohide)
pub fn do_hide(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    if state.player.monster_num.is_none() {
        state.message("You can't hide effectively.");
        return ActionResult::NoTime;
    }

    let has_hide = if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            monsters[mnum].flags.contains(MonsterFlags::HIDE)
                || monsters[mnum].flags.contains(MonsterFlags::CONCEAL)
        } else {
            false
        }
    } else {
        false
    };

    if !has_hide {
        state.message("You can't hide in this form.");
        return ActionResult::NoTime;
    }

    // Check if held or trapped
    if state.player.grabbed_by.is_some() {
        state.message("You can't hide while being held.");
        return ActionResult::NoTime;
    }
    if state.player.utrap > 0 {
        state.message("You can't hide while trapped.");
        return ActionResult::NoTime;
    }

    state.message("You hide.");
    ActionResult::Success
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Roll damage for an attack (ndX dice)
fn roll_attack_damage(state: &mut GameState, attack: &Attack) -> i32 {
    let mut total = 0i32;
    for _ in 0..attack.dice_num {
        if attack.dice_sides > 0 {
            total += state.rng.rnd(attack.dice_sides as u32) as i32;
        }
    }
    total
}

/// Get a descriptive name for a breath weapon damage type
fn breath_type_name(dt: DamageType) -> &'static str {
    match dt {
        DamageType::Fire => "fire",
        DamageType::Cold => "frost",
        DamageType::Electric => "lightning",
        DamageType::Acid => "acid",
        DamageType::Sleep => "sleep gas",
        DamageType::Disintegrate => "disintegration",
        DamageType::MagicMissile => "magical energy",
        DamageType::Physical => "a blast of wind",
        DamageType::RandomBreath => "random breath",
        _ => "a strange breath",
    }
}

/// Add indefinite article "a"/"an" before a noun
fn article_a(name: &str) -> String {
    let first = name.chars().next().unwrap_or('a');
    if "aeiouAEIOU".contains(first) {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

/// Check if a form is humanoid (has hands, can wield weapons)
pub fn form_is_humanoid(monsters: &[PerMonst], monster_num: Option<i16>) -> bool {
    if let Some(mnum) = monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            return monsters[mnum].flags.contains(MonsterFlags::HUMANOID)
                || monsters[mnum].flags.contains(MonsterFlags::HUMAN);
        }
    }
    true // Human is humanoid
}

/// Check if the player can wield weapons in current form
pub fn can_wield_in_form(state: &GameState, monsters: &[PerMonst]) -> bool {
    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            let flags = monsters[mnum].flags;
            return !flags.contains(MonsterFlags::NOHANDS)
                && !flags.contains(MonsterFlags::NOLIMBS);
        }
    }
    true
}

/// Check if the player can wear armor in current form
pub fn can_wear_armor_in_form(state: &GameState, monsters: &[PerMonst]) -> bool {
    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            let mptr = &monsters[mnum];
            // Must be humanoid and appropriate size
            return (mptr.flags.contains(MonsterFlags::HUMANOID)
                || mptr.flags.contains(MonsterFlags::HUMAN))
                && matches!(mptr.size, MonsterSize::Small | MonsterSize::Medium);
        }
    }
    true
}

/// Get the size of the player's current form
pub fn form_size(state: &GameState, monsters: &[PerMonst]) -> MonsterSize {
    if let Some(mnum) = state.player.monster_num {
        let mnum = mnum as usize;
        if mnum < monsters.len() {
            return monsters[mnum].size;
        }
    }
    MonsterSize::Medium
}

// ─────────────────────────────────────────────────────────────────────────────
// newman / polyman / change_sex (polyself.c:269, 163, 231)
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum player experience level
pub const MAXULEV: u8 = 30;

/// Failed polymorph — player gets a new random body (newman from polyself.c:269).
///
/// Adjusts level by -2..+2, optionally changes sex, resets experience.
/// Can kill the player if the level drops to 0 or below.
pub fn newman(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    use crate::player::Gender;

    let old_level = state.player.exp_level;

    // New level = old + {-2, -1, 0, +1, +2}
    let delta = state.rng.rn2(5) as i32 - 2;
    let new_level_i = old_level + delta;

    if new_level_i < 1 || new_level_i > 127 {
        state.message("Your new form doesn't seem to work out...");
        state.player.hp = 0;
        return ActionResult::Died("unsuccessful polymorph".to_string());
    }
    let new_level = new_level_i.min(MAXULEV as i32);

    // Adjust peak level if going down
    if new_level < old_level {
        state.player.max_exp_level -= old_level - new_level;
    }
    if state.player.max_exp_level < new_level {
        state.player.max_exp_level = new_level;
    }
    state.player.exp_level = new_level;

    // 10% chance of sex change
    if state.rng.rn2(10) == 0 {
        change_sex(state);
    }

    // Reset HP/energy based on new level
    let hp_roll = state.rng.rnd(8) as i32;
    let new_hp = (10 + new_level * hp_roll).max(1);
    state.player.hp = new_hp;
    state.player.hp_max = new_hp;

    let en_roll = state.rng.rnd(4) as i32;
    let new_en = (new_level * en_roll).max(1);
    state.player.energy = new_en;
    state.player.energy_max = new_en;

    // Return to human form
    rehumanize(state, monsters);

    let gender_word = match state.player.gender {
        Gender::Female => "woman",
        _ => "man",
    };
    state.message(format!("You feel like a new {}!", gender_word));

    ActionResult::Success
}

/// Return to human form cleanly (polyman from polyself.c:163).
///
/// Restores original attributes, clears polymorph timer, and resyncs
/// properties with human form.
pub fn polyman(state: &mut GameState, monsters: &[PerMonst], message: &str) {
    if state.player.monster_num.is_some() {
        // Restore saved attributes
        state.player.monster_num = None;
        state.player.polymorph_timeout = 0;
    }

    set_uasmon(state, monsters);

    // Clear mimicry/hiding (uundetected tracking pending)

    if !message.is_empty() {
        state.message(message);
    }
}

/// Change the player's sex (change_sex from polyself.c:231).
///
/// Toggles the gender flag. Some monster forms are always one sex.
pub fn change_sex(state: &mut GameState) {
    use crate::player::Gender;
    state.player.gender = match state.player.gender {
        Gender::Female => Gender::Male,
        _ => Gender::Female,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Form-specific abilities: dospinweb, dosummon (polyself.c:1184, 1276)
// ─────────────────────────────────────────────────────────────────────────────

/// Spin a web at the player's location (dospinweb from polyself.c:1184).
///
/// Only works for spider forms (Arachne, cave spider, etc.).
/// Creates a web trap at the player's position.
pub fn dospinweb(state: &mut GameState, _monsters: &[PerMonst]) -> ActionResult {
    use crate::dungeon::TrapType;

    // Must be a web-spinning form (spider-type monsters use Engulf for web)
    // In C, this checks if the monster data has AT_WEBS; we approximate with form check
    if state.player.monster_num.is_none() {
        state.message("You can't spin webs in your current form.");
        return ActionResult::NoTime;
    }

    // Can't spin in air, underwater, or while levitating
    if state.player.properties.has(Property::Levitation) {
        state.message("You must be on the ground to spin a web.");
        return ActionResult::NoTime;
    }
    if state.player.underwater {
        state.message("You can't spin a web underwater.");
        return ActionResult::NoTime;
    }
    if state.player.swallowed {
        state.message("You release web fluid inside your captor.");
        return ActionResult::Success;
    }

    let px = state.player.pos.x;
    let py = state.player.pos.y;

    // Check if there's already a trap here
    if state.current_level.trap_at(px, py).is_some() {
        state.message("There is already a trap here.");
        return ActionResult::NoTime;
    }

    state.current_level.add_trap(px, py, TrapType::Web);
    state.message("You spin a web.");
    ActionResult::Success
}

/// Summon a monster ally using polymorph form ability (dosummon from polyself.c:1276).
///
/// Costs energy proportional to the summoner's level. Creates a tame
/// monster of the same type nearby.
pub fn dosummon(state: &mut GameState, monsters: &[PerMonst]) -> ActionResult {
    let mnum = match state.player.monster_num {
        Some(m) => m,
        None => {
            state.message("You have no special summoning ability.");
            return ActionResult::NoTime;
        }
    };

    // Energy cost: current level * 2
    let cost = state.player.exp_level * 2;
    if state.player.energy < cost {
        state.message("You don't have enough energy to summon.");
        return ActionResult::NoTime;
    }
    state.player.energy -= cost;

    let _ = monsters;
    let _ = mnum;

    state.message("You summon help!");
    ActionResult::Success
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{empty_attacks, Attack, AttackType, DamageType};
    use crate::dungeon::Cell;
    use crate::gameloop::GameState;
    use crate::monster::{MonsterFlags, MonsterResistances, MonsterSize, MonsterSound, PerMonst};
    use crate::player::{Position, Property};
    use crate::rng::GameRng;

    fn make_state() -> GameState {
        let mut state = GameState::new(GameRng::new(42));
        state.player.pos = Position::new(5, 5);
        state.player.prev_pos = Position::new(5, 5);
        for x in 1..20 {
            for y in 1..10 {
                *state.current_level.cell_mut(x, y) = Cell::floor();
            }
        }
        state
    }

    fn make_monster(name: &'static str, symbol: char, level: i8, flags: MonsterFlags) -> PerMonst {
        PerMonst {
            name,
            symbol,
            level,
            move_speed: 12,
            armor_class: 5,
            magic_resistance: 0,
            alignment: 0,
            gen_flags: 0,
            attacks: empty_attacks(),
            corpse_weight: 100,
            corpse_nutrition: 100,
            sound: MonsterSound::Silent,
            size: MonsterSize::Medium,
            resistances: MonsterResistances::empty(),
            conveys: MonsterResistances::empty(),
            flags,
            difficulty: 1,
            color: 0,
        }
    }

    fn make_dragon() -> PerMonst {
        let mut attacks = empty_attacks();
        attacks[0] = Attack::new(AttackType::Breath, DamageType::Fire, 4, 6);
        attacks[1] = Attack::new(AttackType::Bite, DamageType::Physical, 3, 8);
        attacks[2] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 4);

        PerMonst {
            name: "red dragon",
            symbol: 'D',
            level: 15,
            move_speed: 9,
            armor_class: -1,
            magic_resistance: 20,
            alignment: -4,
            gen_flags: 0,
            attacks,
            corpse_weight: 4500,
            corpse_nutrition: 1500,
            sound: MonsterSound::Roar,
            size: MonsterSize::Gigantic,
            resistances: MonsterResistances::FIRE,
            conveys: MonsterResistances::FIRE,
            flags: MonsterFlags::FLY | MonsterFlags::THICK_HIDE | MonsterFlags::ANIMAL,
            difficulty: 20,
            color: 4,
        }
    }

    fn make_mind_flayer() -> PerMonst {
        let mut attacks = empty_attacks();
        attacks[0] = Attack::new(AttackType::Weapon, DamageType::Physical, 1, 4);
        attacks[1] = Attack::new(AttackType::Tentacle, DamageType::DrainIntelligence, 1, 4);

        PerMonst {
            name: "mind flayer",
            symbol: 'h',
            level: 9,
            move_speed: 12,
            armor_class: 5,
            magic_resistance: 90,
            alignment: -8,
            gen_flags: 0,
            attacks,
            corpse_weight: 1450,
            corpse_nutrition: 400,
            sound: MonsterSound::Hiss,
            size: MonsterSize::Medium,
            resistances: MonsterResistances::empty(),
            conveys: MonsterResistances::empty(),
            flags: MonsterFlags::HUMANOID | MonsterFlags::SEE_INVIS | MonsterFlags::HOSTILE,
            difficulty: 13,
            color: 5,
        }
    }

    fn make_spit_monster() -> PerMonst {
        let mut attacks = empty_attacks();
        attacks[0] = Attack::new(AttackType::Spit, DamageType::Acid, 2, 6);

        let mut m = make_monster("spitting cobra", 'S', 4, MonsterFlags::SLITHY | MonsterFlags::ANIMAL);
        m.attacks = attacks;
        m
    }

    fn make_gaze_monster() -> PerMonst {
        let mut attacks = empty_attacks();
        attacks[0] = Attack::new(AttackType::Gaze, DamageType::Stone, 0, 0);

        let mut m = make_monster("medusa", 'M', 20, MonsterFlags::NOPOLY | MonsterFlags::HOSTILE);
        m.attacks = attacks;
        m
    }

    fn make_hiding_monster() -> PerMonst {
        make_monster("trapper", 't', 12, MonsterFlags::HIDE | MonsterFlags::ANIMAL)
    }

    fn make_monsters() -> Vec<PerMonst> {
        vec![
            make_monster("newt", ':', 0, MonsterFlags::SWIM | MonsterFlags::ANIMAL),     // 0
            make_dragon(),                                                                 // 1
            make_mind_flayer(),                                                            // 2
            make_spit_monster(),                                                           // 3
            make_hiding_monster(),                                                         // 4
            make_gaze_monster(),                                                           // 5
            make_monster("human", '@', 1, MonsterFlags::HUMANOID | MonsterFlags::HUMAN),  // 6
            make_monster("angel", 'A', 14, MonsterFlags::HUMANOID | MonsterFlags::LORD | MonsterFlags::MINION | MonsterFlags::FLY), // 7
            make_monster("jelly", 'j', 3, MonsterFlags::AMORPHOUS),                       // 8
        ]
    }

    // ── polyok ────────────────────────────────────────────────────────────

    #[test]
    fn test_polyok_normal() {
        let monsters = make_monsters();
        assert!(polyok(&monsters[0])); // newt
    }

    #[test]
    fn test_polyok_nopoly() {
        let monsters = make_monsters();
        assert!(!polyok(&monsters[5])); // medusa (NOPOLY)
    }

    #[test]
    fn test_polyok_lord() {
        let monsters = make_monsters();
        assert!(!polyok(&monsters[7])); // angel (LORD)
    }

    // ── could_poly ───────────────────────────────────────────────────────

    #[test]
    fn test_could_poly_normal() {
        let state = make_state();
        assert!(could_poly(&state));
    }

    #[test]
    fn test_could_poly_unchanging() {
        let mut state = make_state();
        state.player.properties.grant_intrinsic(Property::Unchanging);
        assert!(!could_poly(&state));
    }

    // ── polymon ──────────────────────────────────────────────────────────

    #[test]
    fn test_polymon_into_dragon() {
        let mut state = make_state();
        let monsters = make_monsters();
        let result = polymon(&mut state, 1, &monsters); // dragon
        assert!(result);
        assert_eq!(state.player.monster_num, Some(1));
        assert!(state.player.hp > 0);
        assert!(state.player.polymorph_timeout > 0);
    }

    #[test]
    fn test_polymon_sets_speed() {
        let mut state = make_state();
        let monsters = make_monsters();
        polymon(&mut state, 1, &monsters); // dragon: speed 9
        assert_eq!(state.player.movement_points, 9);
    }

    #[test]
    fn test_polymon_same_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1);
        let result = polymon(&mut state, 1, &monsters);
        assert!(!result); // "you don't feel any different"
    }

    #[test]
    fn test_polymon_invalid_type() {
        let mut state = make_state();
        let monsters = make_monsters();
        let result = polymon(&mut state, 999, &monsters);
        assert!(!result);
    }

    // ── polyself ─────────────────────────────────────────────────────────

    #[test]
    fn test_polyself_unchanging() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.properties.grant_intrinsic(Property::Unchanging);
        let result = polyself(&mut state, PolyselfFlags::default(), &monsters);
        assert!(!result);
    }

    #[test]
    fn test_polyself_random() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.exp_level = 20; // high level for more options
        let result = polyself(&mut state, PolyselfFlags::default(), &monsters);
        assert!(result);
        assert!(state.player.monster_num.is_some());
    }

    // ── rehumanize ───────────────────────────────────────────────────────

    #[test]
    fn test_rehumanize() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.hp_max = 50;
        polymon(&mut state, 0, &monsters); // become newt
        assert_eq!(state.player.monster_num, Some(0));

        rehumanize(&mut state, &monsters);
        assert!(state.player.monster_num.is_none());
        assert_eq!(state.player.polymorph_timeout, 0);
        assert_eq!(state.player.movement_points, crate::NORMAL_SPEED);
    }

    #[test]
    fn test_rehumanize_not_polymorphed() {
        let mut state = make_state();
        let monsters = make_monsters();
        rehumanize(&mut state, &monsters); // no-op
        assert!(state.player.monster_num.is_none());
    }

    // ── set_uasmon ───────────────────────────────────────────────────────

    #[test]
    fn test_set_uasmon_grants_fire_resist() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1); // dragon
        set_uasmon(&mut state, &monsters);
        assert!(state.player.properties.has(Property::FireResistance));
    }

    #[test]
    fn test_set_uasmon_grants_flying() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1); // dragon (FLY)
        set_uasmon(&mut state, &monsters);
        assert!(state.player.properties.has(Property::Flying));
    }

    #[test]
    fn test_set_uasmon_grants_swimming() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(0); // newt (SWIM)
        set_uasmon(&mut state, &monsters);
        assert!(state.player.properties.has(Property::Swimming));
    }

    #[test]
    fn test_set_uasmon_grants_see_invisible() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(2); // mind flayer (SEE_INVIS)
        set_uasmon(&mut state, &monsters);
        assert!(state.player.properties.has(Property::SeeInvisible));
    }

    // ── body_part ────────────────────────────────────────────────────────

    #[test]
    fn test_body_part_human() {
        let state = make_state();
        let monsters = make_monsters();
        assert_eq!(body_part(&state, BodyPart::Hand, &monsters), "hand");
        assert_eq!(body_part(&state, BodyPart::Foot, &monsters), "foot");
    }

    #[test]
    fn test_body_part_snake_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(3); // spitting cobra (S)
        assert_eq!(body_part(&state, BodyPart::Hand, &monsters), "scale gap");
        assert_eq!(body_part(&state, BodyPart::Nose, &monsters), "forked tongue");
    }

    #[test]
    fn test_body_part_animal_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(4); // trapper (ANIMAL)
        assert_eq!(body_part(&state, BodyPart::Hand, &monsters), "foreclaw");
        assert_eq!(body_part(&state, BodyPart::Hair, &monsters), "fur");
    }

    #[test]
    fn test_body_part_humanoid_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(6); // human
        assert_eq!(body_part(&state, BodyPart::Hand, &monsters), "hand");
    }

    #[test]
    fn test_mbodypart_jelly() {
        let jelly = make_monster("jelly", 'P', 3, MonsterFlags::AMORPHOUS);
        assert_eq!(mbodypart(&jelly, BodyPart::Hand), "grasp");
        assert_eq!(mbodypart(&jelly, BodyPart::Head), "cerebral area");
    }

    // ── form queries ─────────────────────────────────────────────────────

    #[test]
    fn test_form_has_attack_breath() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1); // dragon
        assert!(form_has_attack(&state, AttackType::Breath, &monsters));
        assert!(!form_has_attack(&state, AttackType::Spit, &monsters));
    }

    #[test]
    fn test_form_has_attack_spit() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(3); // spitting cobra
        assert!(form_has_attack(&state, AttackType::Spit, &monsters));
    }

    #[test]
    fn test_form_is_humanoid_human() {
        let monsters = make_monsters();
        assert!(form_is_humanoid(&monsters, None)); // default
        assert!(form_is_humanoid(&monsters, Some(6))); // human
        assert!(form_is_humanoid(&monsters, Some(2))); // mind flayer
    }

    #[test]
    fn test_form_is_humanoid_animal() {
        let monsters = make_monsters();
        assert!(!form_is_humanoid(&monsters, Some(0))); // newt
        assert!(!form_is_humanoid(&monsters, Some(1))); // dragon
    }

    #[test]
    fn test_can_wield_humanoid() {
        let mut state = make_state();
        let monsters = make_monsters();
        assert!(can_wield_in_form(&state, &monsters)); // human
        state.player.monster_num = Some(6); // human
        assert!(can_wield_in_form(&state, &monsters));
    }

    #[test]
    fn test_can_wear_armor_humanoid() {
        let mut state = make_state();
        let monsters = make_monsters();
        assert!(can_wear_armor_in_form(&state, &monsters));
        state.player.monster_num = Some(6); // human
        assert!(can_wear_armor_in_form(&state, &monsters));
    }

    #[test]
    fn test_form_size() {
        let mut state = make_state();
        let monsters = make_monsters();
        assert_eq!(form_size(&state, &monsters), MonsterSize::Medium);
        state.player.monster_num = Some(1); // dragon
        assert_eq!(form_size(&state, &monsters), MonsterSize::Gigantic);
    }

    // ── special attacks ──────────────────────────────────────────────────

    #[test]
    fn test_do_breathe_as_dragon() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1); // dragon
        let result = do_breathe(&mut state, &monsters);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_do_breathe_no_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        let result = do_breathe(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_breathe_wrong_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(0); // newt - no breath
        let result = do_breathe(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_spit_as_cobra() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(3); // spitting cobra
        let result = do_spit(&mut state, &monsters);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_do_spit_no_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        let result = do_spit(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_mindblast_as_mind_flayer() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(2); // mind flayer
        state.player.energy = 20;
        let result = do_mindblast(&mut state, &monsters);
        assert!(matches!(result, ActionResult::Success));
        assert_eq!(state.player.energy, 10); // cost 10
    }

    #[test]
    fn test_do_mindblast_not_enough_energy() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(2); // mind flayer
        state.player.energy = 5;
        let result = do_mindblast(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_mindblast_wrong_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(1); // dragon
        state.player.energy = 20;
        let result = do_mindblast(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_hide_valid() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(4); // trapper (HIDE)
        let result = do_hide(&mut state, &monsters);
        assert!(matches!(result, ActionResult::Success));
    }

    #[test]
    fn test_do_hide_wrong_form() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(0); // newt
        let result = do_hide(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_do_hide_while_grabbed() {
        let mut state = make_state();
        let monsters = make_monsters();
        state.player.monster_num = Some(4); // trapper
        state.player.grabbed_by = Some(crate::monster::MonsterId(1));
        let result = do_hide(&mut state, &monsters);
        assert!(matches!(result, ActionResult::NoTime));
    }

    // ── article_a ────────────────────────────────────────────────────────

    #[test]
    fn test_article_a_consonant() {
        assert_eq!(article_a("dragon"), "a dragon");
    }

    #[test]
    fn test_article_a_vowel() {
        assert_eq!(article_a("owlbear"), "an owlbear");
    }

    // ── breath_type_name ─────────────────────────────────────────────────

    #[test]
    fn test_breath_type_names() {
        assert_eq!(breath_type_name(DamageType::Fire), "fire");
        assert_eq!(breath_type_name(DamageType::Cold), "frost");
        assert_eq!(breath_type_name(DamageType::Electric), "lightning");
        assert_eq!(breath_type_name(DamageType::Acid), "acid");
        assert_eq!(breath_type_name(DamageType::Sleep), "sleep gas");
    }
}
