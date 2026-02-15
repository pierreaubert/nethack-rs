//! Pet/dog handling (dog.c)
//!
//! Handles tame monster behavior, feeding, and pet AI.
//!
//! Core systems:
//! - Pet creation and initialization (newedog, initedog, makedog)
//! - Taming mechanics (tamedog, wary_dog)
//! - Feeding and nutrition (dogfood, dog_eat, dog_nutrition)
//! - Pet tracking and management (keepdogs, losedogs)
//! - Pet behavior and abuse (abuse_dog, mon_catchup_elapsed_time)

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId, MonsterState};
use crate::object::{Object, ObjectClass, ObjectId};
use crate::player::{Alignment, You};
use crate::rng::GameRng;

/// Food quality for pets (maps to C dogfood_types enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DogfoodType {
    /// Best food for this pet
    DogFood = 0,
    /// Corpse (acceptable for carnivores)
    Cadaver = 1,
    /// Acceptable food
    AcceptableFood = 2,
    /// Wrong diet for this species
    ManFood = 3,
    /// Object to carry (apport training)
    Apport = 4,
    /// Poisonous to this pet
    Poison = 5,
    /// Unknown food type
    Unknown = 6,
    /// Forbidden to eat
    Taboo = 7,
}

/// Pet hunger levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PetHunger {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
}

/// Extended pet data structure (mirrors C edog)
/// Stores additional state for tamed monsters
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PetExtension {
    /// Time when last item was dropped
    pub drop_time: u32,
    /// Distance from player when item was dropped
    pub drop_distance: u32,
    /// Apport training level (0-10+, higher = more likely to fetch)
    pub apport_level: i32,
    /// Last time pet was called with whistle
    pub whistle_time: u32,
    /// Game turn when pet will get hungry
    pub hunger_time: u32,
    /// Previous goal location (used for AI pathfinding)
    pub prev_goal: (i8, i8),
    /// Abuse counter (increases when player harms pet)
    pub abuse_count: i32,
    /// Number of times pet has been revived
    pub revival_count: i32,
    /// HP penalty from starvation
    pub starvation_penalty: i32,
    /// True if player killed this pet (affects resurrection behavior)
    pub killed_by_player: bool,
}

impl PetHunger {
    /// Get hunger level from nutrition value
    pub fn from_nutrition(nutrition: i32, max_nutrition: i32) -> Self {
        let percent = (nutrition * 100) / max_nutrition.max(1);
        if percent > 150 {
            PetHunger::Satiated
        } else if percent > 50 {
            PetHunger::NotHungry
        } else if percent > 25 {
            PetHunger::Hungry
        } else if percent > 10 {
            PetHunger::Weak
        } else {
            PetHunger::Fainting
        }
    }
}

impl PetExtension {
    /// Create a new pet extension with default values
    pub fn new() -> Self {
        Self {
            drop_time: 0,
            drop_distance: 10000,
            apport_level: 0,
            whistle_time: 0,
            hunger_time: 1000,
            prev_goal: (-1, -1),
            abuse_count: 0,
            revival_count: 0,
            starvation_penalty: 0,
            killed_by_player: false,
        }
    }

    /// Reset pet goal to invalid state
    pub fn clear_goal(&mut self) {
        self.prev_goal = (-1, -1);
    }

    /// Check if goal is valid
    pub fn has_valid_goal(&self) -> bool {
        self.prev_goal != (-1, -1)
    }
}

/// Check if a monster is a valid pet (newedog/is_pet check)
pub fn is_pet(monster: &Monster) -> bool {
    monster.state.tame
}

/// Get the pet extension if monster is a pet, None otherwise
pub fn get_pet_ext(monster: &Monster) -> Option<&PetExtension> {
    if monster.state.tame {
        monster.pet_extension.as_ref()
    } else {
        None
    }
}

/// Get the mutable pet extension if monster is a pet
pub fn get_pet_ext_mut(monster: &mut Monster) -> Option<&mut PetExtension> {
    if monster.state.tame {
        monster.pet_extension.as_mut()
    } else {
        None
    }
}

/// Initialize or update the pet extension for a newly tamed monster
/// Equivalent to newedog() in C - creates the edog extension structure
pub fn create_pet_extension(monster: &mut Monster) {
    if monster.pet_extension.is_none() {
        monster.pet_extension = Some(PetExtension::new());
    }
}

/// Free the pet extension when pet becomes untamed
/// Equivalent to free_edog() in C
pub fn free_pet_extension(monster: &mut Monster) {
    monster.pet_extension = None;
    monster.state.tame = false;
}

/// Initialize pet after taming (initedog equivalent)
/// Sets up all pet-specific fields and state after a monster is first tamed
pub fn initialize_pet(monster: &mut Monster, player: &You, game_turn: u32) {
    // Determine tameness level (C uses mtame field)
    // Domestic animals start at 10, others at 5
    let is_domestic = matches!(
        monster.name.as_str().to_lowercase().as_str(),
        "little dog" | "kitten" | "pony" | "horse"
    );
    monster.tameness = if is_domestic { 10 } else { 5 };

    // Make pet peaceful and non-vengeful
    monster.state.peaceful = true;
    monster.state.fleeing = false;

    // Recalculate alignment based on being tamed
    update_pet_alignment(monster, player);

    // Initialize pet extension
    if let Some(ext) = get_pet_ext_mut(monster) {
        ext.drop_time = 0;
        ext.drop_distance = 10000;
        ext.apport_level = player.attr_current.get(crate::player::Attribute::Charisma) as i32;
        ext.whistle_time = 0;
        ext.hunger_time = game_turn + 1000;
        ext.prev_goal = (-1, -1);
        ext.abuse_count = 0;
        ext.revival_count = 0;
        ext.starvation_penalty = 0;
        ext.killed_by_player = false;
    }
}

