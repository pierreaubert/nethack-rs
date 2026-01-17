//! Prayer system (pray.c)

use crate::action::ActionResult;
use crate::gameloop::GameState;

/// Pray to the player's god
pub fn do_pray(state: &mut GameState) -> ActionResult {
    if state.player.prayer_timeout > 0 {
        state.message("You are not ready to pray yet.");
        return ActionResult::NoTime;
    }

    state.player.prayer_timeout = 300 + state.rng.rn2(200) as i32;

    if state.player.god_anger > 0 {
        state.message("You sense your god's displeasure.");
        state.player.god_anger = (state.player.god_anger - 1).max(0);
        return ActionResult::Success;
    }

    let hp_percent = (state.player.hp * 100) / state.player.hp_max.max(1);
    if hp_percent < 10 || state.player.hp < 5 {
        state.message("You feel a surge of divine power!");
        state.player.hp = state.player.hp_max;
        return ActionResult::Success;
    }

    state.message("You feel at peace.");
    ActionResult::Success
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::GameRng;

    #[test]
    fn test_prayer_timeout_blocks_prayer() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.prayer_timeout = 100;
        let result = do_pray(&mut state);
        assert!(matches!(result, ActionResult::NoTime));
    }

    #[test]
    fn test_prayer_sets_timeout() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.prayer_timeout = 0;
        let result = do_pray(&mut state);
        assert!(matches!(result, ActionResult::Success));
        assert!(state.player.prayer_timeout > 0);
    }

    #[test]
    fn test_desperate_prayer_heals() {
        let mut state = GameState::new(GameRng::from_entropy());
        state.player.hp = 1;
        state.player.hp_max = 50;
        state.player.prayer_timeout = 0;
        state.player.god_anger = 0;
        let result = do_pray(&mut state);
        assert!(matches!(result, ActionResult::Success));
        assert_eq!(state.player.hp, state.player.hp_max);
    }
}
