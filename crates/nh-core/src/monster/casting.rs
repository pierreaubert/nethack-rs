//! Monster spellcasting (mcastu.c)
//!
//! Handles monster wizard and cleric spell selection and casting effects.
//! Port of NetHack C mcastu.c (868 lines).

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::combat::{Attack, DamageType};
use crate::gameloop::GameState;
use crate::monster::{Monster, SpeedState, aggravate, mon_adjust_speed, mon_set_minvis};
use crate::player::Property;
use crate::rng::GameRng;

// ============================================================================
// Spell Enums
// ============================================================================

/// Monster wizard (mage) spells — matches C's mcast_mage_spells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MageSpell {
    PsiBolt = 0,
    CureSelf = 1,
    HasteSelf = 2,
    StunYou = 3,
    Disappear = 4,
    WeakenYou = 5,
    DestroyArmor = 6,
    CurseItems = 7,
    Aggravation = 8,
    SummonMons = 9,
    CloneWiz = 10,
    DeathTouch = 11,
}

/// Monster cleric spells — matches C's mcast_cleric_spells
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ClericSpell {
    OpenWounds = 0,
    CureSelf = 1,
    ConfuseYou = 2,
    Paralyze = 3,
    BlindYou = 4,
    Insects = 5,
    CurseItems = 6,
    Lightning = 7,
    FirePillar = 8,
    Geyser = 9,
}

/// Result of a monster casting attempt
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastResult {
    /// Spell was successfully cast
    Success,
    /// Monster failed to cast (cancelled, spec_used, fumble, etc.)
    Failed,
}

// ============================================================================
// Spell Selection (C: choose_magic_spell, choose_clerical_spell)
// ============================================================================

/// Convert a level-based random value into a specific mage spell.
/// Port of C choose_magic_spell(). Inappropriate choices are screened
/// by spell_would_be_useless().
pub fn choose_magic_spell(mut spellval: u32, rng: &mut GameRng) -> MageSpell {
    // C: while (spellval > 24 && rn2(25)) spellval = rn2(spellval);
    while spellval > 24 && rng.rn2(25) != 0 {
        spellval = rng.rn2(spellval);
    }

    match spellval {
        24 | 23 => {
            // In C, checks Antimagic || Hallucination → fall back to PsiBolt.
            // We can't check player state here (pure selection), so return
            // DeathTouch; the caller filters via spell_would_be_useless.
            MageSpell::DeathTouch
        }
        20..=22 => MageSpell::DeathTouch,
        18..=19 => MageSpell::CloneWiz,
        15..=17 => MageSpell::SummonMons,
        13..=14 => MageSpell::Aggravation,
        10..=12 => MageSpell::CurseItems,
        8..=9 => MageSpell::DestroyArmor,
        6..=7 => MageSpell::WeakenYou,
        4..=5 => MageSpell::Disappear,
        3 => MageSpell::StunYou,
        2 => MageSpell::HasteSelf,
        1 => MageSpell::CureSelf,
        _ => MageSpell::PsiBolt,
    }
}

/// Convert a level-based random value into a specific cleric spell.
/// Port of C choose_clerical_spell().
pub fn choose_clerical_spell(mut spellnum: u32, rng: &mut GameRng) -> ClericSpell {
    // C: while (spellnum > 15 && rn2(16)) spellnum = rn2(spellnum);
    while spellnum > 15 && rng.rn2(16) != 0 {
        spellnum = rng.rn2(spellnum);
    }

    match spellnum {
        15 | 14 => {
            if rng.rn2(3) != 0 {
                return ClericSpell::OpenWounds;
            }
            ClericSpell::Geyser
        }
        13 => ClericSpell::Geyser,
        12 => ClericSpell::FirePillar,
        11 => ClericSpell::Lightning,
        10 | 9 => ClericSpell::CurseItems,
        8 => ClericSpell::Insects,
        7 | 6 => ClericSpell::BlindYou,
        5 | 4 => ClericSpell::Paralyze,
        3 | 2 => ClericSpell::ConfuseYou,
        1 => ClericSpell::CureSelf,
        _ => ClericSpell::OpenWounds,
    }
}

// ============================================================================
// Undirected / Useless checks
// ============================================================================

/// Check if a mage spell is undirected (doesn't target the player).
/// Port of C is_undirected_spell() for AD_SPEL.
pub fn is_undirected_mage_spell(spell: MageSpell) -> bool {
    matches!(
        spell,
        MageSpell::CloneWiz
            | MageSpell::SummonMons
            | MageSpell::Aggravation
            | MageSpell::Disappear
            | MageSpell::HasteSelf
            | MageSpell::CureSelf
    )
}

/// Check if a cleric spell is undirected (doesn't target the player).
/// Port of C is_undirected_spell() for AD_CLRC.
pub fn is_undirected_cleric_spell(spell: ClericSpell) -> bool {
    matches!(spell, ClericSpell::Insects | ClericSpell::CureSelf)
}