/// Update pet alignment to match current situation
/// Simplified version of C's set_malign() for pets
fn update_pet_alignment(monster: &mut Monster, player: &You) {
    // Pet's alignment shifts toward player's alignment
    // This affects how the pet acts and interacts with alignment-based items
    if let Some(_ext) = get_pet_ext_mut(monster) {
        // Adjust based on player alignment
        monster.alignment = player.alignment.typ.value();
    }
}

/// Make a monster tame (become a pet)
pub fn tame_monster(monster: &mut Monster, tameness_level: i32) {
    create_pet_extension(monster);
    monster.tameness = tameness_level.max(1) as i8;
    monster.state.tame = true;
    monster.state.peaceful = true;
    monster.state.fleeing = false;
}

/// Untame a pet (goes wild) - removes pet status
/// Equivalent to untame operation in C, with optional unleashing
pub fn untame_monster(monster: &mut Monster) {
    monster.tameness = 0;
    monster.state.tame = false;
    free_pet_extension(monster);
}

/// Determine food quality for a pet (dogfood equivalent)
/// Returns food quality enum indicating how much pet likes this food
pub fn food_quality(pet: &Monster, food: &Object) -> DogfoodType {
    // This is a simplified version - the C code has very complex logic
    match food.class {
        ObjectClass::Food => {
            // Check if it's food
            match pet.name.as_str().to_lowercase().as_str() {
                name if name.contains("dog") || name.contains("wolf") => {
                    // Dogs prefer meat
                    if food.object_type >= 100 && food.object_type <= 120 {
                        DogfoodType::DogFood
                    } else if food.object_type >= 50 && food.object_type < 100 {
                        DogfoodType::AcceptableFood
                    } else {
                        DogfoodType::ManFood
                    }
                }
                name if name.contains("cat") || name.contains("kitten") => {
                    // Cats like fish and meat
                    if food.object_type >= 100 && food.object_type <= 125 {
                        DogfoodType::DogFood
                    } else if food.object_type >= 50 && food.object_type < 100 {
                        DogfoodType::AcceptableFood
                    } else {
                        DogfoodType::ManFood
                    }
                }
                _ => DogfoodType::AcceptableFood,
            }
        }
        _ => DogfoodType::Unknown,
    }
}

/// Check if pet will follow player to new level
pub fn pet_will_follow(pet: &Monster, player: &You) -> bool {
    if !pet.state.tame {
        return false;
    }

    // Pet must be adjacent to player
    let dx = (pet.x - player.pos.x).abs();
    let dy = (pet.y - player.pos.y).abs();

    dx <= 1 && dy <= 1
}

/// Feed a pet with food item
/// Returns true if pet ate the food
pub fn feed_pet(pet: &mut Monster, food: &Object, rng: &mut GameRng) -> bool {
    if !pet.state.tame {
        return false;
    }

    // Check if this is food
    if food.class != ObjectClass::Food {
        return false;
    }

    // Pets prefer certain foods based on their type
    let likes_food = match pet.name.as_str() {
        name if name.contains("dog") || name.contains("wolf") || name.contains("hound") => {
            // Dogs like meat
            food.object_type >= 100 && food.object_type <= 120 // Meat range
        }
        name if name.contains("cat") || name.contains("kitten") => {
            // Cats like meat and fish
            food.object_type >= 100 && food.object_type <= 125
        }
        _ => true, // Other pets eat anything
    };

    if !likes_food && !rng.one_in(3) {
        return false; // 2/3 chance to refuse non-preferred food
    }

    // Increase tameness/loyalty
    if pet.state.tame {
        // Feeding increases loyalty (represented by not going wild)
        pet.state.peaceful = true;
    }

    true
}

/// Calculate pet's target position (where it wants to move)
/// Pets try to stay near the player
pub fn pet_target_position(pet: &Monster, player: &You, level: &Level) -> Option<(i8, i8)> {
    if !pet.state.tame {
        return None;
    }

    let px = player.pos.x;
    let py = player.pos.y;
    let mx = pet.x;
    let my = pet.y;

    // If already adjacent, stay put or move randomly
    let dx = (px - mx).abs();
    let dy = (py - my).abs();
    if dx <= 1 && dy <= 1 {
        return None; // Already close enough
    }

    // Move toward player
    let target_x = mx + (px - mx).signum();
    let target_y = my + (py - my).signum();

    if level.is_walkable(target_x, target_y) && level.monster_at(target_x, target_y).is_none() {
        Some((target_x, target_y))
    } else {
        // Try adjacent squares
        for dy in -1i8..=1 {
            for dx in -1i8..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = mx + dx;
                let ny = my + dy;
                if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
                    // Prefer squares closer to player
                    let old_dist = (px - mx).abs() + (py - my).abs();
                    let new_dist = (px - nx).abs() + (py - ny).abs();
                    if new_dist < old_dist {
                        return Some((nx, ny));
                    }
                }
            }
        }
        None
    }
}

/// Check if pet should attack a monster
/// Pets attack hostile monsters but not peaceful ones
pub fn pet_should_attack(pet: &Monster, target: &Monster) -> bool {
    if !pet.state.tame {
        return false;
    }

    // Don't attack other pets
    if target.state.tame {
        return false;
    }

    // Don't attack peaceful monsters unless they attacked first
    if target.state.peaceful && !target.state.fleeing {
        return false;
    }

    true
}

