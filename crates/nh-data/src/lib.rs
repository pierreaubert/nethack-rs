//! nh-data: Static game data for NetHack clone
//!
//! Contains monster definitions, object definitions, artifacts, roles, races, etc.

pub mod artifacts;
pub mod colors;
pub mod monsters;
pub mod objects;
pub mod roles;

pub use artifacts::{
    get_artifact, get_artifact_by_index, get_quest_artifact, is_artifact_base, num_artifacts,
    Alignment, Artifact, ArtifactFlags, InvokeProperty, ARTIFACTS,
};
pub use colors::*;
pub use monsters::{get_monster, num_monsters, MonsterType, MONSTERS};
pub use roles::{
    find_race, find_role, get_race, get_role, num_races, num_roles, race_allows_alignment,
    role_allows_alignment, role_allows_race, Advancement, Race, Role, RoleName, StartingItem,
    RACES, ROLES,
};
