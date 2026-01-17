//! Player system
//!
//! Contains the You struct and all player-related functionality.

mod alignment;
mod attributes;
mod conduct;
mod hunger;
mod properties;
mod role;
mod skills;
mod you;

pub use alignment::{Alignment, AlignmentType};
pub use attributes::{Attribute, Attributes};
pub use conduct::Conduct;
pub use hunger::HungerState;
pub use properties::{Property, PropertyFlags, PropertySet};
pub use role::{Gender, Race, Role};
pub use skills::{Skill, SkillLevel, SkillSet, SkillType};
pub use you::{Encumbrance, Position, You};