/// Move pet toward player or attack enemies
pub fn pet_move(
    pet_id: MonsterId,
    level: &mut Level,
    player: &You,
    _rng: &mut GameRng,
) -> Option<String> {
    let pet = level.monster(pet_id)?;

    if !pet.state.tame {
        return None;
    }

    // Check for enemies to attack
    let pet_x = pet.x;
    let pet_y = pet.y;

    // Look for adjacent enemies
    for dy in -1i8..=1 {
        for dx in -1i8..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = pet_x + dx;
            let ny = pet_y + dy;

            if let Some(target) = level.monster_at(nx, ny) {
                let target_clone = target.clone();
                let pet = level.monster(pet_id)?;
                if pet_should_attack(pet, &target_clone) {
                    // Attack the target
                    return Some(format!(
                        "Your {} attacks the {}!",
                        pet.name, target_clone.name
                    ));
                }
            }
        }
    }

    // Move toward player
    let pet = level.monster(pet_id)?;
    if let Some((tx, ty)) = pet_target_position(pet, player, level) {
        if let Some(pet) = level.monster_mut(pet_id) {
            pet.x = tx;
            pet.y = ty;
        }
    }

    None
}

/// Get all pets on the current level
pub fn get_pets(level: &Level) -> Vec<MonsterId> {
    level
        .monsters
        .iter()
        .filter(|m| m.state.tame)
        .map(|m| m.id)
        .collect()
}

/// Count pets on the current level
pub fn count_pets(level: &Level) -> usize {
    level.monsters.iter().filter(|m| m.state.tame).count()
}

/// Attempt to tame a monster by offering food (tamedog equivalent)
/// Returns true if taming succeeded or pet ate the food
pub fn tame_dog(
    monster: &mut Monster,
    food: Option<&Object>,
    player: &You,
    game_turn: u32,
) -> bool {
    // Cannot tame certain special monsters
    if monster.is_shopkeeper || monster.is_priest || monster.is_guard || monster.is_minion {
        return false;
    }

    // At minimum, make it peaceful
    monster.state.peaceful = true;

    // If already tame, check if we can feed it
    if monster.state.tame && food.is_some() {
        let food = food.unwrap();
        if let Some(ext) = get_pet_ext(monster) {
            // Check if hunger_time has passed
            if game_turn >= ext.hunger_time {
                // Pet will eat - increase tameness
                if monster.tameness < 20 {
                    monster.tameness += 1;
                }
                return true;
            }
        }
        return false;
    }

    // Cannot tame if already tame
    if monster.state.tame {
        return false;
    }

    // Cannot tame if monster is paralyzed
    if monster.state.paralyzed {
        return false;
    }

    // Cannot tame if there's food and it's inappropriate
    if let Some(food) = food {
        if food_quality(monster, food) >= DogfoodType::ManFood {
            return false;
        }
    }

    // Success: create and initialize pet
    create_pet_extension(monster);
    monster.state.tame = true;
    initialize_pet(monster, player, game_turn);

    // If food provided, pet will eat it
    if food.is_some() {
        if let Some(ext) = get_pet_ext_mut(monster) {
            ext.hunger_time = game_turn + 1500; // Set hunger time after eating
        }
    }

    true
}

/// Handle pet abuse and decrease tameness (abuse_dog equivalent)
/// Called when player harms pet
pub fn abuse_pet(monster: &mut Monster) {
    if !monster.state.tame {
        return;
    }

    let new_tameness = if monster.tameness > 0 {
        monster.tameness - 1
    } else {
        0
    };
    monster.tameness = new_tameness;

    // Track abuse
    if let Some(ext) = get_pet_ext_mut(monster) {
        ext.abuse_count += 1;
    }

    // If no longer tame, unleash
    if monster.tameness == 0 {
        untame_monster(monster);
    }
}

/// Handle pet revival/resurrection (wary_dog equivalent)
/// Called when pet is brought back to life
pub fn revive_pet(monster: &mut Monster, was_dead: bool) {
    if !monster.state.tame {
        return;
    }

    // Read ext fields first to avoid borrow issues
    let (killed_by_player, abuse_count, starvation_penalty) =
        if let Some(ext) = get_pet_ext(monster) {
            (
                ext.killed_by_player,
                ext.abuse_count,
                ext.starvation_penalty,
            )
        } else {
            return;
        };

    // Increment revival count
    if let Some(ext) = get_pet_ext_mut(monster) {
        ext.revival_count += 1;
    }

    // If killed by player, revive as hostile
    if killed_by_player {
        untame_monster(monster);
        monster.state.peaceful = false;
        if let Some(ext) = get_pet_ext_mut(monster) {
            ext.killed_by_player = false;
        }
        return;
    }

    // Heavy abuse causes wild revival
    if abuse_count > 2 {
        untame_monster(monster);
        monster.state.peaceful = false;
        if let Some(ext) = get_pet_ext_mut(monster) {
            ext.abuse_count = 0;
        }
        return;
    }

    // Moderate abuse causes untame but potentially peaceful
    if abuse_count > 0 {
        untame_monster(monster);
        // May stay peaceful
        if let Some(ext) = get_pet_ext_mut(monster) {
            ext.abuse_count = 0;
        }
        return;
    }

    // Restore max HP if damaged by starvation
    if starvation_penalty > 0 {
        monster.hp_max = monster.hp_max + starvation_penalty;
    }

    // Clean slate on revival
    if let Some(ext) = get_pet_ext_mut(monster) {
        ext.abuse_count = 0;
        ext.prev_goal = (-1, -1);
        if starvation_penalty > 0 {
            ext.starvation_penalty = 0;
        }
        // Different behavior for death vs. life-saving
        if was_dead {
            // Fully revived, reset hunger
            ext.hunger_time = 0;
        }
    }
}

