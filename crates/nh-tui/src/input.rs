//! Input handling - convert key events to commands

use crossterm::event::{KeyCode, KeyEvent};
use nh_core::action::{Command, Direction};

/// Convert a key event to a game command
pub fn key_to_command(key: KeyEvent, num_pad: bool) -> Option<Command> {
    match key.code {
        // Vi keys (hjklyubn) when not using numpad
        KeyCode::Char('h') if !num_pad => Some(Command::Move(Direction::West)),
        KeyCode::Char('j') if !num_pad => Some(Command::Move(Direction::South)),
        KeyCode::Char('k') if !num_pad => Some(Command::Move(Direction::North)),
        KeyCode::Char('l') if !num_pad => Some(Command::Move(Direction::East)),
        KeyCode::Char('y') if !num_pad => Some(Command::Move(Direction::NorthWest)),
        KeyCode::Char('u') if !num_pad => Some(Command::Move(Direction::NorthEast)),
        KeyCode::Char('b') if !num_pad => Some(Command::Move(Direction::SouthWest)),
        KeyCode::Char('n') if !num_pad => Some(Command::Move(Direction::SouthEast)),

        // Capital Vi keys for running
        KeyCode::Char('H') if !num_pad => Some(Command::Run(Direction::West)),
        KeyCode::Char('J') if !num_pad => Some(Command::Run(Direction::South)),
        KeyCode::Char('K') if !num_pad => Some(Command::Run(Direction::North)),
        KeyCode::Char('L') if !num_pad => Some(Command::Run(Direction::East)),
        KeyCode::Char('Y') if !num_pad => Some(Command::Run(Direction::NorthWest)),
        KeyCode::Char('U') if !num_pad => Some(Command::Run(Direction::NorthEast)),
        KeyCode::Char('B') if !num_pad => Some(Command::Run(Direction::SouthWest)),
        KeyCode::Char('N') if !num_pad => Some(Command::Run(Direction::SouthEast)),

        // Arrow keys
        KeyCode::Up => Some(Command::Move(Direction::North)),
        KeyCode::Down => Some(Command::Move(Direction::South)),
        KeyCode::Left => Some(Command::Move(Direction::West)),
        KeyCode::Right => Some(Command::Move(Direction::East)),

        // Common commands
        KeyCode::Char(',') | KeyCode::Char('g') => Some(Command::Pickup),
        // KeyCode::Char('d') => Some(Command::Drop),
        // KeyCode::Char('e') => Some(Command::Eat),
        // KeyCode::Char('q') => Some(Command::Quaff),
        // KeyCode::Char('r') => Some(Command::Read),
        // KeyCode::Char('z') => Some(Command::Zap),
        // KeyCode::Char('a') => Some(Command::Apply),
        KeyCode::Char('i') => Some(Command::Inventory),
        KeyCode::Char('.') => Some(Command::Rest),
        KeyCode::Char('<') => Some(Command::GoUp),
        KeyCode::Char('>') => Some(Command::GoDown),
        KeyCode::Char('s') => Some(Command::Search),
        // KeyCode::Char('o') => Some(Command::Open),
        // KeyCode::Char('c') => Some(Command::Close),
        KeyCode::Char(':') => Some(Command::Look),
        KeyCode::Char('/') => Some(Command::WhatsHere),
        KeyCode::Char('?') => Some(Command::Help),

        // Wearing/wielding
        // KeyCode::Char('w') => Some(Command::Wield),
        // KeyCode::Char('W') => Some(Command::Wear),
        // KeyCode::Char('T') => Some(Command::TakeOff),
        // KeyCode::Char('P') => Some(Command::PutOn),
        // KeyCode::Char('R') => Some(Command::Remove),

        // Meta
        KeyCode::Char('S') => Some(Command::Save),

        _ => None,
    }
}
