//! Ball and chain mechanics (ball.c)
//!
//! Handles iron ball movement, chain drag, and punishment mechanics.

use crate::dungeon::Level;
use crate::magic::scroll::PunishmentState;

/// Default iron ball weight (matches C: 480)
pub const IRON_BALL_WEIGHT: i32 = 480;

/// Default chain length (matches C: 4 squares)
pub const CHAIN_LENGTH: i32 = 4;

/// Heavy iron ball weight (matches C: 960 for "very heavy iron ball")
pub const HEAVY_BALL_WEIGHT: i32 = 960;

/// Check if the ball is draggable to a new position.
///
/// The iron ball can be dragged within chain_length squares of the player.
/// Walls and other obstacles may block the drag path (ball.c:ballfall, drag_ball).
pub fn can_drag_ball(
    punishment: &PunishmentState,
    player_x: i8,
    player_y: i8,
    dest_x: i8,
    dest_y: i8,
) -> bool {
    if !punishment.punished {
        return true; // Not punished, no restrictions
    }

    let (bx, by) = punishment.ball_position;
    let dist_to_ball = ((dest_x - bx) as i32).abs().max(((dest_y - by) as i32).abs());

    // Player must stay within chain_length of the ball
    if dist_to_ball > punishment.chain_length {
        return false;
    }

    // Also check that the new position isn't at the ball position
    // (the player can stand on the ball, but this affects behavior)
    let _ = (player_x, player_y); // Used for path checking in full implementation
    true
}

/// Calculate the new ball position when the player moves.
///
/// If the player moves away from the ball and exceeds chain length,
/// the ball is dragged along behind.
///
/// Returns (new_ball_x, new_ball_y, was_dragged).
pub fn drag_ball(
    punishment: &PunishmentState,
    new_player_x: i8,
    new_player_y: i8,
) -> (i8, i8, bool) {
    if !punishment.punished {
        return (0, 0, false);
    }

    let (bx, by) = punishment.ball_position;
    let dist = ((new_player_x - bx) as i32).abs().max(((new_player_y - by) as i32).abs());

    if dist <= punishment.chain_length {
        // Ball stays where it is
        (bx, by, false)
    } else {
        // Ball is dragged: move it one step toward the player
        let dx = (new_player_x - bx).signum();
        let dy = (new_player_y - by).signum();
        (bx + dx, by + dy, true)
    }
}

/// Ball falls through a trap or hole (ballfall from ball.c:208).
///
/// When the player falls through a hole, the ball falls too.
/// Returns the message to display.
pub fn ballfall(punishment: &PunishmentState) -> Option<String> {
    if !punishment.punished {
        return None;
    }

    Some("Your iron ball drags you down!".to_string())
}

/// Place the ball on the level (placebc from ball.c).
///
/// Used when entering a new level to position the ball and chain.
pub fn placebc(punishment: &mut PunishmentState, player_x: i8, player_y: i8) {
    if !punishment.punished {
        return;
    }
    // Place ball at player's feet
    punishment.ball_position = (player_x, player_y);
}

/// Remove the ball from the level (unplacebc from ball.c).
pub fn unplacebc(punishment: &mut PunishmentState) {
    if !punishment.punished {
        return;
    }
    punishment.ball_position = (0, 0);
}

/// Set up punishment (set_bc from ball.c).
pub fn set_bc(punishment: &mut PunishmentState, player_x: i8, player_y: i8, heavy: bool) {
    punishment.punished = true;
    punishment.ball_weight = if heavy { HEAVY_BALL_WEIGHT } else { IRON_BALL_WEIGHT };
    punishment.chain_length = CHAIN_LENGTH;
    punishment.ball_position = (player_x, player_y);
}

/// Remove punishment (unpunish from ball.c).
pub fn unpunish(punishment: &mut PunishmentState) {
    punishment.punished = false;
    punishment.ball_weight = 0;
    punishment.chain_length = 0;
    punishment.ball_position = (0, 0);
}

/// Check if the ball is at a given position.
pub fn ball_at(punishment: &PunishmentState, x: i8, y: i8) -> bool {
    punishment.punished && punishment.ball_position == (x, y)
}

/// Get the movement penalty for dragging the ball.
///
/// Returns extra movement cost (0 if not punished or ball at feet).
pub fn ball_movement_penalty(
    punishment: &PunishmentState,
    player_x: i8,
    player_y: i8,
    level: &Level,
) -> i32 {
    if !punishment.punished {
        return 0;
    }

    let (bx, by) = punishment.ball_position;
    if bx == player_x && by == player_y {
        return 0; // Ball at feet — carrying it
    }

    // Penalty increases with distance from ball
    let dist = ((player_x - bx) as i32).abs().max(((player_y - by) as i32).abs());

    let _ = level; // Would check terrain in full implementation

    // C uses a complex formula; simplified to linear penalty
    dist.min(punishment.chain_length)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn punished_state(x: i8, y: i8) -> PunishmentState {
        PunishmentState {
            punished: true,
            ball_weight: IRON_BALL_WEIGHT,
            chain_length: CHAIN_LENGTH,
            ball_position: (x, y),
        }
    }

    #[test]
    fn test_can_drag_ball_not_punished() {
        let state = PunishmentState::new();
        assert!(can_drag_ball(&state, 5, 5, 20, 20));
    }

    #[test]
    fn test_can_drag_ball_in_range() {
        let state = punished_state(5, 5);
        assert!(can_drag_ball(&state, 5, 5, 7, 7)); // 2 squares away
    }

    #[test]
    fn test_can_drag_ball_out_of_range() {
        let state = punished_state(5, 5);
        assert!(!can_drag_ball(&state, 5, 5, 15, 15)); // 10 squares away
    }

    #[test]
    fn test_drag_ball_stays() {
        let state = punished_state(5, 5);
        let (bx, by, dragged) = drag_ball(&state, 7, 7);
        assert_eq!((bx, by), (5, 5));
        assert!(!dragged);
    }

    #[test]
    fn test_drag_ball_follows() {
        let state = punished_state(5, 5);
        let (bx, by, dragged) = drag_ball(&state, 15, 5);
        assert_eq!(bx, 6); // Moved one step toward player
        assert_eq!(by, 5);
        assert!(dragged);
    }

    #[test]
    fn test_set_bc_and_unpunish() {
        let mut state = PunishmentState::new();
        set_bc(&mut state, 10, 10, false);
        assert!(state.punished);
        assert_eq!(state.ball_weight, IRON_BALL_WEIGHT);

        unpunish(&mut state);
        assert!(!state.punished);
        assert_eq!(state.ball_weight, 0);
    }

    #[test]
    fn test_set_bc_heavy() {
        let mut state = PunishmentState::new();
        set_bc(&mut state, 10, 10, true);
        assert_eq!(state.ball_weight, HEAVY_BALL_WEIGHT);
    }

    #[test]
    fn test_ball_at() {
        let state = punished_state(5, 5);
        assert!(ball_at(&state, 5, 5));
        assert!(!ball_at(&state, 6, 5));
    }

    #[test]
    fn test_ballfall_punished() {
        let state = punished_state(5, 5);
        assert!(ballfall(&state).is_some());
    }

    #[test]
    fn test_ballfall_not_punished() {
        let state = PunishmentState::new();
        assert!(ballfall(&state).is_none());
    }

    #[test]
    fn test_placebc() {
        let mut state = punished_state(0, 0);
        placebc(&mut state, 10, 12);
        assert_eq!(state.ball_position, (10, 12));
    }
}
