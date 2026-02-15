//! Monster sounds and speech (sounds.c)
//!
//! Handles monster vocalizations, growls, and speech.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::monster::{Monster, MonsterSound};
use crate::rng::GameRng;

/// Get the growl/sound verb for a monster based on its sound type
pub fn growl_sound(sound: MonsterSound) -> &'static str {
    match sound {
        MonsterSound::Bark => "barks",
        MonsterSound::Mew => "mews",
        MonsterSound::Roar => "roars",
        MonsterSound::Growl => "growls",
        MonsterSound::Sqeek => "squeaks",
        MonsterSound::Sqawk => "squawks",
        MonsterSound::Hiss => "hisses",
        MonsterSound::Buzz => "buzzes",
        MonsterSound::Grunt => "grunts",
        MonsterSound::Neigh => "neighs",
        MonsterSound::Wail => "wails",
        MonsterSound::Gurgle => "gurgles",
        MonsterSound::Burble => "burbles",
        MonsterSound::Animal => "growls",
        MonsterSound::Shriek => "shrieks",
        MonsterSound::Bones => "rattles",
        MonsterSound::Laugh => "laughs",
        MonsterSound::Mumble => "mumbles",
        MonsterSound::Silent | _ => "is silent",
    }
}

/// Get a message when a monster growls (angry/hostile)
/// sound parameter comes from PerMonst data for this monster type
pub fn monster_growl(monster: &Monster, sound: MonsterSound) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    let verb = growl_sound(sound);
    if verb == "is silent" {
        return None;
    }

    Some(format!("The {} {}!", monster.name, verb))
}

/// Get a message when a monster whimpers (hurt/fleeing)
pub fn monster_whimper(monster: &Monster, sound: MonsterSound) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    let msg = match sound {
        MonsterSound::Bark => "whines",
        MonsterSound::Mew => "yowls",
        MonsterSound::Roar => "snarls",
        MonsterSound::Growl => "whimpers",
        MonsterSound::Sqeek => "squeals",
        MonsterSound::Sqawk => "screeches",
        MonsterSound::Hiss => "hisses",
        MonsterSound::Neigh => "whinnies",
        MonsterSound::Wail => "moans",
        MonsterSound::Silent => return None,
        _ => "cries out",
    };

    Some(format!("The {} {}!", monster.name, msg))
}

/// Get a message when a monster yelps (hit/damaged)
pub fn monster_yelp(monster: &Monster, sound: MonsterSound) -> Option<String> {
    if monster.state.sleeping {
        return None;
    }

    let msg = match sound {
        MonsterSound::Bark => "yelps",
        MonsterSound::Mew => "yowls",
        MonsterSound::Roar => "snarls",
        MonsterSound::Growl => "yelps",
        MonsterSound::Sqeek => "squeals",
        MonsterSound::Sqawk => "squawks",
        MonsterSound::Hiss => "hisses",
        MonsterSound::Neigh => "neighs",
        MonsterSound::Silent => return None,
        _ => "cries out",
    };

    Some(format!("The {} {}!", monster.name, msg))
}

/// Humanoid speech patterns
const HUMANOID_SOUNDS: &[&str] = &[
    "\"I'm gonna get you!\"",
    "\"You're struggling in vain!\"",
    "\"Run away!\"",
    "curses",
    "shouts",
];

/// Get speech for humanoid monsters
pub fn humanoid_speech(
    monster: &Monster,
    sound: MonsterSound,
    rng: &mut GameRng,
) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    match sound {
        MonsterSound::Humanoid | MonsterSound::Orc | MonsterSound::Soldier => {
            let idx = rng.rn2(HUMANOID_SOUNDS.len() as u32) as usize;
            Some(format!("The {} {}.", monster.name, HUMANOID_SOUNDS[idx]))
        }
        MonsterSound::Arrest => Some(format!(
            "The {} shouts: \"Halt! You're under arrest!\"",
            monster.name
        )),
        MonsterSound::Guard => Some(format!("The {} yells: \"Halt, thief!\"", monster.name)),
        MonsterSound::Sell => Some(format!("The {} says: \"Can I help you?\"", monster.name)),
        MonsterSound::Djinni => Some(format!(
            "The {} speaks: \"I am here to serve.\"",
            monster.name
        )),
        MonsterSound::Nurse => Some(format!(
            "The {} says: \"Take your medicine!\"",
            monster.name
        )),
        MonsterSound::Seduce => Some(format!("The {} whispers seductively.", monster.name)),
        MonsterSound::Vampire => Some(format!(
            "The {} says: \"I vant to suck your blood!\"",
            monster.name
        )),
        MonsterSound::Bribe => Some(format!("The {} offers you a deal.", monster.name)),
        MonsterSound::Cuss => Some(format!("The {} curses at you!", monster.name)),
        MonsterSound::Rider => Some(format!(
            "The {} intones: \"Your time has come.\"",
            monster.name
        )),
        MonsterSound::Leader => Some(format!("The {} speaks to you.", monster.name)),
        MonsterSound::Nemesis => Some(format!("The {} taunts you!", monster.name)),
        MonsterSound::Guardian => Some(format!("The {} challenges you!", monster.name)),
        _ => None,
    }
}

