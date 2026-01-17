//! Special game systems
//!
//! Shopkeepers, priests, vault guards, quests, etc.

// TODO: Implement these modules
// mod shk;     // Shopkeepers
// mod priest;  // Priests/temples
// mod vault;   // Vault guards
// mod quest;   // Quest system

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
