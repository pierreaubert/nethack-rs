//! Player action system
//!
//! Implements player commands and actions.

#[cfg(not(feature = "std"))]
use crate::compat::*;

pub mod apply;
pub mod commands;
pub mod dig;
pub mod eat;
pub mod engrave;
pub mod extended;
pub mod fountain;
pub mod help;
pub mod helpers;
pub mod jump;
pub mod keybindings;
pub mod kick;
pub mod level_change;
pub mod movement;
pub mod name;
pub mod open_close;
pub mod pickup;
pub mod pray;
pub mod quaff;
pub mod read;
pub mod search;
pub mod teleport;
pub mod throw;
pub mod trap;
pub mod wear;
pub mod music;
pub mod zap;

/// Player command types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Movement
    Move(Direction),
    MoveUntilInteresting(Direction),
    Run(Direction),
    Travel,
    Rest,
    GoUp,
    GoDown,

    // Combat
    Fight(Direction),
    Fire(Direction),
    Throw(char, Direction),
    TwoWeapon,
    SwapWeapon,
    CastSpell,

    // Object manipulation
    Pickup,
    Drop(char),
    Eat(char),
    Quaff(char),
    Read(char),
    Zap(char, Direction),
    Apply(char),
    Wear(char),
    TakeOff(char),
    PutOn(char),
    Remove(char),
    Wield(Option<char>),
    SelectQuiver(char),
    Loot,
    Tip(char),
    Dip,
    Rub(char),
    Wipe,
    Force(Direction),

    // Information
    Inventory,
    Look,
    WhatsHere,
    Help,
    Discoveries,
    History,
    ShowAttributes,
    ShowEquipment,
    ShowSpells,
    ShowConduct,
    DungeonOverview,
    CountGold,
    ClassDiscovery,
    TypeInventory(char),
    Vanquished,

    // Actions
    Open(Direction),
    Close(Direction),
    Kick(Direction),
    Search,
    Pray,
    Offer,
    Engrave(String),
    Pay,
    Chat,
    Feed,
    Sit,
    Jump,
    Invoke,
    Untrap(Direction),
    Ride,
    TurnUndead,
    MonsterAbility,
    EnhanceSkill,
    NameItem(char, String),
    NameLevel(String),
    Organize(char, char),

    // Meta
    Save,
    Quit,
    Options,
    ExtendedCommand(String),
    Redraw,
}

/// Movement directions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
    Up,
    Down,
    Self_,
}

impl Direction {
    /// Get the delta (dx, dy) for this direction
    pub const fn delta(&self) -> (i8, i8) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
            Direction::Up => (0, 0),
            Direction::Down => (0, 0),
            Direction::Self_ => (0, 0),
        }
    }

    /// Check if this is a vertical direction (up/down)
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Direction::Up | Direction::Down)
    }

    /// Get direction from delta values (xytod equivalent)
    ///
    /// Returns the direction corresponding to the given dx, dy deltas,
    /// or None if the deltas don't represent a valid direction.
    pub const fn from_delta(dx: i8, dy: i8) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(Direction::North),
            (0, 1) => Some(Direction::South),
            (1, 0) => Some(Direction::East),
            (-1, 0) => Some(Direction::West),
            (1, -1) => Some(Direction::NorthEast),
            (-1, -1) => Some(Direction::NorthWest),
            (1, 1) => Some(Direction::SouthEast),
            (-1, 1) => Some(Direction::SouthWest),
            (0, 0) => Some(Direction::Self_),
            _ => None,
        }
    }

    /// Get the direction name as a string (directionname equivalent)
    pub const fn name(&self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::East => "east",
            Direction::West => "west",
            Direction::NorthEast => "northeast",
            Direction::NorthWest => "northwest",
            Direction::SouthEast => "southeast",
            Direction::SouthWest => "southwest",
            Direction::Up => "up",
            Direction::Down => "down",
            Direction::Self_ => "self",
        }
    }

    /// Check if this is a cardinal direction (N/S/E/W)
    pub const fn is_cardinal(&self) -> bool {
        matches!(
            self,
            Direction::North | Direction::South | Direction::East | Direction::West
        )
    }

    /// Check if this is a diagonal direction
    pub const fn is_diagonal(&self) -> bool {
        matches!(
            self,
            Direction::NorthEast
                | Direction::NorthWest
                | Direction::SouthEast
                | Direction::SouthWest
        )
    }

    /// Get the opposite direction
    pub const fn opposite(&self) -> Option<Self> {
        match self {
            Direction::North => Some(Direction::South),
            Direction::South => Some(Direction::North),
            Direction::East => Some(Direction::West),
            Direction::West => Some(Direction::East),
            Direction::NorthEast => Some(Direction::SouthWest),
            Direction::NorthWest => Some(Direction::SouthEast),
            Direction::SouthEast => Some(Direction::NorthWest),
            Direction::SouthWest => Some(Direction::NorthEast),
            Direction::Up => Some(Direction::Down),
            Direction::Down => Some(Direction::Up),
            Direction::Self_ => None,
        }
    }
}

/// Get direction from delta values (xytod equivalent)
pub const fn xytod(dx: i8, dy: i8) -> Option<Direction> {
    Direction::from_delta(dx, dy)
}

/// Get direction name as string (directionname equivalent)
pub const fn directionname(dir: Direction) -> &'static str {
    dir.name()
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action completed successfully, time passes
    Success,
    /// Action completed but no time passes
    NoTime,
    /// Action was cancelled
    Cancelled,
    /// Action failed with message
    Failed(String),
    /// Player died
    Died(String),
    /// Game should be saved
    Save,
    /// Game should quit
    Quit,
}
