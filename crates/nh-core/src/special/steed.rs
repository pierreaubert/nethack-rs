//! Steed/riding mechanics (steed.c)
//!
//! Handles mounting, dismounting, and riding behavior.

use crate::monster::Monster;
use crate::player::You;

/// Result of a mount attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountResult {
    /// Successfully mounted
    Mounted(String),
    /// Can't mount this creature
    CantMount(String),
    /// No creature to mount
    NoTarget,
    /// Player is in a state that prevents mounting
    PlayerCantRide(String),
}

/// Result of a dismount attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DismountResult {
    /// Successfully dismounted
    Dismounted(String),
    /// Can't dismount (e.g., trapped)
    CantDismount(String),
    /// Not currently riding
    NotRiding,
}

/// Check if a monster can be ridden (can_saddle from steed.c:95).
///
/// Requirements: monster must be tame, large enough, and capable of carrying the player.
pub fn can_ride(monster: &Monster, player: &You) -> bool {
    // Must be tame
    if !monster.state.peaceful {
        return false;
    }

    // Can't ride tiny or small monsters (need at least medium/large)
    // Monster level roughly correlates to size
    if monster.level < 3 {
        return false;
    }

    // Can't ride while levitating or polymorphed into something huge
    if player.monster_num.is_some() {
        return false;
    }

    // Can't ride while swallowed
    if player.swallowed {
        return false;
    }

    true
}

/// Attempt to mount a steed (doride from steed.c:200).
pub fn doride(player: &You, monster: &Monster) -> MountResult {
    if player.steed.is_some() {
        return MountResult::CantMount("You are already riding.".to_string());
    }

    if player.swallowed {
        return MountResult::PlayerCantRide("You can't ride while engulfed!".to_string());
    }

    if player.underwater {
        return MountResult::PlayerCantRide("You can't ride underwater!".to_string());
    }

    if !can_ride(monster, player) {
        return MountResult::CantMount(format!(
            "You can't ride {}.", monster.name
        ));
    }

    MountResult::Mounted(format!("You mount {}.", monster.name))
}

/// Dismount from steed (dismount_steed from steed.c:338).
pub fn dismount(player: &You) -> DismountResult {
    if player.steed.is_none() {
        return DismountResult::NotRiding;
    }

    if player.utrap > 0 {
        return DismountResult::CantDismount(
            "You can't dismount while trapped!".to_string()
        );
    }

    DismountResult::Dismounted("You dismount.".to_string())
}

/// Get the speed bonus from riding (rider_speed from steed.c).
///
/// Riding increases movement speed based on steed's speed.
pub fn rider_speed_bonus(steed_speed: i32) -> i32 {
    // Riding gives a fraction of the steed's speed as bonus
    steed_speed / 4
}

/// Check if riding affects the player's armor class.
///
/// Riding provides some protection (skill-dependent in C).
pub fn riding_ac_bonus(riding_skill: i32) -> i8 {
    match riding_skill {
        0 => 0,       // Unskilled
        1 => 0,       // Basic
        2 => -1,      // Skilled
        _ => -2,      // Expert
    }
}

/// Get landing position when forcibly dismounted (landing_spot from steed.c:700).
///
/// Tries to find an adjacent safe square. Returns offset (dx, dy).
pub fn landing_spot(
    player_x: i8,
    player_y: i8,
    level: &crate::dungeon::Level,
) -> Option<(i8, i8)> {
    // Try all 8 adjacent squares
    for dx in -1..=1i8 {
        for dy in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = player_x + dx;
            let ny = player_y + dy;
            if level.is_valid_pos(nx, ny) {
                let cell = &level.cells[nx as usize][ny as usize];
                if cell.typ.is_passable() {
                    return Some((dx, dy));
                }
            }
        }
    }
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;
    use crate::player::{Gender, Race, Role};

    fn test_player() -> You {
        You::new("Test".into(), Role::Valkyrie, Race::Human, Gender::Female)
    }

    fn test_monster(_level: u8, _peaceful: bool) -> Monster {
        Monster::new(
            MonsterId(1),
            0, // monster_type
            5, 5, // x, y
        )
    }

    #[test]
    fn test_doride_not_riding() {
        let player = test_player();
        let mut monster = test_monster(5, true);
        monster.state.peaceful = true;
        monster.level = 5;
        let result = doride(&player, &monster);
        assert!(matches!(result, MountResult::Mounted(_)));
    }

    #[test]
    fn test_doride_already_riding() {
        let mut player = test_player();
        player.steed = Some(MonsterId(99));
        let monster = test_monster(5, true);
        let result = doride(&player, &monster);
        assert!(matches!(result, MountResult::CantMount(_)));
    }

    #[test]
    fn test_doride_swallowed() {
        let mut player = test_player();
        player.swallowed = true;
        let monster = test_monster(5, true);
        let result = doride(&player, &monster);
        assert!(matches!(result, MountResult::PlayerCantRide(_)));
    }

    #[test]
    fn test_dismount_not_riding() {
        let player = test_player();
        let result = dismount(&player);
        assert!(matches!(result, DismountResult::NotRiding));
    }

    #[test]
    fn test_dismount_ok() {
        let mut player = test_player();
        player.steed = Some(MonsterId(1));
        let result = dismount(&player);
        assert!(matches!(result, DismountResult::Dismounted(_)));
    }

    #[test]
    fn test_rider_speed_bonus() {
        assert_eq!(rider_speed_bonus(12), 3);
        assert_eq!(rider_speed_bonus(24), 6);
    }

    #[test]
    fn test_riding_ac_bonus() {
        assert_eq!(riding_ac_bonus(0), 0);
        assert_eq!(riding_ac_bonus(2), -1);
        assert_eq!(riding_ac_bonus(3), -2);
    }

    #[test]
    fn test_can_ride_hostile() {
        let player = test_player();
        let mut monster = test_monster(5, false);
        monster.state.peaceful = false;
        monster.level = 5;
        assert!(!can_ride(&monster, &player));
    }

    #[test]
    fn test_can_ride_too_small() {
        let player = test_player();
        let mut monster = test_monster(1, true);
        monster.state.peaceful = true;
        monster.level = 1;
        assert!(!can_ride(&monster, &player));
    }
}
