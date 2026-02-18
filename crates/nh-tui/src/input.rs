//! Input handling - convert key events to commands
//!
//! Key bindings follow the original NetHack cmd.c conventions.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use nh_core::action::{Command, Direction};

/// Convert a key event to a game command.
///
/// These are the "simple" bindings that map directly to a Command
/// without needing additional input (item selection, direction, or text).
/// More complex bindings (d, e, t, o, c, #, etc.) are handled in app.rs.
pub fn key_to_command(key: KeyEvent, num_pad: bool) -> Option<Command> {
    // Ctrl key combos
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('p') => Some(Command::History),       // Ctrl+P: message history
            KeyCode::Char('r') => Some(Command::Redraw),        // Ctrl+R: redraw screen
            KeyCode::Char('x') => Some(Command::ShowAttributes), // Ctrl+X: show attributes
            _ => None,
        };
    }

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

        // Pickup / movement
        KeyCode::Char(',') => Some(Command::Pickup),           // , : pickup
        KeyCode::Char('.') => Some(Command::Rest),              // . : rest/wait
        KeyCode::Char('<') => Some(Command::GoUp),              // < : go up stairs
        KeyCode::Char('>') => Some(Command::GoDown),            // > : go down stairs
        KeyCode::Char('_') => Some(Command::Travel),            // _ : travel
        KeyCode::Char('s') => Some(Command::Search),            // s : search

        // Information
        KeyCode::Char('i') => Some(Command::Inventory),         // i : inventory
        KeyCode::Char(':') => Some(Command::Look),               // : : look here
        KeyCode::Char('/') => Some(Command::WhatsHere),          // / : what is symbol
        KeyCode::Char('?') => Some(Command::Help),               // ? : help
        KeyCode::Char('\\') => Some(Command::Discoveries),       // \ : discoveries
        KeyCode::Char('$') => Some(Command::CountGold),          // $ : count gold
        KeyCode::Char('V') => Some(Command::History),            // V : version/history

        // Simple actions (no extra input)
        KeyCode::Char('p') => Some(Command::Pay),               // p : pay shopkeeper
        KeyCode::Char('x') => Some(Command::SwapWeapon),        // x : swap weapons
        KeyCode::Char('X') => Some(Command::TwoWeapon),         // X : two-weapon mode
        KeyCode::Char('Z') => Some(Command::ShowSpells),        // Z : cast spell
        KeyCode::Char('+') => Some(Command::EnhanceSkill),      // + : enhance weapon skill

        // Meta
        KeyCode::Char('S') => Some(Command::Save),              // S : save game

        _ => None,
    }
}