/// Handle pet starvation and time tracking (mon_catchup_elapsed_time equivalent)
/// Called when pet re-enters after being on different level
/// Returns true if pet died from starvation
pub fn update_pet_time(monster: &mut Monster, turns_elapsed: u32) -> bool {
    if !monster.state.tame {
        return false;
    }

    // Read ext fields first to avoid borrow issues
    let hunger_time = if let Some(ext) = get_pet_ext(monster) {
        ext.hunger_time
    } else {
        return false;
    };

    // Check starvation - reduce max HP to 1/3
    let starving = hunger_time > 0 && hunger_time + 500 < turns_elapsed;
    if starving {
        let starvation_penalty = (monster.hp_max * 2 / 3).max(1);
        monster.hp_max = monster.hp_max / 3;
        if let Some(ext) = get_pet_ext_mut(monster) {
            ext.starvation_penalty = starvation_penalty;
        }
    }

    // Check severe starvation
    if hunger_time > 0 && hunger_time + 750 < turns_elapsed {
        // Pet dies
        monster.hp = 0;
        return true;
    }

    // Reduce tameness slowly over time
    if turns_elapsed > 5000 && monster.tameness > 0 {
        monster.tameness -= 1;
    }

    false
}

/// Create starting pet for new game (makedog equivalent)
/// Returns None if pet was genocided or player chose no pet
pub fn create_starting_pet(player: &You, rng: &mut GameRng) -> Option<Monster> {
    // Check player preference for pet type
    // This would be set from user preferences
    let pet_names = [("little dog", "dog"), ("kitten", "cat"), ("pony", "horse")];

    // Select random or preferred pet type
    let pet_choice = rng.rn2(pet_names.len() as u32) as usize;
    let (pet_name, _category) = pet_names[pet_choice];

    // Create the monster at player start position
    let mut pet = Monster::new(MonsterId(rng.rn2(u32::MAX)), 0, player.pos.x, player.pos.y);
    pet.name = pet_name.to_string();

    // Initialize pet system
    tame_monster(&mut pet, 5);
    initialize_pet(&mut pet, player, 0);

    // For horses, add saddle
    if pet_name == "pony" {
        // In full implementation, create saddle object and equip
    }

    Some(pet)
}

/// Move pets on the level toward player or attack enemies
/// Handles all pet AI movement
pub fn update_pets(level: &mut Level, player: &You, game_turn: u32) {
    let pet_ids: Vec<_> = level
        .monsters
        .iter()
        .filter(|m| m.state.tame)
        .map(|m| m.id)
        .collect();

    for pet_id in pet_ids {
        if let Some(pet) = level.monster_mut(pet_id) {
            // Update hunger
            if let Some(ext) = get_pet_ext_mut(pet) {
                if game_turn >= ext.hunger_time {
                    // Pet is hungry - would seek food
                    // For now, just update hunger time
                    ext.hunger_time = game_turn + 1000;
                }
            }
        }
    }
}

// ============================================================================
// Pet Level Transition Functions (keepdogs, losedogs from dog.c)
// ============================================================================

/// Collect pets that should follow player to new level (keepdogs equivalent)
///
/// Returns a list of pet IDs that will follow the player. These pets should
/// be removed from the current level and added to the new level.
///
/// # Arguments
/// * `level` - Current level
/// * `player` - Player reference
/// * `pets_only` - If true, only collect tame pets (for ascension/escape)
pub fn keepdogs(level: &Level, player: &You, pets_only: bool) -> Vec<MonsterId> {
    let mut following_pets = Vec::new();

    for monster in &level.monsters {
        // Skip dead monsters
        if monster.hp <= 0 {
            continue;
        }

        // If pets_only, skip non-pets
        if pets_only && !monster.state.tame {
            continue;
        }

        // Check if monster should follow
        let should_follow = if monster.state.tame {
            // Pets follow if adjacent and can move
            pet_will_follow(monster, player) && monster.can_act()
        } else {
            false
        };

        if should_follow {
            // Check for conditions that prevent following
            let stay_behind = monster.state.sleeping || monster.state.paralyzed;

            if !stay_behind {
                following_pets.push(monster.id);
            }
        }
    }

    following_pets
}

/// Restore pets when arriving at a new level (losedogs equivalent)
///
/// Places pets that followed the player onto the new level near the player.
///
/// # Arguments
/// * `level` - New level to place pets on
/// * `player` - Player reference
/// * `pet_ids` - List of pet monster IDs to restore
/// * `pets` - The actual pet monsters to place
/// * `game_turn` - Current game turn for time tracking
pub fn losedogs(level: &mut Level, player: &You, pets: Vec<Monster>, game_turn: u32) {
    for mut pet in pets {
        // Find a spot near the player
        let (px, py) = (player.pos.x, player.pos.y);

        // Try to place adjacent to player
        let mut placed = false;
        for dy in -1i8..=1 {
            for dx in -1i8..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = px + dx;
                let ny = py + dy;

                if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
                    pet.x = nx;
                    pet.y = ny;
                    placed = true;
                    break;
                }
            }
            if placed {
                break;
            }
        }

        // If couldn't place adjacent, try random nearby spot
        if !placed {
            pet.x = px;
            pet.y = py;
        }

        // Update pet time tracking
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = game_turn + 1000;
        }

        level.monsters.push(pet);
    }
}

// ============================================================================
// Pet Hunger and Nutrition Functions
// ============================================================================

/// Get pet hunger state (dog_hunger equivalent)
///
/// Returns the hunger level of a pet based on time since last feeding.
pub fn dog_hunger(pet: &Monster, game_turn: u32) -> PetHunger {
    if let Some(ext) = get_pet_ext(pet) {
        if game_turn < ext.hunger_time {
            PetHunger::Satiated
        } else if game_turn < ext.hunger_time + 300 {
            PetHunger::NotHungry
        } else if game_turn < ext.hunger_time + 600 {
            PetHunger::Hungry
        } else if game_turn < ext.hunger_time + 900 {
            PetHunger::Weak
        } else {
            PetHunger::Fainting
        }
    } else {
        PetHunger::NotHungry
    }
}

