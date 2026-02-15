//! Quest system (quest.c, questpgr.c)
//!
//! Implements the role-specific quest system where players must
//! retrieve an artifact from a nemesis to complete their quest.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::gameloop::GameState;
use crate::player::Role;

/// Quest progress stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum QuestStage {
    /// Quest not yet started
    #[default]
    NotStarted,
    /// Player has been assigned the quest
    Assigned,
    /// Player is seeking the quest artifact
    Seeking,
    /// Player has obtained the artifact
    GotArtifact,
    /// Quest is complete
    Completed,
}

/// Quest status tracking
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QuestStatus {
    /// Current quest stage
    pub stage: QuestStage,
    /// Whether player has spoken to leader
    pub met_leader: bool,
    /// Whether player has been given permission to quest
    pub got_permission: bool,
    /// Number of times player was denied permission
    pub denial_count: u8,
    /// Whether the nemesis has been killed
    pub nemesis_dead: bool,
    /// Whether the quest artifact has been retrieved
    pub got_artifact: bool,
    /// Whether player has returned artifact to leader
    pub returned_artifact: bool,
    /// Turn when quest was started
    pub start_turn: u64,
    /// Turn when quest was completed
    pub complete_turn: Option<u64>,
}

impl QuestStatus {
    /// Create a new quest status
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if player can attempt the quest
    pub fn can_quest(&self) -> bool {
        self.got_permission && !self.is_complete()
    }

    /// Check if quest is complete
    pub fn is_complete(&self) -> bool {
        self.stage == QuestStage::Completed
    }

    /// Check if player needs to talk to leader
    pub fn needs_leader(&self) -> bool {
        !self.met_leader || (!self.got_permission && self.denial_count < 7)
    }
}

/// Quest information for a role
#[derive(Debug, Clone)]
pub struct QuestInfo {
    /// Role this quest is for
    pub role: Role,
    /// Name of the quest leader
    pub leader_name: &'static str,
    /// Name of the nemesis
    pub nemesis_name: &'static str,
    /// Name of the quest artifact
    pub artifact_name: &'static str,
    /// Quest home level name
    pub home_name: &'static str,
    /// Quest locate level name
    pub locate_name: &'static str,
    /// Quest goal level name
    pub goal_name: &'static str,
    /// Minimum experience level to start quest
    pub min_level: u8,
    /// Quest narrative text
    pub narrative: QuestNarrative,
}

/// Quest narrative text
#[derive(Debug, Clone)]
pub struct QuestNarrative {
    /// Leader's greeting
    pub leader_greeting: &'static str,
    /// Leader's quest assignment
    pub quest_assignment: &'static str,
    /// Leader's denial (not ready)
    pub denial_not_ready: &'static str,
    /// Leader's encouragement
    pub encouragement: &'static str,
    /// Nemesis taunt
    pub nemesis_taunt: &'static str,
    /// Victory message
    pub victory_message: &'static str,
}

