//! Monster ranged attacks (mthrowu.c)
//!
//! Handles monsters throwing objects and using ranged weapons against the player.

use crate::monster::Monster;
use crate::object::Object;
use crate::rng::GameRng;

/// Result of a monster throwing an object
#[derive(Debug, Clone)]
pub struct MonsterThrowResult {
    /// Whether the throw hit the player
    pub hit: bool,
    /// Damage dealt (0 if miss)
    pub damage: i32,
    /// Message to display
    pub message: String,
    /// Whether the thrown object broke on impact
    pub object_broke: bool,
}

/// Maximum range for monster throws (matches C: BOLT_LIM = 8)
pub const MAX_THROW_RANGE: i32 = 8;

/// Check if a monster can throw an object at the player (thrwmu from mthrowu.c:50).
///
/// Monsters need a clear line of sight and an appropriate item.
pub fn can_monster_throw(
    monster: &Monster,
    player_x: i8,
    player_y: i8,
) -> bool {
    let dx = (player_x - monster.x) as i32;
    let dy = (player_y - monster.y) as i32;
    let dist = dx.abs().max(dy.abs());

    // Must be within throwing range
    if dist > MAX_THROW_RANGE || dist == 0 {
        return false;
    }

    // Must be in a straight line or diagonal (for accuracy)
    dx == 0 || dy == 0 || dx.abs() == dy.abs()
}

/// Calculate throw damage for a monster's projectile.
///
/// Damage depends on the object type, monster strength, and distance.
pub fn throw_damage(obj: &Object, monster_level: i32, distance: i32) -> i32 {
    let base_dmg = (obj.damage_dice as i32).max(1);
    let level_bonus = monster_level / 4;

    // Damage reduced by distance (long throws are weaker)
    let distance_penalty = distance / 3;

    (base_dmg + level_bonus - distance_penalty).max(1)
}

/// Calculate throw to-hit bonus for a monster.
///
/// Higher level monsters and closer range = better accuracy.
pub fn throw_to_hit(monster_level: i32, distance: i32) -> i32 {
    let base = monster_level;
    let range_penalty = distance;
    base - range_penalty
}

/// Pick the best throwable item from a monster's inventory (m_pick_throw from mthrowu.c).
///
/// Prefers: daggers, darts, shuriken, rocks, arrows, spears.
/// Returns the index of the selected item, or None.
pub fn pick_throw_item(items: &[Object]) -> Option<usize> {
    // Priority: specific weapons first
    let priority_names = [
        "dagger", "dart", "shuriken", "throwing star",
        "spear", "javelin", "rock", "arrow", "crossbow bolt",
    ];

    for priority in &priority_names {
        if let Some(idx) = items.iter().position(|o| {
            o.display_name().to_lowercase().contains(priority)
        }) {
            return Some(idx);
        }
    }

    // Fall back to any weapon class item
    items.iter().position(|o| {
        matches!(o.class, crate::object::ObjectClass::Weapon | crate::object::ObjectClass::Gem)
    })
}

/// Monster throws an object at the player (thrwmu from mthrowu.c).
pub fn thrwmu(
    monster: &Monster,
    obj: &Object,
    player_x: i8,
    player_y: i8,
    player_ac: i8,
    rng: &mut GameRng,
) -> MonsterThrowResult {
    let dx = (player_x - monster.x) as i32;
    let dy = (player_y - monster.y) as i32;
    let distance = dx.abs().max(dy.abs());

    let to_hit = throw_to_hit(monster.level as i32, distance);
    let roll = rng.rn2(20) as i32 + 1; // d20
    let hit = (roll + to_hit) as i8 > player_ac;

    if hit {
        let damage = throw_damage(obj, monster.level as i32, distance);
        let obj_name = obj.display_name();
        let broke = obj.class == crate::object::ObjectClass::Potion;

        MonsterThrowResult {
            hit: true,
            damage,
            message: format!("{} throws {} and hits you!", monster.name, obj_name),
            object_broke: broke,
        }
    } else {
        let obj_name = obj.display_name();
        MonsterThrowResult {
            hit: false,
            damage: 0,
            message: format!("{} throws {} and misses!", monster.name, obj_name),
            object_broke: false,
        }
    }
}

/// Check if a potion should break and create a vapor effect on impact.
///
/// Potions always break when thrown. The vapor effect depends on BUC status.
pub fn potion_breaks_on_throw(obj: &Object) -> bool {
    obj.class == crate::object::ObjectClass::Potion
}

/// Monster breathes a ranged attack (breamu from mthrowu.c:580).
///
/// Dragon breath, fire breath, etc. Returns damage and message.
pub fn breamu(
    monster: &Monster,
    breath_type: BreathType,
    distance: i32,
    rng: &mut GameRng,
) -> (i32, String) {
    let mlev = monster.level as i32;
    let base_damage = match breath_type {
        BreathType::Fire => rng.rnd(6) as i32 * (mlev / 2 + 1),
        BreathType::Cold => rng.rnd(6) as i32 * (mlev / 2 + 1),
        BreathType::Sleep => 0,
        BreathType::Disintegration => 100, // Instant kill if not resistant
        BreathType::Lightning => rng.rnd(6) as i32 * (mlev / 2 + 1),
        BreathType::Poison => rng.rnd(6) as i32 * 2,
        BreathType::Acid => rng.rnd(6) as i32 * 2,
    };

    let name = breath_type.name();
    let _ = distance; // Would reduce damage with range in full implementation

    (base_damage, format!("{} breathes {}!", monster.name, name))
}

/// Types of breath weapons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreathType {
    Fire,
    Cold,
    Sleep,
    Disintegration,
    Lightning,
    Poison,
    Acid,
}

impl BreathType {
    pub const fn name(&self) -> &'static str {
        match self {
            BreathType::Fire => "fire",
            BreathType::Cold => "frost",
            BreathType::Sleep => "sleep gas",
            BreathType::Disintegration => "a disintegration blast",
            BreathType::Lightning => "lightning",
            BreathType::Poison => "poison gas",
            BreathType::Acid => "acid",
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;

    fn test_monster() -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.level = 5;
        m.name = "kobold".to_string();
        m
    }

    #[test]
    fn test_can_monster_throw_in_range() {
        let monster = test_monster();
        assert!(can_monster_throw(&monster, 10, 5)); // 5 tiles east
    }

    #[test]
    fn test_can_monster_throw_out_of_range() {
        let monster = test_monster();
        assert!(!can_monster_throw(&monster, 20, 5)); // 15 tiles
    }

    #[test]
    fn test_can_monster_throw_diagonal() {
        let monster = test_monster();
        assert!(can_monster_throw(&monster, 10, 10)); // Diagonal
    }

    #[test]
    fn test_can_monster_throw_non_line() {
        let monster = test_monster();
        assert!(!can_monster_throw(&monster, 7, 8)); // Not on a line
    }

    #[test]
    fn test_throw_damage() {
        let mut obj = Object::default();
        obj.damage_dice = 4;
        assert!(throw_damage(&obj, 5, 3) >= 1);
    }

    #[test]
    fn test_throw_to_hit() {
        assert!(throw_to_hit(10, 3) > throw_to_hit(10, 8));
    }

    #[test]
    fn test_breath_type_names() {
        assert_eq!(BreathType::Fire.name(), "fire");
        assert_eq!(BreathType::Cold.name(), "frost");
        assert_eq!(BreathType::Disintegration.name(), "a disintegration blast");
    }
}