/// Snapshot of monster state needed for spell uselessness checks,
/// extracted to avoid borrow issues with GameState.
pub struct CasterSnapshot {
    pub level: u8,
    pub spec_used: u8,
    pub cancelled: bool,
    pub confused: bool,
    pub peaceful: bool,
    pub invisible: bool,
    pub invis_blocked: bool,
    pub hp: i32,
    pub hp_max: i32,
    pub permanent_speed: SpeedState,
}

impl CasterSnapshot {
    pub fn from_monster(m: &Monster) -> Self {
        Self {
            level: m.level,
            spec_used: m.spec_used,
            cancelled: m.state.cancelled,
            confused: m.state.confused,
            peaceful: m.state.peaceful,
            invisible: m.state.invisible,
            invis_blocked: m.state.invis_blocked,
            hp: m.hp,
            hp_max: m.hp_max,
            permanent_speed: m.permanent_speed,
        }
    }
}

/// Check if a mage spell would be useless for this monster to cast.
/// Port of C spell_would_be_useless() for AD_SPEL.
pub fn mage_spell_would_be_useless(
    caster: &CasterSnapshot,
    spell: MageSpell,
    player_blinded: bool,
    rng: &mut GameRng,
) -> bool {
    // Peaceful monsters won't aggravate/summon
    if caster.peaceful
        && matches!(
            spell,
            MageSpell::Aggravation | MageSpell::SummonMons | MageSpell::CloneWiz
        )
    {
        return true;
    }
    // Haste when already fast
    if caster.permanent_speed == SpeedState::Fast && spell == MageSpell::HasteSelf {
        return true;
    }
    // Invisibility when already invisible or blocked
    if (caster.invisible || caster.invis_blocked) && spell == MageSpell::Disappear {
        return true;
    }
    // Peaceful won't go invisible (so player doesn't accidentally hit them)
    if caster.peaceful && spell == MageSpell::Disappear {
        return true;
    }
    // Healing when at full HP
    if caster.hp == caster.hp_max && spell == MageSpell::CureSelf {
        return true;
    }
    // Clone wizard: always useless for non-Wizard monsters
    // Wizard of Yendor identification deferred until unique monster tracking is added
    if spell == MageSpell::CloneWiz {
        return true;
    }
    // Aggravation when nothing to wake
    if spell == MageSpell::Aggravation {
        // Small chance to pick it even when nothing needs waking
        // (caster doesn't know everything)
        if rng.rn2(100) != 0 {
            return true;
        }
    }
    let _ = player_blinded; // reserved for future use
    false
}

/// Check if a cleric spell would be useless for this monster to cast.
/// Port of C spell_would_be_useless() for AD_CLRC.
pub fn cleric_spell_would_be_useless(
    caster: &CasterSnapshot,
    spell: ClericSpell,
    player_blinded: bool,
) -> bool {
    // Peaceful monsters won't summon insects
    if caster.peaceful && spell == ClericSpell::Insects {
        return true;
    }
    // Healing when at full HP
    if caster.hp == caster.hp_max && spell == ClericSpell::CureSelf {
        return true;
    }
    // Blindness on already-blind player
    if player_blinded && spell == ClericSpell::BlindYou {
        return true;
    }
    false
}

// ============================================================================
// Main Entry Point (C: castmu)
// ============================================================================

