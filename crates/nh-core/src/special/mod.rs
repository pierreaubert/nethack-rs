//! Special game systems
//!
//! Shopkeepers, priests, vault guards, quests, mail, etc.

pub mod dog; // Pet handling
pub mod integration; // Integration guide for all systems
pub mod mail; // Mail daemon delivery system
pub mod priest; // Priests and temples
pub mod quest; // Quest system
pub mod shk; // Shopkeepers
pub mod sounds; // Monster sounds and speech
pub mod summon; // Summoning monsters
pub mod vault; // Vault guards

pub use summon::{SummonResult, dosummon, msummon, nasty};

/// Room types for special rooms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Ordinary,
    Shop(ShopType),
    Vault,
    Court,
    Swamp,
    Morgue,
    Beehive,
    Barracks,
    Zoo,
    Temple,
    LeprehallHall,
    CockatriceNest,
    AntHole,
}

/// Shop types
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ShopType {
    General,
    Armor,
    Weapon,
    Food,
    Scroll,
    Potion,
    Wand,
    Tool,
    Book,
    Ring,
    Candle,
    Tin,
}
