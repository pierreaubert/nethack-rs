//! Throwing objects (dothrow.c)
//!
//! Handles all projectile mechanics: throwing, firing from quiver,
//! multishot, boomerangs, potion shattering, gem/unicorn interactions,
//! object breakage, cockatrice eggs, monster throwing, and wand breaking.

use crate::action::{ActionResult, Direction};
use crate::combat::CombatEffect;
use crate::gameloop::GameState;
use crate::monster::{Monster, MonsterId};
use crate::object::{BucStatus, Material, Object, ObjectClass};
use crate::player::{Attribute, Encumbrance, Race, Role, SkillLevel, SkillType, You};
use crate::rng::GameRng;

// ============================================================================
// Throw range calculation
// ============================================================================

/// Calculate the maximum throw range for an object.
///
/// Based on C dothrow.c throwit() range calculation.
/// Range = STR/2 - weight/40 for most objects, capped at [1,10].
/// Special cases: iron balls use weight/100, boulders get 20.
pub fn throw_range(player: &You, obj: &Object, ammo_with_launcher: bool) -> i32 {
    let str_val = player.attr_current.get(Attribute::Strength) as i32;
    let urange = str_val / 2;

    let weight = obj.weight as i32;
    let mut range = if obj.class == ObjectClass::Ball {
        urange - (weight / 100)
    } else {
        urange - (weight / 40)
    };

    // Ammo with matching launcher gets extended range
    if ammo_with_launcher {
        range += 1;
    }

    // Ammo without launcher or non-weapon halves range
    if !ammo_with_launcher && obj.class != ObjectClass::Gem {
        range /= 2;
    }

    range.clamp(1, 10)
}

// ============================================================================
// To-hit calculation for thrown objects
// ============================================================================

/// Calculate the to-hit bonus for a thrown projectile.
///
/// Based on C uhitm.c thitmonst() — uses dexterity instead of strength,
/// applies distance penalty, monster size bonus, and weapon skill.
pub fn throw_to_hit(
    player: &You,
    target: &Monster,
    obj: &Object,
    distance: i32,
    ammo_with_launcher: bool,
) -> i32 {
    let mut tmp: i32 = -1;
    tmp += player.luck as i32;
    tmp += target.ac as i32; // find_mac equivalent
    tmp += player.hit_bonus as i32;
    tmp += player.exp_level;

    // Dexterity adjustments (not strength — thrown objects use dex)
    let dex = player.attr_current.get(Attribute::Dexterity) as i32;
    if dex < 4 {
        tmp -= 3;
    } else if dex < 6 {
        tmp -= 2;
    } else if dex < 8 {
        tmp -= 1;
    } else if dex >= 14 {
        tmp += dex - 14;
    }

    // Distance penalty
    let disttmp = (3 - distance).max(-4);
    tmp += disttmp;

    // Monster size bonus
    // MonsterSize: Tiny=0, Small=1, Medium=2, Large=3, Huge=4, Gigantic=7
    // Bonus = size - 2 (medium is baseline)
    tmp += target.level as i32 / 3; // Proxy for size

    // Sleeping/immobile bonus
    if target.state.sleeping {
        tmp += 2;
    }
    if target.frozen_timeout > 0 {
        tmp += 4;
    }

    // Weapon type adjustments
    if ammo_with_launcher {
        // Matched ammo+launcher: use launcher enchantment
        tmp += obj.weapon_tohit as i32;
    } else if is_throwing_weapon(obj) {
        tmp += 2;
    } else if obj.class != ObjectClass::Weapon && obj.class != ObjectClass::Gem {
        tmp -= 2; // Non-weapon penalty
    }

    // Enchantment bonus
    tmp += obj.enchantment.max(0) as i32;

    tmp
}

// ============================================================================
// Multishot calculation
// ============================================================================

/// Calculate how many shots the player fires per throw action.
///
/// Based on C dothrow.c throw_obj() multishot logic.
/// Returns 1 for single shot, 2+ for multishot (capped by quantity).
pub fn multishot_count(
    player: &You,
    obj: &Object,
    ammo_with_launcher: bool,
    rng: &mut GameRng,
) -> i32 {
    // Must have quantity > 1 and proper ammo/weapon type
    if obj.quantity <= 1 {
        return 1;
    }

    // Must be ammo+launcher or stackable missile weapon
    let is_stackable_missile = matches!(
        obj.class,
        ObjectClass::Weapon | ObjectClass::Gem
    ) && obj.quantity > 1;

    if !ammo_with_launcher && !is_stackable_missile {
        return 1;
    }

    let mut multishot: i32 = 1;

    // Determine if the player has a "weak multishot" penalty
    let weak = matches!(
        player.role,
        Role::Wizard | Role::Priest | Role::Healer
    ) || (player.role == Role::Tourist)
        || player.attr_current.get(Attribute::Dexterity) <= 6;

    // Weapon skill bonuses
    let skill_level = weapon_skill_for_throw(player, obj);
    match skill_level {
        SkillLevel::Expert => {
            multishot += 1;
            if !weak {
                multishot += 1;
            }
        }
        SkillLevel::Skilled => {
            if !weak {
                multishot += 1;
            }
        }
        _ => {}
    }

    // Role bonuses
    match player.role {
        Role::Caveman => {
            // Sling or spear bonus
            multishot += 1;
        }
        Role::Monk => {
            // Shuriken bonus
            multishot += 1;
        }
        Role::Ranger => {
            // General ranged bonus (not dagger)
            multishot += 1;
        }
        Role::Rogue => {
            // Dagger bonus
            multishot += 1;
        }
        Role::Samurai => {
            // Ya + yumi bonus
            if ammo_with_launcher {
                multishot += 1;
            }
        }
        _ => {}
    }

    // Race bonuses (only if not weak)
    if !weak {
        match player.race {
            Race::Elf => {
                if ammo_with_launcher {
                    multishot += 1;
                }
            }
            Race::Orc => {
                if ammo_with_launcher {
                    multishot += 1;
                }
            }
            Race::Gnome => {
                // Crossbow bonus
                multishot += 1;
            }
            _ => {}
        }
    }

    // Randomize: multishot = rnd(multishot)
    multishot = rng.rnd(multishot as u32) as i32;

    // Cap to available quantity
    multishot = multishot.min(obj.quantity);

    multishot.max(1)
}