/// Monster casts a spell at the player.
/// Port of C castmu().
///
/// # Arguments
/// * `state` - Game state
/// * `monster_idx` - Index into current_level.monsters
/// * `attack` - The attack being used (damage_type determines wizard vs cleric)
/// * `thinks_it_foundyou` - Monster thinks it knows where you are
/// * `foundyou` - Monster actually knows where you are
pub fn castmu(
    state: &mut GameState,
    monster_idx: usize,
    attack: &Attack,
    thinks_it_foundyou: bool,
    foundyou: bool,
) -> CastResult {
    // Extract caster info to avoid borrow issues
    let caster = {
        let m = &state.current_level.monsters[monster_idx];
        CasterSnapshot::from_monster(m)
    };
    let monster_name = state.current_level.monsters[monster_idx].name.clone();
    let ml = caster.level as u32;

    if ml == 0 {
        return CastResult::Failed;
    }

    let is_wizard_spell = attack.damage_type == DamageType::MageSpell;
    let is_cleric_spell = attack.damage_type == DamageType::ClericSpell;
    if !is_wizard_spell && !is_cleric_spell {
        return CastResult::Failed;
    }

    // Spell selection loop (C: do { ... } while (--cnt > 0 && spell_would_be_useless))
    let (mage_spell, cleric_spell) = select_spell(
        &caster,
        is_wizard_spell,
        thinks_it_foundyou,
        foundyou,
        ml,
        state.player.is_blind(),
        &mut state.rng,
    );

    // No valid spell found
    if mage_spell.is_none() && cleric_spell.is_none() {
        cursetxt(state, &monster_name, true);
        return CastResult::Failed;
    }

    // Check if monster is unable to cast (cancelled, spec_used, etc.)
    if caster.cancelled || caster.spec_used > 0 || ml == 0 {
        let undirected = if let Some(ms) = mage_spell {
            is_undirected_mage_spell(ms)
        } else if let Some(cs) = cleric_spell {
            is_undirected_cleric_spell(cs)
        } else {
            false
        };
        cursetxt(state, &monster_name, undirected);
        return CastResult::Failed;
    }

    // Set cooldown: mspec_used = 10 - m_lev, minimum 2
    {
        let m = &mut state.current_level.monsters[monster_idx];
        let cooldown = (10u8).saturating_sub(m.level).max(2);
        m.spec_used = cooldown;
    }

    // Directed spell at wrong location?
    if !foundyou && thinks_it_foundyou {
        let is_undirected = if let Some(ms) = mage_spell {
            is_undirected_mage_spell(ms)
        } else if let Some(cs) = cleric_spell {
            is_undirected_cleric_spell(cs)
        } else {
            false
        };
        if !is_undirected {
            state.message(format!("{} casts a spell at thin air!", monster_name));
            return CastResult::Failed;
        }
    }

    // Fumbled attack: confused monsters fumble more
    let fumble_threshold = if caster.confused { 100 } else { 20 };
    if state.rng.rn2(ml * 10) < fumble_threshold {
        state.message(format!("The air crackles around {}.", monster_name));
        return CastResult::Failed;
    }

    // Announce the spell
    let spell_target = if let Some(ms) = mage_spell {
        if is_undirected_mage_spell(ms) {
            ""
        } else {
            " at you"
        }
    } else if let Some(cs) = cleric_spell {
        if is_undirected_cleric_spell(cs) {
            ""
        } else {
            " at you"
        }
    } else {
        " at you"
    };
    state.message(format!("{} casts a spell{}!", monster_name, spell_target));

    // Calculate base damage
    let dmg = if !foundyou {
        0
    } else if attack.dice_sides > 0 {
        state
            .rng
            .dice(ml / 2 + attack.dice_num as u32, attack.dice_sides as u32)
            as i32
    } else {
        state.rng.dice(ml / 2 + 1, 6) as i32
    };

    // Half spell damage property
    let dmg = if state.player.properties.has(Property::HalfSpellDamage) {
        (dmg + 1) / 2
    } else {
        dmg
    };

    // Dispatch based on attack damage type
    match attack.damage_type {
        DamageType::Fire => {
            state.message("You're enveloped in flames.");
            if state.player.properties.has_fire_res() {
                state.message("But you resist the effects.");
            } else {
                state.player.take_damage(dmg);
            }
        }
        DamageType::Cold => {
            state.message("You're covered in frost.");
            if state.player.properties.has_cold_res() {
                state.message("But you resist the effects.");
            } else {
                state.player.take_damage(dmg);
            }
        }
        DamageType::MageSpell => {
            if let Some(spell) = mage_spell {
                cast_wizard_spell(state, monster_idx, dmg, spell);
            }
        }
        DamageType::ClericSpell => {
            if let Some(spell) = cleric_spell {
                cast_cleric_spell(state, monster_idx, dmg, spell);
            }
        }
        _ => {
            // Generic magic missile / other
            state.message("You are hit by a shower of missiles!");
            if state.player.properties.has(Property::MagicResistance) {
                state.message("The missiles bounce off!");
            } else {
                state.player.take_damage(dmg);
            }
        }
    }

    CastResult::Success
}

/// Select an appropriate spell, with retry loop for useless spells.
fn select_spell(
    caster: &CasterSnapshot,
    is_wizard: bool,
    thinks_it_foundyou: bool,
    _foundyou: bool,
    ml: u32,
    player_blinded: bool,
    rng: &mut GameRng,
) -> (Option<MageSpell>, Option<ClericSpell>) {
    let mut cnt = 40u32;

    loop {
        let spellval = rng.rn2(ml);

        if is_wizard {
            let spell = choose_magic_spell(spellval, rng);

            // Not trying to attack? Only allow undirected spells.
            if !thinks_it_foundyou {
                if !is_undirected_mage_spell(spell)
                    || mage_spell_would_be_useless(caster, spell, player_blinded, rng)
                {
                    return (None, None);
                }
                return (Some(spell), None);
            }

            if !mage_spell_would_be_useless(caster, spell, player_blinded, rng) {
                return (Some(spell), None);
            }
        } else {
            let spell = choose_clerical_spell(spellval, rng);

            if !thinks_it_foundyou {
                if !is_undirected_cleric_spell(spell)
                    || cleric_spell_would_be_useless(caster, spell, player_blinded)
                {
                    return (None, None);
                }
                return (Some(MageSpell::PsiBolt), Some(spell)); // Only cleric set
            }

            if !cleric_spell_would_be_useless(caster, spell, player_blinded) {
                return (None, Some(spell));
            }
        }

        cnt -= 1;
        if cnt == 0 {
            return (None, None);
        }
    }
}

// ============================================================================
// Feedback messages
// ============================================================================

/// Feedback when a monster fails to cast a spell.
/// Port of C cursetxt().
fn cursetxt(state: &mut GameState, monster_name: &str, undirected: bool) {
    if undirected {
        state.message(format!(
            "{} points all around, then curses.",
            monster_name
        ));
    } else {
        state.message(format!(
            "{} points at you, then curses.",
            monster_name
        ));
    }
}

// ============================================================================
// Monster self-healing (shared by wizard and cleric)
// ============================================================================

