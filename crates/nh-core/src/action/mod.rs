//! Player action system
//!
//! Implements player commands and actions.

pub mod apply;
pub mod eat;
pub mod pickup;
pub mod wear;
pub mod pray;
pub mod open_close;
pub mod kick;
pub mod engrave;
pub mod teleport;
pub mod trap;
pub mod quaff;
pub mod read;
pub mod zap;
pub mod throw;

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

    // Information
    Inventory,
    Look,
    WhatsHere,
    Help,
    Discoveries,
    History,

    // Actions
    Open(Direction),
    Close(Direction),
    Kick(Direction),
    Search,
    Pray,
    Offer,
    Dip,
    Engrave(String),
    Pay,
    Chat,
    Sit,

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