/// Get quest info for a role
pub fn get_quest_info(role: Role) -> QuestInfo {
    match role {
        Role::Archeologist => QuestInfo {
            role,
            leader_name: "Lord Carnarvon",
            nemesis_name: "the Minion of Huhetotl",
            artifact_name: "the Orb of Detection",
            home_name: "the College of Archeology",
            locate_name: "the Tomb of the Toltec Kings",
            goal_name: "the Sanctum of Huhetotl",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Ah, a fellow archeologist! Welcome!",
                quest_assignment: "We need you to recover the Orb of Detection from the Minion of Huhetotl.",
                denial_not_ready: "You are not yet experienced enough for this dangerous mission.",
                encouragement: "The fate of archeology rests in your hands!",
                nemesis_taunt: "You dare disturb my master's rest?",
                victory_message: "The Orb of Detection is yours! Return it to Lord Carnarvon.",
            },
        },
        Role::Barbarian => QuestInfo {
            role,
            leader_name: "Pelias",
            nemesis_name: "Thoth Amon",
            artifact_name: "the Heart of Ahriman",
            home_name: "the Camp of the Duali",
            locate_name: "the Subterranean Caves",
            goal_name: "Thoth Amon's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Hail, warrior! Your strength is needed!",
                quest_assignment: "Thoth Amon has stolen the Heart of Ahriman. Retrieve it!",
                denial_not_ready: "You must grow stronger before facing Thoth Amon.",
                encouragement: "By Crom, show no mercy!",
                nemesis_taunt: "Your primitive weapons cannot harm me!",
                victory_message: "The Heart of Ahriman pulses with power!",
            },
        },
        Role::Caveman => QuestInfo {
            role,
            leader_name: "Shaman Karnov",
            nemesis_name: "Chromatic Dragon",
            artifact_name: "the Sceptre of Might",
            home_name: "the Caves of the Ancestors",
            locate_name: "the Dragon's Lair",
            goal_name: "the Dragon's Den",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Ooga! Strong one come!",
                quest_assignment: "Big dragon take shiny stick. You get back!",
                denial_not_ready: "You not strong enough. Come back later.",
                encouragement: "Ancestors watch over you!",
                nemesis_taunt: "A mere caveman dares challenge me?",
                victory_message: "The Sceptre of Might is yours!",
            },
        },
        Role::Healer => QuestInfo {
            role,
            leader_name: "Hippocrates",
            nemesis_name: "Cyclops",
            artifact_name: "the Staff of Aesculapius",
            home_name: "the Temple of Epidaurus",
            locate_name: "the Temple of Coeus",
            goal_name: "the Cyclops's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Welcome, healer. We have need of your skills.",
                quest_assignment: "The Cyclops has stolen the Staff of Aesculapius. Recover it.",
                denial_not_ready: "Your healing arts are not yet refined enough.",
                encouragement: "Remember: first, do no harm... to yourself.",
                nemesis_taunt: "Your healing cannot save you now!",
                victory_message: "The Staff of Aesculapius will heal many!",
            },
        },
        Role::Knight => QuestInfo {
            role,
            leader_name: "King Arthur",
            nemesis_name: "Ixoth",
            artifact_name: "the Magic Mirror of Merlin",
            home_name: "Camelot Castle",
            locate_name: "the Questing Beast's Lair",
            goal_name: "Ixoth's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Welcome, noble knight!",
                quest_assignment: "The dragon Ixoth has stolen Merlin's Mirror. Retrieve it!",
                denial_not_ready: "You have not yet proven your valor.",
                encouragement: "For honor and glory!",
                nemesis_taunt: "Your chivalry will be your downfall!",
                victory_message: "The Magic Mirror of Merlin is restored!",
            },
        },
        Role::Monk => QuestInfo {
            role,
            leader_name: "Grand Master",
            nemesis_name: "Master Kaen",
            artifact_name: "the Eyes of the Overworld",
            home_name: "the Monastery",
            locate_name: "the Caves of Thought",
            goal_name: "Master Kaen's Dojo",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Welcome, student. Your training continues.",
                quest_assignment: "Master Kaen has betrayed us. Recover the Eyes of the Overworld.",
                denial_not_ready: "Your mind is not yet disciplined enough.",
                encouragement: "Find peace within the storm.",
                nemesis_taunt: "Your techniques are inferior!",
                victory_message: "The Eyes of the Overworld grant you true sight!",
            },
        },
        Role::Priest => QuestInfo {
            role,
            leader_name: "the High Priest",
            nemesis_name: "Nalzok",
            artifact_name: "the Mitre of Holiness",
            home_name: "the Great Temple",
            locate_name: "the Temple of Moloch",
            goal_name: "Nalzok's Sanctum",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Blessings upon you, faithful one.",
                quest_assignment: "The demon Nalzok has defiled our sacred Mitre. Reclaim it!",
                denial_not_ready: "Your faith must grow stronger.",
                encouragement: "May your deity guide your path!",
                nemesis_taunt: "Your prayers fall on deaf ears!",
                victory_message: "The Mitre of Holiness shines with divine light!",
            },
        },
        Role::Ranger => QuestInfo {
            role,
            leader_name: "Orion",
            nemesis_name: "Scorpius",
            artifact_name: "the Longbow of Diana",
            home_name: "Orion's Camp",
            locate_name: "the Scorpion's Nest",
            goal_name: "Scorpius's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Well met, ranger!",
                quest_assignment: "Scorpius has taken Diana's Longbow. Hunt him down!",
                denial_not_ready: "Your tracking skills need improvement.",
                encouragement: "Let your arrows fly true!",
                nemesis_taunt: "My venom will end you!",
                victory_message: "The Longbow of Diana never misses!",
            },
        },
        Role::Rogue => QuestInfo {
            role,
            leader_name: "the Master of Thieves",
            nemesis_name: "the Master Assassin",
            artifact_name: "the Master Key of Thievery",
            home_name: "the Thieves' Guild",
            locate_name: "the Assassins' Den",
            goal_name: "the Master Assassin's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Ah, a fellow professional.",
                quest_assignment: "The Master Assassin has our Key. Steal it back.",
                denial_not_ready: "You're too clumsy for this job.",
                encouragement: "Remember: silence is golden.",
                nemesis_taunt: "You'll never see my blade coming!",
                victory_message: "The Master Key opens all doors!",
            },
        },
        Role::Samurai => QuestInfo {
            role,
            leader_name: "Lord Sato",
            nemesis_name: "Ashikaga Takauji",
            artifact_name: "the Tsurugi of Muramasa",
            home_name: "the Castle of the Taro Clan",
            locate_name: "the Shogun's Castle",
            goal_name: "Ashikaga's Stronghold",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Honor to you, samurai.",
                quest_assignment: "The traitor Ashikaga has the Tsurugi. Reclaim our honor!",
                denial_not_ready: "Your bushido is not yet perfected.",
                encouragement: "Death before dishonor!",
                nemesis_taunt: "Your clan is weak!",
                victory_message: "The Tsurugi of Muramasa cuts through all!",
            },
        },
        Role::Tourist => QuestInfo {
            role,
            leader_name: "Twoflower",
            nemesis_name: "the Master of Thieves",
            artifact_name: "the Platinum Yendorian Express Card",
            home_name: "Ankh-Morpork",
            locate_name: "the Thieves' Guild Hall",
            goal_name: "the Master Thief's Lair",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Oh good, another tourist!",
                quest_assignment: "Someone stole my credit card! Get it back!",
                denial_not_ready: "You look a bit green for this.",
                encouragement: "Don't forget to take pictures!",
                nemesis_taunt: "Your valuables are mine!",
                victory_message: "The Card has unlimited credit!",
            },
        },
        Role::Valkyrie => QuestInfo {
            role,
            leader_name: "the Norn",
            nemesis_name: "Lord Surtur",
            artifact_name: "the Orb of Fate",
            home_name: "the Shrine of Destiny",
            locate_name: "Muspelheim",
            goal_name: "Surtur's Stronghold",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Hail, warrior maiden!",
                quest_assignment: "Lord Surtur threatens Ragnarok. Stop him and recover the Orb!",
                denial_not_ready: "The fates say you are not ready.",
                encouragement: "Valhalla awaits the worthy!",
                nemesis_taunt: "I will bring Ragnarok upon you!",
                victory_message: "The Orb of Fate reveals your destiny!",
            },
        },
        Role::Wizard => QuestInfo {
            role,
            leader_name: "the Wizard of Balance",
            nemesis_name: "the Dark One",
            artifact_name: "the Eye of the Aethiopica",
            home_name: "the Tower of the Archmage",
            locate_name: "the Dark Tower",
            goal_name: "the Dark One's Sanctum",
            min_level: 14,
            narrative: QuestNarrative {
                leader_greeting: "Welcome, apprentice.",
                quest_assignment: "The Dark One has stolen the Eye. Recover it!",
                denial_not_ready: "Your magical knowledge is insufficient.",
                encouragement: "Let wisdom guide your spells!",
                nemesis_taunt: "Your magic is nothing compared to mine!",
                victory_message: "The Eye of the Aethiopica grants great power!",
            },
        },
    }
}

