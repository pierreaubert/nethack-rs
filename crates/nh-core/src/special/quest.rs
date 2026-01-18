//! Quest system (quest.c, questpgr.c)
//!
//! Implements the role-specific quest system where players must
//! retrieve an artifact from a nemesis to complete their quest.

use crate::gameloop::GameState;
use crate::player::Role;

/// Quest progress stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone, Default)]
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
            quest_info.narrative.leader_greeting,
            quest_info.narrative.quest_assignment
        )
    } else {
        quest_status.denial_count += 1;
        format!(
            "{}\n\n{}",
            quest_info.narrative.leader_greeting,
            quest_info.narrative.denial_not_ready
        )
    }
}

/// Handle defeating the nemesis
pub fn defeat_nemesis(quest_status: &mut QuestStatus) {
    quest_status.nemesis_dead = true;
}

/// Handle obtaining the quest artifact
pub fn obtain_artifact(
    quest_status: &mut QuestStatus,
    quest_info: &QuestInfo,
) -> String {
    quest_status.got_artifact = true;
    quest_status.stage = QuestStage::GotArtifact;
    quest_info.narrative.victory_message.to_string()
}

/// Handle returning the artifact to the leader
pub fn return_artifact(
    quest_status: &mut QuestStatus,
    current_turn: u64,
) -> String {
    quest_status.returned_artifact = true;
    quest_status.stage = QuestStage::Completed;
    quest_status.complete_turn = Some(current_turn);
    "Your quest is complete! You have proven yourself worthy.".to_string()
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
}