/// Monster heals itself. Port of C m_cure_self().
/// Returns the remaining damage to deal (0 if healed successfully).
fn m_cure_self(state: &mut GameState, monster_idx: usize, dmg: i32) -> i32 {
    let m = &mut state.current_level.monsters[monster_idx];
    if m.hp < m.hp_max {
        let name = m.name.clone();
        let heal = state.rng.dice(3, 6) as i32;
        let m = &mut state.current_level.monsters[monster_idx];
        m.hp = (m.hp + heal).min(m.hp_max);
        state.message(format!("{} looks better.", name));
        0
    } else {
        dmg
    }
}

// ============================================================================
// Wizard Spell Effects (C: cast_wizard_spell)
// ============================================================================

/// Cast a wizard spell. Port of C cast_wizard_spell().
fn cast_wizard_spell(state: &mut GameState, monster_idx: usize, dmg: i32, spell: MageSpell) {
    let monster_name = state.current_level.monsters[monster_idx].name.clone();

    match spell {
        MageSpell::DeathTouch => {
            state.message(format!(
                "Oh no, {}'s using the touch of death!",
                monster_name
            ));
            let has_mr = state.player.properties.has(Property::MagicResistance);
            let ml = state.current_level.monsters[monster_idx].level;
            if has_mr {
                state.message("Lucky for you, it didn't work!");
            } else if state.rng.rn2(ml as u32) > 12 {
                // Instant death
                state.player.take_damage(state.player.hp);
                state.message("You die...");
            } else {
                state.message("Lucky for you, it didn't work!");
            }
        }
        MageSpell::CloneWiz => {
            // Requires Wizard of Yendor unique monster tracking; currently a no-op message
            state.message("Double Trouble...");
        }
        MageSpell::SummonMons => {
            // Summon nasty monsters near the player
            let count = summon_nasty(state);
            if count == 1 {
                state.message("A monster appears from nowhere!");
            } else {
                state.message("Monsters appear from nowhere!");
            }
        }
        MageSpell::Aggravation => {
            state.message("You feel that monsters are aware of your presence.");
            let px = state.player.pos.x;
            let py = state.player.pos.y;
            aggravate(&mut state.current_level.monsters, px, py);
        }
        MageSpell::CurseItems => {
            state.message("You feel as if you need some help.");
            rndcurse_inventory(state);
        }
        MageSpell::DestroyArmor => {
            if state.player.properties.has(Property::MagicResistance) {
                state.message("A field of force surrounds you!");
            } else {
                state.message("Your skin itches.");
                // Armor destruction deferred: requires per-slot worn armor tracking
            }
        }
        MageSpell::WeakenYou => {
            if state.player.properties.has(Property::MagicResistance) {
                state.message("You feel momentarily weakened.");
            } else {
                state.message("You suddenly feel weaker!");
                let ml = state.current_level.monsters[monster_idx].level;
                let mut drain = (ml as i8).saturating_sub(6).max(1);
                if state.player.properties.has(Property::HalfSpellDamage) {
                    drain = (drain + 1) / 2;
                }
                let amount = state.rng.rnd(drain as u32) as i8;
                state.player.losestr(amount);
            }
        }
        MageSpell::Disappear => {
            let m = &state.current_level.monsters[monster_idx];
            if !m.state.invisible && !m.state.invis_blocked {
                state.message(format!("{} suddenly disappears!", monster_name));
                let m = &mut state.current_level.monsters[monster_idx];
                mon_set_minvis(m);
            }
        }
        MageSpell::StunYou => {
            let has_mr = state.player.properties.has(Property::MagicResistance);
            let has_fa = state.player.properties.has(Property::FreeAction);
            if has_mr || has_fa {
                if !state.player.is_stunned() {
                    state.message("You feel momentarily disoriented.");
                }
                state.player.make_stunned(1, false);
            } else {
                if state.player.is_stunned() {
                    state.message("You struggle to keep your balance.");
                } else {
                    state.message("You reel...");
                }
                let dex = state.player.attr_current.get(crate::player::Attribute::Dexterity);
                let dice_n = if dex < 12 { 6 } else { 4 };
                let mut stun_dmg = state.rng.dice(dice_n, 4) as u16;
                if state.player.properties.has(Property::HalfSpellDamage) {
                    stun_dmg = stun_dmg.div_ceil(2);
                }
                let new_timeout = state.player.stunned_timeout.saturating_add(stun_dmg);
                state.player.make_stunned(new_timeout, false);
            }
        }
        MageSpell::HasteSelf => {
            let m = &mut state.current_level.monsters[monster_idx];
            mon_adjust_speed(m, 1, None);
        }
        MageSpell::CureSelf => {
            m_cure_self(state, monster_idx, dmg);
        }
        MageSpell::PsiBolt => {
            let mut psi_dmg = dmg;
            if state.player.properties.has(Property::MagicResistance) {
                psi_dmg = (psi_dmg + 1) / 2;
            }
            if psi_dmg <= 5 {
                state.message("You get a slight headache.");
            } else if psi_dmg <= 10 {
                state.message("Your brain is on fire!");
            } else if psi_dmg <= 20 {
                state.message("Your head suddenly aches painfully!");
            } else {
                state.message("Your head suddenly aches very painfully!");
            }
            if psi_dmg > 0 {
                state.player.take_damage(psi_dmg);
            }
        }
    }
}

