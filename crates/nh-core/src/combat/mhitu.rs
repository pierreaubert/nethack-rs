//! Monster attacks player combat (mhitu.c)
//!
//! Handles all combat initiated by monsters against the player.
//!
//! Main entry point is `mattacku()` which orchestrates all monster attacks.

use super::{Attack, AttackType, CombatEffect, CombatResult, DamageType};
use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId};
use crate::object::Object;
use crate::player::You;
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
pub fn wild_miss_message(attacker_name: &str, player_displaced: bool, player_invisible: bool) -> String {
    if player_displaced {
        if player_invisible {
            format!("The {} strikes at your invisible displaced image and misses!", attacker_name)
        } else {
            format!("The {} strikes at your displaced image and misses!", attacker_name)
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
    let distance = ((attacker.x - player.pos.x).abs().max((attacker.y - player.pos.y).abs())) as i32;

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
        let attack_result = process_single_attack(
            attacker,
            player,
            inventory,
            level,
            attack,
            rng,
        );

        // Accumulate results
        if attack_result.hit {
            result.any_hit = true;
            result.total_damage += attack_result.damage;

            // Add weapon swing message for weapon attacks (before hit message)
            if attack.attack_type == AttackType::Weapon {
                // Check if monster has a wielded weapon
                if let Some(weapon_idx) = attacker.wielded
                    && let Some(weapon) = attacker.inventory.get(weapon_idx) {
                    let weapon_name_str = weapon.name.as_deref().unwrap_or("weapon");
                    let is_thrust = weapon_name_str.contains("spear")
                        || weapon_name_str.contains("lance")
                        || weapon_name_str.contains("trident");
                    let display = weapon.display_name();
                    result.messages.push(weapon_swing_message(&attacker_name, &display, is_thrust));
                }
            }

            // Add hit message
            result.messages.push(hit_message(&attacker_name, attack.attack_type));

            // Add damage-specific message
            if let Some(msg) = damage_effect_message(&attacker_name, attack.damage_type) {
                result.messages.push(msg);
            }

            // Track special effects
            if let Some(effect) = attack_result.special_effect {
                result.effects.push(effect);
            }
        } else {
            // Miss message
            let near_miss = rng.one_in(2);
            result.messages.push(miss_message(&attacker_name, near_miss));
        }

        // Check for player death
        if attack_result.defender_died {
            result.player_died = true;
            result.messages.push("You die...".to_string());
            break;
        }

        // Check for attacker death (passive damage)
        if attack_result.attacker_died {
            result.monster_died = true;
            break;
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
            let (result, _msg) = monster_attack_player_full(
                attacker, player, inventory, level, attack, rng,
            );
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
    // Explosion always hits if in range, and kills the attacker
    let mut result = monster_attack_player(attacker, player, attack, rng);
    result.hit = true; // Explosions always hit
    result.attacker_died = true; // Attacker dies from explosion
    result
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
    let distance = ((attacker.x - player.pos.x).abs().max((attacker.y - player.pos.y).abs())) as u32;

    // Miss chance increases with distance
    if distance > 1 && rng.one_in(distance) {
        return CombatResult::MISS;
    }

    monster_attack_player(attacker, player, attack, rng)
}

/// Calculate monster's to-hit bonus
///
/// Based on find_roll_to_hit() in mhitu.c
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

    // Calculate to-hit
    let to_hit = calculate_monster_to_hit(attacker, player);

    // Roll to hit
    // Formula: roll + to_hit > 10 - AC means hit
    // With AC 10 (no armor), need roll + to_hit > 0 (always hits with any to_hit > -19)
    // With AC -10 (good armor), need roll + to_hit > 20 (harder to hit)
    let roll = rng.rnd(20) as i32;
    if roll + to_hit <= 10 - player.armor_class as i32 {
        return CombatResult::MISS;
    }

    // Calculate base damage
    let mut damage = rng.dice(attack.dice_num as u32, attack.dice_sides as u32) as i32;

    // Apply resistance-based damage reduction
    let (mult_num, mult_den) = damage_multiplier_for_resistance(attack.damage_type, player);
    damage = damage * mult_num / mult_den;

    // Apply special damage effects based on damage type
    let special_effect = apply_damage_effect(attack.damage_type, player, damage, rng);

    // Apply damage to player (minimum 0 after resistance)
    if damage > 0 {
        player.hp -= damage;
    }

    CombatResult {
        hit: true,
        defender_died: player.hp <= 0,
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
        if is_nymph && (obj.class == crate::object::ObjectClass::Ring 
                     || obj.class == crate::object::ObjectClass::Amulet) {
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
        let adjusted_weight = if is_nymph && (obj.class == crate::object::ObjectClass::Ring 
                                           || obj.class == crate::object::ObjectClass::Amulet) {
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
pub fn apply_grab_damage(
    player: &mut You,
    grabber: &Monster,
    rng: &mut GameRng,
) -> i32 {
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
                Some(format!("The {} stole your {}!", attacker.name, stolen.display_name()))
            } else {
                Some(format!("The {} couldn't find anything to steal.", attacker.name))
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
        _ => None,
    };

    (result, message)
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

        let attack = Attack::new(
            crate::combat::AttackType::Claw,
            DamageType::Physical,
            1,
            6,
        );

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

        let attack = Attack::new(
            crate::combat::AttackType::Claw,
            DamageType::Physical,
            1,
            6,
        );

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
        assert!(player.confused_timeout >= 10, "Should be confused for at least 10 turns");
        assert!(player.confused_timeout <= 19, "Should be confused for at most 19 turns");
    }

    #[test]
    fn test_stun_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.stunned_timeout, 0);

        let effect = apply_damage_effect(DamageType::Stun, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Stunned));
        assert!(player.stunned_timeout >= 5, "Should be stunned for at least 5 turns");
        assert!(player.stunned_timeout <= 9, "Should be stunned for at most 9 turns");
    }

    #[test]
    fn test_blind_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.blinded_timeout, 0);

        let effect = apply_damage_effect(DamageType::Blind, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Blinded));
        assert!(player.blinded_timeout >= 20, "Should be blinded for at least 20 turns");
        assert!(player.blinded_timeout <= 119, "Should be blinded for at most 119 turns");
    }

    #[test]
    fn test_paralyze_effect() {
        let mut player = test_player();
        let mut rng = GameRng::new(42);

        assert_eq!(player.paralyzed_timeout, 0);

        let effect = apply_damage_effect(DamageType::Paralyze, &mut player, 0, &mut rng);

        assert_eq!(effect, Some(CombatEffect::Paralyzed));
        assert!(player.paralyzed_timeout >= 3, "Should be paralyzed for at least 3 turns");
        assert!(player.paralyzed_timeout <= 7, "Should be paralyzed for at most 7 turns");
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
        assert_eq!(player.attr_current.get(Attribute::Strength), 15, "Should lose 1 strength");
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

        assert_eq!(effect, None, "Stone resistance should protect from petrification");
    }

    #[test]
    fn test_poison_resistance_blocks_strength_drain() {
        let mut player = test_player();
        player.attr_current.set(Attribute::Strength, 16);
        player.properties.grant_intrinsic(Property::PoisonResistance);
        let mut rng = GameRng::new(42);

        let effect = apply_damage_effect(DamageType::DrainStrength, &mut player, 0, &mut rng);

        assert_eq!(effect, None, "Poison resistance should protect from strength drain");
        assert_eq!(player.attr_current.get(Attribute::Strength), 16, "Strength should not change");
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
        player.properties.grant_intrinsic(Property::DisintResistance);
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

        assert_eq!(effect, None, "Acid resistance should protect from acid effects");
    }

    // Damage reduction tests
    #[test]
    fn test_fire_resistance_reduces_damage() {
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Fire, &test_player());
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
        assert_eq!((mult_num, mult_den), (1, 2), "Acid resistance = half damage");
    }

    #[test]
    fn test_half_physical_damage() {
        let mut player = test_player();
        player.properties.grant_intrinsic(Property::HalfPhysDamage);
        let (mult_num, mult_den) = damage_multiplier_for_resistance(DamageType::Physical, &player);
        assert_eq!((mult_num, mult_den), (1, 2), "Half physical damage property");
    }

    #[test]
    fn test_fire_attack_with_resistance() {
        let mut player = test_player();
        player.hp = 100;
        player.armor_class = 10;
        player.properties.grant_intrinsic(Property::FireResistance);
        let monster = test_monster(10);
        let mut rng = GameRng::new(42);

        let attack = Attack::new(
            crate::combat::AttackType::Breath,
            DamageType::Fire,
            3,
            6,
        );

        let result = monster_attack_player(&monster, &mut player, &attack, &mut rng);

        assert!(result.hit, "Should still hit");
        assert_eq!(result.damage, 0, "Fire damage should be reduced to 0");
        assert_eq!(player.hp, 100, "HP should not change with fire resistance");
    }

    // ========================================================================
    // Tests for message functions
    // ========================================================================

    #[test]
    fn test_hit_message() {
        assert_eq!(hit_message("goblin", AttackType::Bite), "The goblin bites!");
        assert_eq!(hit_message("troll", AttackType::Claw), "The troll claws!");
        assert_eq!(hit_message("dragon", AttackType::Breath), "The dragon breathes on you!");
        assert_eq!(hit_message("soldier", AttackType::Weapon), "The soldier hits!");
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
        assert!(result.any_hit || !result.messages.is_empty(), "Should have attempted attack");
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
        assert!(result.messages.len() >= 2, "Should process multiple attacks");
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
        assert!(!result.any_hit, "Melee attack should not reach distant player");
        assert!(result.messages.is_empty(), "No messages for out-of-range attack");
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
            assert!(has_swing_msg, "Should have weapon swing message for weapon attack");
        }
    }
}