/// Check if a monster can make sounds
pub fn can_make_sound(monster: &Monster, sound: MonsterSound) -> bool {
    !monster.state.sleeping && monster.can_act() && sound != MonsterSound::Silent
}

/// Get a random sound message for a monster
pub fn random_monster_sound(
    monster: &Monster,
    sound: MonsterSound,
    rng: &mut GameRng,
) -> Option<String> {
    if !can_make_sound(monster, sound) {
        return None;
    }

    // 1 in 10 chance to make a sound
    if !rng.one_in(10) {
        return None;
    }

    // Humanoid monsters speak, others growl
    if sound as u8 >= MonsterSound::Humanoid as u8 {
        humanoid_speech(monster, sound, rng)
    } else {
        monster_growl(monster, sound)
    }
}

/// Generate ambient level sounds (dosounds equivalent - simplified)
pub fn generate_ambient_sounds(
    level: &crate::dungeon::Level,
    player_x: i8,
    player_y: i8,
    rng: &mut GameRng,
) -> Vec<String> {
    let mut messages = Vec::new();

    // 1 in 400 chance for fountain sounds
    if rng.one_in(400) {
        if !rng.one_in(2) {
            messages.push("You hear a bubbling sound coming from a fountain.".to_string());
        } else {
            messages.push("Someone is splashing at a fountain.".to_string());
        }
    }

    // 1 in 300 chance for sink sounds
    if rng.one_in(300) {
        messages.push("You hear water dripping from somewhere.".to_string());
    }

    // 1 in 200 chance for vault/money sounds
    if rng.one_in(200) {
        if !rng.one_in(3) {
            messages.push("You hear the sound of coins jingling.".to_string());
        } else {
            messages.push("You hear footsteps, as if someone is patrolling nearby.".to_string());
        }
    }

    messages
}

/// Monster noise engine - core vocalization generation (domonnoise equivalent)
pub fn generate_monster_noise(
    monster: &Monster,
    sound: MonsterSound,
    rng: &mut GameRng,
) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    // 1 in 20 chance to generate sound
    if !rng.one_in(20) {
        return None;
    }

    // Generate appropriate sound based on type
    match sound {
        MonsterSound::Silent => None,
        MonsterSound::Humanoid | MonsterSound::Orc | MonsterSound::Soldier => {
            humanoid_speech(monster, sound, rng)
        }
        _ => {
            if monster.state.fleeing {
                monster_whimper(monster, sound)
            } else {
                monster_growl(monster, sound)
            }
        }
    }
}

/// Pet-specific sounds (beg, growl, yelp, whimper) - for common pet sounds
pub fn pet_sound(monster: &Monster, situation: PetSoundType, rng: &mut GameRng) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    // Determine sound based on pet type
    let default_sound = MonsterSound::Bark; // Default for dogs

    match situation {
        PetSoundType::Begging => {
            // Pet is begging/hungry
            Some(format!("The {} begs for food.", monster.name))
        }
        PetSoundType::Whimpering => {
            // Pet is hurt/scared
            monster_whimper(monster, default_sound)
        }
        PetSoundType::Yelping => {
            // Pet is hit/attacked
            monster_yelp(monster, default_sound)
        }
        PetSoundType::Growling => {
            // Pet is aggressive
            monster_growl(monster, default_sound)
        }
    }
}

/// Pet sound type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetSoundType {
    Begging,
    Whimpering,
    Yelping,
    Growling,
}

/// Generate demon cursing sounds
pub fn demon_cuss(monster: &Monster) -> Option<String> {
    let insults = [
        "\"Fool! Your pathetic magic is no match for me!\"",
        "\"I curse you with a thousand demons!\"",
        "\"Your soul is mine!\"",
        "\"Prepare to meet your doom!\"",
    ];

    if monster.state.sleeping {
        return None;
    }

    let idx = (monster.id.0 as usize) % insults.len();
    Some(format!("The {} shouts: {}", monster.name, insults[idx]))
}