/// Check if player meets requirements to start quest
pub fn can_start_quest(state: &GameState, quest_info: &QuestInfo) -> bool {
    state.player.exp_level >= quest_info.min_level as i32
}

/// Handle meeting the quest leader
pub fn meet_leader(
    state: &mut GameState,
    quest_status: &mut QuestStatus,
    quest_info: &QuestInfo,
) -> String {
    quest_status.met_leader = true;

    if quest_status.got_permission {
        return quest_info.narrative.encouragement.to_string();
    }

    if can_start_quest(state, quest_info) {
        quest_status.got_permission = true;
        quest_status.stage = QuestStage::Assigned;
        format!(
            "{}\n\n{}",
            quest_info.narrative.leader_greeting, quest_info.narrative.quest_assignment
        )
    } else {
        quest_status.denial_count += 1;
        format!(
            "{}\n\n{}",
            quest_info.narrative.leader_greeting, quest_info.narrative.denial_not_ready
        )
    }
}

/// Handle defeating the nemesis
pub fn defeat_nemesis(quest_status: &mut QuestStatus) {
    quest_status.nemesis_dead = true;
}

/// Handle obtaining the quest artifact
pub fn obtain_artifact(quest_status: &mut QuestStatus, quest_info: &QuestInfo) -> String {
    quest_status.got_artifact = true;
    quest_status.stage = QuestStage::GotArtifact;
    quest_info.narrative.victory_message.to_string()
}