/// Get the weapon skill level relevant for a thrown object.
fn weapon_skill_for_throw(player: &You, obj: &Object) -> SkillLevel {
    // Map object class/type to a skill type
    let skill_type = match obj.class {
        ObjectClass::Weapon => {
            // Use object's inherent skill type (simplified)
            if obj.damage_sides <= 3 {
                SkillType::Dart
            } else {
                SkillType::Spear
            }
        }
        ObjectClass::Gem | ObjectClass::Rock => SkillType::Sling,
        _ => return SkillLevel::Unskilled,
    };

    player.skills.get(skill_type).level
}

// ============================================================================
// Object classification helpers
// ============================================================================

/// Check if an object is designed to be thrown (missile weapon).
///
/// Based on C throwing_weapon(): spears, daggers, knives, war hammers, aklys.
pub fn is_throwing_weapon(obj: &Object) -> bool {
    // Simplified: weapons with low damage dice tend to be missiles
    // In full implementation, check specific object types
    obj.class == ObjectClass::Weapon && obj.weight < 40
}

/// Check if an object will break on impact.
///
/// Based on C breaktest(): glass, potions, eggs, pies, venoms, camera.
pub fn breaktest(obj: &Object) -> bool {
    match obj.class {
        ObjectClass::Potion => true,
        ObjectClass::Food => {
            // Eggs and cream pies break
            obj.object_type >= 0 // Simplified: all thrown food can break
                && obj.weight <= 10 // Light food items (eggs, pies)
        }
        ObjectClass::Venom => true,
        _ => false,
    }
}

/// Check if a monster is a unicorn (symbol 'u' in C).
///
/// Unicorns have special gem-catching behavior.
pub fn is_unicorn(monster: &Monster) -> bool {
    // In C, unicorns are identified by S_UNICORN class ('u')
    // In our system, check monster flags or type range
    // Monster types for unicorns in NetHack: white/gray/black unicorn
    // Simplified: check the name
    monster.name.contains("unicorn")
}

// ============================================================================
// Potion hit effects
// ============================================================================

/// Result of a potion shattering on a monster.
#[derive(Debug, Clone)]
pub struct PotionHitResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the target was affected
    pub affected: bool,
    /// Effects applied to target
    pub effects: Vec<CombatEffect>,
    /// Damage dealt
    pub damage: i32,
    /// Whether the monster died
    pub monster_died: bool,
}

/// Apply potion shatter effects to a monster.
///
/// Based on C potion.c potionhit(). Each potion type has a unique
/// effect when shattered on a monster.
pub fn potionhit(
    target: &mut Monster,
    potion: &Object,
    rng: &mut GameRng,
) -> PotionHitResult {
    let mut result = PotionHitResult {
        messages: Vec::new(),
        affected: false,
        effects: Vec::new(),
        damage: 1, // Base shatter damage
        monster_died: false,
    };

    result.messages.push(format!(
        "The potion shatters on {}!",
        target.name
    ));

    // Apply potion-specific effects based on object_type
    // Potion types: 257=GainAbility..282=Water (from PotionType enum)
    match potion.object_type {
        259 => {
            // Confusion
            if target.confused_timeout == 0 {
                result.messages.push(format!("{} looks confused.", target.name));
            }
            target.confused_timeout += rng.rnd(12) as u16 + 4;
            result.affected = true;
            result.effects.push(CombatEffect::Confused);
        }
        260 => {
            // Blindness
            if target.blinded_timeout == 0 {
                result.messages.push(format!("{} is blinded!", target.name));
            }
            target.blinded_timeout += rng.rnd(25) as u16 + 20;
            result.affected = true;
            result.effects.push(CombatEffect::Blinded);
        }
        261 => {
            // Paralysis
            if !target.resists_sleep() {
                target.frozen_timeout += rng.rnd(10) as u16 + 5;
                result.messages.push(format!("{} is frozen!", target.name));
                result.affected = true;
                result.effects.push(CombatEffect::Paralyzed);
            } else {
                result.messages.push(format!("{} resists.", target.name));
            }
        }
        262 => {
            // Speed
            if target.speed == crate::monster::SpeedState::Normal {
                target.speed = crate::monster::SpeedState::Fast;
                result.messages.push(format!("{} speeds up!", target.name));
                result.affected = true;
            }
        }
        267 => {
            // Healing - heals monster!
            let heal = rng.dice(6, 4) as i32;
            target.hp = (target.hp + heal).min(target.hp_max);
            result.messages.push(format!("{} looks better.", target.name));
            result.affected = true;
            result.damage = 0; // No damage
        }
        268 => {
            // Extra healing
            let heal = rng.dice(6, 8) as i32;
            target.hp = (target.hp + heal).min(target.hp_max);
            result.messages.push(format!("{} looks much better.", target.name));
            result.affected = true;
            result.damage = 0;
        }
        274 => {
            // Sleeping
            if !target.resists_sleep() {
                target.state.sleeping = true;
                target.sleep_timeout = rng.rnd(12) as u16 + 5;
                result.messages.push(format!("{} falls asleep.", target.name));
                result.affected = true;
            } else {
                result.messages.push(format!("{} resists.", target.name));
            }
        }
        275 => {
            // Full healing
            target.hp = target.hp_max;
            result.messages.push(format!("{} looks completely healed!", target.name));
            result.affected = true;
            result.damage = 0;
        }
        280 => {
            // Acid
            let acid_dmg = rng.dice(3, 6) as i32;
            if target.resists_acid() {
                result.messages.push(format!("{} is not affected.", target.name));
            } else {
                result.damage += acid_dmg;
                result.messages.push(format!("{} is burned by acid!", target.name));
                result.affected = true;
            }
        }
        282 => {
            // Holy/unholy water
            if potion.buc == BucStatus::Blessed {
                // Holy water damages undead/demons
                if target.is_undead() || target.is_demon() {
                    let holy_dmg = rng.dice(2, 6) as i32;
                    result.damage += holy_dmg;
                    result.messages.push(format!(
                        "{} is burned by the holy water!",
                        target.name
                    ));
                    result.affected = true;
                }
            } else if potion.buc == BucStatus::Cursed {
                // Unholy water damages non-undead
                if !target.is_undead() && !target.is_demon() {
                    let unholy_dmg = rng.dice(2, 6) as i32;
                    result.damage += unholy_dmg;
                    result.messages.push(format!(
                        "{} is burned by the unholy water!",
                        target.name
                    ));
                    result.affected = true;
                }
            }
        }
        _ => {
            // Other potions: just the shatter damage
        }
    }

    // Apply damage
    target.hp -= result.damage;
    if target.hp <= 0 {
        result.monster_died = true;
    }

    result
}

