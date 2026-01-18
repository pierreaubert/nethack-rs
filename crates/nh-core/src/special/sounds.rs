//! Monster sounds and speech (sounds.c)
//!
//! Handles monster vocalizations, growls, and speech.

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
pub fn humanoid_speech(monster: &Monster, sound: MonsterSound, rng: &mut GameRng) -> Option<String> {
    if monster.state.sleeping || !monster.can_act() {
        return None;
    }

    match sound {
        MonsterSound::Humanoid | MonsterSound::Orc | MonsterSound::Soldier => {
            let idx = rng.rn2(HUMANOID_SOUNDS.len() as u32) as usize;
            Some(format!("The {} {}.", monster.name, HUMANOID_SOUNDS[idx]))
        }
        MonsterSound::Arrest => {
            Some(format!("The {} shouts: \"Halt! You're under arrest!\"", monster.name))
        }
        MonsterSound::Guard => {
            Some(format!("The {} yells: \"Halt, thief!\"", monster.name))
        }
        MonsterSound::Sell => {
            Some(format!("The {} says: \"Can I help you?\"", monster.name))
        }
        MonsterSound::Djinni => {
            Some(format!("The {} speaks: \"I am here to serve.\"", monster.name))
        }
        MonsterSound::Nurse => {
            Some(format!("The {} says: \"Take your medicine!\"", monster.name))
        }
        MonsterSound::Seduce => {
            Some(format!("The {} whispers seductively.", monster.name))
        }
        MonsterSound::Vampire => {
            Some(format!("The {} says: \"I vant to suck your blood!\"", monster.name))
        }
        MonsterSound::Bribe => {
            Some(format!("The {} offers you a deal.", monster.name))
        }
        MonsterSound::Cuss => {
            Some(format!("The {} curses at you!", monster.name))
        }
        MonsterSound::Rider => {
            Some(format!("The {} intones: \"Your time has come.\"", monster.name))
        }
        MonsterSound::Leader => {
            Some(format!("The {} speaks to you.", monster.name))
        }
        MonsterSound::Nemesis => {
            Some(format!("The {} taunts you!", monster.name))
        }
        MonsterSound::Guardian => {
            Some(format!("The {} challenges you!", monster.name))
        }
        _ => None,
    }
}

/// Check if a monster can make sounds
pub fn can_make_sound(monster: &Monster, sound: MonsterSound) -> bool {
    !monster.state.sleeping && monster.can_act() && sound != MonsterSound::Silent
}

/// Get a random sound message for a monster
pub fn random_monster_sound(monster: &Monster, sound: MonsterSound, rng: &mut GameRng) -> Option<String> {
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
}