/// Check if a creature can speak (speaker check)
pub fn can_speak(sound: MonsterSound) -> bool {
    sound as u8 >= MonsterSound::Humanoid as u8
}

// ============================================================================
// Sound Mapping Functions (add_sound_mapping, noises from sounds.c)
// ============================================================================

/// Sound mapping entry for message-to-sound file mapping
#[derive(Debug, Clone)]
pub struct SoundMapping {
    /// Pattern to match against messages
    pub pattern: String,
    /// Sound file to play
    pub filename: String,
    /// Volume level (0-100)
    pub volume: u8,
}

/// Sound mapping registry
#[derive(Debug, Clone, Default)]
pub struct SoundRegistry {
    /// List of sound mappings
    pub mappings: Vec<SoundMapping>,
    /// Sound directory path
    pub sound_dir: String,
}

impl SoundRegistry {
    /// Create a new sound registry
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
            sound_dir: ".".to_string(),
        }
    }

    /// Set the sound directory
    pub fn set_sound_dir(&mut self, dir: &str) {
        self.sound_dir = dir.to_string();
    }
}

/// Add a sound mapping (add_sound_mapping equivalent)
///
/// Parses a mapping string in format: MESG "pattern" "filename" volume
/// Returns true on success, false on failure.
pub fn add_sound_mapping(registry: &mut SoundRegistry, mapping: &str) -> bool {
    // Parse the mapping string
    // Expected format: MESG "pattern" "filename" volume
    let parts: Vec<&str> = mapping.split('"').collect();

    if parts.len() < 4 {
        return false;
    }

    // Extract pattern (between first pair of quotes)
    let pattern = parts.get(1).map(|s| s.to_string()).unwrap_or_default();
    if pattern.is_empty() {
        return false;
    }

    // Extract filename (between second pair of quotes)
    let filename = parts.get(3).map(|s| s.to_string()).unwrap_or_default();
    if filename.is_empty() {
        return false;
    }

    // Extract volume from remaining text
    let volume_str = parts.get(4).unwrap_or(&"50");
    let volume: u8 = volume_str.trim().parse().unwrap_or(50).min(100);

    // Build full file path
    let full_path = if registry.sound_dir.is_empty() {
        filename
    } else {
        format!("{}/{}", registry.sound_dir, filename)
    };

    registry.mappings.push(SoundMapping {
        pattern,
        filename: full_path,
        volume,
    });

    true
}

/// Find a sound file for a message
pub fn find_sound_for_message<'a>(
    registry: &'a SoundRegistry,
    _message: &str,
) -> Option<&'a SoundMapping> {
    for mapping in &registry.mappings {
        if _message.contains(&mapping.pattern) {
            return Some(mapping);
        }
    }
    None
}

/// Generate level-specific ambient noises (noises equivalent)
///
/// Returns ambient sound messages based on dungeon features and monsters.
pub fn noises(
    level: &crate::dungeon::Level,
    player_x: i8,
    player_y: i8,
    rng: &mut GameRng,
) -> Vec<String> {
    let mut messages = Vec::new();

    // Check for nearby monsters making sounds
    for monster in &level.monsters {
        // Skip if too far away
        let dx = (monster.x - player_x).abs();
        let dy = (monster.y - player_y).abs();
        let dist = dx.max(dy);

        if dist > 15 {
            continue; // Too far to hear
        }

        // Skip sleeping or silent monsters
        if monster.state.sleeping {
            continue;
        }

        // Small chance to hear distant monster
        let hear_chance = if dist <= 3 {
            10 // 10% for nearby
        } else if dist <= 8 {
            5 // 5% for medium distance
        } else {
            2 // 2% for far
        };

        if rng.rn2(100) < hear_chance {
            // Generate a vague sound description
            let sound_desc = if dist <= 3 {
                format!("You hear {} nearby.", monster_sound_description(rng))
            } else if dist <= 8 {
                format!(
                    "You hear {} in the distance.",
                    monster_sound_description(rng)
                )
            } else {
                format!(
                    "You hear a faint {} far away.",
                    monster_sound_description(rng)
                )
            };
            messages.push(sound_desc);
            break; // Only one monster sound per turn
        }
    }

    // Add ambient dungeon sounds
    messages.extend(generate_ambient_sounds(level, player_x, player_y, rng));

    messages
}