// ============================================================================
// Boomerang handling
// ============================================================================

/// Result of a boomerang throw.
#[derive(Debug, Clone)]
pub struct BoomerangResult {
    /// Whether the boomerang returned to the thrower
    pub returned: bool,
    /// Whether a monster was hit during flight
    pub hit_monster: Option<MonsterId>,
    /// Damage dealt to monster
    pub damage: i32,
    /// Whether the boomerang broke
    pub broke: bool,
    /// Messages
    pub messages: Vec<String>,
}

/// Handle a boomerang throw and return path.
///
/// Based on C boomhit() in dothrow.c.
/// Boomerangs travel in a curved path and return to the thrower.
/// 1/10 chance of missing the catch on return.
pub fn handle_boomerang(
    player: &You,
    level: &mut crate::dungeon::Level,
    dx: i8,
    dy: i8,
    range: i32,
    rng: &mut GameRng,
) -> BoomerangResult {
    let mut result = BoomerangResult {
        returned: true,
        hit_monster: None,
        damage: 0,
        broke: false,
        messages: Vec::new(),
    };

    // Trace the boomerang path (outward leg)
    let mut x = player.pos.x;
    let mut y = player.pos.y;

    for _ in 0..range {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) || !level.is_walkable(x, y) {
            break;
        }

        // Check for monster hit on outward path
        if let Some(monster) = level.monster_at_mut(x, y) {
            let monster_id = monster.id;
            let damage = rng.rnd(6) as i32 + 1; // Boomerang: 1d6+1
            monster.hp -= damage;
            result.hit_monster = Some(monster_id);
            result.damage = damage;
            result.messages.push(format!(
                "The boomerang hits {}!",
                monster.name
            ));
            if monster.hp <= 0 {
                result.messages.push(format!(
                    "You kill {}!",
                    monster.name
                ));
            }
            // Boomerang continues on return path even after hit
            break;
        }
    }

    // Return path: 1/10 chance of fumbling the catch
    let dex = player.attr_current.get(Attribute::Dexterity) as i32;
    if rng.rn2(10) == 0 && dex < 14 {
        result.returned = false;
        result.messages.push("You fumble and fail to catch the boomerang!".to_string());
    } else {
        result.returned = true;
        result.messages.push("You skillfully catch the boomerang.".to_string());
    }

    result
}

// ============================================================================
// Monster catching projectiles
// ============================================================================

/// Check if a monster catches a thrown projectile.
///
/// Based on C moncatchobj(). Monsters that are awake and dexterous
/// have a chance to catch thrown objects.
pub fn monster_catches(target: &Monster, _obj: &Object, rng: &mut GameRng) -> bool {
    // Sleeping or frozen monsters can't catch
    if target.state.sleeping || target.frozen_timeout > 0 {
        return false;
    }

    // Only shopkeepers and high-level monsters catch
    if target.is_shopkeeper {
        // Shopkeepers always catch
        return true;
    }

    // High-level monsters have a chance
    if target.level >= 10 && rng.rn2(3) == 0 {
        return true;
    }

    false
}

// ============================================================================
// Gem/unicorn interaction
// ============================================================================

/// Result of a gem being thrown at a unicorn.
#[derive(Debug, Clone)]
pub struct GemAcceptResult {
    /// Messages
    pub messages: Vec<String>,
    /// Change in player luck
    pub luck_change: i32,
    /// Whether the unicorn became peaceful
    pub became_peaceful: bool,
    /// Whether the unicorn picked up the gem
    pub gem_taken: bool,
    /// Whether the unicorn teleported away
    pub teleported: bool,
}

/// Handle a unicorn catching a gem.
///
/// Based on C gem_accept() in dothrow.c.
/// Luck changes depend on gem quality, identification, and alignment match.
pub fn gem_accept(
    target: &mut Monster,
    obj: &Object,
    obj_material: Material,
    player_alignment: i8,
    gem_identified: bool,
    rng: &mut GameRng,
) -> GemAcceptResult {
    let mut result = GemAcceptResult {
        messages: Vec::new(),
        luck_change: 0,
        became_peaceful: false,
        gem_taken: true,
        teleported: false,
    };

    let is_buddy = target.alignment == player_alignment;
    let is_real_gem = obj_material == Material::Gemstone;

    // Make the unicorn peaceful
    if !target.is_peaceful() {
        target.state.peaceful = true;
        result.became_peaceful = true;
    }

    if gem_identified {
        // Known gem quality
        if is_real_gem {
            if is_buddy {
                result.luck_change = 5;
                result.messages.push("You feel exceptionally lucky!".to_string());
            } else {
                result.luck_change = rng.rn2(7) as i32 - 3; // -3 to +3
            }
        } else {
            // Fake gem (glass, worthless)
            result.messages.push(format!(
                "{} is not interested in your junk.",
                target.name
            ));
            result.gem_taken = false;
        }
    } else {
        // Unknown gem
        if is_real_gem {
            if is_buddy {
                result.luck_change = 1;
            } else {
                result.luck_change = rng.rn2(3) as i32 - 1; // -1 to +1
            }
        } else {
            // Fake but unknown: unicorn graciously accepts
            result.messages.push(format!(
                "{} graciously accepts your gift.",
                target.name
            ));
        }
    }

    if result.gem_taken {
        // Add gem to unicorn inventory
        target.inventory.push(obj.clone());
        result.messages.push(format!(
            "{} catches the gem.",
            target.name
        ));
    }

    // Unicorn may teleport away after accepting
    if result.gem_taken && rng.rn2(3) == 0 {
        result.teleported = true;
        result.messages.push(format!(
            "{} vanishes!",
            target.name
        ));
    }

    result
}

// ============================================================================
// Egg throwing (cockatrice petrification)
// ============================================================================

/// Result of a thrown egg hitting something.
#[derive(Debug, Clone)]
pub struct EggHitResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether the target was petrified
    pub petrified: bool,
    /// Whether the egg was consumed
    pub consumed: bool,
    /// Damage dealt
    pub damage: i32,
}

