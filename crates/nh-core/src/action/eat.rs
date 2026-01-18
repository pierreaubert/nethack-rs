//! Eating food and corpses (eat.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;
use crate::object::{BucStatus, Object, ObjectClass};

/// Hunger state thresholds (from NetHack)
pub const SATIATED: i32 = 2000;
pub const NOT_HUNGRY: i32 = 900;
pub const HUNGRY: i32 = 150;
pub const WEAK: i32 = 50;
pub const FAINTING: i32 = 0;

/// Hunger state names for display
pub const HUNGER_STATES: &[&str] = &[
    "Satiated", "", "Hungry", "Weak", "Fainting", "Fainted", "Starved",
];

/// Hunger state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HungerState {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
    Fainted,
    Starved,
}

impl HungerState {
    /// Get hunger state from nutrition value
    pub fn from_nutrition(nutrition: i32) -> Self {
        if nutrition > SATIATED {
            HungerState::Satiated
        } else if nutrition > NOT_HUNGRY {
            HungerState::NotHungry
        } else if nutrition > HUNGRY {
            HungerState::Hungry
        } else if nutrition > WEAK {
            HungerState::Weak
        } else if nutrition > FAINTING {
            HungerState::Fainting
        } else if nutrition > -10 {
            HungerState::Fainted
        } else {
            HungerState::Starved
        }
    }

    /// Get display string for hunger state
    pub fn as_str(&self) -> &'static str {
        match self {
            HungerState::Satiated => "Satiated",
            HungerState::NotHungry => "",
            HungerState::Hungry => "Hungry",
            HungerState::Weak => "Weak",
            HungerState::Fainting => "Fainting",
            HungerState::Fainted => "Fainted",
            HungerState::Starved => "Starved",
        }
    }
}

/// Tin preparation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinType {
    Rotten,
    Homemade,
    Soup,
    FrenchFried,
    Pickled,
    Boiled,
    Smoked,
    Dried,
    DeepFried,
    Szechuan,
    Broiled,
    StirFried,
    Sauteed,
    Candied,
    Pureed,
    Spinach,
}

impl TinType {
    /// Get nutrition modifier for tin type
    pub fn nutrition(&self) -> i32 {
        match self {
            TinType::Rotten => -50,
            TinType::Homemade => 50,
            TinType::Soup => 20,
            TinType::FrenchFried => 40,
            TinType::Pickled => 40,
            TinType::Boiled => 50,
            TinType::Smoked => 50,
            TinType::Dried => 55,
            TinType::DeepFried => 60,
            TinType::Szechuan => 70,
            TinType::Broiled => 80,
            TinType::StirFried => 80,
            TinType::Sauteed => 95,
            TinType::Candied => 100,
            TinType::Pureed => 500,
            TinType::Spinach => 600,
        }
    }

    /// Get description for tin type
    pub fn description(&self) -> &'static str {
        match self {
            TinType::Rotten => "rotten",
            TinType::Homemade => "homemade",
            TinType::Soup => "soup made from",
            TinType::FrenchFried => "french fried",
            TinType::Pickled => "pickled",
            TinType::Boiled => "boiled",
            TinType::Smoked => "smoked",
            TinType::Dried => "dried",
            TinType::DeepFried => "deep fried",
            TinType::Szechuan => "szechuan",
            TinType::Broiled => "broiled",
            TinType::StirFried => "stir fried",
            TinType::Sauteed => "sauteed",
            TinType::Candied => "candied",
            TinType::Pureed => "pureed",
            TinType::Spinach => "spinach",
        }
    }
}

/// Base nutrition values for common foods
pub fn base_nutrition(object_type: i16) -> i32 {
    // These should match nh-data object definitions
    // For now, provide reasonable defaults
    match object_type {
        // Corpses vary by monster
        _ if object_type < 100 => 100, // Generic food
        _ => 50,
    }
}

/// Check if food is edible
pub fn is_edible(obj: &Object) -> bool {
    obj.class == ObjectClass::Food
}