/// Calculate nutrition value of food for a pet (dog_nutrition equivalent)
///
/// Returns how much nutrition the pet gains from eating this food.
pub fn dog_nutrition(pet: &Monster, food: &Object) -> i32 {
    // Use object_type as a proxy for nutrition value
    let base_nutrition = (food.object_type as i32).max(50);

    // Adjust based on food quality for this pet
    let quality = food_quality(pet, food);
    match quality {
        DogfoodType::DogFood => base_nutrition * 2,
        DogfoodType::Cadaver => base_nutrition + base_nutrition / 2,
        DogfoodType::AcceptableFood => base_nutrition,
        DogfoodType::ManFood => base_nutrition / 2,
        DogfoodType::Apport => 0,               // Not food
        DogfoodType::Poison => -base_nutrition, // Harmful
        DogfoodType::Unknown => base_nutrition / 4,
        DogfoodType::Taboo => 0,
    }
}

/// Check pet's inventory for food (dog_invent equivalent - simplified)
///
/// Returns true if pet has food in its inventory.
pub fn dog_invent(_pet: &Monster) -> bool {
    // In full implementation, would check pet's minvent
    // For now, pets don't carry items in our simplified model
    false
}

// ============================================================================
// Pet Taming and Behavior Functions
// ============================================================================

/// Attempt to tame a monster with a chance of failure (maybe_tame equivalent)
///
/// Returns true if taming succeeded.
pub fn maybe_tame(monster: &mut Monster, player: &You, game_turn: u32, rng: &mut GameRng) -> bool {
    // Cannot tame certain special monsters
    if monster.is_shopkeeper || monster.is_priest || monster.is_guard || monster.is_minion {
        return false;
    }

    // Already tame
    if monster.state.tame {
        return false;
    }

    // Base chance depends on charisma
    let cha = player.attr_current.get(crate::player::Attribute::Charisma) as i32;
    let chance = 10 + cha * 2; // 10-46% base chance

    if rng.rn2(100) < chance as u32 {
        tame_monster(monster, 5);
        initialize_pet(monster, player, game_turn);
        true
    } else {
        // Failed taming might make monster peaceful
        if rng.one_in(3) {
            monster.state.peaceful = true;
        }
        false
    }
}

/// Pet begging for food (beg equivalent)
///
/// Returns a message if pet is begging, None otherwise.
pub fn beg(pet: &Monster, game_turn: u32) -> Option<String> {
    if !pet.state.tame {
        return None;
    }

    let hunger = dog_hunger(pet, game_turn);
    match hunger {
        PetHunger::Hungry | PetHunger::Weak | PetHunger::Fainting => {
            Some(format!("{} is looking at you expectantly.", pet.name))
        }
        _ => None,
    }
}

// ============================================================================
// Pet Naming Functions
// ============================================================================

/// Get a cute name for a young pet (strkitten equivalent)
///
/// Returns an appropriate diminutive name based on pet type.
pub fn strkitten(pet: &Monster) -> &'static str {
    let name_lower = pet.name.to_lowercase();
    if name_lower.contains("cat") || name_lower.contains("kitten") {
        "kitten"
    } else if name_lower.contains("dog") || name_lower.contains("puppy") {
        "puppy"
    } else if name_lower.contains("pony") || name_lower.contains("horse") {
        "foal"
    } else if name_lower.contains("wolf") {
        "wolf pup"
    } else {
        "pet"
    }
}

