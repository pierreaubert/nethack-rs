//! Monster attacks player combat (mhitu.c)
//!
//! Handles all combat initiated by monsters against the player.
//!
//! Main entry point is `mattacku()` which orchestrates all monster attacks.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use super::{
    ArmorProficiency, ArmorType, Attack, AttackType, CombatEffect, CombatResult, CriticalHitType,
    DamageType, DefenseCalculation, DodgeSkill, RangedAttack, RangedCombatResult, RangedWeaponType,
    SkillLevel, SpecialCombatEffect, StatusEffect, apply_damage_reduction, apply_special_effect,
    apply_status_effect, attempt_dodge, award_monster_xp, calculate_armor_damage_reduction,
    calculate_skill_enhanced_damage, calculate_status_damage, determine_critical_hit,
    effect_severity_from_skill, execute_ranged_attack, should_trigger_special_effect,
};
use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId};
use crate::object::Object;
use crate::player::{Property, You};
use crate::rng::GameRng;

/// Result of a full monster attack sequence
#[derive(Debug, Clone, Default)]
pub struct MonsterAttackResult {
    /// Whether any attack connected
    pub any_hit: bool,
    /// Whether the player died
    pub player_died: bool,
    /// Whether the monster died (e.g., from passive damage)
    pub monster_died: bool,
    /// Total damage dealt
    pub total_damage: i32,
    /// Messages generated during the attack
    pub messages: Vec<String>,
    /// Special effects triggered
    pub effects: Vec<CombatEffect>,
}



// ============================================================================
// Message Functions (hitmsg, missmu, wildmiss, mswings)
// ============================================================================

/// Generate hit message based on attack type (hitmsg in C)
pub fn hit_message(attacker_name: &str, attack_type: AttackType) -> String {
    match attack_type {
        AttackType::Bite => format!("The {} bites!", attacker_name),
        AttackType::Kick => format!("The {} kicks!", attacker_name),
        AttackType::Sting => format!("The {} stings!", attacker_name),
        AttackType::Butt => format!("The {} butts!", attacker_name),
        AttackType::Touch => format!("The {} touches you!", attacker_name),
        AttackType::Tentacle => format!("The {}'s tentacles suck you!", attacker_name),
        AttackType::Claw => format!("The {} claws!", attacker_name),
        AttackType::Hug => format!("The {} squeezes you!", attacker_name),
        AttackType::Engulf => format!("The {} engulfs you!", attacker_name),
        AttackType::Breath => format!("The {} breathes on you!", attacker_name),
        AttackType::Spit => format!("The {} spits at you!", attacker_name),
        AttackType::Gaze => format!("The {} gazes at you!", attacker_name),
        AttackType::Explode | AttackType::ExplodeOnDeath => {
            format!("The {} explodes!", attacker_name)
        }
        AttackType::Weapon => format!("The {} hits!", attacker_name),
        AttackType::Magic => format!("The {} casts a spell!", attacker_name),
        _ => format!("The {} hits!", attacker_name),
    }
}

/// Generate miss message (missmu in C)
pub fn miss_message(attacker_name: &str, near_miss: bool) -> String {
    if near_miss {
        format!("The {} just misses!", attacker_name)
    } else {
        format!("The {} misses.", attacker_name)
    }
}

/// Generate wild miss message for displaced/invisible player (wildmiss in C)
pub fn wild_miss_message(
    attacker_name: &str,
    player_displaced: bool,
    player_invisible: bool,
) -> String {
    if player_displaced {
        if player_invisible {
            format!(
                "The {} strikes at your invisible displaced image and misses!",
                attacker_name
            )
        } else {
            format!(
                "The {} strikes at your displaced image and misses!",
                attacker_name
            )
        }
    } else if player_invisible {
        format!("The {} swings wildly and misses!", attacker_name)
    } else {
        format!("The {} attacks a spot beside you.", attacker_name)
    }
}

/// Generate weapon swing message (mswings in C)
pub fn weapon_swing_message(attacker_name: &str, weapon_name: &str, is_thrust: bool) -> String {
    if is_thrust {
        format!("The {} thrusts its {}.", attacker_name, weapon_name)
    } else {
        format!("The {} swings its {}.", attacker_name, weapon_name)
    }
}

/// Generate damage type specific message
pub fn damage_effect_message(attacker_name: &str, damage_type: DamageType) -> Option<String> {
    match damage_type {
        DamageType::Fire => Some("You're covered in flames!".to_string()),
        DamageType::Cold => Some("You're covered in frost!".to_string()),
        DamageType::Electric => Some("You get zapped!".to_string()),
        DamageType::Acid => Some("You're covered in acid! It burns!".to_string()),
        DamageType::Sleep => Some(format!("The {} puts you to sleep!", attacker_name)),
        DamageType::Paralyze => Some("You are frozen!".to_string()),
        DamageType::DrainLife => Some("You feel your life force draining away...".to_string()),
        DamageType::Stone => Some("You are turning to stone!".to_string()),
        DamageType::Disintegrate => Some("You are disintegrating!".to_string()),
        DamageType::Confuse => Some("You feel confused.".to_string()),
        DamageType::Stun => Some("You stagger...".to_string()),
        DamageType::Blind => Some("You can't see!".to_string()),
        DamageType::DrainStrength => Some("You feel weaker!".to_string()),
        DamageType::DrainDexterity => Some("You feel clumsy!".to_string()),
        DamageType::DrainConstitution => Some("You feel fragile!".to_string()),
        DamageType::Disease => Some("You feel very sick.".to_string()),
        DamageType::StealGold => Some("Your purse feels lighter.".to_string()),
        DamageType::StealItem => Some("Something was stolen from you!".to_string()),
        DamageType::Teleport => Some("Your position suddenly seems uncertain!".to_string()),
        DamageType::Digest => Some("You are swallowed!".to_string()),
        DamageType::Wrap | DamageType::Stick => Some("You are being held!".to_string()),
        _ => None,
    }
}

/// Generate resistance message
pub fn resistance_message(damage_type: DamageType) -> Option<String> {
    match damage_type {
        DamageType::Fire => Some("The fire doesn't feel hot!".to_string()),
        DamageType::Cold => Some("The frost doesn't seem cold!".to_string()),
        DamageType::Electric => Some("The zap doesn't shock you!".to_string()),
        DamageType::Acid => Some("The acid doesn't burn much.".to_string()),
        DamageType::Sleep => Some("You yawn.".to_string()),
        DamageType::Paralyze => Some("You momentarily stiffen.".to_string()),
        DamageType::DrainLife => Some("You feel a strange tingle.".to_string()),
        DamageType::Stone => Some("You feel sluggish for a moment.".to_string()),
        DamageType::Disintegrate => Some("You feel a mild tingle.".to_string()),
        _ => None,
    }
}

// ============================================================================
// Seduction Functions (could_seduce, doseduce)
// ============================================================================

/// Seduction compatibility result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeduceResult {
    /// Cannot seduce
    No,
    /// Can seduce (opposite gender)
    Yes,
    /// Wrong gender for nymph (same gender)
    WrongGender,
}

/// Check if attacker could seduce defender (could_seduce in C).
///
/// Returns:
/// - `SeduceResult::Yes` if opposite gender and can seduce
/// - `SeduceResult::WrongGender` if same gender (nymph only)
/// - `SeduceResult::No` if cannot seduce
///
/// # Arguments
/// * `attacker` - The attacking monster
/// * `attacker_gender` - The attacker's gender (0=male, 1=female, 2=neuter)
/// * `defender_gender` - The defender's gender (0=male, 1=female, 2=neuter)
/// * `defender_sees_invisible` - Whether defender can see invisible
/// * `attack` - The attack being used (or None for general capability check)
/// * `is_nymph` - Whether attacker is a nymph
/// * `is_demon_seducer` - Whether attacker is an incubus/succubus
pub fn could_seduce(
    attacker: &Monster,
    attacker_gender: i8,
    defender_gender: i8,
    defender_sees_invisible: bool,
    attack: Option<&Attack>,
    is_nymph: bool,
    is_demon_seducer: bool,
) -> SeduceResult {
    let attacker_invisible = attacker.state.invisible;

    // Determine attack damage type
    let ad_type = if let Some(atk) = attack {
        atk.damage_type
    } else {
        // Check if monster has seduction capability by scanning attacks
        let has_seduce = attacker
            .attacks
            .iter()
            .any(|a| a.damage_type == DamageType::Seduce);
        if has_seduce {
            DamageType::Seduce
        } else {
            DamageType::Physical
        }
    };

    // Invisible attacker can't seduce if defender can't see them (for regular seduction)
    if attacker_invisible && !defender_sees_invisible && ad_type == DamageType::Seduce {
        return SeduceResult::No;
    }

    // Check if this is a seduction-capable monster
    if !is_nymph && !is_demon_seducer {
        return SeduceResult::No;
    }

    // Check attack type is seduction-related
    if !matches!(ad_type, DamageType::Seduce | DamageType::StealItem) {
        return SeduceResult::No;
    }

    // Gender check: opposite genders can seduce, same gender cannot (except nymphs)
    // Gender: 0=male, 1=female, 2=neuter
    if attacker_gender == 1 - defender_gender {
        // Opposite gender
        SeduceResult::Yes
    } else if is_nymph {
        // Nymphs can still interact with same gender, just differently
        SeduceResult::WrongGender
    } else {
        SeduceResult::No
    }
}

// ============================================================================
// Main Entry Point: mattacku()
// ============================================================================

