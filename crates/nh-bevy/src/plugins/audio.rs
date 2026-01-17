//! Audio system for sound effects and music
//!
//! This plugin provides:
//! - Sound effect events for game actions
//! - Music management (ambient, combat)
//! - Volume control integration with GameSettings

use bevy::prelude::*;

use crate::plugins::game::AppState;
use crate::plugins::ui::GameSettings;
use crate::resources::GameStateResource;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioState>()
            .add_event::<SoundEffect>()
            .add_systems(Update, play_sound_effects.run_if(in_state(AppState::Playing)))
            .add_systems(Update, detect_sound_triggers.run_if(in_state(AppState::Playing)));
    }
}

/// Current audio state
#[derive(Resource, Default)]
pub struct AudioState {
    /// Currently playing music track
    pub current_music: Option<MusicTrack>,
    /// Previous player HP (for damage sound detection)
    pub prev_player_hp: Option<i32>,
    /// Previous player position (for footstep detection)
    pub prev_player_pos: Option<(i8, i8)>,
}

/// Music tracks
#[derive(Clone, Copy, PartialEq)]
pub enum MusicTrack {
    Ambient,
    Combat,
    Menu,
    GameOver,
}

/// Sound effect events
#[derive(Event, Clone, Debug)]
pub enum SoundEffect {
    // Movement
    Footstep,
    FootstepWater,
    FootstepStone,

    // Combat
    Hit,
    Miss,
    CriticalHit,
    PlayerHurt,
    MonsterDeath,
    PlayerDeath,

    // Items
    Pickup,
    Drop,
    Equip,
    Unequip,
    Eat,
    Drink,

    // Environment
    DoorOpen,
    DoorClose,
    DoorLocked,
    StairsUp,
    StairsDown,

    // UI
    MenuSelect,
    MenuBack,

    // Special
    LevelUp,
    SecretFound,
    TrapTriggered,
}

impl SoundEffect {
    /// Get the asset path for this sound effect
    /// Returns None if the sound file doesn't exist yet
    pub fn asset_path(&self) -> Option<&'static str> {
        match self {
            SoundEffect::Footstep => Some("sounds/footstep.ogg"),
            SoundEffect::FootstepWater => Some("sounds/footstep_water.ogg"),
            SoundEffect::FootstepStone => Some("sounds/footstep_stone.ogg"),
            SoundEffect::Hit => Some("sounds/hit.ogg"),
            SoundEffect::Miss => Some("sounds/miss.ogg"),
            SoundEffect::CriticalHit => Some("sounds/critical.ogg"),
            SoundEffect::PlayerHurt => Some("sounds/hurt.ogg"),
            SoundEffect::MonsterDeath => Some("sounds/monster_death.ogg"),
            SoundEffect::PlayerDeath => Some("sounds/player_death.ogg"),
            SoundEffect::Pickup => Some("sounds/pickup.ogg"),
            SoundEffect::Drop => Some("sounds/drop.ogg"),
            SoundEffect::Equip => Some("sounds/equip.ogg"),
            SoundEffect::Unequip => Some("sounds/equip.ogg"),
            SoundEffect::Eat => Some("sounds/eat.ogg"),
            SoundEffect::Drink => Some("sounds/drink.ogg"),
            SoundEffect::DoorOpen => Some("sounds/door_open.ogg"),
            SoundEffect::DoorClose => Some("sounds/door_close.ogg"),
            SoundEffect::DoorLocked => Some("sounds/door_close.ogg"),
            SoundEffect::StairsUp => Some("sounds/stairs.ogg"),
            SoundEffect::StairsDown => Some("sounds/stairs.ogg"),
            SoundEffect::MenuSelect => Some("sounds/menu_select.ogg"),
            SoundEffect::MenuBack => Some("sounds/menu_back.ogg"),
            SoundEffect::LevelUp => Some("sounds/level_up.ogg"),
            SoundEffect::SecretFound => Some("sounds/secret.ogg"),
            SoundEffect::TrapTriggered => Some("sounds/trap.ogg"),
        }
    }
}