/// Handle a thrown egg hitting a target.
///
/// Based on C thitmonst() egg handling in dothrow.c.
/// Cockatrice eggs can petrify the target if they don't resist.
pub fn throw_egg(
    target: &mut Monster,
    is_cockatrice_egg: bool,
    _rng: &mut GameRng,
) -> EggHitResult {
    let mut result = EggHitResult {
        messages: Vec::new(),
        petrified: false,
        consumed: true,
        damage: 1,
    };

    result.messages.push(format!(
        "The egg splatters on {}!",
        target.name
    ));

    if is_cockatrice_egg {
        // Petrification check
        if target.resists_stone() {
            result.messages.push(format!(
                "{} is not affected.",
                target.name
            ));
        } else {
            // Monster turns to stone
            result.petrified = true;
            result.damage = target.hp + 200; // Fatal
            result.messages.push(format!(
                "{} turns to stone!",
                target.name
            ));
        }
    }

    // Apply damage
    target.hp -= result.damage;

    result
}

// ============================================================================
// Wand breaking effects
// ============================================================================

/// Result of a wand breaking (thrown and shattered, or snapped).
#[derive(Debug, Clone)]
pub struct WandBreakResult {
    /// Messages
    pub messages: Vec<String>,
    /// Damage radius (in squares)
    pub radius: i32,
    /// Damage at center
    pub center_damage: i32,
    /// Whether this is an explosive break (fire/lightning/etc.)
    pub explosive: bool,
    /// Monsters affected (id, damage)
    pub affected_monsters: Vec<(MonsterId, i32)>,
}

/// Calculate the effect of a wand breaking/exploding.
///
/// Based on C breakobj() and wand break logic from zap.c.
/// Wand damage = 4 * charges * (number_of_charges + 1).
pub fn wand_break_effect(
    wand: &Object,
    x: i8,
    y: i8,
    level: &crate::dungeon::Level,
    rng: &mut GameRng,
) -> WandBreakResult {
    let mut result = WandBreakResult {
        messages: Vec::new(),
        radius: 1,
        center_damage: 0,
        explosive: false,
        affected_monsters: Vec::new(),
    };

    let charges = wand.enchantment.max(0) as i32;
    if charges == 0 {
        result.messages.push("The wand breaks with a fizzle.".to_string());
        return result;
    }

    // Base damage scales with charges
    let base_damage = 4 * charges * (charges + 1);
    result.center_damage = base_damage;
    result.explosive = true;
    result.radius = if charges >= 4 { 2 } else { 1 };

    result.messages.push("The wand explodes!".to_string());

    // Find monsters within blast radius
    for dx in -result.radius..=result.radius {
        for dy in -result.radius..=result.radius {
            let mx = x + dx as i8;
            let my = y + dy as i8;
            if !level.is_valid_pos(mx, my) {
                continue;
            }
            if let Some(monster) = level.monster_at(mx, my) {
                // Damage falls off with distance
                let dist = (dx.abs() + dy.abs()).max(1);
                let dmg = base_damage / dist;
                // Randomize a bit
                let final_dmg = (rng.rnd(dmg.max(1) as u32) as i32).max(1);
                result.affected_monsters.push((monster.id, final_dmg));
            }
        }
    }

    result
}

// ============================================================================
// Monster throwing at player
// ============================================================================

/// Result of a monster throwing at the player.
#[derive(Debug, Clone)]
pub struct MonsterThrowResult {
    /// Messages
    pub messages: Vec<String>,
    /// Whether the player was hit
    pub hit: bool,
    /// Damage dealt to player
    pub player_damage: i32,
    /// Effects applied to player
    pub effects: Vec<CombatEffect>,
    /// Object that was thrown (if lands on ground)
    pub object_landed: Option<(i8, i8)>,
}

/// Handle a monster throwing an object at the player.
///
/// Based on C m_throw() in mthrowu.c.
/// Monsters throw projectiles with to-hit based on their level.
#[allow(clippy::too_many_arguments)]
pub fn monster_throw(
    thrower: &Monster,
    obj: &Object,
    player: &You,
    player_ac: i8,
    dx: i8,
    dy: i8,
    _max_range: i32,
    rng: &mut GameRng,
) -> MonsterThrowResult {
    let mut result = MonsterThrowResult {
        messages: Vec::new(),
        hit: false,
        player_damage: 0,
        effects: Vec::new(),
        object_landed: None,
    };

    let obj_name = obj.name.clone().unwrap_or_else(|| "something".to_string());

    // To-hit: monster level + 1d20 vs player AC
    let to_hit = thrower.level as i32 + 5; // Base to-hit
    let roll = rng.rnd(20) as i32;
    let target_ac = player_ac as i32;

    // Check if projectile reaches the player
    // (simplified: assume monster is within range)
    if roll + to_hit >= 10 + target_ac {
        result.hit = true;

        // Calculate damage
        let damage = match obj.class {
            ObjectClass::Weapon => {
                let dice = obj.damage_dice.max(1);
                let sides = obj.damage_sides.max(4);
                rng.dice(dice as u32, sides as u32) as i32
            }
            ObjectClass::Gem | ObjectClass::Rock => rng.rnd(3) as i32,
            ObjectClass::Potion => 1,
            _ => rng.rnd(2) as i32,
        };

        result.player_damage = damage + obj.enchantment.max(0) as i32;
        result.messages.push(format!(
            "{} throws {} and hits you!",
            thrower.name, obj_name
        ));

        // Special effects from projectile type
        if obj.poisoned
            && !player.properties.has(crate::player::Property::PoisonResistance)
        {
            result.effects.push(CombatEffect::Poisoned);
            result.player_damage += rng.rnd(6) as i32;
            result.messages.push("The projectile was poisoned!".to_string());
        }
    } else {
        result.messages.push(format!(
            "{} throws {} and misses you.",
            thrower.name, obj_name
        ));
        // Object lands near the player
        let land_x = player.pos.x + dx;
        let land_y = player.pos.y + dy;
        result.object_landed = Some((land_x, land_y));
    }

    result
}

// ============================================================================
// Projectile path tracing
// ============================================================================

/// Result of tracing a projectile path through the level.
#[derive(Debug, Clone)]
pub struct ProjectilePath {
    /// Each cell the projectile passed through
    pub cells: Vec<(i8, i8)>,
    /// Final resting position
    pub end_x: i8,
    pub end_y: i8,
    /// Monster hit (if any)
    pub hit_monster: Option<MonsterId>,
    /// Whether the projectile hit a wall
    pub hit_wall: bool,
}

