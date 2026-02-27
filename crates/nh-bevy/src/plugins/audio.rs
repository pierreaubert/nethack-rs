//! Audio system for sound effects and music
//!
//! This plugin provides:
//! - Sound effect events for game actions
//! - Music management (ambient, combat)
//! - Volume control integration with GameSettings

use bevy::prelude::*;

use crate::plugins::game::AppState;
use crate::plugins::ui::GameSettings;
use crate::resources::{AssetsConfig, GameStateResource};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioState>()
            .add_message::<SoundEffect>()
            .add_systems(
                Update,
                play_sound_effects.run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                detect_sound_triggers.run_if(in_state(AppState::Playing)),
            );
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
#[derive(Message, Clone, Debug)]
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
        // Use .wav files from game_sfx/ - the .ogg files use FLAC codec which Bevy doesn't support
        match self {
            SoundEffect::Footstep => Some("sounds/game_sfx/footstep.wav"),
            SoundEffect::FootstepWater => Some("sounds/game_sfx/footstep_water.wav"),
            SoundEffect::FootstepStone => Some("sounds/game_sfx/footstep_stone.wav"),
            SoundEffect::Hit => Some("sounds/game_sfx/hit.wav"),
            SoundEffect::Miss => Some("sounds/game_sfx/miss.wav"),
            SoundEffect::CriticalHit => Some("sounds/game_sfx/critical.wav"),
            SoundEffect::PlayerHurt => Some("sounds/game_sfx/hurt.wav"),
            SoundEffect::MonsterDeath => Some("sounds/game_sfx/monster_death.wav"),
            SoundEffect::PlayerDeath => Some("sounds/game_sfx/player_death.wav"),
            SoundEffect::Pickup => Some("sounds/game_sfx/pickup.wav"),
            SoundEffect::Drop => Some("sounds/game_sfx/drop.wav"),
            SoundEffect::Equip => Some("sounds/game_sfx/equip.wav"),
            SoundEffect::Unequip => Some("sounds/game_sfx/equip.wav"),
            SoundEffect::Eat => Some("sounds/game_sfx/eat.wav"),
            SoundEffect::Drink => Some("sounds/game_sfx/drink.wav"),
            SoundEffect::DoorOpen => Some("sounds/game_sfx/door_open.wav"),
            SoundEffect::DoorClose => Some("sounds/game_sfx/door_close.wav"),
            SoundEffect::DoorLocked => Some("sounds/game_sfx/door_close.wav"),
            SoundEffect::StairsUp => Some("sounds/game_sfx/stairs.wav"),
            SoundEffect::StairsDown => Some("sounds/game_sfx/stairs.wav"),
            SoundEffect::MenuSelect => Some("sounds/game_sfx/menu_select.wav"),
            SoundEffect::MenuBack => Some("sounds/game_sfx/menu_back.wav"),
            SoundEffect::LevelUp => Some("sounds/game_sfx/level_up.wav"),
            SoundEffect::SecretFound => Some("sounds/game_sfx/secret.wav"),
            SoundEffect::TrapTriggered => Some("sounds/game_sfx/trap.wav"),
        }
    }
}

/// Play sound effects when events are triggered
fn play_sound_effects(
    mut sound_events: MessageReader<SoundEffect>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
    assets_config: Option<Res<AssetsConfig>>,
    mut commands: Commands,
    mut warned: Local<std::collections::HashSet<String>>,
) {
    // Skip audio if volume is zero
    if settings.sfx_volume <= 0.0 {
        sound_events.clear();
        return;
    }

    let base_path = assets_config
        .map(|c| c.base_path.clone())
        .unwrap_or_else(|| std::path::PathBuf::from("assets"));

    for effect in sound_events.read() {
        if let Some(path) = effect.asset_path() {
            // Check if file exists in the asset folder
            let asset_path = base_path.join(path);
            if !asset_path.exists() {
                if !warned.contains(path) {
                    warn!("Sound file not found at {:?}: {:?}", asset_path, effect);
                    warned.insert(path.to_string());
                }
                continue;
            }

            let sound = asset_server.load(path);
            commands.spawn((
                AudioPlayer::new(sound),
                PlaybackSettings {
                    volume: bevy::audio::Volume::Linear(settings.sfx_volume),
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
    mut sound_events: MessageWriter<SoundEffect>,
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
        let cell = state
            .current_level
            .cell(curr_pos.0 as usize, curr_pos.1 as usize);
        let sound = match cell.typ {
            nh_core::dungeon::CellType::Pool
            | nh_core::dungeon::CellType::Moat
            | nh_core::dungeon::CellType::Water => SoundEffect::FootstepWater,
            nh_core::dungeon::CellType::Stone => SoundEffect::FootstepStone,
            _ => SoundEffect::Footstep,
        };
        sound_events.write(sound);
    }

    // Detect player damage
    let curr_hp = state.player.hp;
    if curr_hp < prev_hp {
        sound_events.write(SoundEffect::PlayerHurt);
    }

    // Detect messages for other sound triggers
    for msg in &state.messages {
        let lower = msg.to_lowercase();
        if lower.contains("you hit") || lower.contains("hits") {
            sound_events.write(SoundEffect::Hit);
        } else if lower.contains("miss") {
            sound_events.write(SoundEffect::Miss);
        } else if lower.contains("killed") || lower.contains("destroyed") {
            sound_events.write(SoundEffect::MonsterDeath);
        } else if lower.contains("pick up") {
            sound_events.write(SoundEffect::Pickup);
        } else if lower.contains("drop") {
            sound_events.write(SoundEffect::Drop);
        } else if lower.contains("level up") || lower.contains("welcome to experience level") {
            sound_events.write(SoundEffect::LevelUp);
        } else if lower.contains("locked") {
            sound_events.write(SoundEffect::DoorLocked);
        } else if lower.contains("door opens") {
            sound_events.write(SoundEffect::DoorOpen);
        } else if lower.contains("door closes") {
            sound_events.write(SoundEffect::DoorClose);
        } else if lower.contains("secret") {
            sound_events.write(SoundEffect::SecretFound);
        } else if lower.contains("trap") {
            sound_events.write(SoundEffect::TrapTriggered);
        }
    }

    // Update state
    audio_state.prev_player_hp = Some(curr_hp);
    audio_state.prev_player_pos = Some(curr_pos);
}