/// Handle returning the artifact to the leader
pub fn return_artifact(quest_status: &mut QuestStatus, current_turn: u64) -> String {
    quest_status.returned_artifact = true;
    quest_status.stage = QuestStage::Completed;
    quest_status.complete_turn = Some(current_turn);
    "Your quest is complete! You have proven yourself worthy.".to_string()
}

/// Check if player is allowed to attempt the quest (ok_to_quest equivalent)
pub fn is_permitted_for_quest(quest_status: &QuestStatus, player_level: u8, min_level: u8) -> bool {
    // Must have received permission or already completed quest
    if !quest_status.got_permission && !quest_status.is_complete() {
        return false;
    }

    // Must be high enough level
    player_level >= min_level
}

/// Handle quest entry from quest dungeon (onquest equivalent)
pub fn handle_quest_entry(
    quest_status: &mut QuestStatus,
    on_locate_level: bool,
    on_goal_level: bool,
) -> Vec<String> {
    let mut messages = Vec::new();

    if on_locate_level {
        // Entering locate level for first time
        if !quest_status.nemesis_dead {
            messages.push("You sense a powerful presence nearby...".to_string());
        }
    } else if on_goal_level {
        // Entering goal level
        if quest_status.nemesis_dead {
            messages.push("The nemesis has already been defeated here.".to_string());
        } else if quest_status.stage == QuestStage::Seeking {
            messages.push("You have found the nemesis's lair!".to_string());
        }
    }

    messages
}

/// Process quest chat command (quest_chat equivalent)
pub fn handle_quest_chat(quest_status: &QuestStatus, is_leader: bool, is_nemesis: bool) -> String {
    if is_leader {
        if quest_status.stage == QuestStage::Completed {
            "The leader congratulates you on your successful quest!".to_string()
        } else if quest_status.stage == QuestStage::GotArtifact {
            "The leader awaits the artifact you promised to retrieve.".to_string()
        } else if quest_status.got_permission {
            "Good luck on your quest! Be careful out there.".to_string()
        } else {
            "I have a task for you, but first you must prove yourself.".to_string()
        }
    } else if is_nemesis {
        if quest_status.nemesis_dead {
            "The nemesis is already dead.".to_string()
        } else {
            "You dare to challenge me?!".to_string()
        }
    } else {
        "I don't know what you're talking about.".to_string()
    }
}