/// Trace a projectile's path through the level.
///
/// Returns each cell traversed and the final position.
/// Used by throw, fire, and boomerang paths.
pub fn trace_projectile(
    level: &crate::dungeon::Level,
    start_x: i8,
    start_y: i8,
    dx: i8,
    dy: i8,
    range: i32,
) -> ProjectilePath {
    let mut path = ProjectilePath {
        cells: Vec::new(),
        end_x: start_x,
        end_y: start_y,
        hit_monster: None,
        hit_wall: false,
    };

    let mut x = start_x;
    let mut y = start_y;

    for _ in 0..range {
        x += dx;
        y += dy;

        if !level.is_valid_pos(x, y) {
            path.hit_wall = true;
            break;
        }

        if !level.is_walkable(x, y) {
            path.hit_wall = true;
            break;
        }

        path.cells.push((x, y));
        path.end_x = x;
        path.end_y = y;

        // Check for monster at this position
        if let Some(monster) = level.monster_at(x, y) {
            path.hit_monster = Some(monster.id);
            break;
        }
    }

    path
}

// ============================================================================
// Returning weapons (Mjollnir, aklys)
// ============================================================================

/// Result of a returning weapon check.
#[derive(Debug, Clone)]
pub enum ReturnResult {
    /// Weapon returns normally to hand
    Returned,
    /// Weapon fumbled on return, hits thrower for damage
    Fumbled(i32),
    /// Weapon fails to return, drops at impact
    Failed,
}

/// Check if a returning weapon (Mjollnir/aklys) returns to the thrower.
///
/// Based on C dothrow.c throwit() returning missile logic.
/// 99/100: normal return. 1/100: fails. 2/100: fumbled return with self-damage.
pub fn returning_weapon_check(rng: &mut GameRng) -> ReturnResult {
    let roll = rng.rn2(100);
    if roll == 0 {
        ReturnResult::Failed
    } else if roll <= 2 {
        let self_damage = rng.rnd(2) as i32;
        ReturnResult::Fumbled(self_damage)
    } else {
        ReturnResult::Returned
    }
}

// ============================================================================
// Slip chance (cursed/greased)
// ============================================================================

/// Check if a thrown object slips and goes in a random direction.
///
/// Based on C throwit() slip check: cursed+greased weapons have 1/7 chance.
pub fn check_throw_slip(obj: &Object, rng: &mut GameRng) -> Option<Direction> {
    let slippery = obj.buc == BucStatus::Cursed || obj.greased;
    if slippery && rng.rn2(7) == 0 {
        // Pick a random direction
        let dirs = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
            Direction::NorthEast,
            Direction::NorthWest,
            Direction::SouthEast,
            Direction::SouthWest,
        ];
        let idx = rng.rn2(8) as usize;
        Some(dirs[idx])
    } else {
        None
    }
}

// ============================================================================
// Encumbrance check (heavy throw)
// ============================================================================

/// Check if the player is too encumbered to throw effectively.
///
/// Based on C throwit() stamina check: heavily encumbered + low HP
/// + heavy object = drop instead of throw.
pub fn too_encumbered_to_throw(player: &You, obj: &Object) -> bool {
    let encumbered = matches!(
        player.encumbrance(),
        Encumbrance::Strained | Encumbrance::Overtaxed | Encumbrance::Overloaded
    );
    let low_hp = player.hp < player.hp_max / 4;
    let heavy = obj.weight > 100;
    encumbered && low_hp && heavy
}

// ============================================================================
// Main throw function (expanded)
// ============================================================================

