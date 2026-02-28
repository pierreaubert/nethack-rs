//! nh-ui: Terminal UI layer using ratatui
//!
//! Provides the terminal interface for the game.

pub mod app;
pub mod display;
pub mod input;
pub mod theme;
pub mod widgets;

pub use app::{App, AppEvent, CharacterChoices, CharacterCreationState, StartMenuAction, UiMode};
pub use theme::Theme;
pub use display::GraphicsMode;