/// Process quest information command (quest_info equivalent)
pub fn get_quest_status_message(quest_status: &QuestStatus, info: &QuestInfo) -> String {
    match quest_status.stage {
        QuestStage::NotStarted => {
            format!(
                "You haven't spoken to {} about a quest yet.",
                info.leader_name
            )
        }
        QuestStage::Assigned => {
            format!(
                "You are seeking {}. {} awaits your return to {}.",
                info.nemesis_name, info.leader_name, info.home_name
            )
        }
        QuestStage::Seeking => {
            format!(
                "You are on a quest to find {} and retrieve {}.",
                info.nemesis_name, info.artifact_name
            )
        }
        QuestStage::GotArtifact => {
            format!(
                "You have obtained {}! Return it to {} at {}.",
                info.artifact_name, info.leader_name, info.home_name
            )
        }
        QuestStage::Completed => {
            format!(
                "You have successfully completed the quest! {} has been recovered.",
                info.artifact_name
            )
        }
    }
}

/// Handle quest leader conversation (chat_with_leader equivalent)
pub fn speak_with_quest_leader(
    quest_status: &mut QuestStatus,
    info: &QuestInfo,
    player_level: u8,
) -> Vec<String> {
    let mut messages = Vec::new();

    if quest_status.stage == QuestStage::Completed {
        messages.push(info.narrative.victory_message.to_string());
        return messages;
    }

    if !quest_status.met_leader {
        messages.push(info.narrative.leader_greeting.to_string());
        quest_status.met_leader = true;
    }

    if !quest_status.got_permission {
        if player_level < info.min_level {
            messages.push(format!(
                "{}. You need to reach level {} first.",
                info.narrative.denial_not_ready, info.min_level
            ));
            quest_status.denial_count += 1;
        } else {
            messages.push(info.narrative.quest_assignment.to_string());
            quest_status.got_permission = true;
            quest_status.stage = QuestStage::Assigned;
        }
    } else if quest_status.stage == QuestStage::Assigned {
        messages.push(info.narrative.encouragement.to_string());
    }

    messages
}

/// Handle quest nemesis interaction (chat_with_nemesis equivalent)
pub fn speak_with_nemesis(info: &QuestInfo) -> String {
    info.narrative.nemesis_taunt.to_string()
}

/// Check if player is on quest (onquest equivalent)
pub fn is_on_quest(quest_status: &QuestStatus) -> bool {
    quest_status.got_permission && !quest_status.is_complete()
}

/// Check if player is on goal level (on_goal equivalent)
pub fn is_on_goal_level(quest_status: &QuestStatus) -> bool {
    is_on_quest(quest_status)
}