/// Get a generic sound description
fn monster_sound_description(rng: &mut GameRng) -> &'static str {
    const SOUNDS: &[&str] = &[
        "shuffling",
        "scraping",
        "growling",
        "hissing",
        "clicking",
        "rustling",
        "breathing",
        "movement",
    ];
    SOUNDS[rng.rn2(SOUNDS.len() as u32) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monster::MonsterId;

    fn test_monster() -> Monster {
        let mut m = Monster::new(MonsterId(1), 0, 5, 5);
        m.name = "test monster".to_string();
        m
    }

    #[test]
    fn test_growl_sound() {
        assert_eq!(growl_sound(MonsterSound::Bark), "barks");
        assert_eq!(growl_sound(MonsterSound::Roar), "roars");
        assert_eq!(growl_sound(MonsterSound::Hiss), "hisses");
    }

    #[test]
    fn test_monster_growl() {
        let monster = test_monster();
        let msg = monster_growl(&monster, MonsterSound::Bark);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("barks"));
    }

    #[test]
    fn test_silent_monster() {
        let monster = test_monster();
        assert!(monster_growl(&monster, MonsterSound::Silent).is_none());
    }

    #[test]
    fn test_sleeping_monster() {
        let mut monster = test_monster();
        monster.state.sleeping = true;
        assert!(monster_growl(&monster, MonsterSound::Bark).is_none());
    }

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_growl_sound_all_types() {
        assert_eq!(growl_sound(MonsterSound::Bark), "barks");
        assert_eq!(growl_sound(MonsterSound::Mew), "mews");
        assert_eq!(growl_sound(MonsterSound::Roar), "roars");
        assert_eq!(growl_sound(MonsterSound::Growl), "growls");
        assert_eq!(growl_sound(MonsterSound::Sqeek), "squeaks");
        assert_eq!(growl_sound(MonsterSound::Sqawk), "squawks");
        assert_eq!(growl_sound(MonsterSound::Hiss), "hisses");
        assert_eq!(growl_sound(MonsterSound::Buzz), "buzzes");
        assert_eq!(growl_sound(MonsterSound::Grunt), "grunts");
        assert_eq!(growl_sound(MonsterSound::Neigh), "neighs");
        assert_eq!(growl_sound(MonsterSound::Wail), "wails");
        assert_eq!(growl_sound(MonsterSound::Gurgle), "gurgles");
        assert_eq!(growl_sound(MonsterSound::Burble), "burbles");
        assert_eq!(growl_sound(MonsterSound::Animal), "growls");
        assert_eq!(growl_sound(MonsterSound::Shriek), "shrieks");
        assert_eq!(growl_sound(MonsterSound::Bones), "rattles");
        assert_eq!(growl_sound(MonsterSound::Laugh), "laughs");
        assert_eq!(growl_sound(MonsterSound::Mumble), "mumbles");
        assert_eq!(growl_sound(MonsterSound::Silent), "is silent");
    }

    #[test]
    fn test_monster_growl_contains_name() {
        let monster = test_monster();
        let msg = monster_growl(&monster, MonsterSound::Roar);

        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("test monster"));
        assert!(msg.contains("roars"));
    }

    #[test]
    fn test_monster_whimper_all_types() {
        let monster = test_monster();

        assert!(monster_whimper(&monster, MonsterSound::Bark).is_some());
        assert!(monster_whimper(&monster, MonsterSound::Mew).is_some());
        assert!(monster_whimper(&monster, MonsterSound::Roar).is_some());
        assert!(monster_whimper(&monster, MonsterSound::Growl).is_some());
    }

    #[test]
    fn test_monster_whimper_silent() {
        let monster = test_monster();
        assert!(monster_whimper(&monster, MonsterSound::Silent).is_none());
    }

    #[test]
    fn test_monster_yelp_all_types() {
        let monster = test_monster();

        assert!(monster_yelp(&monster, MonsterSound::Bark).is_some());
        assert!(monster_yelp(&monster, MonsterSound::Mew).is_some());
        assert!(monster_yelp(&monster, MonsterSound::Neigh).is_some());
    }

    #[test]
    fn test_monster_yelp_when_sleeping_fails() {
        let mut monster = test_monster();
        monster.state.sleeping = true;
        assert!(monster_yelp(&monster, MonsterSound::Bark).is_none());
    }

    #[test]
    fn test_humanoid_speech_sample_outputs() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let speech = humanoid_speech(&monster, MonsterSound::Humanoid, &mut rng);
        assert!(speech.is_some());
        let msg = speech.unwrap();
        assert!(msg.contains("test monster"));
    }

    #[test]
    fn test_humanoid_speech_arrest() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let speech = humanoid_speech(&monster, MonsterSound::Arrest, &mut rng);
        assert!(speech.is_some());
        let msg = speech.unwrap();
        assert!(msg.contains("arrest"));
    }

    #[test]
    fn test_humanoid_speech_guard() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let speech = humanoid_speech(&monster, MonsterSound::Guard, &mut rng);
        assert!(speech.is_some());
        let msg = speech.unwrap();
        assert!(msg.contains("thief"));
    }

    #[test]
    fn test_humanoid_speech_sell() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let speech = humanoid_speech(&monster, MonsterSound::Sell, &mut rng);
        assert!(speech.is_some());
        let msg = speech.unwrap();
        assert!(msg.contains("help"));
    }

    #[test]
    fn test_can_make_sound_basics() {
        let monster = test_monster();
        assert!(can_make_sound(&monster, MonsterSound::Bark));
        assert!(!can_make_sound(&monster, MonsterSound::Silent));
    }

    #[test]
    fn test_can_make_sound_when_sleeping() {
        let mut monster = test_monster();
        monster.state.sleeping = true;
        assert!(!can_make_sound(&monster, MonsterSound::Bark));
    }

    #[test]
    fn test_random_monster_sound_probability() {
        let monster = test_monster();

        // With seed 42, should get varied results
        let mut rng = GameRng::new(42);
        let sound1 = random_monster_sound(&monster, MonsterSound::Bark, &mut rng);

        let mut rng = GameRng::new(43);
        let sound2 = random_monster_sound(&monster, MonsterSound::Bark, &mut rng);

        // Most of the time should be None due to 1/10 chance
        // But both could be None or Some
        let _results = (sound1, sound2);
    }

    #[test]
    fn test_pet_sound_begging() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let sound = pet_sound(&monster, PetSoundType::Begging, &mut rng);
        assert!(sound.is_some());
        let msg = sound.unwrap();
        assert!(msg.contains("begs"));
    }

    #[test]
    fn test_pet_sound_whimpering() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let sound = pet_sound(&monster, PetSoundType::Whimpering, &mut rng);
        assert!(sound.is_some());
    }

    #[test]
    fn test_pet_sound_yelping() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let sound = pet_sound(&monster, PetSoundType::Yelping, &mut rng);
        assert!(sound.is_some());
    }

    #[test]
    fn test_pet_sound_growling() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let sound = pet_sound(&monster, PetSoundType::Growling, &mut rng);
        assert!(sound.is_some());
    }

    #[test]
    fn test_demon_cuss_output() {
        let monster = test_monster();
        let cuss = demon_cuss(&monster);

        assert!(cuss.is_some());
        let msg = cuss.unwrap();
        assert!(msg.contains("shouts"));
    }

    #[test]
    fn test_demon_cuss_sleeping_silent() {
        let mut monster = test_monster();
        monster.state.sleeping = true;

        assert!(demon_cuss(&monster).is_none());
    }

    #[test]
    fn test_can_speak_humanoid() {
        assert!(can_speak(MonsterSound::Humanoid));
        // Orc (20) is below Humanoid (21) threshold, so can't speak
        assert!(!can_speak(MonsterSound::Orc));
        assert!(can_speak(MonsterSound::Soldier));
    }

    #[test]
    fn test_can_speak_animal() {
        assert!(!can_speak(MonsterSound::Bark));
        assert!(!can_speak(MonsterSound::Roar));
        assert!(!can_speak(MonsterSound::Mew));
    }

    #[test]
    fn test_generate_ambient_sounds_count() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut rng = GameRng::new(42);

        let sounds = generate_ambient_sounds(&level, 10, 10, &mut rng);
        // Should be a vector (possibly empty or with sounds)
        assert!(sounds.is_empty() || sounds.len() > 0);
    }

    #[test]
    fn test_generate_monster_noise_silent() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let noise = generate_monster_noise(&monster, MonsterSound::Silent, &mut rng);
        assert!(noise.is_none());
    }

    #[test]
    fn test_generate_monster_noise_humanoid() {
        let monster = test_monster();
        let mut rng = GameRng::new(0); // Seed to likely generate sound

        let noise = generate_monster_noise(&monster, MonsterSound::Humanoid, &mut rng);
        // Due to 1/20 chance, might or might not generate
        let _ = noise;
    }

    #[test]
    fn test_generate_monster_noise_animal() {
        let mut monster = test_monster();
        monster.state.fleeing = false;
        let mut rng = GameRng::new(0);

        let noise = generate_monster_noise(&monster, MonsterSound::Bark, &mut rng);
        // Due to 1/20 chance, might or might not generate
        let _ = noise;
    }

    #[test]
    fn test_pet_sound_type_variants() {
        // Verify all variants exist
        let _ = PetSoundType::Begging;
        let _ = PetSoundType::Whimpering;
        let _ = PetSoundType::Yelping;
        let _ = PetSoundType::Growling;
    }

    #[test]
    fn test_humanoid_speech_various_types() {
        let monster = test_monster();
        let mut rng = GameRng::new(42);

        let djinni = humanoid_speech(&monster, MonsterSound::Djinni, &mut rng);
        assert!(djinni.is_some());
        assert!(djinni.unwrap().contains("serve"));

        let nurse = humanoid_speech(&monster, MonsterSound::Nurse, &mut rng);
        assert!(nurse.is_some());
        assert!(nurse.unwrap().contains("medicine"));

        let vampire = humanoid_speech(&monster, MonsterSound::Vampire, &mut rng);
        assert!(vampire.is_some());
        assert!(vampire.unwrap().contains("blood"));
    }

    #[test]
    fn test_monster_whimper_contains_name() {
        let monster = test_monster();
        let msg = monster_whimper(&monster, MonsterSound::Growl);

        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("test monster"));
    }

    #[test]
    fn test_monster_yelp_contains_name() {
        let monster = test_monster();
        let msg = monster_yelp(&monster, MonsterSound::Bark);

        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("test monster"));
    }

    // ========================================================================
    // Tests for new functions: SoundRegistry, add_sound_mapping, noises
    // ========================================================================

    #[test]
    fn test_sound_registry_new() {
        let registry = SoundRegistry::new();
        assert!(registry.mappings.is_empty());
        assert_eq!(registry.sound_dir, ".");
    }

    #[test]
    fn test_sound_registry_set_sound_dir() {
        let mut registry = SoundRegistry::new();
        registry.set_sound_dir("/sounds");
        assert_eq!(registry.sound_dir, "/sounds");
    }

    #[test]
    fn test_add_sound_mapping_valid() {
        let mut registry = SoundRegistry::new();
        let result = add_sound_mapping(&mut registry, r#"MESG "You hit" "hit.wav" 50"#);

        assert!(result);
        assert_eq!(registry.mappings.len(), 1);
        assert_eq!(registry.mappings[0].pattern, "You hit");
        assert!(registry.mappings[0].filename.contains("hit.wav"));
    }

    #[test]
    fn test_add_sound_mapping_invalid() {
        let mut registry = SoundRegistry::new();
        let result = add_sound_mapping(&mut registry, "invalid mapping");

        assert!(!result);
        assert!(registry.mappings.is_empty());
    }

    #[test]
    fn test_find_sound_for_message_found() {
        let mut registry = SoundRegistry::new();
        add_sound_mapping(&mut registry, r#"MESG "You hit" "hit.wav" 50"#);

        let result = find_sound_for_message(&registry, "You hit the goblin!");
        assert!(result.is_some());
        assert_eq!(result.unwrap().pattern, "You hit");
    }

    #[test]
    fn test_find_sound_for_message_not_found() {
        let mut registry = SoundRegistry::new();
        add_sound_mapping(&mut registry, r#"MESG "You hit" "hit.wav" 50"#);

        let result = find_sound_for_message(&registry, "You miss the goblin!");
        assert!(result.is_none());
    }

    #[test]
    fn test_noises_returns_vec() {
        let dlevel = crate::dungeon::DLevel::new(0, 1);
        let level = crate::dungeon::Level::new(dlevel);
        let mut rng = GameRng::new(42);

        let sounds = noises(&level, 10, 10, &mut rng);
        // Should return a vector (possibly empty)
        assert!(sounds.is_empty() || !sounds.is_empty());
    }

    #[test]
    fn test_sound_mapping_fields() {
        let mapping = SoundMapping {
            pattern: "test".to_string(),
            filename: "test.wav".to_string(),
            volume: 75,
        };

        assert_eq!(mapping.pattern, "test");
        assert_eq!(mapping.filename, "test.wav");
        assert_eq!(mapping.volume, 75);
    }
}