/// Play sound effects when events are triggered
fn play_sound_effects(
    mut sound_events: EventReader<SoundEffect>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
    mut commands: Commands,
) {
    for effect in sound_events.read() {
        if let Some(path) = effect.asset_path() {
            // Check if file exists to avoid Bevy's ERROR log spam
            // Bevy typically looks in 'assets/' relative to the executable or crate root
            let asset_path = std::path::Path::new("crates/nh-bevy/assets").join(path);
            if !asset_path.exists() {
                #[cfg(debug_assertions)]
                warn!("Sound file not found: {:?} (expected at {:?})", effect, asset_path);
                continue;
            }

            let sound = asset_server.load(path);
            commands.spawn((
                AudioPlayer::new(sound),
                PlaybackSettings {
                    volume: bevy::audio::Volume::new(settings.sfx_volume),
                    mode: bevy::audio::PlaybackMode::Despawn,
                    ..default()
                },
            ));
        }
    }
}

/// Detect game events that should trigger sounds
fn detect_sound_triggers(
    game_state: Res<GameStateResource>,
    mut audio_state: ResMut<AudioState>,
    mut sound_events: EventWriter<SoundEffect>,
) {
    if !game_state.is_changed() {
        return;
    }

    let state = &game_state.0;

    // Initialize state on first run
    if audio_state.prev_player_hp.is_none() {
        audio_state.prev_player_hp = Some(state.player.hp);
        audio_state.prev_player_pos = Some((state.player.pos.x, state.player.pos.y));
        return;
    }

    let prev_hp = audio_state.prev_player_hp.unwrap();
    let prev_pos = audio_state.prev_player_pos.unwrap();

    // Detect player movement (footsteps)
    let curr_pos = (state.player.pos.x, state.player.pos.y);
    if curr_pos != prev_pos {
        // Check terrain type for footstep sound
        let cell = state.current_level.cell(curr_pos.0 as usize, curr_pos.1 as usize);
        let sound = match cell.typ {
            nh_core::dungeon::CellType::Pool
            | nh_core::dungeon::CellType::Moat
            | nh_core::dungeon::CellType::Water => SoundEffect::FootstepWater,
            nh_core::dungeon::CellType::Stone => SoundEffect::FootstepStone,
            _ => SoundEffect::Footstep,
        };
        sound_events.send(sound);
    }

    // Detect player damage
    let curr_hp = state.player.hp;
    if curr_hp < prev_hp {
        sound_events.send(SoundEffect::PlayerHurt);
    }

    // Detect messages for other sound triggers
    for msg in &state.messages {
        let lower = msg.to_lowercase();
        if lower.contains("you hit") || lower.contains("hits") {
            sound_events.send(SoundEffect::Hit);
        } else if lower.contains("miss") {
            sound_events.send(SoundEffect::Miss);
        } else if lower.contains("killed") || lower.contains("destroyed") {
            sound_events.send(SoundEffect::MonsterDeath);
        } else if lower.contains("pick up") {
            sound_events.send(SoundEffect::Pickup);
        } else if lower.contains("drop") {
            sound_events.send(SoundEffect::Drop);
        } else if lower.contains("level up") || lower.contains("welcome to experience level") {
            sound_events.send(SoundEffect::LevelUp);
        } else if lower.contains("locked") {
            sound_events.send(SoundEffect::DoorLocked);
        } else if lower.contains("door opens") {
            sound_events.send(SoundEffect::DoorOpen);
        } else if lower.contains("door closes") {
            sound_events.send(SoundEffect::DoorClose);
        } else if lower.contains("secret") {
            sound_events.send(SoundEffect::SecretFound);
        } else if lower.contains("trap") {
            sound_events.send(SoundEffect::TrapTriggered);
        }
    }

    // Update state
    audio_state.prev_player_hp = Some(curr_hp);
    audio_state.prev_player_pos = Some(curr_pos);
}