/// Throw an object from inventory (expanded version).
///
/// Handles multishot, slip chance, boomerangs, gem/unicorn,
/// breakage, potion effects, and object landing.
pub fn do_throw(state: &mut GameState, obj_letter: char, direction: Direction) -> ActionResult {
    // Get the object from inventory
    let obj = match state.get_inventory_item(obj_letter) {
        Some(o) => o.clone(),
        None => return ActionResult::Failed("You don't have that item.".to_string()),
    };

    let (dx, dy) = direction.delta();

    // Can't throw at self
    if dx == 0 && dy == 0 {
        state.message("You can't throw something at yourself.");
        return ActionResult::NoTime;
    }

    // Encumbrance check
    if too_encumbered_to_throw(&state.player, &obj) {
        state.message("You are too strained to throw that.");
        return ActionResult::NoTime;
    }

    // Slip check (cursed/greased)
    let actual_direction = if let Some(slip_dir) = check_throw_slip(&obj, &mut state.rng) {
        state.message("The object slips from your grip!");
        slip_dir
    } else {
        direction
    };
    let (dx, dy) = actual_direction.delta();
    if dx == 0 && dy == 0 {
        // Slipped to self direction
        state.message("The object falls at your feet.");
        return ActionResult::Success;
    }

    // Calculate multishot
    let ammo_with_launcher = false; // Simplified: full launcher check would go here
    let shots = multishot_count(&state.player, &obj, ammo_with_launcher, &mut state.rng);

    let obj_name = obj.name.clone().unwrap_or_else(|| "object".to_string());
    let max_range = throw_range(&state.player, &obj, ammo_with_launcher);

    // Process each shot
    for shot in 0..shots {
        // Remove from inventory (or reduce quantity)
        if shot == 0 {
            if obj.quantity > shots {
                if let Some(inv_obj) = state.get_inventory_item_mut(obj_letter) {
                    inv_obj.quantity -= shots;
                }
            } else {
                state.remove_from_inventory(obj_letter);
            }
        }

        // Create the thrown projectile
        let mut projectile = obj.clone();
        projectile.quantity = 1;

        // Trace the projectile path
        let path = trace_projectile(
            &state.current_level,
            state.player.pos.x,
            state.player.pos.y,
            dx,
            dy,
            max_range,
        );

        if let Some(monster_id) = path.hit_monster {
            // Hit a monster — process hit
            let to_hit = throw_to_hit(
                &state.player,
                // We need monster ref, get it
                state.current_level.monster(monster_id).unwrap(),
                &projectile,
                path.cells.len() as i32,
                ammo_with_launcher,
            );
            let roll = state.rng.rnd(20) as i32;

            if roll <= to_hit {
                // Hit!
                if obj.class == ObjectClass::Potion {
                    // Potion shatters on hit — collect messages to avoid borrow conflict
                    let (messages, killed) = {
                        let monster = state.current_level.monster_mut(monster_id).unwrap();
                        let pot_result = potionhit(monster, &projectile, &mut state.rng);
                        let died = pot_result.monster_died || monster.hp <= 0;
                        let mut msgs = pot_result.messages;
                        if died {
                            msgs.push(format!("You kill the {}!", monster.name));
                        }
                        (msgs, died)
                    };
                    for msg in messages {
                        state.message(msg);
                    }
                    if killed {
                        state.current_level.remove_monster(monster_id);
                    }
                } else if obj.class == ObjectClass::Gem {
                    // Gem hit — check unicorn interaction
                    let damage = calculate_throw_damage(&projectile, &mut state.rng);
                    let align_val = state.player.alignment.typ.value();
                    let gem_identified = state.discoveries.contains(&obj.object_type);
                    let (messages, luck_change, killed, teleported) = {
                        let monster = state.current_level.monster_mut(monster_id).unwrap();
                        if is_unicorn(monster) {
                            let gem_result = gem_accept(
                                monster,
                                &projectile,
                                Material::Gemstone,
                                align_val,
                                gem_identified,
                                &mut state.rng,
                            );
                            (gem_result.messages, gem_result.luck_change, false, gem_result.teleported)
                        } else {
                            monster.hp -= damage;
                            let died = monster.hp <= 0;
                            let mut msgs = vec![format!("The {} hits {}!", obj_name, monster.name)];
                            if died {
                                msgs.push(format!("You kill {}!", monster.name));
                            } else if monster.state.sleeping {
                                monster.state.sleeping = false;
                                monster.sleep_timeout = 0;
                            }
                            (msgs, 0, died, false)
                        }
                    };
                    for msg in messages {
                        state.message(msg);
                    }
                    if luck_change != 0 {
                        state.player.luck = (state.player.luck as i32 + luck_change).clamp(-13, 13) as i8;
                    }
                    if killed || teleported {
                        state.current_level.remove_monster(monster_id);
                    }
                } else {
                    // Normal damage
                    let damage = calculate_throw_damage(&projectile, &mut state.rng);
                    let (messages, killed) = {
                        let monster = state.current_level.monster_mut(monster_id).unwrap();
                        monster.hp -= damage;
                        let died = monster.hp <= 0;
                        let mut msgs = vec![format!("The {} hits {}!", obj_name, monster.name)];
                        if died {
                            msgs.push(format!("You kill {}!", monster.name));
                        } else if monster.state.sleeping {
                            monster.state.sleeping = false;
                            monster.sleep_timeout = 0;
                        }
                        (msgs, died)
                    };
                    for msg in messages {
                        state.message(msg);
                    }
                    if killed {
                        state.current_level.remove_monster(monster_id);
                    }
                }
            } else {
                // Miss
                let monster_name = state.current_level.monster(monster_id)
                    .map(|m| m.name.clone())
                    .unwrap_or_else(|| "monster".to_string());
                state.message(format!("The {} misses {}.", obj_name, monster_name));
                // Object lands at monster position
                projectile.x = path.end_x;
                projectile.y = path.end_y;
                state.current_level.add_object(projectile, path.end_x, path.end_y);
            }
        } else if path.hit_wall {
            // Hit a wall
            state.message(format!("The {} hits the wall.", obj_name));

            // Check breakage
            if breaktest(&projectile) {
                state.message(format!("The {} shatters!", obj_name));
            } else {
                // Land at last valid position
                let (lx, ly) = if path.cells.is_empty() {
                    (state.player.pos.x, state.player.pos.y)
                } else {
                    *path.cells.last().unwrap()
                };
                projectile.x = lx;
                projectile.y = ly;
                state.current_level.add_object(projectile, lx, ly);
            }
        } else {
            // Traveled full range, lands on ground
            projectile.x = path.end_x;
            projectile.y = path.end_y;
            state.current_level.add_object(projectile, path.end_x, path.end_y);
            if shot == 0 {
                state.message(format!("The {} lands on the ground.", obj_name));
            }
        }
    }

    ActionResult::Success
}