/// Main monster attack function - processes all attacks from a monster (mattacku in C)
///
/// This is the main entry point for monster-vs-player combat.
/// It iterates through all of the monster's attacks and processes each one.
pub fn mattacku(
    attacker: &Monster,
    player: &mut You,
    inventory: &mut Vec<Object>,
    level: &mut Level,
    rng: &mut GameRng,
) -> MonsterAttackResult {
    let mut result = MonsterAttackResult::default();
    let attacker_name = attacker.name.clone();

    // Check if monster can attack
    if !can_monster_attack(attacker, player) {
        return result;
    }

    // Check distance - most attacks require adjacency
    let distance = ((attacker.x - player.pos.x)
        .abs()
        .max((attacker.y - player.pos.y).abs())) as i32;

    // Check for auto-miss conditions (displacement, etc.)
    if automiss(player, attacker) {
        result
            .messages
            .push(format!("The {} misses wildly!", attacker_name));
        return result;
    }

    // Process each attack in the monster's attack set
    for attack in &attacker.attacks {
        if !attack.is_active() {
            continue;
        }

        // Check if attack can reach
        if attack.attack_type.requires_adjacency() && distance > 1 {
            continue;
        }

        // Skip passive attacks (they trigger when monster is attacked)
        if attack.attack_type.is_passive() {
            continue;
        }

        // Process the attack based on type
        let attack_result = process_single_attack(attacker, player, inventory, level, attack, rng);

        // Accumulate results
        if attack_result.hit {
            result.any_hit = true;
            result.total_damage += attack_result.damage;

            // Add weapon swing message for weapon attacks (before hit message)
            if attack.attack_type == AttackType::Weapon {
                // Check if monster has a wielded weapon
                if let Some(weapon_idx) = attacker.wielded {
                    if let Some(weapon) = attacker.inventory.get(weapon_idx) {
                        let weapon_name_str = weapon.name.as_deref().unwrap_or("weapon");
                        let is_thrust = weapon_name_str.contains("spear")
                            || weapon_name_str.contains("lance")
                            || weapon_name_str.contains("trident");
                        let display = weapon.display_name();
                        result.messages.push(weapon_swing_message(
                            &attacker_name,
                            &display,
                            is_thrust,
                        ));
                    }
                }
            }

            // Add hit message
            result
                .messages
                .push(hit_message(&attacker_name, attack.attack_type));

            // Add damage-specific message
            if let Some(msg) = damage_effect_message(&attacker_name, attack.damage_type) {
                result.messages.push(msg);
            }

            // Track special effects
            if let Some(effect) = attack_result.special_effect {
                result.effects.push(effect);
            }

            #[cfg(feature = "extensions")]
            {
                use crate::monster::combat_hooks;
                combat_hooks::on_monster_hit_player(
                    attacker.id,
                    level,
                    attack_result.damage,
                    attack.attack_type,
                    attack.damage_type,
                );
            }
        } else {
            // Miss message
            let near_miss = rng.one_in(2);
            result
                .messages
                .push(miss_message(&attacker_name, near_miss));

            #[cfg(feature = "extensions")]
            {
                use crate::monster::combat_hooks;
                combat_hooks::on_monster_miss_player(attacker.id, level, attack.attack_type);
            }
        }

        // Check for player death
        if attack_result.defender_died {
            result.player_died = true;
            result.messages.push("You die...".to_string());
            break;
        }

        // Check for attacker death (passive damage from player)
        if attack_result.attacker_died {
            result.monster_died = true;
            break;
        }

        // Apply passive damage to attacker (e.g., acid blood, fire body)
        if attack_result.hit && attack.attack_type.is_melee() {
            let passive_dmg = passiveum(player, attacker, rng);
            if passive_dmg > 0 {
                result.messages.push(format!(
                    "The {} is hurt by your passive defense!",
                    attacker_name
                ));
            }
        }
    }

    result
}

/// Check if a monster can attack the player
fn can_monster_attack(attacker: &Monster, player: &You) -> bool {
    // Can't attack if peaceful or tame
    if attacker.state.peaceful || attacker.state.tame {
        return false;
    }

    // Can't attack if sleeping or paralyzed
    if attacker.state.sleeping || attacker.state.paralyzed {
        return false;
    }

    // Can't attack if fleeing
    if attacker.state.fleeing {
        return false;
    }

    // Can't attack if cancelled (for some attack types)
    // This is checked per-attack in the C code

    // Player can't be attacked if buried (unless attacker can dig)
    if player.buried {
        return false;
    }

    true
}

/// Process a single attack from a monster
fn process_single_attack(
    attacker: &Monster,
    player: &mut You,
    inventory: &mut Vec<Object>,
    level: &mut Level,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    match attack.attack_type {
        AttackType::Engulf => process_engulf_attack(attacker, player, attack, rng),
        AttackType::Explode => process_explode_attack(attacker, player, attack, rng),
        AttackType::Gaze => process_gaze_attack(attacker, player, attack, rng),
        AttackType::Breath | AttackType::Spit => {
            process_ranged_attack(attacker, player, attack, rng)
        }
        _ => {
            // Standard melee attack - use the full version for special effects
            let (result, _msg) =
                monster_attack_player_full(attacker, player, inventory, level, attack, rng);
            result
        }
    }
}

/// Process engulf attack (gulpmu in C)
fn process_engulf_attack(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Engulfing attack - swallow the player
    let mut result = monster_attack_player(attacker, player, attack, rng);

    if result.hit {
        player.swallowed = true;
        result.special_effect = Some(CombatEffect::Engulfed);

        // Check for blindness when engulfed
        if gulp_blnd_check(player) {
            player.blinded_timeout = player.blinded_timeout.saturating_add(1);
        }
    }

    result
}

/// Process explosion attack (explmu in C)
fn process_explode_attack(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    explmu(player, attacker, attack, rng)
}

/// Process gaze attack (gazemu in C)
fn process_gaze_attack(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    use crate::player::Property;

    // Gaze attacks can be blocked by blindness or reflection
    if player.is_blind() {
        return CombatResult::MISS; // Can't see the gaze
    }

    // Reflection blocks some gaze attacks
    if player.properties.has(Property::Reflection) {
        // Reflected back at monster - could damage them
        // For now, just miss
        return CombatResult::MISS;
    }

    monster_attack_player(attacker, player, attack, rng)
}

/// Process ranged attack (breath/spit)
fn process_ranged_attack(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Ranged attacks have a chance to miss based on distance
    let distance = ((attacker.x - player.pos.x)
        .abs()
        .max((attacker.y - player.pos.y).abs())) as u32;

    // Miss chance increases with distance
    if distance > 1 && rng.one_in(distance) {
        return CombatResult::MISS;
    }

    monster_attack_player(attacker, player, attack, rng)
}

/// Calculate monster's to-hit bonus
///
/// Based on find_roll_to_hit() in mhitu.c
/// Get monster's attack skill level based on type and experience
///
/// Determines how skilled a monster is at combat based on its type and level
fn get_monster_attack_skill(attacker: &Monster) -> SkillLevel {
    match attacker.level {
        0..=2 => SkillLevel::Unskilled,
        3..=6 => SkillLevel::Basic,
        7..=12 => SkillLevel::Skilled,
        13..=20 => SkillLevel::Expert,
        _ => SkillLevel::Master,
    }
}

fn calculate_monster_to_hit(attacker: &Monster, player: &You) -> i32 {
    // Base is monster level
    let mut to_hit = attacker.level as i32;

    // Monster state penalties
    if attacker.state.confused {
        to_hit -= 2;
    }
    if attacker.state.stunned {
        to_hit -= 2;
    }
    if attacker.state.blinded {
        to_hit -= 2;
    }

    // Bonus vs disabled player
    if player.is_stunned() {
        to_hit += 2;
    }
    if player.is_confused() {
        to_hit += 2;
    }
    if player.is_blind() {
        to_hit += 2;
    }
    if player.sleeping_timeout > 0 {
        to_hit += 4;
    }
    if player.paralyzed_timeout > 0 {
        to_hit += 4;
    }

    to_hit
}