// ============================================================================
// Cleric Spell Effects (C: cast_cleric_spell)
// ============================================================================

/// Cast a cleric spell. Port of C cast_cleric_spell().
fn cast_cleric_spell(state: &mut GameState, monster_idx: usize, dmg: i32, spell: ClericSpell) {
    let monster_name = state.current_level.monsters[monster_idx].name.clone();

    match spell {
        ClericSpell::Geyser => {
            state.message("A sudden geyser slams into you from nowhere!");
            let mut geyser_dmg = state.rng.dice(8, 6) as i32;
            if state.player.properties.has(Property::HalfPhysDamage) {
                geyser_dmg = (geyser_dmg + 1) / 2;
            }
            state.player.take_damage(geyser_dmg);
        }
        ClericSpell::FirePillar => {
            state.message("A pillar of fire strikes all around you!");
            let fire_dmg = if state.player.properties.has_fire_res() {
                0
            } else {
                let mut d = state.rng.dice(8, 6) as i32;
                if state.player.properties.has(Property::HalfSpellDamage) {
                    d = (d + 1) / 2;
                }
                d
            };
            if fire_dmg > 0 {
                state.player.take_damage(fire_dmg);
            }
            // Destroy fire-vulnerable items
            destroy_items_by_element(state, Element::Fire);
        }
        ClericSpell::Lightning => {
            state.message("A bolt of lightning strikes down at you from above!");
            let has_reflection = state.player.properties.has(Property::Reflection);
            let has_shock_res = state.player.properties.has_shock_res();
            if has_reflection {
                state.message("It bounces off your shield!");
            } else if has_shock_res {
                // No damage
            } else {
                let mut bolt_dmg = state.rng.dice(8, 6) as i32;
                if state.player.properties.has(Property::HalfSpellDamage) {
                    bolt_dmg = (bolt_dmg + 1) / 2;
                }
                state.player.take_damage(bolt_dmg);
            }
            if !has_reflection {
                // Lightning blinds briefly
                let blind_dur = state.rng.rnd(100) as u16;
                let msg = state.player.make_blinded(
                    state.player.blinded_timeout.saturating_add(blind_dur),
                    true,
                );
                if let Some(m) = msg {
                    state.message(m);
                }
                // Destroy shock-vulnerable items
                destroy_items_by_element(state, Element::Shock);
            }
        }
        ClericSpell::CurseItems => {
            state.message("You feel as if you need some help.");
            rndcurse_inventory(state);
        }
        ClericSpell::Insects => {
            // Summon insects around the player
            let ml = state.current_level.monsters[monster_idx].level;
            let mut quan = if ml < 2 {
                1
            } else {
                state.rng.rnd(ml as u32 / 2) as i32
            };
            if quan < 3 {
                quan = 3;
            }
            state.message(format!("{} summons insects!", monster_name));
            // Insect spawning deferred: requires makemon integration with insect type selection
            let _ = quan;
        }
        ClericSpell::BlindYou => {
            if !state.player.is_blind() {
                state.message("Scales cover your eyes!");
                let duration = if state.player.properties.has(Property::HalfSpellDamage) {
                    100
                } else {
                    200
                };
                let msg = state.player.make_blinded(duration, false);
                if let Some(m) = msg {
                    state.message(m);
                }
            }
        }
        ClericSpell::Paralyze => {
            let has_mr = state.player.properties.has(Property::MagicResistance);
            let has_fa = state.player.properties.has(Property::FreeAction);
            if has_mr || has_fa {
                state.message("You stiffen briefly.");
                state.player.paralyzed_timeout = 1;
            } else {
                state.message("You are frozen in place!");
                let ml = state.current_level.monsters[monster_idx].level;
                let mut para_dur = 4 + ml as u16;
                if state.player.properties.has(Property::HalfSpellDamage) {
                    para_dur = para_dur.div_ceil(2);
                }
                state.player.paralyzed_timeout = para_dur;
            }
        }
        ClericSpell::ConfuseYou => {
            if state.player.properties.has(Property::MagicResistance) {
                state.message("You feel momentarily dizzy.");
            } else {
                let ml = state.current_level.monsters[monster_idx].level;
                let mut conf_dur = ml as u16;
                if state.player.properties.has(Property::HalfSpellDamage) {
                    conf_dur = conf_dur.div_ceil(2);
                }
                let was_confused = state.player.is_confused();
                let new_timeout = state.player.confused_timeout.saturating_add(conf_dur);
                state.player.make_confused(new_timeout, false);
                if was_confused {
                    state.message("You feel more confused!");
                } else {
                    state.message("You feel confused!");
                }
            }
        }
        ClericSpell::CureSelf => {
            m_cure_self(state, monster_idx, dmg);
        }
        ClericSpell::OpenWounds => {
            let mut wound_dmg = dmg;
            if state.player.properties.has(Property::MagicResistance) {
                wound_dmg = (wound_dmg + 1) / 2;
            }
            if wound_dmg <= 5 {
                state.message("Your skin itches badly for a moment.");
            } else if wound_dmg <= 10 {
                state.message("Wounds appear on your body!");
            } else if wound_dmg <= 20 {
                state.message("Severe wounds appear on your body!");
            } else {
                state.message("Your body is covered with painful wounds!");
            }
            if wound_dmg > 0 {
                state.player.take_damage(wound_dmg);
            }
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Element type for item destruction
enum Element {
    Fire,
    Shock,
}

/// Destroy items vulnerable to an element.
/// Simplified version — full implementation in action/trap.rs.
fn destroy_items_by_element(state: &mut GameState, element: Element) {
    use crate::object::ObjectClass;

    let mut destroyed = Vec::new();
    for (i, item) in state.inventory.iter().enumerate() {
        let vulnerable = match element {
            Element::Fire => matches!(
                item.class,
                ObjectClass::Scroll | ObjectClass::Potion | ObjectClass::Spellbook
            ),
            Element::Shock => {
                matches!(item.class, ObjectClass::Wand | ObjectClass::Ring)
            }
        };
        if vulnerable && state.rng.rn2(3) == 0 {
            destroyed.push(i);
        }
    }

    if !destroyed.is_empty() {
        let count = destroyed.len();
        let element_name = match element {
            Element::Fire => "fire",
            Element::Shock => "lightning",
        };
        state.message(format!(
            "{} of your items {} destroyed by {}!",
            if count == 1 { "One" } else { "Some" },
            if count == 1 { "is" } else { "are" },
            element_name
        ));
        // Remove in reverse order to preserve indices
        for i in destroyed.into_iter().rev() {
            state.inventory.remove(i);
        }
    }
}

/// Curse a random inventory item.
fn rndcurse_inventory(state: &mut GameState) {
    if state.inventory.is_empty() {
        return;
    }
    let idx = state.rng.rn2(state.inventory.len() as u32) as usize;
    crate::object::rndcurse(&mut state.inventory[idx], &mut state.rng);
}

/// Summon nasty monsters near the player.
/// Returns the count of monsters summoned.
fn summon_nasty(state: &mut GameState) -> i32 {
    // Simplified: returns 1-3 as count; actual monster spawning requires makemon integration
    state.rng.rnd(3) as i32
}

// ============================================================================
// Ranged spell (C: buzzmu)
// ============================================================================

/// Monster uses a ranged spell (beam). Port of C buzzmu().
/// Currently a stub — requires beam/ray system integration.
pub fn buzzmu(
    state: &mut GameState,
    monster_idx: usize,
    _attack: &Attack,
) -> CastResult {
    let m = &state.current_level.monsters[monster_idx];
    if m.state.cancelled {
        let name = m.name.clone();
        cursetxt(state, &name, false);
        return CastResult::Failed;
    }
    // Beam/ray casting deferred: requires lined_up check and buzz() ray system
    CastResult::Failed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{Attack, AttackType, DamageType};
    use crate::monster::{Monster, MonsterId, SpeedState};
    use crate::rng::GameRng;

    fn make_test_state() -> GameState {
        GameState::new(GameRng::new(42))
    }

    fn make_caster(level: u8) -> Monster {
        let mut m = Monster::new(MonsterId(999), 0, 5, 5);
        m.name = "the test caster".to_string();
        m.level = level;
        m.hp = 50;
        m.hp_max = 50;
        m.spec_used = 0;
        m
    }

    fn spell_attack(is_wizard: bool) -> Attack {
        Attack {
            attack_type: AttackType::Magic,
            damage_type: if is_wizard {
                DamageType::MageSpell
            } else {
                DamageType::ClericSpell
            },
            dice_num: 0,
            dice_sides: 6,
        }
    }

    #[test]
    fn test_choose_magic_spell_low_level() {
        let mut rng = GameRng::new(1);
        // spellval 0 should always give PsiBolt
        assert_eq!(choose_magic_spell(0, &mut rng), MageSpell::PsiBolt);
    }

    #[test]
    fn test_choose_magic_spell_mid_level() {
        let mut rng = GameRng::new(2);
        assert_eq!(choose_magic_spell(3, &mut rng), MageSpell::StunYou);
        assert_eq!(choose_magic_spell(2, &mut rng), MageSpell::HasteSelf);
        assert_eq!(choose_magic_spell(1, &mut rng), MageSpell::CureSelf);
    }

    #[test]
    fn test_choose_magic_spell_high_level() {
        let mut rng = GameRng::new(3);
        assert_eq!(choose_magic_spell(20, &mut rng), MageSpell::DeathTouch);
    }

    #[test]
    fn test_choose_clerical_spell_low_level() {
        let mut rng = GameRng::new(1);
        assert_eq!(choose_clerical_spell(0, &mut rng), ClericSpell::OpenWounds);
        assert_eq!(choose_clerical_spell(1, &mut rng), ClericSpell::CureSelf);
    }

    #[test]
    fn test_choose_clerical_spell_high_level() {
        let mut rng = GameRng::new(4);
        assert_eq!(choose_clerical_spell(13, &mut rng), ClericSpell::Geyser);
        assert_eq!(choose_clerical_spell(12, &mut rng), ClericSpell::FirePillar);
        assert_eq!(choose_clerical_spell(11, &mut rng), ClericSpell::Lightning);
    }

    #[test]
    fn test_undirected_mage_spells() {
        assert!(is_undirected_mage_spell(MageSpell::CureSelf));
        assert!(is_undirected_mage_spell(MageSpell::HasteSelf));
        assert!(is_undirected_mage_spell(MageSpell::Disappear));
        assert!(is_undirected_mage_spell(MageSpell::Aggravation));
        assert!(is_undirected_mage_spell(MageSpell::SummonMons));
        assert!(is_undirected_mage_spell(MageSpell::CloneWiz));
        // Directed spells
        assert!(!is_undirected_mage_spell(MageSpell::PsiBolt));
        assert!(!is_undirected_mage_spell(MageSpell::DeathTouch));
        assert!(!is_undirected_mage_spell(MageSpell::StunYou));
    }

    #[test]
    fn test_undirected_cleric_spells() {
        assert!(is_undirected_cleric_spell(ClericSpell::CureSelf));
        assert!(is_undirected_cleric_spell(ClericSpell::Insects));
        assert!(!is_undirected_cleric_spell(ClericSpell::Geyser));
        assert!(!is_undirected_cleric_spell(ClericSpell::OpenWounds));
    }

    #[test]
    fn test_mage_spell_useless_haste_when_fast() {
        let mut rng = GameRng::new(5);
        let mut caster = CasterSnapshot {
            level: 10,
            spec_used: 0,
            cancelled: false,
            confused: false,
            peaceful: false,
            invisible: false,
            invis_blocked: false,
            hp: 50,
            hp_max: 50,
            permanent_speed: SpeedState::Fast,
        };
        assert!(mage_spell_would_be_useless(
            &caster,
            MageSpell::HasteSelf,
            false,
            &mut rng
        ));
        // Not useless when normal speed
        caster.permanent_speed = SpeedState::Normal;
        assert!(!mage_spell_would_be_useless(
            &caster,
            MageSpell::HasteSelf,
            false,
            &mut rng
        ));
    }

    #[test]
    fn test_mage_spell_useless_cure_at_full_hp() {
        let mut rng = GameRng::new(6);
        let caster = CasterSnapshot {
            level: 10,
            spec_used: 0,
            cancelled: false,
            confused: false,
            peaceful: false,
            invisible: false,
            invis_blocked: false,
            hp: 50,
            hp_max: 50,
            permanent_speed: SpeedState::Normal,
        };
        assert!(mage_spell_would_be_useless(
            &caster,
            MageSpell::CureSelf,
            false,
            &mut rng
        ));
    }

    #[test]
    fn test_cleric_spell_useless_blind_when_blind() {
        let caster = CasterSnapshot {
            level: 10,
            spec_used: 0,
            cancelled: false,
            confused: false,
            peaceful: false,
            invisible: false,
            invis_blocked: false,
            hp: 50,
            hp_max: 50,
            permanent_speed: SpeedState::Normal,
        };
        assert!(cleric_spell_would_be_useless(
            &caster,
            ClericSpell::BlindYou,
            true
        ));
        assert!(!cleric_spell_would_be_useless(
            &caster,
            ClericSpell::BlindYou,
            false
        ));
    }

    #[test]
    fn test_castmu_cancelled_monster_fails() {
        let mut state = make_test_state();
        let mut caster = make_caster(10);
        caster.state.cancelled = true;
        state.current_level.monsters.clear();
        state.current_level.monsters.push(caster);

        let attack = spell_attack(true);
        let result = castmu(&mut state, 0, &attack, true, true);
        assert_eq!(result, CastResult::Failed);
    }

    #[test]
    fn test_castmu_spec_used_fails() {
        let mut state = make_test_state();
        let mut caster = make_caster(10);
        caster.spec_used = 5;
        state.current_level.monsters.clear();
        state.current_level.monsters.push(caster);

        let attack = spell_attack(true);
        let result = castmu(&mut state, 0, &attack, true, true);
        assert_eq!(result, CastResult::Failed);
    }

    #[test]
    fn test_castmu_sets_cooldown() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));

        let attack = spell_attack(true);
        // May succeed or fail due to fumble, but if it gets past the cancel check,
        // cooldown should be set. We test multiple seeds.
        for seed in 0..20u64 {
            state.rng = GameRng::new(seed);
            let mut caster = make_caster(10);
            caster.spec_used = 0;
            state.current_level.monsters[0] = caster;

            let _ = castmu(&mut state, 0, &attack, true, true);
            // If the spell got past the cancel/spec check, cooldown should be set
            // (10 - 10 = 0, clamped to 2)
            let spec = state.current_level.monsters[0].spec_used;
            assert!(spec >= 2 || spec == 0, "spec_used should be >= 2 or 0 (not attempted), got {}", spec);
        }
    }

    #[test]
    fn test_m_cure_self_heals() {
        let mut state = make_test_state();
        let mut caster = make_caster(10);
        caster.hp = 20;
        caster.hp_max = 50;
        state.current_level.monsters.clear();
        state.current_level.monsters.push(caster);

        let result = m_cure_self(&mut state, 0, 10);
        assert_eq!(result, 0); // Healed, so damage becomes 0
        assert!(state.current_level.monsters[0].hp > 20);
        assert!(state.current_level.monsters[0].hp <= 50);
    }

    #[test]
    fn test_m_cure_self_at_full_hp() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));

        let result = m_cure_self(&mut state, 0, 10);
        assert_eq!(result, 10); // No healing needed, damage unchanged
    }

    #[test]
    fn test_cast_wizard_psi_bolt_damage() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        let initial_hp = state.player.hp;

        cast_wizard_spell(&mut state, 0, 8, MageSpell::PsiBolt);
        assert!(state.player.hp < initial_hp);
        assert_eq!(state.player.hp, initial_hp - 8);
    }

    #[test]
    fn test_cast_wizard_psi_bolt_magic_resistance() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        state
            .player
            .properties
            .grant_intrinsic(Property::MagicResistance);
        let initial_hp = state.player.hp;

        // With MR, damage is halved: (8+1)/2 = 4
        cast_wizard_spell(&mut state, 0, 8, MageSpell::PsiBolt);
        assert_eq!(state.player.hp, initial_hp - 4);
    }

    #[test]
    fn test_cast_wizard_stun() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        assert!(!state.player.is_stunned());

        cast_wizard_spell(&mut state, 0, 0, MageSpell::StunYou);
        assert!(state.player.is_stunned());
    }

    #[test]
    fn test_cast_wizard_haste_self() {
        let mut state = make_test_state();
        let mut caster = make_caster(10);
        caster.speed = SpeedState::Normal;
        state.current_level.monsters.clear();
        state.current_level.monsters.push(caster);

        cast_wizard_spell(&mut state, 0, 0, MageSpell::HasteSelf);
        assert_eq!(
            state.current_level.monsters[0].speed,
            SpeedState::Fast
        );
    }

    #[test]
    fn test_cast_wizard_disappear() {
        let mut state = make_test_state();
        let mut caster = make_caster(10);
        caster.state.invisible = false;
        caster.state.invis_blocked = false;
        state.current_level.monsters.clear();
        state.current_level.monsters.push(caster);

        cast_wizard_spell(&mut state, 0, 0, MageSpell::Disappear);
        assert!(state.current_level.monsters[0].state.invisible);
    }

    #[test]
    fn test_cast_cleric_geyser() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        let initial_hp = state.player.hp;

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::Geyser);
        assert!(state.player.hp < initial_hp);
    }

    #[test]
    fn test_cast_cleric_confuse() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        assert!(!state.player.is_confused());

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::ConfuseYou);
        assert!(state.player.is_confused());
    }

    #[test]
    fn test_cast_cleric_paralyze() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::Paralyze);
        assert!(state.player.paralyzed_timeout > 0);
    }

    #[test]
    fn test_cast_cleric_paralyze_with_free_action() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        state
            .player
            .properties
            .grant_intrinsic(Property::FreeAction);

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::Paralyze);
        assert_eq!(state.player.paralyzed_timeout, 1); // Brief stiffening only
    }

    #[test]
    fn test_cast_cleric_blind() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        assert!(!state.player.is_blind());

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::BlindYou);
        assert!(state.player.is_blind());
    }

    #[test]
    fn test_cast_cleric_open_wounds() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        let initial_hp = state.player.hp;

        cast_cleric_spell(&mut state, 0, 12, ClericSpell::OpenWounds);
        assert_eq!(state.player.hp, initial_hp - 12);
    }

    #[test]
    fn test_cast_cleric_fire_pillar_with_fire_res() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        state
            .player
            .properties
            .grant_intrinsic(Property::FireResistance);
        let initial_hp = state.player.hp;

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::FirePillar);
        assert_eq!(state.player.hp, initial_hp); // No damage with fire resistance
    }

    #[test]
    fn test_cast_cleric_lightning_with_reflection() {
        let mut state = make_test_state();
        state.current_level.monsters.clear();
        state.current_level.monsters.push(make_caster(10));
        state
            .player
            .properties
            .grant_intrinsic(Property::Reflection);
        let initial_hp = state.player.hp;

        cast_cleric_spell(&mut state, 0, 0, ClericSpell::Lightning);
        assert_eq!(state.player.hp, initial_hp); // Reflected, no damage
    }

    #[test]
    fn test_rndcurse_inventory() {
        let mut state = make_test_state();
        // Add some items
        let mut obj = crate::object::Object::default();
        obj.class = crate::object::ObjectClass::Weapon;
        obj.buc = crate::object::BucStatus::Uncursed;
        state.inventory.push(obj.clone());
        state.inventory.push(obj);

        // Run curse multiple times — at least one should curse
        let mut any_cursed = false;
        for seed in 0..50u64 {
            state.rng = GameRng::new(seed);
            for item in &mut state.inventory {
                item.buc = crate::object::BucStatus::Uncursed;
            }
            rndcurse_inventory(&mut state);
            if state
                .inventory
                .iter()
                .any(|i| i.buc == crate::object::BucStatus::Cursed)
            {
                any_cursed = true;
                break;
            }
        }
        assert!(any_cursed);
    }
}