/// Calculate damage for thrown object
fn calculate_throw_damage(obj: &Object, rng: &mut GameRng) -> i32 {
    let base_damage = match obj.class {
        ObjectClass::Weapon => {
            // Use weapon damage dice
            let dice = obj.damage_dice.max(1);
            let sides = obj.damage_sides.max(4);
            rng.dice(dice as u32, sides as u32) as i32
        }
        ObjectClass::Gem | ObjectClass::Rock => {
            // Rocks and gems do 1d3
            rng.dice(1, 3) as i32
        }
        ObjectClass::Potion => {
            // Potions shatter for 1 damage
            1
        }
        _ => {
            // Other objects do 1d2
            rng.dice(1, 2) as i32
        }
    };

    // Add enchantment bonus
    base_damage + obj.enchantment.max(0) as i32
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::DLevel;
    use crate::monster::{MonsterFlags, MonsterResistances};
    use crate::object::ObjectId;

    fn test_player() -> You {
        let mut player = You::default();
        player.exp_level = 1;
        player.attr_current.set(Attribute::Strength, 14);
        player.attr_current.set(Attribute::Dexterity, 12);
        player.luck = 0;
        player
    }

    fn test_monster() -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = "kobold".to_string();
        m.hp = 10;
        m.hp_max = 10;
        m.level = 1;
        m.ac = 7;
        m
    }

    fn test_weapon() -> Object {
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.damage_dice = 1;
        obj.damage_sides = 6;
        obj.weight = 30;
        obj
    }

    // ---- Throw range tests ----

    #[test]
    fn test_throw_range_basic() {
        let player = test_player();
        let obj = test_weapon();
        let range = throw_range(&player, &obj, false);
        // STR 14 / 2 = 7, weight 30/40 = 0, no launcher halves: 7/2=3
        assert!(range >= 1 && range <= 10);
    }

    #[test]
    fn test_throw_range_ammo_with_launcher() {
        let player = test_player();
        let obj = test_weapon();
        let range_without = throw_range(&player, &obj, false);
        let range_with = throw_range(&player, &obj, true);
        assert!(range_with >= range_without);
    }

    #[test]
    fn test_throw_range_heavy_object() {
        let player = test_player();
        let mut obj = test_weapon();
        obj.weight = 400;
        let range = throw_range(&player, &obj, false);
        assert_eq!(range, 1); // Clamped minimum
    }

    #[test]
    fn test_throw_range_clamped() {
        let player = test_player();
        let mut obj = test_weapon();
        obj.weight = 0;
        let range = throw_range(&player, &obj, true);
        assert!(range <= 10);
    }

    // ---- To-hit tests ----

    #[test]
    fn test_throw_to_hit_basic() {
        let player = test_player();
        let target = test_monster();
        let obj = test_weapon();
        let to_hit = throw_to_hit(&player, &target, &obj, 1, false);
        // Should be positive for a reasonable setup
        assert!(to_hit > 0);
    }

    #[test]
    fn test_throw_to_hit_distance_penalty() {
        let player = test_player();
        let target = test_monster();
        let obj = test_weapon();
        let close = throw_to_hit(&player, &target, &obj, 1, false);
        let far = throw_to_hit(&player, &target, &obj, 8, false);
        assert!(close > far);
    }

    #[test]
    fn test_throw_to_hit_sleeping_bonus() {
        let player = test_player();
        let mut target = test_monster();
        let obj = test_weapon();

        let awake_hit = throw_to_hit(&player, &target, &obj, 1, false);
        target.state.sleeping = true;
        let sleep_hit = throw_to_hit(&player, &target, &obj, 1, false);
        assert_eq!(sleep_hit, awake_hit + 2);
    }

    // ---- Multishot tests ----

    #[test]
    fn test_multishot_single_item() {
        let player = test_player();
        let obj = test_weapon();
        let mut rng = GameRng::new(42);
        assert_eq!(multishot_count(&player, &obj, false, &mut rng), 1);
    }

    #[test]
    fn test_multishot_stack() {
        let player = test_player();
        let mut obj = test_weapon();
        obj.quantity = 20;
        let mut rng = GameRng::new(42);
        let shots = multishot_count(&player, &obj, false, &mut rng);
        assert!(shots >= 1);
        assert!(shots <= 20);
    }

    #[test]
    fn test_multishot_ranger_bonus() {
        let mut player = test_player();
        player.role = Role::Ranger;
        let mut obj = test_weapon();
        obj.quantity = 20;
        let mut rng = GameRng::new(42);
        let shots = multishot_count(&player, &obj, false, &mut rng);
        // Ranger should get at least 1 shot, often more
        assert!(shots >= 1);
    }

    // ---- Breaktest tests ----

    #[test]
    fn test_breaktest_potion() {
        let obj = Object::new(ObjectId(1), 267, ObjectClass::Potion);
        assert!(breaktest(&obj));
    }

    #[test]
    fn test_breaktest_weapon() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        assert!(!breaktest(&obj));
    }

    #[test]
    fn test_breaktest_venom() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Venom);
        assert!(breaktest(&obj));
    }

    // ---- Potion hit tests ----

    #[test]
    fn test_potionhit_confusion() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let mut potion = Object::new(ObjectId(1), 259, ObjectClass::Potion);
        potion.buc = BucStatus::Uncursed;

        let result = potionhit(&mut target, &potion, &mut rng);
        assert!(result.affected);
        assert!(target.confused_timeout > 0);
    }

    #[test]
    fn test_potionhit_blindness() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let potion = Object::new(ObjectId(1), 260, ObjectClass::Potion);

        let result = potionhit(&mut target, &potion, &mut rng);
        assert!(result.affected);
        assert!(target.blinded_timeout > 0);
    }

    #[test]
    fn test_potionhit_healing_heals_monster() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.hp = 3;
        let potion = Object::new(ObjectId(1), 267, ObjectClass::Potion);

        let result = potionhit(&mut target, &potion, &mut rng);
        assert!(result.affected);
        assert_eq!(result.damage, 0);
        assert!(target.hp > 3); // Monster was healed
    }

    #[test]
    fn test_potionhit_acid_damage() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let starting_hp = target.hp;
        let potion = Object::new(ObjectId(1), 280, ObjectClass::Potion);

        let result = potionhit(&mut target, &potion, &mut rng);
        assert!(result.affected);
        assert!(result.damage > 1); // Acid does extra
        assert!(target.hp < starting_hp);
    }

    #[test]
    fn test_potionhit_holy_water_undead() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.flags = MonsterFlags::UNDEAD;
        let starting_hp = target.hp;
        let mut potion = Object::new(ObjectId(1), 282, ObjectClass::Potion);
        potion.buc = BucStatus::Blessed;

        let result = potionhit(&mut target, &potion, &mut rng);
        assert!(result.affected);
        assert!(target.hp < starting_hp);
    }

    #[test]
    fn test_potionhit_sleeping_resisted() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.resistances = MonsterResistances::SLEEP;
        let potion = Object::new(ObjectId(1), 274, ObjectClass::Potion);

        let _result = potionhit(&mut target, &potion, &mut rng);
        assert!(!target.state.sleeping);
    }

    // ---- Boomerang tests ----

    #[test]
    fn test_boomerang_return_usually() {
        let player = test_player();
        let mut level = crate::dungeon::Level::new(DLevel::main_dungeon_start());
        let mut returned_count = 0;
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let result = handle_boomerang(&player, &mut level, 1, 0, 5, &mut rng);
            if result.returned {
                returned_count += 1;
            }
        }
        // Should return most of the time
        assert!(returned_count >= 80);
    }

    // ---- Monster catching tests ----

    #[test]
    fn test_shopkeeper_always_catches() {
        let mut target = test_monster();
        target.is_shopkeeper = true;
        let obj = test_weapon();
        let mut rng = GameRng::new(42);
        assert!(monster_catches(&target, &obj, &mut rng));
    }

    #[test]
    fn test_sleeping_monster_no_catch() {
        let mut target = test_monster();
        target.state.sleeping = true;
        let obj = test_weapon();
        let mut rng = GameRng::new(42);
        assert!(!monster_catches(&target, &obj, &mut rng));
    }

    // ---- Gem/unicorn tests ----

    #[test]
    fn test_gem_accept_aligned_real_gem() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.alignment = 1;
        let gem = Object::new(ObjectId(1), 100, ObjectClass::Gem);

        let result = gem_accept(
            &mut target,
            &gem,
            Material::Gemstone,
            1, // Same alignment
            true,
            &mut rng,
        );
        assert_eq!(result.luck_change, 5);
        assert!(result.gem_taken);
    }

    #[test]
    fn test_gem_accept_fake_gem_identified() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let gem = Object::new(ObjectId(1), 100, ObjectClass::Gem);

        let result = gem_accept(
            &mut target,
            &gem,
            Material::Glass,
            0,
            true,
            &mut rng,
        );
        assert_eq!(result.luck_change, 0);
        assert!(!result.gem_taken);
    }

    #[test]
    fn test_gem_accept_makes_peaceful() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.state.peaceful = false;
        target.alignment = 0;
        let gem = Object::new(ObjectId(1), 100, ObjectClass::Gem);

        let result = gem_accept(
            &mut target,
            &gem,
            Material::Gemstone,
            0,
            false,
            &mut rng,
        );
        assert!(result.became_peaceful);
        assert!(target.is_peaceful());
    }

    // ---- Egg throw tests ----

    #[test]
    fn test_egg_normal() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let result = throw_egg(&mut target, false, &mut rng);
        assert!(!result.petrified);
        assert_eq!(result.damage, 1);
    }

    #[test]
    fn test_egg_cockatrice_petrifies() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        let result = throw_egg(&mut target, true, &mut rng);
        assert!(result.petrified);
        assert!(target.hp <= 0);
    }

    #[test]
    fn test_egg_cockatrice_resisted() {
        let mut rng = GameRng::new(42);
        let mut target = test_monster();
        target.resistances = MonsterResistances::STONE;
        let result = throw_egg(&mut target, true, &mut rng);
        assert!(!result.petrified);
    }

    // ---- Wand break tests ----

    #[test]
    fn test_wand_break_no_charges() {
        let mut rng = GameRng::new(42);
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 0;
        let level = crate::dungeon::Level::new(DLevel::main_dungeon_start());
        let result = wand_break_effect(&wand, 5, 5, &level, &mut rng);
        assert!(!result.explosive);
        assert_eq!(result.center_damage, 0);
    }

    #[test]
    fn test_wand_break_with_charges() {
        let mut rng = GameRng::new(42);
        let mut wand = Object::new(ObjectId(1), 0, ObjectClass::Wand);
        wand.enchantment = 3;
        let level = crate::dungeon::Level::new(DLevel::main_dungeon_start());
        let result = wand_break_effect(&wand, 5, 5, &level, &mut rng);
        assert!(result.explosive);
        assert_eq!(result.center_damage, 48); // 4 * 3 * 4
    }

    // ---- Monster throw tests ----

    #[test]
    fn test_monster_throw_damage() {
        let mut rng = GameRng::new(42);
        let thrower = test_monster();
        let obj = test_weapon();
        let player = test_player();

        let result = monster_throw(&thrower, &obj, &player, 10, 1, 0, 5, &mut rng);
        assert!(!result.messages.is_empty());
    }

    // ---- Returning weapon tests ----

    #[test]
    fn test_returning_weapon_mostly_returns() {
        let mut return_count = 0;
        for seed in 0..1000u64 {
            let mut rng = GameRng::new(seed);
            if matches!(returning_weapon_check(&mut rng), ReturnResult::Returned) {
                return_count += 1;
            }
        }
        // ~97% should return
        assert!(return_count >= 950);
    }

    #[test]
    fn test_returning_weapon_fail_exists() {
        let mut fail_count = 0;
        for seed in 0..10000u64 {
            let mut rng = GameRng::new(seed);
            if matches!(returning_weapon_check(&mut rng), ReturnResult::Failed) {
                fail_count += 1;
            }
        }
        assert!(fail_count > 0); // Should happen ~1%
    }

    // ---- Slip check tests ----

    #[test]
    fn test_slip_cursed_weapon() {
        let mut slip_count = 0;
        for seed in 0..700u64 {
            let mut rng = GameRng::new(seed);
            let mut obj = test_weapon();
            obj.buc = BucStatus::Cursed;
            if check_throw_slip(&obj, &mut rng).is_some() {
                slip_count += 1;
            }
        }
        // ~1/7 chance = ~100 out of 700
        assert!(slip_count > 50 && slip_count < 200);
    }

    #[test]
    fn test_slip_blessed_never() {
        for seed in 0..100u64 {
            let mut rng = GameRng::new(seed);
            let mut obj = test_weapon();
            obj.buc = BucStatus::Blessed;
            assert!(check_throw_slip(&obj, &mut rng).is_none());
        }
    }

    // ---- Encumbrance tests ----

    #[test]
    fn test_not_encumbered_normally() {
        let player = test_player();
        let obj = test_weapon();
        assert!(!too_encumbered_to_throw(&player, &obj));
    }

    // ---- Projectile path tests ----

    #[test]
    fn test_trace_projectile_empty_level() {
        let level = crate::dungeon::Level::new(DLevel::main_dungeon_start());
        let path = trace_projectile(&level, 5, 5, 1, 0, 5);
        // Depends on level geometry; just check structure
        assert!(path.cells.len() <= 5);
        assert!(path.hit_monster.is_none());
    }

    // ---- Throw damage tests ----

    #[test]
    fn test_throw_damage_weapon() {
        let mut rng = GameRng::new(12345);
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.damage_dice = 2;
        obj.damage_sides = 6;
        obj.enchantment = 1;

        let damage = calculate_throw_damage(&obj, &mut rng);
        assert!(damage >= 3 && damage <= 13); // 2d6 + 1
    }

    #[test]
    fn test_throw_damage_rock() {
        let mut rng = GameRng::new(12345);
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Rock);

        let damage = calculate_throw_damage(&obj, &mut rng);
        assert!(damage >= 1 && damage <= 3); // 1d3
    }

    #[test]
    fn test_throw_damage_enchanted_weapon() {
        let mut rng = GameRng::new(42);
        let mut obj = Object::new(ObjectId(1), 0, ObjectClass::Weapon);
        obj.damage_dice = 1;
        obj.damage_sides = 4;
        obj.enchantment = 3;

        let damage = calculate_throw_damage(&obj, &mut rng);
        assert!(damage >= 4 && damage <= 7); // 1d4 + 3
    }

    // ---- Unicorn check tests ----

    #[test]
    fn test_is_unicorn() {
        let mut m = test_monster();
        m.name = "white unicorn".to_string();
        assert!(is_unicorn(&m));
    }

    #[test]
    fn test_is_not_unicorn() {
        let m = test_monster();
        assert!(!is_unicorn(&m));
    }

    // ---- Is throwing weapon tests ----

    #[test]
    fn test_is_throwing_weapon_light() {
        let mut obj = test_weapon();
        obj.weight = 20;
        assert!(is_throwing_weapon(&obj));
    }

    #[test]
    fn test_is_throwing_weapon_heavy() {
        let mut obj = test_weapon();
        obj.weight = 100;
        assert!(!is_throwing_weapon(&obj));
    }

    #[test]
    fn test_not_throwing_weapon_potion() {
        let obj = Object::new(ObjectId(1), 0, ObjectClass::Potion);
        assert!(!is_throwing_weapon(&obj));
    }
}