/// Calculate damage multiplier based on player's elemental resistances
/// Returns (multiplier_num, multiplier_den) where damage = damage * num / den
fn damage_multiplier_for_resistance(damage_type: DamageType, player: &You) -> (i32, i32) {
    use crate::player::Property;

    match damage_type {
        DamageType::Fire => {
            if player.properties.has(Property::FireResistance) {
                (0, 1) // No damage
            } else {
                (1, 1) // Full damage
            }
        }
        DamageType::Cold => {
            if player.properties.has(Property::ColdResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Electric => {
            if player.properties.has(Property::ShockResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::Acid => {
            if player.properties.has(Property::AcidResistance) {
                (1, 2) // Half damage with acid resistance
            } else {
                (1, 1)
            }
        }
        DamageType::Disintegrate => {
            if player.properties.has(Property::DisintResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        DamageType::MagicMissile => {
            if player.properties.has(Property::MagicResistance) {
                (0, 1)
            } else {
                (1, 1)
            }
        }
        _ => {
            // Check for half physical damage for physical attacks
            if damage_type == DamageType::Physical
                && player.properties.has(Property::HalfPhysDamage)
            {
                (1, 2)
            } else {
                (1, 1)
            }
        }
    }
}

/// Monster melee attack against player
pub fn monster_attack_player(
    attacker: &Monster,
    player: &mut You,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Check if monster can reach player (must be adjacent for melee)
    let dx = (attacker.x - player.pos.x).abs();
    let dy = (attacker.y - player.pos.y).abs();
    if dx > 1 || dy > 1 {
        return CombatResult::MISS;
    }

    // Paralyzed/frozen monsters can't attack
    if attacker.frozen_timeout > 0 || attacker.state.paralyzed {
        return CombatResult::MISS;
    }

    // Phase 13: Check if player is incapacitated by status effects
    if player.status_effects.is_incapacitated() {
        // Apply passive damage from status effects to player
        let status_damage = calculate_status_damage(&player.status_effects);
        player.hp = (player.hp - status_damage).max(0);
        // Don't let incapacitated player dodge or defend effectively
    }

    // Get monster's attack skill level
    let skill_level = get_monster_attack_skill(attacker);

    // Calculate to-hit
    let base_to_hit = calculate_monster_to_hit(attacker, player);

    // Phase 13: Apply player status effect penalties to monster's to-hit
    let player_ac_penalty = player.status_effects.ac_penalty();
    let effective_to_hit = base_to_hit + player_ac_penalty;

    // Add skill-based to-hit bonus
    let enhanced_to_hit = effective_to_hit + skill_level.hit_bonus();

    // Roll to hit
    // Formula: roll + to_hit > 10 - AC means hit
    // With AC 10 (no armor), need roll + to_hit > 0 (always hits with any to_hit > -19)
    // With AC -10 (good armor), need roll + to_hit > 20 (harder to hit)
    let roll = rng.rnd(20) as i32;
    if roll + enhanced_to_hit <= 10 - player.armor_class as i32 {
        return CombatResult::MISS;
    }

    // Roll for critical hit
    let critical = determine_critical_hit(roll, skill_level, rng);

    // Calculate base damage
    let mut damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply resistance-based damage reduction
    let (mult_num, mult_den) = damage_multiplier_for_resistance(attack.damage_type, player);
    damage = damage * mult_num / mult_den;

    // Apply skill-enhanced damage with critical multiplier
    damage = calculate_skill_enhanced_damage(damage, skill_level, critical);

    // Handle instant kill
    let player_died = if critical == CriticalHitType::InstantKill {
        player.hp = 0;
        true
    } else {
        // Apply damage to player (minimum 0 after resistance)
        if damage > 0 {
            player.hp -= damage;
            // Check if spell is interrupted by damage
            if player.casting_spell.is_some() && !player.properties.has(Property::Concentration) {
                // DC based on damage amount (10 + damage/10)
                let dc = 10 + (damage / 10);
                if rng.percent(dc as u32) {
                    player.casting_interrupted = true;
                }
            }
        }
        player.hp <= 0
    };

    // Apply special damage effects based on damage type (enhanced by criticals)
    let mut special_effect = apply_damage_effect(attack.damage_type, player, damage, rng);

    // Phase 13: On critical hits, trigger status effects using new system
    if critical.is_critical() && skill_level as u8 >= SkillLevel::Skilled as u8 {
        let effect_severity = effect_severity_from_skill(&skill_level);

        // Try to trigger poison/disease based on attack type
        match attack.attack_type {
            AttackType::Bite | AttackType::Sting => {
                if should_trigger_special_effect(&SpecialCombatEffect::Poison, &skill_level, rng) {
                    apply_special_effect(
                        &SpecialCombatEffect::Poison,
                        &mut player.status_effects,
                        &format!("monster {} attack", attack.attack_type),
                        effect_severity,
                    );
                    if special_effect.is_none() {
                        special_effect = Some(CombatEffect::Poisoned);
                    }
                }
            }
            AttackType::Claw => {
                if should_trigger_special_effect(&SpecialCombatEffect::Disease, &skill_level, rng) {
                    apply_special_effect(
                        &SpecialCombatEffect::Disease,
                        &mut player.status_effects,
                        "monster claw wound",
                        effect_severity,
                    );
                    if special_effect.is_none() {
                        special_effect = Some(CombatEffect::ItemDestroyed);
                    }
                }
            }
            AttackType::Gaze => {
                if should_trigger_special_effect(&SpecialCombatEffect::Stun, &skill_level, rng) {
                    apply_special_effect(
                        &SpecialCombatEffect::Stun,
                        &mut player.status_effects,
                        "monster gaze attack",
                        1,
                    );
                    if special_effect.is_none() {
                        special_effect = Some(CombatEffect::Blinded);
                    }
                }
            }
            _ => {}
        }
    }

    // Phase 13: Apply passive damage from monster's status effects
    let monster_status_damage = calculate_status_damage(&attacker.status_effects);
    // Note: Can't damage monster here (immutable reference), would need separate call

    CombatResult {
        hit: true,
        defender_died: player_died,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Apply special effects based on damage type
/// Returns (effect, damage_multiplier) where damage_multiplier adjusts the base damage
fn apply_damage_effect(
    damage_type: DamageType,
    player: &mut You,
    _damage: i32,
    rng: &mut GameRng,
) -> Option<CombatEffect> {
    use crate::player::{Attribute, Property};

    match damage_type {
        DamageType::Physical => None,

        DamageType::Fire => {
            // Fire resistance negates fire damage effects
            if player.properties.has(Property::FireResistance) {
                // With resistance, 1/20 chance to still burn items
                if rng.one_in(20) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            } else {
                // Without resistance, 1/3 chance to burn scrolls/potions
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Cold => {
            // Cold resistance negates cold damage effects
            if player.properties.has(Property::ColdResistance) {
                None
            } else {
                // 1/3 chance to freeze and shatter potions
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Electric => {
            // Shock resistance negates electric damage effects
            if player.properties.has(Property::ShockResistance) {
                None
            } else {
                // 1/3 chance to destroy rings or wands
                if rng.one_in(3) {
                    Some(CombatEffect::ItemDestroyed)
                } else {
                    None
                }
            }
        }

        DamageType::Sleep => {
            // Sleep resistance protects against sleep attacks
            if player.properties.has(Property::SleepResistance) {
                None
            } else if rng.one_in(3) {
                // Put player to sleep for 5-14 turns
                let duration = rng.rnd(10) as u16 + 5;
                player.sleeping_timeout = player.sleeping_timeout.saturating_add(duration);
                Some(CombatEffect::Paralyzed)
            } else {
                None
            }
        }

        DamageType::DrainLife => {
            // Drain resistance protects against level drain
            if player.properties.has(Property::DrainResistance) {
                None
            } else if player.exp_level > 1 {
                // Drain one experience level (minimum 1)
                player.exp_level -= 1;
                // Also reduce max HP slightly
                player.hp_max = (player.hp_max - rng.rnd(5) as i32).max(1);
                player.hp = player.hp.min(player.hp_max);
                Some(CombatEffect::Drained)
            } else {
                None
            }
        }

        DamageType::Stone => {
            // Stone resistance protects against petrification
            if player.properties.has(Property::StoneResistance) {
                None
            } else {
                // Petrification is usually instant death if not resisted
                Some(CombatEffect::Petrifying)
            }
        }

        DamageType::DrainStrength => {
            // Poison resistance protects against strength drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                // Drain 1 point of strength
                let current_str = player.attr_current.get(Attribute::Strength);
                if current_str > 3 {
                    player.attr_current.modify(Attribute::Strength, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::DrainDexterity => {
            // Poison resistance protects against dexterity drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                let current_dex = player.attr_current.get(Attribute::Dexterity);
                if current_dex > 3 {
                    player.attr_current.modify(Attribute::Dexterity, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::DrainConstitution => {
            // Poison resistance protects against constitution drain
            if player.properties.has(Property::PoisonResistance) {
                None
            } else {
                let current_con = player.attr_current.get(Attribute::Constitution);
                if current_con > 3 {
                    player.attr_current.modify(Attribute::Constitution, -1);
                    Some(CombatEffect::Poisoned)
                } else {
                    None
                }
            }
        }

        DamageType::Disease => {
            // Sick resistance protects against disease
            if player.properties.has(Property::SickResistance) {
                None
            } else {
                // Apply sickness - drain constitution over time
                let current_con = player.attr_current.get(Attribute::Constitution);
                if current_con > 3 {
                    player.attr_current.modify(Attribute::Constitution, -1);
                }
                Some(CombatEffect::Diseased)
            }
        }

        DamageType::Acid => {
            // Acid resistance negates acid damage effects
            if player.properties.has(Property::AcidResistance) {
                None
            } else {
                // Corrode armor - reduce AC temporarily
                // In real NetHack this would erode specific armor pieces
                if rng.one_in(3) {
                    player.armor_class = player.armor_class.saturating_add(1);
                    Some(CombatEffect::ArmorCorroded)
                } else {
                    None
                }
            }
        }

        DamageType::Disintegrate => {
            // Disintegration resistance protects completely
            if player.properties.has(Property::DisintResistance) {
                None
            } else {
                // Disintegration is usually instant death
                Some(CombatEffect::Petrifying) // Reusing for instant death effect
            }
        }

        DamageType::Confuse => {
            // No direct resistance, but half spell damage might help
            let duration = rng.rnd(10) as u16 + 10;
            player.confused_timeout = player.confused_timeout.saturating_add(duration);
            Some(CombatEffect::Confused)
        }

        DamageType::Stun => {
            // Stun player for 5-9 turns
            let duration = rng.rnd(5) as u16 + 5;
            player.stunned_timeout = player.stunned_timeout.saturating_add(duration);
            Some(CombatEffect::Stunned)
        }

        DamageType::Blind => {
            // Blind player for 20-119 turns
            let duration = rng.rnd(100) as u16 + 20;
            player.blinded_timeout = player.blinded_timeout.saturating_add(duration);
            Some(CombatEffect::Blinded)
        }

        DamageType::Paralyze => {
            // Free action protects against paralysis
            if player.properties.has(Property::FreeAction) {
                None
            } else {
                // Paralyze player for 3-7 turns
                let duration = rng.rnd(5) as u16 + 3;
                player.paralyzed_timeout = player.paralyzed_timeout.saturating_add(duration);
                Some(CombatEffect::Paralyzed)
            }
        }

        DamageType::StealGold => {
            // Steal some gold (10-50%)
            if player.gold > 0 {
                let steal_percent = rng.rnd(40) as i32 + 10;
                let stolen = (player.gold * steal_percent) / 100;
                player.gold -= stolen.max(1);
                Some(CombatEffect::GoldStolen)
            } else {
                None
            }
        }

        DamageType::StealItem => {
            // Stealing is handled by steal_from_player() which needs inventory access
            Some(CombatEffect::ItemStolen)
        }

        DamageType::Teleport => {
            // Teleport is handled by teleport_player_attack() which needs level access
            Some(CombatEffect::Teleported)
        }

        DamageType::Digest => {
            player.swallowed = true;
            Some(CombatEffect::Engulfed)
        }

        DamageType::Wrap | DamageType::Stick => {
            // Grab is handled by grab_player() which needs attacker info
            Some(CombatEffect::Grabbed)
        }

        _ => None,
    }
}

/// Monster steals an item from player's inventory (from steal.c)
/// Returns the stolen item if successful, None otherwise
pub fn steal_from_player(
    attacker: &Monster,
    inventory: &mut Vec<Object>,
    rng: &mut GameRng,
) -> Option<Object> {
    if inventory.is_empty() {
        return None;
    }

    // Nymphs prefer rings and amulets, monkeys take anything
    let is_nymph = attacker.name.contains("nymph");

    // Weight items - worn/wielded items are harder to steal (weight 5 vs 1)
    let mut total_weight = 0;
    for obj in inventory.iter() {
        let weight = if obj.worn_mask != 0 { 5 } else { 1 };
        // Nymphs prefer jewelry
        if is_nymph
            && (obj.class == crate::object::ObjectClass::Ring
                || obj.class == crate::object::ObjectClass::Amulet)
        {
            total_weight += weight * 3; // Triple weight for preferred items
        } else {
            total_weight += weight;
        }
    }

    if total_weight == 0 {
        return None;
    }

    // Pick a random item based on weights
    let mut pick = rng.rn2(total_weight as u32) as i32;
    let mut steal_idx = None;

    for (idx, obj) in inventory.iter().enumerate() {
        let weight = if obj.worn_mask != 0 { 5 } else { 1 };
        let adjusted_weight = if is_nymph
            && (obj.class == crate::object::ObjectClass::Ring
                || obj.class == crate::object::ObjectClass::Amulet)
        {
            weight * 3
        } else {
            weight
        };

        pick -= adjusted_weight;
        if pick < 0 {
            steal_idx = Some(idx);
            break;
        }
    }

    // Remove and return the stolen item
    steal_idx.map(|idx| inventory.remove(idx))
}

/// Monster teleports the player randomly (from mhitu.c)
/// Returns the new position if teleported, None if teleport failed
pub fn teleport_player_attack(
    player: &mut You,
    level: &Level,
    rng: &mut GameRng,
) -> Option<(i8, i8)> {
    use crate::player::Property;

    // Teleport control lets player resist
    if player.properties.has(Property::TeleportControl) && rng.one_in(3) {
        return None; // Resisted
    }

    // Find a random valid position
    for _ in 0..100 {
        let x = rng.rn2(crate::COLNO as u32) as i8;
        let y = rng.rn2(crate::ROWNO as u32) as i8;

        if level.is_walkable(x, y) && level.monster_at(x, y).is_none() {
            player.prev_pos = player.pos;
            player.pos.x = x;
            player.pos.y = y;
            return Some((x, y));
        }
    }

    None // Failed to find valid position
}

/// Monster grabs the player (wrap/stick attacks from mhitu.c)
/// Sets the grabbed_by field on the player
pub fn grab_player(player: &mut You, attacker_id: MonsterId) {
    player.grabbed_by = Some(attacker_id);
}

/// Check if player can escape from grab
/// Returns true if player escaped
pub fn try_escape_grab(player: &mut You, rng: &mut GameRng) -> bool {
    use crate::player::Attribute;

    if player.grabbed_by.is_none() {
        return true; // Not grabbed
    }

    // Escape chance based on strength and dexterity
    let str_val = player.attr_current.get(Attribute::Strength) as i32;
    let dex_val = player.attr_current.get(Attribute::Dexterity) as i32;

    // Base 10% + 2% per point of STR+DEX above 20
    let escape_chance = 10 + ((str_val + dex_val - 20) * 2).max(0);

    if rng.percent(escape_chance as u32) {
        player.grabbed_by = None;
        true
    } else {
        false
    }
}

/// Apply grab damage each turn while grabbed
pub fn apply_grab_damage(player: &mut You, grabber: &Monster, rng: &mut GameRng) -> i32 {
    // Crushing damage based on monster level
    let damage = rng.dice(1, grabber.level as u32 / 2 + 1) as i32;
    player.hp -= damage;
    damage
}

/// Full monster attack with context for special attacks
/// This version has access to inventory and level for stealing/teleport
pub fn monster_attack_player_full(
    attacker: &Monster,
    player: &mut You,
    inventory: &mut Vec<Object>,
    level: &mut Level,
    attack: &Attack,
    rng: &mut GameRng,
) -> (CombatResult, Option<String>) {
    // First do the basic attack
    let result = monster_attack_player(attacker, player, attack, rng);

    if !result.hit {
        return (result, None);
    }

    // Handle special effects that need context
    let message = match attack.damage_type {
        DamageType::StealItem => {
            if let Some(stolen) = steal_from_player(attacker, inventory, rng) {
                Some(format!(
                    "The {} stole your {}!",
                    attacker.name,
                    stolen.display_name()
                ))
            } else {
                Some(format!(
                    "The {} couldn't find anything to steal.",
                    attacker.name
                ))
            }
        }
        DamageType::Teleport => {
            if let Some((x, y)) = teleport_player_attack(player, level, rng) {
                Some(format!("You are teleported to ({}, {})!", x, y))
            } else {
                Some("You resist the teleportation.".to_string())
            }
        }
        DamageType::Wrap | DamageType::Stick => {
            grab_player(player, attacker.id);
            Some(format!("The {} grabs you!", attacker.name))
        }
        DamageType::Slow => {
            let msg = u_slow_down(player);
            Some(msg)
        }
        DamageType::Disease => {
            let (msg, _fatal) = diseasemu(player, &attacker.name, rng);
            Some(msg)
        }
        DamageType::Seduce | DamageType::SeduceSpecial => {
            match doseduce(player, attacker, inventory, rng) {
                SeduceResult::Yes => Some(format!("The {} seduces you!", attacker.name)),
                SeduceResult::WrongGender => Some(format!("The {} ugly thing tries to seduce you.", attacker.name)),
                SeduceResult::No => None,
            }
        }
        DamageType::StealGold => {
            let amount = stealgold(inventory, player.gold, rng);
            if amount > 0 {
                player.gold -= amount;
                Some(format!("The {} steals {} gold!", attacker.name, amount))
            } else {
                Some(format!("The {} couldn't find any gold.", attacker.name))
            }
        }
        _ => None,
    };

    (result, message)
}

// ============================================================================
// Enhanced Ranged Attack System (Phase 11)
// ============================================================================

/// Monster ranged attack with distance considerations
pub fn monster_ranged_attack_enhanced(
    attacker: &Monster,
    player: &mut You,
    distance: i32,
    rng: &mut GameRng,
) -> CombatResult {
    // Get monster skill level
    let skill_level = get_monster_attack_skill(attacker);

    // Monster ranged attack (thrown rocks, etc.)
    let base_to_hit = attacker.level as i32;

    let ranged_attack = RangedAttack {
        weapon_type: RangedWeaponType::Thrown,
        distance,
        skill_level,
        base_to_hit,
    };

    // Check if in range
    if !ranged_attack.in_range() {
        return CombatResult::MISS;
    }

    // Execute ranged attack
    let mut ranged_result = execute_ranged_attack(&ranged_attack, player.armor_class, rng);

    if !ranged_result.hit {
        return CombatResult::MISS;
    }

    // Calculate damage
    let base_damage = rng.dice(1, 4) as i32;
    let damage = ranged_attack.calculate_damage(base_damage, ranged_result.critical);

    // Apply damage to player
    let player_died = if ranged_result.critical == CriticalHitType::InstantKill {
        player.hp = 0;
        true
    } else {
        player.hp -= damage;
        player.hp <= 0
    };

    // Apply special effects on critical ranged hits
    let mut special_effect = None;
    if ranged_result.critical.is_critical() && skill_level as u8 >= SkillLevel::Skilled as u8 {
        if rng.one_in(3) {
            special_effect = Some(CombatEffect::ItemDestroyed);
        }
    }

    CombatResult {
        hit: true,
        defender_died: player_died,
        attacker_died: false,
        damage,
        special_effect,
    }
}

/// Improved process_ranged_attack with new system
pub fn process_ranged_attack_enhanced(
    attacker: &Monster,
    player: &mut You,
    rng: &mut GameRng,
) -> CombatResult {
    // Calculate distance (Chebyshev distance for grid)
    let distance = (attacker.x - player.pos.x)
        .abs()
        .max(attacker.y - player.pos.y) as i32;

    // Use enhanced ranged attack
    monster_ranged_attack_enhanced(attacker, player, distance, rng)
}

// ============================================================================
// Monster Defense System (Phase 12 Integration)
// ============================================================================

/// Get monster armor proficiency based on level
pub fn get_monster_armor_proficiency(monster: &Monster) -> ArmorProficiency {
    match monster.level {
        0..=2 => ArmorProficiency::Untrained,
        3..=6 => ArmorProficiency::Novice,
        7..=12 => ArmorProficiency::Trained,
        13..=20 => ArmorProficiency::Expert,
        _ => ArmorProficiency::Master,
    }
}

/// Get monster dodge skill
pub fn get_monster_dodge_skill(monster: &Monster) -> DodgeSkill {
    match monster.level {
        0..=2 => DodgeSkill::Untrained,
        3..=6 => DodgeSkill::Basic,
        7..=12 => DodgeSkill::Practiced,
        13..=20 => DodgeSkill::Expert,
        _ => DodgeSkill::Master,
    }
}

/// Calculate monster defense
pub fn calculate_monster_defense(monster: &Monster) -> DefenseCalculation {
    let armor_prof = get_monster_armor_proficiency(monster);
    let dodge_skill = get_monster_dodge_skill(monster);

    // Base AC from monster
    let base_ac = monster.ac as i32;

    // Monster armor degradation
    let degradation = super::ArmorDegradation::new(5);

    DefenseCalculation::calculate(base_ac, armor_prof, dodge_skill, degradation)
}

/// Apply monster defense to incoming player damage
pub fn apply_monster_defense(
    monster: &Monster,
    incoming_damage: i32,
    damage_type: DamageType,
    rng: &mut crate::rng::GameRng,
) -> i32 {
    let defense = calculate_monster_defense(monster);

    // Try to dodge
    let dodge_skill = get_monster_dodge_skill(monster);
    if attempt_dodge(dodge_skill, 0, rng) {
        return 0;
    }

    // Calculate armor reduction
    let armor_type = ArmorType::Medium; // Most monsters have medium natural armor
    let reduction = calculate_armor_damage_reduction(
        defense.base_ac,
        defense.proficiency,
        damage_type,
        armor_type,
    );

    // Apply reduction
    apply_damage_reduction(incoming_damage, reduction)
}

// ============================================================================
// Phase 15: Monster Spell Casting in Combat
// ============================================================================

/// Check if a monster can cast a specific combat spell
pub fn can_monster_cast_spell(monster: &Monster, spell: super::CombatSpell) -> bool {
    // Simple heuristic: high-level monsters can cast spells
    // Spellcasting monsters (wizards, clerics, priests) have level >= 5
    monster.level >= 5
}

/// Monster attempts to cast a combat spell at the player
pub fn monster_cast_spell(
    attacker: &mut Monster,
    target: &mut You,
    spell: super::CombatSpell,
    rng: &mut crate::rng::GameRng,
) -> super::SpellCastResult {
    // Check if monster can cast (simplified)
    if !can_monster_cast_spell(attacker, spell) {
        return super::SpellCastResult::failed();
    }

    // Simulate "mana" as energy (monsters use a simple pool)
    // Assume monsters have enough energy
    let mana_cost = spell.mana_cost() / 2; // Monsters pay half cost

    // Monster spell failure chance based on level
    let failure_chance = 20 - (attacker.level as i32);
    if rng.rnd(100) as i32 <= failure_chance {
        return super::SpellCastResult::failed();
    }

    // Calculate damage - monsters do fixed damage based on spell
    let base_damage = spell.base_damage() / 2; // Monsters do half damage
    let damage = (base_damage as f32 * (attacker.level as f32 / 10.0)).max(1.0) as i32;

    let mut result = super::SpellCastResult::success().with_damage(damage);

    // Apply spell effects
    match spell {
        super::CombatSpell::ForceBolt | super::CombatSpell::MagicMissile => {
            target.hp -= damage;
        }
        super::CombatSpell::Fireball => {
            target.hp -= damage;
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Stunned,
                1,
                "spell",
            );
        }
        super::CombatSpell::Sleep => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Stunned,
                2,
                "sleep spell",
            );
            result = result.with_effect(StatusEffect::Stunned);
        }
        super::CombatSpell::Slow => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Paralyzed,
                1,
                "slow spell",
            );
        }
        super::CombatSpell::Confuse => {
            apply_status_effect(
                &mut target.status_effects,
                StatusEffect::Paralyzed,
                2,
                "confusion",
            );
        }
        _ => {}
    }

    result
}

// ============================================================================
// Additional mhitu functions (expels, gulp_blnd_check, u_slow_down, etc.)
// ============================================================================

/// Player is expelled from an engulfing monster (expels in C).
///
/// # Arguments
/// * `player` - The player being expelled
/// * `attacker_name` - Name of the engulfing monster
/// * `is_animal` - Whether the engulfer is an animal (regurgitate vs expelled)
/// * `damage_type` - The type of damage from engulf attack (for message flavor)
///
/// # Returns
/// Message describing the expulsion
pub fn expels(
    player: &mut You,
    attacker_name: &str,
    is_animal: bool,
    damage_type: DamageType,
) -> String {
    // Clear swallowed state
    player.swallowed = false;

    if is_animal {
        "You get regurgitated!".to_string()
    } else {
        let blast = match damage_type {
            DamageType::Electric => " in a shower of sparks",
            DamageType::Cold => " in a blast of frost",
            _ => " with a squelch",
        };
        format!("You get expelled from {}{}!", attacker_name, blast)
    }
}

/// Check for blindness when engulfed (gulp_blnd_check in C).
///
/// Player can be blinded if they have a light source that gets snuffed.
///
/// # Arguments
/// * `player` - The player to check
///
/// # Returns
/// Whether the player becomes blinded
pub fn gulp_blnd_check(player: &You) -> bool {
    // In NetHack, if the player has a lit light source that gets
    // snuffed when engulfed, they may become temporarily blinded
    // For now, simplified check - engulfing causes darkness
    !player.properties.has(Property::Infravision) && !player.properties.has(Property::SeeInvisible)
}

/// Slow down the player (u_slow_down in C).
///
/// Called when player's intrinsic speed is removed.
///
/// # Arguments
/// * `player` - The player to slow down
///
/// # Returns
/// Message about slowing down
pub fn u_slow_down(player: &mut You) -> String {
    // Remove intrinsic speed
    player.properties.remove_intrinsic(Property::Speed);

    // Check if player still has extrinsic speed
    if player.properties.has(Property::Speed) {
        "Your quickness feels less natural.".to_string()
    } else {
        "You slow down.".to_string()
    }
}

/// Disease attack against the player (diseasemu in C).
///
/// Applies disease effects like those from rats, zombies, etc.
///
/// # Arguments
/// * `player` - The player being diseased
/// * `attacker_name` - Name of the diseasing monster
/// * `rng` - Random number generator
///
/// # Returns
/// Result with message and whether player was diseased
pub fn diseasemu(player: &mut You, attacker_name: &str, rng: &mut GameRng) -> (String, bool) {
    // Check for disease resistance (poison resistance helps)
    if player.properties.has(Property::PoisonResistance) {
        return (
            format!("You feel a slight illness from {}'s attack.", attacker_name),
            false,
        );
    }

    // Check for sick resistance
    if player.properties.has(Property::SickResistance) {
        return (format!("You resist {}'s disease.", attacker_name), false);
    }

    // Apply disease - reduces max HP over time
    // In full implementation, would track disease state
    let hp_loss = rng.rnd(4) as i32;
    player.hp_max = (player.hp_max - hp_loss).max(1);
    player.hp = player.hp.min(player.hp_max);

    (
        format!("You feel very sick from {}'s attack!", attacker_name),
        true,
    )
}

/// Monster engulfs the player (gulpmu in C).
///
/// Handles the engulfing attack where monster swallows the player.
///
/// # Arguments
/// * `player` - The player being engulfed
/// * `attacker` - The engulfing monster
/// * `attack` - The engulf attack
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result with messages
pub fn gulpmu(
    player: &mut You,
    attacker: &Monster,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Check if player is too big to engulf (polymorphed into something huge)
    // For now, always allow engulf

    // Set swallowed state
    player.swallowed = true;

    // Calculate engulf damage
    let damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Check player resistance
    let final_damage = match attack.damage_type {
        DamageType::Fire => {
            if player.properties.has(Property::FireResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Cold => {
            if player.properties.has(Property::ColdResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Electric => {
            if player.properties.has(Property::ShockResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Acid => {
            if player.properties.has(Property::AcidResistance) {
                damage / 2
            } else {
                damage
            }
        }
        _ => damage,
    };

    // Apply damage
    player.hp -= final_damage;

    CombatResult {
        hit: true,
        defender_died: player.hp <= 0,
        attacker_died: false,
        damage: final_damage,
        special_effect: Some(CombatEffect::Engulfed),
    }
}

/// Monster explodes against the player (explmu in C).
///
/// Handles explosion attacks like gas spores.
///
/// # Arguments
/// * `player` - The player caught in explosion
/// * `attacker` - The exploding monster
/// * `attack` - The explosion attack
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result (attacker always dies)
pub fn explmu(
    player: &mut You,
    attacker: &Monster,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Can't explode if cancelled
    if attacker.state.cancelled {
        return CombatResult::MISS;
    }

    // Calculate explosion damage
    let damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Check player resistance
    let final_damage = match attack.damage_type {
        DamageType::Fire => {
            if player.properties.has(Property::FireResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Cold => {
            if player.properties.has(Property::ColdResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Electric => {
            if player.properties.has(Property::ShockResistance) {
                0
            } else {
                damage
            }
        }
        DamageType::Acid => {
            if player.properties.has(Property::AcidResistance) {
                damage / 2
            } else {
                damage
            }
        }
        _ => damage,
    };

    // Apply damage
    player.hp -= final_damage;

    CombatResult {
        hit: true,
        defender_died: player.hp <= 0,
        attacker_died: true, // Exploding monster always dies
        damage: final_damage,
        special_effect: None,
    }
}

/// Monster gaze attack against the player (gazemu in C).
///
/// Handles gaze attacks like medusa's petrifying gaze.
///
/// # Arguments
/// * `player` - The player being gazed at
/// * `attacker` - The gazing monster
/// * `attack` - The gaze attack
/// * `rng` - Random number generator
///
/// # Returns
/// Combat result
pub fn gazemu(
    player: &mut You,
    attacker: &Monster,
    attack: &Attack,
    rng: &mut GameRng,
) -> CombatResult {
    // Can't gaze if cancelled or blind
    if attacker.state.cancelled || attacker.state.blinded {
        return CombatResult::MISS;
    }

    // Player must be able to see (blind players are immune)
    if player.is_blind() {
        return CombatResult::MISS;
    }

    // Process gaze based on damage type
    match attack.damage_type {
        DamageType::Stone => {
            // Petrifying gaze - check for resistance
            if player.properties.has(Property::StoneResistance) {
                return CombatResult::MISS;
            }

            // Check for reflection (from shield or amulet)
            if player.properties.has(Property::Reflection) {
                // Gaze reflected back
                return CombatResult {
                    hit: false,
                    defender_died: false,
                    attacker_died: !attacker.resists_stone(),
                    damage: 0,
                    special_effect: Some(CombatEffect::Petrifying),
                };
            }

            // Player is petrified
            CombatResult {
                hit: true,
                defender_died: true,
                attacker_died: false,
                damage: 0,
                special_effect: Some(CombatEffect::Petrifying),
            }
        }
        DamageType::Confuse => {
            // Confusing gaze
            if player.properties.has(Property::HalfSpellDamage) {
                // Magic resistance helps
                if rng.one_in(2) {
                    return CombatResult::MISS;
                }
            }

            let duration = rng.rnd(10) as i32 + 5;
            player.confused_timeout = player.confused_timeout.saturating_add(duration as u16);

            CombatResult {
                hit: true,
                defender_died: false,
                attacker_died: false,
                damage: 0,
                special_effect: Some(CombatEffect::Confused),
            }
        }
        DamageType::Paralyze => {
            // Paralysis gaze (floating eye)
            if player.properties.has(Property::FreeAction) {
                return CombatResult::MISS;
            }

            let duration = rng.rnd(50) as i32 + 50;
            player.paralyzed_timeout = player.paralyzed_timeout.saturating_add(duration as u16);

            CombatResult {
                hit: true,
                defender_died: false,
                attacker_died: false,
                damage: 0,
                special_effect: Some(CombatEffect::Paralyzed),
            }
        }
        _ => {
            // Other gaze attacks - apply damage
            let damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;
            player.hp -= damage;

            CombatResult {
                hit: true,
                defender_died: player.hp <= 0,
                attacker_died: false,
                damage,
                special_effect: None,
            }
        }
    }
}

/// Player passive damage to attacking monster (passiveum in C).
///
/// Some players (polymorphed) have passive attacks that damage attackers.
///
/// # Arguments
/// * `player` - The player
/// * `attacker` - The monster that attacked the player
/// * `rng` - Random number generator
///
/// # Returns
/// Damage dealt to the attacker (0 if none)
pub fn passiveum(_player: &You, _attacker: &Monster, _rng: &mut GameRng) -> i32 {
    // Passive damage when polymorphed (acid blob, cockatrice, etc.)
    // requires polymorph form attack data; returns 0 until polymorph system tracks attacks
    0
}

/// Player is hit by a thrown/ranged object (thitu in C).
///
/// # Arguments
/// * `player` - The player being hit
/// * `damage` - Base damage
/// * `obj_name` - Name of the hitting object
/// * `rng` - Random number generator
///
/// # Returns
/// Whether the hit was successful and message
pub fn thitu(
    player: &mut You,
    mut damage: i32,
    obj_name: &str,
    _rng: &mut GameRng,
) -> (bool, String) {
    // Check for miss conditions
    // In full implementation, would check AC, displacement, etc.

    // Apply AC reduction
    let ac_reduction = (-player.armor_class as i32).max(0);
    damage = (damage - ac_reduction).max(1);

    // Apply damage
    player.hp -= damage;

    let msg = format!("The {} hits you!", obj_name);
    (true, msg)
}

/// Check auto-miss conditions (automiss in C).
///
/// # Arguments
/// * `player` - The player being attacked
/// * `attacker` - The attacking monster
///
/// # Returns
/// Whether attack automatically misses
pub fn automiss(player: &You, attacker: &Monster) -> bool {
    // Check if player is displaced
    if player.properties.has(Property::Displaced) {
        return true;
    }

    // Check if player is invisible and attacker can't see invisible
    if player.properties.has(Property::Invisibility) && !attacker.sees_invisible() {
        return true;
    }

    false
}

/// Steal a specific item from player's inventory (steal_it in C).
///
/// # Arguments
/// * `inventory` - Player's inventory
/// * `index` - Index of item to steal
///
/// # Returns
/// The stolen item if successful
pub fn steal_it(inventory: &mut Vec<Object>, index: usize) -> Option<Object> {
    if index < inventory.len() {
        Some(inventory.remove(index))
    } else {
        None
    }
}

/// Steal the amulet from the player (stealamulet in C).
///
/// Specifically targets the Amulet of Yendor or other worn amulets.
///
/// # Arguments
/// * `inventory` - Player's inventory
///
/// # Returns
/// The stolen amulet if one was worn
pub fn stealamulet(inventory: &mut Vec<Object>) -> Option<Object> {
    // Find worn amulet (worn_mask includes amulet slot)
    let amulet_idx = inventory
        .iter()
        .position(|obj| obj.class == crate::object::ObjectClass::Amulet && obj.worn_mask != 0);

    if let Some(idx) = amulet_idx {
        Some(inventory.remove(idx))
    } else {
        None
    }
}

/// Steal armor from the player (stealarm in C).
///
/// Steals a random piece of worn armor.
///
/// # Arguments
/// * `inventory` - Player's inventory
/// * `rng` - Random number generator
///
/// # Returns
/// The stolen armor if successful
pub fn stealarm(inventory: &mut Vec<Object>, rng: &mut GameRng) -> Option<Object> {
    // Find all worn armor
    let armor_indices: Vec<usize> = inventory
        .iter()
        .enumerate()
        .filter_map(|(i, obj)| {
            if obj.class == crate::object::ObjectClass::Armor && obj.worn_mask != 0 {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if armor_indices.is_empty() {
        return None;
    }

    // Pick random armor to steal
    let idx = armor_indices[rng.rn2(armor_indices.len() as u32) as usize];
    Some(inventory.remove(idx))
}

/// Steal gold from the player (stealgold in C).
///
/// # Arguments
/// * `inventory` - Player's inventory
/// * `max_amount` - Maximum gold to steal
/// * `rng` - Random number generator
///
/// # Returns
/// Amount of gold stolen
pub fn stealgold(inventory: &mut Vec<Object>, max_amount: i32, rng: &mut GameRng) -> i32 {
    // Find gold in inventory
    let gold_idx = inventory
        .iter()
        .position(|obj| obj.class == crate::object::ObjectClass::Coin);

    if let Some(idx) = gold_idx {
        let gold = &mut inventory[idx];
        let steal_amount = (rng.rnd(max_amount as u32) as i32).min(gold.quantity);

        if steal_amount >= gold.quantity {
            // Steal all gold
            let amount = gold.quantity;
            inventory.remove(idx);
            amount
        } else {
            // Steal some gold
            gold.quantity -= steal_amount;
            steal_amount
        }
    } else {
        0
    }
}

/// Seduction attempt against the player (doseduce in C).
///
/// Handles seduction by nymphs, incubi, succubi.
///
/// # Arguments
/// * `player` - The player being seduced
/// * `attacker` - The seducing monster
/// * `inventory` - Player's inventory
/// * `rng` - Random number generator
///
/// # Returns
/// Result of seduction (item stolen, effect applied, etc.)
pub fn doseduce(
    player: &mut You,
    attacker: &Monster,
    inventory: &mut Vec<Object>,
    rng: &mut GameRng,
) -> SeduceResult {
    // Check if seduction is possible
    let seduce_check = could_seduce(
        attacker,
        0, // Attacker gender - simplified
        0, // Defender gender - simplified
        !player.is_blind(),
        None,
        attacker.state.invisible,
        false, // Not necessarily a demon
    );

    if seduce_check == SeduceResult::No {
        return SeduceResult::No;
    }

    // Seduction effects based on monster type
    // Nymphs steal items, demons have... other effects

    // For nymphs, steal an item
    if !inventory.is_empty() {
        let idx = rng.rn2(inventory.len() as u32) as usize;
        if steal_it(inventory, idx).is_some() {
            return SeduceResult::Yes;
        }
    }

    // For demons, could apply stat drain or other effects
    // Simplified for now

    SeduceResult::Yes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;
    use crate::player::Attribute;

    fn test_player() -> You {
        let mut player = You::default();
        player.attr_current.set(Attribute::Dexterity, 10); // Neutral AC bonus
        player
    }

    fn test_monster(level: u8) -> Monster {
        // Place adjacent to player (default pos 0,0) so melee reach check passes
        let mut monster = Monster::new(MonsterId(1), level as i16, 1, 0);
        monster.level = level;
        monster
    }

    #[test]
    fn test_monster_to_hit_base() {
        let player = test_player();
        let monster = test_monster(5);

        // Level 5 monster has to-hit of 5
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 5);
    }

    #[test]
    fn test_monster_to_hit_high_level() {
        let player = test_player();
        let monster = test_monster(15);

        // Level 15 monster has to-hit of 15
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 15);
    }

    #[test]
    fn test_monster_confused_penalty() {
        let player = test_player();
        let mut monster = test_monster(5);
        monster.state.confused = true;

        // Level 5 monster confused: 5 - 2 = 3
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 3);
    }

    #[test]
    fn test_monster_vs_stunned_player() {
        let mut player = test_player();
        player.stunned_timeout = 10;
        let monster = test_monster(5);

        // Level 5 monster vs stunned player: 5 + 2 = 7
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 7);
    }

    #[test]
    fn test_monster_vs_sleeping_player() {
        let mut player = test_player();
        player.sleeping_timeout = 10;
        let monster = test_monster(5);

        // Level 5 monster vs sleeping player: 5 + 4 = 9
        let to_hit = calculate_monster_to_hit(&monster, &player);
        assert_eq!(to_hit, 9);
    }

    #[test]
    fn test_monster_attack_hits_with_ac() {
        let mut player = test_player();
        let monster = test_monster(10);
        let mut rng = GameRng::new(42);

        // Player with AC 10 (no armor)
        player.armor_class = 10;

        let attack = Attack::new(crate::combat::AttackType::Claw, DamageType::Physical, 1, 6);

        // Level 10 monster vs AC 10 player
        // Roll + 10 > 10 - 10 = 0, so need roll > -10, always hits
        let mut hits = 0;
        for _ in 0..100 {
            player.hp = 100;
            let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);
            if result.hit {
                hits += 1;
            }
        }
        assert_eq!(hits, 100, "Level 10 monster should always hit AC 10");
    }

    #[test]
    fn test_monster_attack_misses_good_ac() {
        let mut player = test_player();
        let monster = test_monster(1);
        let mut rng = GameRng::new(42);

        // Player with AC -10 (very good armor)
        player.armor_class = -10;

        let attack = Attack::new(crate::combat::AttackType::Claw, DamageType::Physical, 1, 6);

        // Level 1 monster vs AC -10 player
        // Roll + 1 > 10 - (-10) = 20, so need roll > 19, only roll of 20 hits (5% chance)
        let mut hits = 0;
        for _ in 0..1000 {
            player.hp = 100;
            let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);
            if result.hit {
                hits += 1;
            }
        }
        // Should hit about 5% of the time (1 in 20)
        assert!(
            hits > 20 && hits < 100,
            "Level 1 monster vs AC -10 should hit about 5%, got {}",
            hits
        );
    }

    #[test]
    fn test_confuse_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.confused_timeout, 0);

        let effect = apply_damage_effect(DamageType::Confuse, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Confused));
        assert!(
            player.confused_timeout >= 10,
            "Should be confused for at least 10 turns"
        );
        assert!(
            player.confused_timeout <= 19,
            "Should be confused for at most 19 turns"
        );
    }

    #[test]
    fn test_stun_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.stunned_timeout, 0);

        let effect = apply_damage_effect(DamageType::Stun, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Stunned));
        assert!(
            player.stunned_timeout >= 5,
            "Should be stunned for at least 5 turns"
        );
        assert!(
            player.stunned_timeout <= 9,
            "Should be stunned for at most 9 turns"
        );
    }

    #[test]
    fn test_blind_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.blinded_timeout, 0);

        let effect = apply_damage_effect(DamageType::Blind, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Blinded));
        assert!(
            player.blinded_timeout >= 20,
            "Should be blinded for at least 20 turns"
        );
        assert!(
            player.blinded_timeout <= 119,
            "Should be blinded for at most 119 turns"
        );
    }

    #[test]
    fn test_paralyze_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.paralyzed_timeout, 0);

        let effect = apply_damage_effect(DamageType::Paralyze, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Paralyzed));
        assert!(
            player.paralyzed_timeout >= 3,
            "Should be paralyzed for at least 3 turns"
        );
        assert!(
            player.paralyzed_timeout <= 7,
            "Should be paralyzed for at most 7 turns"
        );
    }

    #[test]
    fn test_drain_life_effect() {
        let mut player = test_player();
        player.exp_level = 5;
        player.hp_max = 50;
        player.hp = 50;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Drained));
        assert_eq!(player.exp_level, 4, "Should lose one experience level");
        assert!(player.hp_max < 50, "Max HP should be reduced");
    }

    #[test]
    fn test_drain_life_at_level_1() {
        let mut player = test_player();
        player.exp_level = 1;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not drain below level 1");
        assert_eq!(player.exp_level, 1, "Should stay at level 1");
    }

    #[test]
    fn test_drain_strength_effect() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 16);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Poisoned));
        assert_eq!(
            player.attr_current.get(Attribute::Strength),
            15,
            "Should lose 1 strength"
        );
    }

    #[test]
    fn test_drain_strength_at_minimum() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 3);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not drain below 3 strength");
        assert_eq!(player.attr_current.get(Attribute::Strength), 3);
    }

    #[test]
    fn test_steal_gold_effect() {
        let mut player = test_player();
        player.gold = 1000;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::StealGold, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::GoldStolen));
        assert!(player.gold < 1000, "Should have lost some gold");
        assert!(player.gold >= 500, "Should have lost at most 50%");
    }

    #[test]
    fn test_steal_gold_no_gold() {
        let mut player = test_player();
        player.gold = 0;
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::StealGold, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Should not steal if no gold");
    }

    #[test]
    fn test_engulf_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert!(!player.swallowed);

        let effect = apply_damage_effect(DamageType::Digest, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Engulfed));
        assert!(player.swallowed, "Player should be swallowed");
    }

    // Resistance tests
    use crate::player::Property;

    #[test]
    fn test_sleep_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::SleepResistance);
        let mut rng = GameRng::new(42);

        // Try many times - with resistance, should never sleep
        for _ in 0..100 {
            let effect = apply_damage_effect(DamageType::Sleep, &mut player, 0, &mut rng);
            assert_eq!(effect, None, "Sleep resistance should protect");
        }
        assert_eq!(player.sleeping_timeout, 0);
    }

    #[test]
    fn test_drain_resistance() {
        let mut player = test_player();
        player.exp_level = 5;
        player.properties.grant_intrinsic(Property::DrainResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainLife, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Drain resistance should protect");
        assert_eq!(player.exp_level, 5, "Level should not change");
    }

    #[test]
    fn test_stone_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::StoneResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Stone, &mut player, 0, &mut rng);

        assert_eq!(
            effect, None,
            "Stone resistance should protect from petrification"
        );
    }

    #[test]
    fn test_poison_resistance_blocks_strength_drain() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 16);
        player
            .properties
            .grant_intrinsic(Property::PoisonResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(
            effect, None,
            "Poison resistance should protect from strength drain"
        );
        assert_eq!(
            player.attr_current.get(Attribute::Strength),
            16,
            "Strength should not change"
        );
    }

    #[test]
    fn test_free_action_blocks_paralysis() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::FreeAction);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Paralyze, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Free action should protect from paralysis");
        assert_eq!(player.paralyzed_timeout, 0);
    }

    #[test]
    fn test_disintegration_resistance() {
        let mut player = test_player();
        player
            .properties
            .grant_intrinsic(Property::DisintResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Disintegrate, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Disintegration resistance should protect");
    }

    #[test]
    fn test_acid_resistance() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::AcidResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::Acid, &mut player, 0, &mut rng);

        assert_eq!(
            effect, None,
            "Acid resistance should protect from acid effects"
        );
    }

    // Damage reduction tests
    #[test]
    fn test_fire_resistance_reduces_damage() {
        let (mult_num, mult_den) =
            damage_multiplier_for_resistance(DamageType::Fire, &test_player());
        assert_eq!((mult_num, mult_den), (1, 1), "No resistance = full damage");

        let mut player = test_player();
        player.properties.grant_intrinsic(Property::FireResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Fire, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Fire resistance = no damage");
    }

    #[test]
    fn test_cold_resistance_reduces_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::ColdResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Cold, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Cold resistance = no damage");
    }

    #[test]
    fn test_shock_resistance_reduces_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::ShockResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Electric, &player);
        assert_eq!((mult_num, mult_den), (0, 1), "Shock resistance = no damage");
    }

    #[test]
    fn test_acid_resistance_halves_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::AcidResistance);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Acid, &player);
        assert_eq!(
            (mult_num, mult_den),
            (1, 2),
            "Acid resistance = half damage"
        );
    }

    #[test]
    fn test_half_physical_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::HalfPhysDamage);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Physical, &player);
        assert_eq!(
            (mult_num, mult_den),
            (1, 2),
            "Half physical damage property"
        );
    }

    #[test]
    fn test_fire_attack_with_resistance() {
        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10;
        player.properties.grant_intrinsic(Property::FireResistance);
        let monster = test_monster(10);
        let mut rng = GameRng::new(42);

        let attack = Attack::new(crate::combat::AttackType::Breath, DamageType::Fire, 3, 6);

        let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);

        // Fire resistance negates dice damage, but the skill system (level 10 = Skilled)
        // adds a damage bonus (+2) on top. With possible critical multiplier, damage
        // is small but nonzero.
        assert!(result.hit, "Should still hit");
        assert!(
            result.damage <= 5,
            "Fire damage {} should be mostly negated by resistance (only skill bonus)",
            result.damage
        );
    }

    // ========================================================================
    // Tests for message functions
    // ========================================================================

    #[test]
    fn test_hit_message() {
        assert_eq!(hit_message("goblin", AttackType::Bite), "The goblin bites!");
        assert_eq!(hit_message("troll", AttackType::Claw), "The troll claws!");
        assert_eq!(
            hit_message("dragon", AttackType::Breath),
            "The dragon breathes on you!"
        );
        assert_eq!(
            hit_message("soldier", AttackType::Weapon),
            "The soldier hits!"
        );
    }

    #[test]
    fn test_miss_message() {
        assert_eq!(miss_message("goblin", false), "The goblin misses.");
        assert_eq!(miss_message("goblin", true), "The goblin just misses!");
    }

    #[test]
    fn test_wild_miss_message() {
        // Displaced player
        assert!(wild_miss_message("goblin", true, false).contains("displaced image"));
        // Invisible displaced player
        assert!(wild_miss_message("goblin", true, true).contains("invisible"));
        // Just invisible
        assert!(wild_miss_message("goblin", false, true).contains("wildly"));
    }

    #[test]
    fn test_damage_effect_message() {
        assert!(damage_effect_message("dragon", DamageType::Fire).is_some());
        assert!(damage_effect_message("vampire", DamageType::DrainLife).is_some());
        assert!(damage_effect_message("goblin", DamageType::Physical).is_none());
    }

    #[test]
    fn test_resistance_message() {
        assert!(resistance_message(DamageType::Fire).is_some());
        assert!(resistance_message(DamageType::Cold).is_some());
        assert!(resistance_message(DamageType::Physical).is_none());
    }

    // ========================================================================
    // Tests for mattacku and related functions
    // ========================================================================

    #[test]
    fn test_can_monster_attack_peaceful() {
        let mut monster = test_monster(5);
        monster.state.peaceful = true;
        let player = test_player();

        assert!(!can_monster_attack(&monster, &player));
    }

    #[test]
    fn test_can_monster_attack_sleeping() {
        let mut monster = test_monster(5);
        monster.state.sleeping = true;
        let player = test_player();

        assert!(!can_monster_attack(&monster, &player));
    }

    #[test]
    fn test_can_monster_attack_hostile() {
        let mut monster = test_monster(5);
        monster.state = crate::monster::MonsterState::active();
        let player = test_player();

        assert!(can_monster_attack(&monster, &player));
    }

    #[test]
    fn test_mattacku_peaceful_monster() {
        let mut monster = test_monster(5);
        monster.state.peaceful = true;
        monster.attacks[0] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 6);

        let mut player = test_player();
        player.hp = 100;
        player.pos.x = 6;
        player.pos.y = 5;

        let mut inventory = Vec::new();
        let mut level = Level::default();
        let mut rng = GameRng::new(42);

        let result = mattacku(&monster, &mut player, &mut inventory, &mut level, &mut rng);

        assert!(!result.any_hit, "Peaceful monster should not attack");
        assert!(result.messages.is_empty());
    }

    #[test]
    fn test_mattacku_hostile_monster() {
        let mut monster = test_monster(10);
        monster.state = crate::monster::MonsterState::active();
        monster.x = 5;
        monster.y = 5;
        monster.attacks[0] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 6);

        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10; // Easy to hit
        player.pos.x = 6;
        player.pos.y = 5;

        let mut inventory = Vec::new();
        let mut level = Level::default();
        let mut rng = GameRng::new(42);

        let result = mattacku(&monster, &mut player, &mut inventory, &mut level, &mut rng);

        // With level 10 monster vs AC 10, should hit
        assert!(
            result.any_hit || !result.messages.is_empty(),
            "Should have attempted attack"
        );
    }

    #[test]
    fn test_mattacku_multiple_attacks() {
        let mut monster = test_monster(10);
        monster.state = crate::monster::MonsterState::active();
        monster.x = 5;
        monster.y = 5;
        // Give monster two attacks
        monster.attacks[0] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 4);
        monster.attacks[1] = Attack::new(AttackType::Bite, DamageType::Physical, 1, 6);

        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10;
        player.pos.x = 6;
        player.pos.y = 5;

        let mut inventory = Vec::new();
        let mut level = Level::default();
        let mut rng = GameRng::new(42);

        let result = mattacku(&monster, &mut player, &mut inventory, &mut level, &mut rng);

        // Should have messages for both attacks (hit or miss)
        assert!(
            result.messages.len() >= 2,
            "Should process multiple attacks"
        );
    }

    #[test]
    fn test_mattacku_out_of_range() {
        let mut monster = test_monster(10);
        monster.state = crate::monster::MonsterState::active();
        monster.x = 5;
        monster.y = 5;
        monster.attacks[0] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 6);

        let mut player = test_player();
        player.hp = 100;
        player.pos.x = 20; // Far away
        player.pos.y = 20;

        let mut inventory = Vec::new();
        let mut level = Level::default();
        let mut rng = GameRng::new(42);

        let result = mattacku(&monster, &mut player, &mut inventory, &mut level, &mut rng);

        // Melee attack should not reach
        assert!(
            !result.any_hit,
            "Melee attack should not reach distant player"
        );
        assert!(
            result.messages.is_empty(),
            "No messages for out-of-range attack"
        );
    }

    #[test]
    fn test_weapon_swing_message() {
        // Test thrust weapons
        assert!(weapon_swing_message("orc", "spear", true).contains("thrusts"));
        assert!(weapon_swing_message("orc", "long sword", false).contains("swings"));
    }

    #[test]
    fn test_mattacku_weapon_attack_with_weapon() {
        use crate::object::{Object, ObjectClass, ObjectId};

        let mut monster = test_monster(10);
        monster.state = crate::monster::MonsterState::active();
        monster.x = 5;
        monster.y = 5;
        monster.attacks[0] = Attack::new(AttackType::Weapon, DamageType::Physical, 1, 8);

        // Give monster a weapon
        let mut sword = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        sword.name = Some("long sword".to_string());
        monster.inventory.push(sword);
        monster.wielded = Some(0);

        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10;
        player.pos.x = 6;
        player.pos.y = 5;

        let mut inventory = Vec::new();
        let mut level = Level::default();
        let mut rng = GameRng::new(42);

        let result = mattacku(&monster, &mut player, &mut inventory, &mut level, &mut rng);

        // Should have weapon swing message if hit
        if result.any_hit {
            let has_swing_msg = result.messages.iter().any(|m| m.contains("swings"));
            assert!(
                has_swing_msg,
                "Should have weapon swing message for weapon attack"
            );
        }
    }

    // Tests for could_seduce function

    #[test]
    fn test_could_seduce_opposite_gender_nymph() {
        let mut monster = test_monster(5);
        monster.state.invisible = false;
        // Give monster a seduce attack
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Attacker female (1), defender male (0) - opposite genders
        let result = could_seduce(&monster, 1, 0, true, None, true, false);
        assert_eq!(
            result,
            SeduceResult::Yes,
            "Opposite gender nymph should seduce"
        );
    }

    #[test]
    fn test_could_seduce_same_gender_nymph() {
        let mut monster = test_monster(5);
        monster.state.invisible = false;
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Attacker female (1), defender female (1) - same gender
        let result = could_seduce(&monster, 1, 1, true, None, true, false);
        assert_eq!(
            result,
            SeduceResult::WrongGender,
            "Same gender nymph should return WrongGender"
        );
    }

    #[test]
    fn test_could_seduce_non_seducer() {
        let mut monster = test_monster(5);
        monster.state.invisible = false;
        monster.attacks[0] = Attack::new(AttackType::Claw, DamageType::Physical, 1, 4);

        // Not a nymph or demon seducer
        let result = could_seduce(&monster, 1, 0, true, None, false, false);
        assert_eq!(result, SeduceResult::No, "Non-seducer should not seduce");
    }

    #[test]
    fn test_could_seduce_invisible_unseen() {
        let mut monster = test_monster(5);
        monster.state.invisible = true;
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Invisible attacker, defender can't see invisible
        let result = could_seduce(&monster, 1, 0, false, None, true, false);
        assert_eq!(
            result,
            SeduceResult::No,
            "Invisible unseen attacker should not seduce"
        );
    }

    #[test]
    fn test_could_seduce_invisible_seen() {
        let mut monster = test_monster(5);
        monster.state.invisible = true;
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Invisible attacker, defender CAN see invisible
        let result = could_seduce(&monster, 1, 0, true, None, true, false);
        assert_eq!(
            result,
            SeduceResult::Yes,
            "Invisible seen attacker should seduce"
        );
    }

    #[test]
    fn test_could_seduce_demon_seducer() {
        let mut monster = test_monster(5);
        monster.state.invisible = false;
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Incubus/succubus (demon seducer)
        let result = could_seduce(&monster, 0, 1, true, None, false, true);
        assert_eq!(
            result,
            SeduceResult::Yes,
            "Demon seducer with opposite gender should seduce"
        );
    }

    #[test]
    fn test_could_seduce_demon_same_gender() {
        let mut monster = test_monster(5);
        monster.state.invisible = false;
        monster.attacks[0] = Attack::new(AttackType::Touch, DamageType::Seduce, 0, 0);

        // Incubus/succubus same gender - demons can't seduce same gender
        let result = could_seduce(&monster, 0, 0, true, None, false, true);
        assert_eq!(
            result,
            SeduceResult::No,
            "Demon seducer with same gender should not seduce"
        );
    }
}
