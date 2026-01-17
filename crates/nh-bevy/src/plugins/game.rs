//! Main game plugin that orchestrates all sub-plugins

use bevy::prelude::*;

use crate::plugins::{
    animation::AnimationPlugin, audio::AudioPlugin, camera::CameraPlugin, effects::EffectsPlugin,
    entities::EntityPlugin, fog::FogOfWarPlugin, input::InputPlugin, lighting::LightingPlugin,
    map::MapPlugin, navigation::NavigationPlugin, ui::UiPlugin,
};
use crate::resources::GameStateResource;

/// Main game plugin
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Initialize game state from nh-core
        let game_state = nh_core::GameState::default();
        app.insert_resource(GameStateResource(game_state));

        // Add sub-plugins in dependency order
        app.add_plugins((
            MapPlugin,
            EntityPlugin,
            CameraPlugin,
            InputPlugin,
            NavigationPlugin,
            UiPlugin,
            AnimationPlugin,
            AudioPlugin,
            FogOfWarPlugin,
            LightingPlugin,
            EffectsPlugin,
        ));

        // Game state management
        app.init_state::<AppState>();

        // Core systems
        app.add_systems(Update, handle_escape.run_if(in_state(AppState::Playing)));
    }
}

/// Application state
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    /// Main menu screen
    MainMenu,
    /// Active gameplay
    #[default]
    Playing,
    /// Game is paused
    Paused,
    /// Player died
    GameOver,
}

/// Handle ESC key - open pause menu instead of quitting
fn handle_escape(
    input: Res<ButtonInput<KeyCode>>,
    inv_state: Res<crate::plugins::ui::InventoryState>,
    dir_state: Res<crate::plugins::ui::DirectionSelectState>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    // Only pause if no UI panels are open
    if input.just_pressed(KeyCode::Escape) && !inv_state.open && !dir_state.active {
        next_state.set(AppState::Paused);
    }
}