/// Generate a litter of pets (litter equivalent - simplified)
///
/// Creates multiple young pets. Returns the number created.
pub fn litter(level: &mut Level, parent: &Monster, player: &You, rng: &mut GameRng) -> u32 {
    // Determine litter size (1-4)
    let count = rng.rn2(4) + 1;

    let mut created = 0;
    for _ in 0..count {
        // Find spot near parent
        let (px, py) = (parent.x, parent.y);
        for dy in -1i8..=1 {
            for dx in -1i8..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = px + dx;
                let ny = py + dy;

                if level.is_walkable(nx, ny) && level.monster_at(nx, ny).is_none() {
                    // Create young pet
                    let mut baby =
                        Monster::new(MonsterId(rng.rn2(u32::MAX)), parent.monster_type, nx, ny);
                    baby.name = strkitten(parent).to_string();
                    tame_monster(&mut baby, 10); // Young pets are very loyal
                    initialize_pet(&mut baby, player, 0);

                    level.monsters.push(baby);
                    created += 1;
                    break;
                }
            }
            if created > 0 {
                break;
            }
        }
    }

    created
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pet() -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = "little dog".to_string();
        m.state.tame = true;
        m.state.peaceful = true;
        m.tameness = 10;
        create_pet_extension(&mut m);
        m
    }

    #[test]
    fn test_is_pet() {
        let pet = test_pet();
        assert!(is_pet(&pet));
    }

    #[test]
    fn test_is_not_pet() {
        let monster = Monster::new(MonsterId(2), 0, 5, 5);
        assert!(!is_pet(&monster));
    }

    #[test]
    fn test_tame_monster() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!monster.state.tame);

        tame_monster(&mut monster, 5);

        assert!(monster.state.tame);
        assert!(monster.state.peaceful);
        assert_eq!(monster.tameness, 5);
    }

    #[test]
    fn test_pet_extension_creation() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(monster.pet_extension.is_none());

        monster.state.tame = true;
        create_pet_extension(&mut monster);

        assert!(monster.pet_extension.is_some());
        assert_eq!(get_pet_ext(&monster).unwrap().abuse_count, 0);
    }

    #[test]
    fn test_pet_hunger() {
        assert_eq!(PetHunger::from_nutrition(200, 100), PetHunger::Satiated);
        assert_eq!(PetHunger::from_nutrition(75, 100), PetHunger::NotHungry);
        assert_eq!(PetHunger::from_nutrition(30, 100), PetHunger::Hungry);
        assert_eq!(PetHunger::from_nutrition(15, 100), PetHunger::Weak);
        assert_eq!(PetHunger::from_nutrition(5, 100), PetHunger::Fainting);
    }

    #[test]
    fn test_pet_will_follow() {
        let pet = test_pet();
        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;

        assert!(pet_will_follow(&pet, &player)); // Adjacent

        let mut far_pet = test_pet();
        far_pet.x = 10;
        far_pet.y = 10;
        assert!(!pet_will_follow(&far_pet, &player)); // Too far
    }

    #[test]
    fn test_abuse_pet() {
        let mut pet = test_pet();
        pet.tameness = 5;

        abuse_pet(&mut pet);

        assert_eq!(pet.tameness, 4);
        if let Some(ext) = get_pet_ext(&pet) {
            assert_eq!(ext.abuse_count, 1);
        }
    }

    #[test]
    fn test_revive_pet() {
        let mut pet = test_pet();

        revive_pet(&mut pet, true);

        if let Some(ext) = get_pet_ext(&pet) {
            assert_eq!(ext.revival_count, 1);
        }
    }

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_pet_extension_initialization() {
        let ext = PetExtension::new();
        assert_eq!(ext.abuse_count, 0);
        assert_eq!(ext.revival_count, 0);
        assert_eq!(ext.drop_time, 0);
        assert_eq!(ext.drop_distance, 10000);
        assert_eq!(ext.apport_level, 0);
        assert_eq!(ext.whistle_time, 0);
        assert_eq!(ext.hunger_time, 1000);
        assert_eq!(ext.prev_goal, (-1, -1));
        assert_eq!(ext.starvation_penalty, 0);
        assert!(!ext.killed_by_player);
    }

    #[test]
    fn test_pet_extension_clear_goal() {
        let mut ext = PetExtension::new();
        ext.prev_goal = (10, 20);
        assert!(ext.has_valid_goal());

        ext.clear_goal();
        assert!(!ext.has_valid_goal());
        assert_eq!(ext.prev_goal, (-1, -1));
    }

    #[test]
    fn test_untame_monster() {
        let mut pet = test_pet();
        assert!(pet.state.tame);
        assert!(pet.pet_extension.is_some());

        untame_monster(&mut pet);

        assert!(!pet.state.tame);
        assert!(pet.pet_extension.is_none());
        assert_eq!(pet.tameness, 0);
    }

    #[test]
    fn test_get_pet_ext_none_for_wild_monster() {
        let monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(get_pet_ext(&monster).is_none());
        let mut m = monster;
        assert!(get_pet_ext_mut(&mut m).is_none());
    }

    #[test]
    fn test_tame_with_zero_tameness_becomes_one() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        tame_monster(&mut monster, 0);
        assert_eq!(monster.tameness, 1); // Clamped to minimum 1
    }

    #[test]
    fn test_food_quality_dog_preferences() {
        let mut dog = Monster::new(MonsterId(1), 0, 5, 5);
        dog.name = "little dog".to_string();
        dog.state.tame = true;
        create_pet_extension(&mut dog);

        let mut meat = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        meat.object_type = 110; // Meat type

        assert_eq!(food_quality(&dog, &meat), DogfoodType::DogFood);
    }

    #[test]
    fn test_food_quality_cat_preferences() {
        let mut cat = Monster::new(MonsterId(1), 0, 5, 5);
        cat.name = "kitten".to_string();
        cat.state.tame = true;
        create_pet_extension(&mut cat);

        let mut fish = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        fish.object_type = 115; // Fish type

        assert_eq!(food_quality(&cat, &fish), DogfoodType::DogFood);
    }

    #[test]
    fn test_food_quality_non_food_object() {
        let pet = test_pet();
        let weapon = Object::new(ObjectId::NONE, 0, ObjectClass::Weapon);

        assert_eq!(food_quality(&pet, &weapon), DogfoodType::Unknown);
    }

    #[test]
    fn test_feed_pet_success() {
        let mut pet = test_pet();
        let mut food = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        food.object_type = 105; // Dog food
        let mut rng = GameRng::new(42);

        assert!(feed_pet(&mut pet, &food, &mut rng));
        assert!(pet.state.peaceful);
    }

    #[test]
    fn test_feed_pet_non_food_fails() {
        let mut pet = test_pet();
        let weapon = Object::new(ObjectId::NONE, 0, ObjectClass::Weapon);
        let mut rng = GameRng::new(42);

        assert!(!feed_pet(&mut pet, &weapon, &mut rng));
    }

    #[test]
    fn test_feed_wild_monster_fails() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        let mut food = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        let mut rng = GameRng::new(42);

        assert!(!feed_pet(&mut monster, &food, &mut rng));
    }

    #[test]
    fn test_pet_will_follow_at_various_distances() {
        let pet = test_pet();
        let mut player = You::default();
        player.pos.x = 5;
        player.pos.y = 5;

        // Adjacent - should follow
        assert!(pet_will_follow(&pet, &player));

        // 2 squares away - should not follow
        let mut far_pet = test_pet();
        far_pet.x = 7;
        far_pet.y = 5;
        assert!(!pet_will_follow(&far_pet, &player));

        // Diagonal adjacent - should follow
        let mut diag_pet = test_pet();
        diag_pet.x = 6;
        diag_pet.y = 6;
        assert!(pet_will_follow(&diag_pet, &player));
    }

    #[test]
    fn test_pet_should_attack_hostile_monsters() {
        let pet = test_pet();
        let mut hostile = Monster::new(MonsterId(2), 0, 6, 5);
        hostile.state.tame = false;
        hostile.state.peaceful = false;

        assert!(pet_should_attack(&pet, &hostile));
    }

    #[test]
    fn test_pet_should_not_attack_other_pets() {
        let pet = test_pet();
        let other_pet = test_pet();

        assert!(!pet_should_attack(&pet, &other_pet));
    }

    #[test]
    fn test_pet_should_not_attack_peaceful_monsters() {
        let pet = test_pet();
        let mut peaceful = Monster::new(MonsterId(2), 0, 6, 5);
        peaceful.state.peaceful = true;
        peaceful.state.tame = false;

        assert!(!pet_should_attack(&pet, &peaceful));
    }

    #[test]
    fn test_get_pets_on_level() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let mut level = crate::dungeon::Level::new(dlevel);

        let mut pet1 = test_pet();
        pet1.id = MonsterId(1);
        let mut pet2 = test_pet();
        pet2.id = MonsterId(2);
        let mut wild = Monster::new(MonsterId(3), 0, 5, 5);

        level.monsters.push(pet1);
        level.monsters.push(pet2);
        level.monsters.push(wild);

        let pet_ids = get_pets(&level);
        assert_eq!(pet_ids.len(), 2);
    }

    #[test]
    fn test_count_pets_on_level() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let mut level = crate::dungeon::Level::new(dlevel);

        let pet1 = test_pet();
        let pet2 = test_pet();
        let wild = Monster::new(MonsterId(3), 0, 5, 5);

        level.monsters.push(pet1);
        level.monsters.push(pet2);
        level.monsters.push(wild);

        assert_eq!(count_pets(&level), 2);
    }

    #[test]
    fn test_abuse_pet_multiple_times() {
        let mut pet = test_pet();
        pet.tameness = 10;

        abuse_pet(&mut pet);
        assert_eq!(pet.tameness, 9);

        abuse_pet(&mut pet);
        assert_eq!(pet.tameness, 8);

        if let Some(ext) = get_pet_ext(&pet) {
            assert_eq!(ext.abuse_count, 2);
        }
    }

    #[test]
    fn test_abuse_pet_to_zero_untames() {
        let mut pet = test_pet();
        pet.tameness = 1;

        abuse_pet(&mut pet);

        assert!(!pet.state.tame);
        assert_eq!(pet.tameness, 0);
    }

    #[test]
    fn test_revive_pet_killed_by_player() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.killed_by_player = true;
        }

        revive_pet(&mut pet, true);

        assert!(!pet.state.tame);
        assert!(!pet.state.peaceful);
    }

    #[test]
    fn test_revive_pet_with_heavy_abuse() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.abuse_count = 3; // Heavy abuse
        }

        revive_pet(&mut pet, true);

        assert!(!pet.state.tame);
        assert!(!pet.state.peaceful);
    }

    #[test]
    fn test_revive_pet_with_moderate_abuse() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.abuse_count = 1; // Moderate abuse
        }

        revive_pet(&mut pet, true);

        assert!(!pet.state.tame);
    }

    #[test]
    fn test_update_pet_time_normal_passage() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        let died = update_pet_time(&mut pet, 1200); // 200 turns passed

        assert!(!died);
    }

    #[test]
    fn test_update_pet_time_starvation_penalty() {
        let mut pet = test_pet();
        pet.hp_max = 100;
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        let died = update_pet_time(&mut pet, 1600); // 600 turns passed (> 500)

        assert!(!died);
        assert!(pet.hp_max < 100); // HP reduced
    }

    #[test]
    fn test_update_pet_time_death_from_starvation() {
        let mut pet = test_pet();
        pet.hp = 50;
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        let died = update_pet_time(&mut pet, 1800); // 800 turns passed (> 750)

        assert!(died);
        assert_eq!(pet.hp, 0);
    }

    #[test]
    fn test_tame_dog_cannot_tame_shopkeeper() {
        let mut shopkeeper = Monster::new(MonsterId(1), 0, 5, 5);
        shopkeeper.is_shopkeeper = true;
        let player = You::default();

        assert!(!tame_dog(&mut shopkeeper, None, &player, 0));
    }

    #[test]
    fn test_tame_dog_cannot_tame_priest() {
        let mut priest = Monster::new(MonsterId(1), 0, 5, 5);
        priest.is_priest = true;
        let player = You::default();

        assert!(!tame_dog(&mut priest, None, &player, 0));
    }

    #[test]
    fn test_tame_dog_cannot_tame_paralyzed() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.state.paralyzed = true;
        let player = You::default();

        assert!(!tame_dog(&mut monster, None, &player, 0));
    }

    #[test]
    fn test_tame_dog_already_tame_with_food() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 100;
        }
        let mut food = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        food.object_type = 110;
        let player = You::default();

        assert!(tame_dog(&mut pet, Some(&food), &player, 200));
    }

    #[test]
    fn test_create_starting_pet() {
        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;
        let mut rng = GameRng::new(42);

        if let Some(pet) = create_starting_pet(&player, &mut rng) {
            assert!(pet.state.tame);
            assert!(pet.pet_extension.is_some());
            assert!(!pet.name.is_empty());
        }
    }

    #[test]
    fn test_dogfood_type_ordering() {
        assert!(DogfoodType::DogFood < DogfoodType::Cadaver);
        assert!(DogfoodType::Cadaver < DogfoodType::AcceptableFood);
        assert!(DogfoodType::ManFood > DogfoodType::DogFood);
    }

    #[test]
    fn test_pet_hunger_level_ordering() {
        assert!(PetHunger::Satiated < PetHunger::NotHungry);
        assert!(PetHunger::NotHungry < PetHunger::Hungry);
        assert!(PetHunger::Hungry < PetHunger::Weak);
        assert!(PetHunger::Weak < PetHunger::Fainting);
    }

    #[test]
    fn test_pet_hunger_boundary_values() {
        // Test exact boundaries
        assert_eq!(PetHunger::from_nutrition(151, 100), PetHunger::Satiated);
        assert_eq!(PetHunger::from_nutrition(150, 100), PetHunger::NotHungry);
        assert_eq!(PetHunger::from_nutrition(51, 100), PetHunger::NotHungry);
        assert_eq!(PetHunger::from_nutrition(50, 100), PetHunger::Hungry);
        assert_eq!(PetHunger::from_nutrition(26, 100), PetHunger::Hungry);
        assert_eq!(PetHunger::from_nutrition(25, 100), PetHunger::Weak);
        assert_eq!(PetHunger::from_nutrition(11, 100), PetHunger::Weak);
        assert_eq!(PetHunger::from_nutrition(10, 100), PetHunger::Fainting);
    }

    // ========================================================================
    // Tests for new functions: keepdogs, losedogs, dog_hunger, etc.
    // ========================================================================

    #[test]
    fn test_keepdogs_collects_adjacent_pets() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let mut level = crate::dungeon::Level::new(dlevel);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // Create adjacent pet
        let mut pet = test_pet();
        pet.x = 11;
        pet.y = 10;
        level.monsters.push(pet);

        let following = keepdogs(&level, &player, true);
        assert_eq!(following.len(), 1);
    }

    #[test]
    fn test_keepdogs_ignores_distant_pets() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let mut level = crate::dungeon::Level::new(dlevel);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // Create distant pet
        let mut pet = test_pet();
        pet.x = 20;
        pet.y = 20;
        level.monsters.push(pet);

        let following = keepdogs(&level, &player, true);
        assert_eq!(following.len(), 0);
    }

    #[test]
    fn test_keepdogs_ignores_sleeping_pets() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let mut level = crate::dungeon::Level::new(dlevel);

        let mut player = You::default();
        player.pos.x = 10;
        player.pos.y = 10;

        // Create adjacent but sleeping pet
        let mut pet = test_pet();
        pet.x = 11;
        pet.y = 10;
        pet.state.sleeping = true;
        level.monsters.push(pet);

        let following = keepdogs(&level, &player, true);
        assert_eq!(following.len(), 0);
    }

    #[test]
    fn test_dog_hunger_satiated() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        assert_eq!(dog_hunger(&pet, 500), PetHunger::Satiated);
    }

    #[test]
    fn test_dog_hunger_progression() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        assert_eq!(dog_hunger(&pet, 1100), PetHunger::NotHungry);
        assert_eq!(dog_hunger(&pet, 1400), PetHunger::Hungry);
        assert_eq!(dog_hunger(&pet, 1700), PetHunger::Weak);
        assert_eq!(dog_hunger(&pet, 2000), PetHunger::Fainting);
    }

    #[test]
    fn test_dog_nutrition_preferred_food() {
        let mut pet = test_pet();
        let mut food = Object::new(ObjectId::NONE, 0, ObjectClass::Food);
        food.object_type = 110; // Meat type

        let nutrition = dog_nutrition(&pet, &food);
        assert!(nutrition > 100); // Should get bonus for preferred food
    }

    #[test]
    fn test_dog_invent_returns_false() {
        let pet = test_pet();
        assert!(!dog_invent(&pet));
    }

    #[test]
    fn test_beg_when_hungry() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 100;
        }

        let msg = beg(&pet, 500); // 400 turns past hunger time
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("expectantly"));
    }

    #[test]
    fn test_beg_when_satiated() {
        let mut pet = test_pet();
        if let Some(ext) = get_pet_ext_mut(&mut pet) {
            ext.hunger_time = 1000;
        }

        let msg = beg(&pet, 500); // Before hunger time
        assert!(msg.is_none());
    }

    #[test]
    fn test_strkitten_cat() {
        let mut pet = Monster::new(MonsterId(1), 0, 5, 5);
        pet.name = "large cat".to_string();
        assert_eq!(strkitten(&pet), "kitten");
    }

    #[test]
    fn test_strkitten_dog() {
        let mut pet = Monster::new(MonsterId(1), 0, 5, 5);
        pet.name = "little dog".to_string();
        assert_eq!(strkitten(&pet), "puppy");
    }

    #[test]
    fn test_strkitten_horse() {
        let mut pet = Monster::new(MonsterId(1), 0, 5, 5);
        pet.name = "pony".to_string();
        assert_eq!(strkitten(&pet), "foal");
    }

    #[test]
    fn test_strkitten_wolf() {
        let mut pet = Monster::new(MonsterId(1), 0, 5, 5);
        pet.name = "winter wolf".to_string();
        assert_eq!(strkitten(&pet), "wolf pup");
    }

    #[test]
    fn test_strkitten_unknown() {
        let mut pet = Monster::new(MonsterId(1), 0, 5, 5);
        pet.name = "dragon".to_string();
        assert_eq!(strkitten(&pet), "pet");
    }

    #[test]
    fn test_maybe_tame_cannot_tame_shopkeeper() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        monster.is_shopkeeper = true;
        let player = You::default();
        let mut rng = GameRng::new(42);

        assert!(!maybe_tame(&mut monster, &player, 0, &mut rng));
    }

    #[test]
    fn test_maybe_tame_cannot_tame_already_tame() {
        let mut pet = test_pet();
        let player = You::default();
        let mut rng = GameRng::new(42);

        assert!(!maybe_tame(&mut pet, &player, 0, &mut rng));
    }
}