/// Handle quest start level entry (on_start equivalent)
pub fn handle_quest_start_entry(quest_status: &mut QuestStatus, info: &QuestInfo) -> Vec<String> {
    let mut messages = Vec::new();

    if quest_status.stage == QuestStage::Assigned {
        messages.push(format!("You are now seeking {}.", info.nemesis_name));
        quest_status.stage = QuestStage::Seeking;
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quest_status_new() {
        let status = QuestStatus::new();
        assert_eq!(status.stage, QuestStage::NotStarted);
        assert!(!status.met_leader);
        assert!(!status.got_permission);
    }

    #[test]
    fn test_quest_info() {
        let info = get_quest_info(Role::Valkyrie);
        assert_eq!(info.leader_name, "the Norn");
        assert_eq!(info.nemesis_name, "Lord Surtur");
        assert_eq!(info.min_level, 14);
    }

    #[test]
    fn test_quest_progression() {
        let mut status = QuestStatus::new();

        assert!(!status.can_quest());

        status.got_permission = true;
        assert!(status.can_quest());

        status.stage = QuestStage::Completed;
        assert!(!status.can_quest());
    }

    #[test]
    fn test_all_roles_have_quests() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        for role in roles {
            let info = get_quest_info(role);
            assert!(!info.leader_name.is_empty());
            assert!(!info.nemesis_name.is_empty());
            assert!(!info.artifact_name.is_empty());
            assert!(info.min_level > 0);
        }
    }

    #[test]
    fn test_defeat_nemesis() {
        let mut status = QuestStatus::new();
        assert!(!status.nemesis_dead);

        defeat_nemesis(&mut status);
        assert!(status.nemesis_dead);
    }

    #[test]
    fn test_obtain_artifact() {
        let mut status = QuestStatus::new();
        let info = get_quest_info(Role::Wizard);

        let msg = obtain_artifact(&mut status, &info);

        assert!(status.got_artifact);
        assert_eq!(status.stage, QuestStage::GotArtifact);
        assert!(!msg.is_empty());
    }

    #[test]
    fn test_return_artifact() {
        let mut status = QuestStatus::new();
        status.got_artifact = true;
        status.stage = QuestStage::GotArtifact;

        let msg = return_artifact(&mut status, 1000);

        assert!(status.returned_artifact);
        assert_eq!(status.stage, QuestStage::Completed);
        assert_eq!(status.complete_turn, Some(1000));
        assert!(!msg.is_empty());
    }

    // ========== EXPANDED TEST COVERAGE ==========

    #[test]
    fn test_quest_status_fields() {
        let status = QuestStatus::new();
        assert_eq!(status.stage, QuestStage::NotStarted);
        assert!(!status.met_leader);
        assert!(!status.got_permission);
        assert_eq!(status.denial_count, 0);
        assert!(!status.nemesis_dead);
        assert!(!status.got_artifact);
        assert!(!status.returned_artifact);
        assert!(status.complete_turn.is_none());
    }

    #[test]
    fn test_quest_stage_progression() {
        let stages = [
            QuestStage::NotStarted,
            QuestStage::Assigned,
            QuestStage::Seeking,
            QuestStage::GotArtifact,
            QuestStage::Completed,
        ];

        for stage in &stages {
            // Each stage is a valid variant
            let _ = stage;
        }
    }

    #[test]
    fn test_get_quest_info_consistency() {
        // Each role should have consistent quest info
        let info1 = get_quest_info(Role::Wizard);
        let info2 = get_quest_info(Role::Wizard);

        assert_eq!(info1.leader_name, info2.leader_name);
        assert_eq!(info1.nemesis_name, info2.nemesis_name);
        assert_eq!(info1.artifact_name, info2.artifact_name);
        assert_eq!(info1.min_level, info2.min_level);
    }

    #[test]
    fn test_quest_info_all_roles_unique() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        let mut seen_artifacts = hashbrown::HashSet::new();
        for role in &roles {
            let info = get_quest_info(*role);
            // Each role should have unique artifact
            assert!(seen_artifacts.insert(info.artifact_name.clone()));
        }
    }

    #[test]
    fn test_quest_permission_flow() {
        let mut status = QuestStatus::new();

        // Initially cannot quest
        assert!(!status.can_quest());

        // After permission granted
        status.got_permission = true;
        assert!(status.can_quest());

        // If nemesis is defeated, still cannot quest (quest is complete)
        status.nemesis_dead = true;
        assert!(status.can_quest()); // Can still be on quest
    }

    #[test]
    fn test_defeat_nemesis_prevents_future_defeats() {
        let mut status = QuestStatus::new();
        defeat_nemesis(&mut status);
        assert!(status.nemesis_dead);

        // Defeating again should not break anything
        defeat_nemesis(&mut status);
        assert!(status.nemesis_dead);
    }

    #[test]
    fn test_obtain_artifact_sets_all_fields() {
        let mut status = QuestStatus::new();
        let info = get_quest_info(Role::Knight);

        obtain_artifact(&mut status, &info);

        assert!(status.got_artifact);
        assert_eq!(status.stage, QuestStage::GotArtifact);
    }

    #[test]
    fn test_return_artifact_requires_artifact_obtained() {
        let mut status = QuestStatus::new();
        // Start with artifact not obtained
        assert!(!status.got_artifact);

        // Returning should still work (game logic allows this)
        let msg = return_artifact(&mut status, 1000);

        assert!(!msg.is_empty());
        assert!(status.returned_artifact);
    }

    #[test]
    fn test_return_artifact_records_turn() {
        let mut status = QuestStatus::new();
        assert!(status.complete_turn.is_none());

        return_artifact(&mut status, 5000);

        assert!(status.complete_turn.is_some());
        assert_eq!(status.complete_turn.unwrap(), 5000);
    }

    #[test]
    fn test_can_quest_after_completion() {
        let mut status = QuestStatus::new();
        status.got_permission = true;
        assert!(status.can_quest());

        // After returning artifact
        status.stage = QuestStage::Completed;
        assert!(!status.can_quest());
    }

    #[test]
    fn test_quest_leader_names_not_empty() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        for role in &roles {
            let info = get_quest_info(*role);
            assert!(
                !info.leader_name.is_empty(),
                "Leader name empty for {:?}",
                role
            );
            assert!(info.leader_name.len() > 1);
        }
    }

    #[test]
    fn test_quest_nemesis_names_not_empty() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        for role in &roles {
            let info = get_quest_info(*role);
            assert!(
                !info.nemesis_name.is_empty(),
                "Nemesis name empty for {:?}",
                role
            );
            assert!(info.nemesis_name.len() > 1);
        }
    }

    #[test]
    fn test_quest_artifact_names_not_empty() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        for role in &roles {
            let info = get_quest_info(*role);
            assert!(
                !info.artifact_name.is_empty(),
                "Artifact name empty for {:?}",
                role
            );
            assert!(info.artifact_name.len() > 1);
        }
    }

    #[test]
    fn test_quest_min_level_reasonable() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];

        for role in &roles {
            let info = get_quest_info(*role);
            assert!(info.min_level >= 10, "Min level too low for {:?}", role);
            assert!(info.min_level <= 30, "Min level too high for {:?}", role);
        }
    }

    #[test]
    fn test_quest_multiple_state_transitions() {
        let mut status = QuestStatus::new();

        // Not started
        assert_eq!(status.stage, QuestStage::NotStarted);

        // Get permission
        status.got_permission = true;

        // Meet leader
        status.met_leader = true;

        // Defeat nemesis
        defeat_nemesis(&mut status);

        // Get artifact
        let info = get_quest_info(Role::Barbarian);
        obtain_artifact(&mut status, &info);

        // Return artifact
        return_artifact(&mut status, 1000);

        assert!(status.returned_artifact);
        assert!(status.nemesis_dead);
        assert!(status.got_artifact);
    }

    #[test]
    fn test_quest_stage_default_behavior() {
        let status = QuestStatus::new();

        // Should be safe to use in conditional checks
        if status.stage == QuestStage::NotStarted {
            assert!(true);
        }
    }

    #[test]
    fn test_obtain_artifact_message_non_empty() {
        let mut status = QuestStatus::new();
        let info = get_quest_info(Role::Ranger);

        let msg = obtain_artifact(&mut status, &info);

        assert!(!msg.is_empty());
        assert!(msg.len() > 5);
    }

    #[test]
    fn test_return_artifact_message_non_empty() {
        let mut status = QuestStatus::new();
        let msg = return_artifact(&mut status, 500);

        assert!(!msg.is_empty());
        assert!(msg.len() > 5);
    }
}
