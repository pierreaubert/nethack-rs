//! nh-data: Static game data for NetHack clone
//!
//! Contains monster definitions, object definitions, artifacts, roles, races, etc.

pub mod artifacts;
pub mod colors;
pub mod monsters;
pub mod objects;
pub mod roles;
pub mod tile;

pub use artifacts::{
    ARTIFACTS, Alignment, Artifact, ArtifactFlags, InvokeProperty, get_artifact,
    get_artifact_by_index, get_quest_artifact, is_artifact_base, num_artifacts,
};
pub use colors::*;
pub use monsters::{MONSTERS, MonsterType, get_monster, num_monsters};
pub use roles::{
    Advancement, RACES, ROLES, Race, Role, RoleName, StartingItem, find_race, find_role, get_race,
    get_role, num_races, num_roles, race_allows_alignment, role_allows_alignment, role_allows_race,
};