/// Check if food is rotten (based on age and type)
pub fn is_rotten(obj: &Object, current_turn: i64) -> bool {
    // Corpses rot after ~250 turns
    // Blessed food lasts longer, cursed food rots faster
    let age = current_turn - obj.age;
    let rot_time = match obj.buc {
        BucStatus::Blessed => 350,
        BucStatus::Uncursed => 250,
        BucStatus::Cursed => 150,
    };
    age > rot_time
}

/// Calculate nutrition from eating an object
pub fn calculate_nutrition(obj: &Object) -> i32 {
    let base = base_nutrition(obj.object_type);

    // Blessed food gives more nutrition
    let buc_modifier = match obj.buc {
        BucStatus::Blessed => 1.5,
        BucStatus::Uncursed => 1.0,
        BucStatus::Cursed => 0.75,
    };

    (base as f32 * buc_modifier) as i32
}

/// Eat food from inventory
pub fn do_eat(state: &mut GameState, obj_letter: char) -> ActionResult {
    // First pass: validation
    let (obj_name, is_cursed, nutrition) = {
        let obj = match state.get_inventory_item(obj_letter) {
            Some(o) => o,
            None => return ActionResult::Failed("You don't have that item.".to_string()),
        };

        if obj.class != ObjectClass::Food {
            return ActionResult::Failed("That's not something you can eat.".to_string());
        }

        let name = obj.name.clone().unwrap_or_else(|| "food".to_string());
        let nutrition = calculate_nutrition(obj);

        (name, obj.is_cursed(), nutrition)
    };

    // Check for choking (eating while satiated)
    let hunger_state = HungerState::from_nutrition(state.player.nutrition);
    if hunger_state == HungerState::Satiated {
        state.message("You're having a hard time getting all of it down.");
        // Could potentially choke - for now just warn
    }

    // Eating message
    state.message(format!("You eat the {}.", obj_name));

    // Apply nutrition
    state.player.nutrition += nutrition;

    // Cursed food might cause problems
    if is_cursed {
        state.message("Ulch - that food was tainted!");
        // Could cause sickness, vomiting, etc.
        state.player.nutrition -= nutrition / 2; // Lose some nutrition
    }

    // Update hunger state
    state.player.update_hunger();

    // Check new hunger state and give feedback
    let new_state = HungerState::from_nutrition(state.player.nutrition);
    match new_state {
        HungerState::Satiated => {
            state.message("You're having a hard time getting all of it down.");
        }
        HungerState::NotHungry => {
            // No message needed
        }
        _ => {
            // Still hungry
        }
    }

    // Remove the food item
    state.remove_from_inventory(obj_letter);

    ActionResult::Success
}

/// Vomit (lose nutrition, possibly drop items)
pub fn vomit(state: &mut GameState) {
    state.message("You vomit!");
    state.player.nutrition -= 1000;
    if state.player.nutrition < 0 {
        state.player.nutrition = 0;
    }
    state.player.update_hunger();
}

/// Choke on food (potentially fatal)
pub fn choke(state: &mut GameState, food_name: &str) {
    state.message(format!("You choke on the {}!", food_name));
    // In full implementation, this could be fatal
    // For now, just cause vomiting
    vomit(state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{Object, ObjectClass, ObjectId};
    use crate::rng::GameRng;

    #[test]
    fn test_eat_non_food_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_missing_item_fails() {
        let mut state = GameState::new(GameRng::from_entropy());
        let result = do_eat(&mut state, 'z');
        assert!(matches!(result, ActionResult::Failed(_)));
    }

    #[test]
    fn test_eat_food_increases_nutrition() {
        let mut state = GameState::new(GameRng::from_entropy());
        let initial_nutrition = state.player.nutrition;
        
        let mut obj = Object::default();
        obj.id = ObjectId(1);
        obj.class = ObjectClass::Food;
        obj.inv_letter = 'a';
        state.inventory.push(obj);

        let result = do_eat(&mut state, 'a');
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.nutrition > initial_nutrition);
    }
}
