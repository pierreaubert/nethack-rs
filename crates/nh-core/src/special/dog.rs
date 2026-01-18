//! Pet/dog handling (dog.c)
//!
//! Handles tame monster behavior, feeding, and pet AI.

use crate::dungeon::Level;
use crate::monster::{Monster, MonsterId};
use crate::object::{Object, ObjectClass};
use crate::player::You;
use crate::rng::GameRng;

/// Pet hunger levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PetHunger {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
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

/// Check if a monster is a valid pet
pub fn is_pet(monster: &Monster) -> bool {
    monster.state.tame
}

/// Make a monster tame (become a pet)
pub fn tame_monster(monster: &mut Monster) {
    monster.state.tame = true;
    monster.state.peaceful = true;
    monster.state.fleeing = false;
}

/// Untame a pet (goes wild)
pub fn untame_monster(monster: &mut Monster) {
    monster.state.tame = false;
    // May or may not stay peaceful
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
                    return Some(format!("Your {} attacks the {}!", pet.name, target_clone.name));
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
    level.monsters
        .iter()
        .filter(|m| m.state.tame)
        .map(|m| m.id)
        .collect()
}

/// Count pets on the current level
pub fn count_pets(level: &Level) -> usize {
    level.monsters.iter().filter(|m| m.state.tame).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pet() -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = "little dog".to_string();
        m.state.tame = true;
        m.state.peaceful = true;
        m
    }

    #[test]
    fn test_is_pet() {
        let pet = test_pet();
        assert!(is_pet(&pet));
    }

    #[test]
    fn test_tame_monster() {
        let mut monster = Monster::new(MonsterId(1), 0, 5, 5);
        assert!(!monster.state.tame);
        
        tame_monster(&mut monster);
        
        assert!(monster.state.tame);
        assert!(monster.state.peaceful);
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
}
