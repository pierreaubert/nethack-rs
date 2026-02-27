//! Glyph system for TUI rendering
//!
//! Provides support for both classic ASCII and fancy Unicode box-drawing characters.

use nh_core::dungeon::{CellType, DoorState};
use nh_core::data::tile::{DungeonTile, Tile};
use strum::{Display, EnumString, VariantNames};

/// Available graphics modes for the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, VariantNames, Default)]
#[strum(serialize_all = "lowercase")]
pub enum GraphicsMode {
    /// Classic ASCII characters.
    Classic,
    /// Fancy Unicode box-drawing characters.
    Fancy,
    /// Automatically detect support.
    #[default]
    Auto,
}

/// Set of glyphs used for rendering map features.
pub trait GlyphSet: Send + Sync {
    /// Get the character for a dungeon cell.
    fn cell_char(&self, typ: CellType, flags: u8) -> char;
    
    /// Get the character for a general tile (monsters, objects, etc).
    fn tile_char(&self, tile: &Tile) -> char;
}

/// Classic NetHack ASCII glyph set.
pub struct ClassicGlyphs;

impl GlyphSet for ClassicGlyphs {
    fn cell_char(&self, typ: CellType, _flags: u8) -> char {
        typ.symbol()
    }

    fn tile_char(&self, tile: &Tile) -> char {
        tile.to_ascii()
    }
}

/// Fancy Unicode box-drawing glyph set.
pub struct FancyGlyphs;

impl GlyphSet for FancyGlyphs {
    fn cell_char(&self, typ: CellType, flags: u8) -> char {
        match typ {
            CellType::VWall => '│',
            CellType::HWall => '─',
            CellType::TLCorner => '┌',
            CellType::TRCorner => '┐',
            CellType::BLCorner => '└',
            CellType::BRCorner => '┘',
            CellType::CrossWall => '┼',
            CellType::TUWall => '┴',
            CellType::TDWall => '┬',
            CellType::TLWall => '┤',
            CellType::TRWall => '├',
            CellType::DBWall => '║',
            CellType::Tree => '♣',
            CellType::Door => {
                let state = DoorState::from_bits_truncate(flags);
                if state.contains(DoorState::OPEN) || state.contains(DoorState::BROKEN) {
                    '·' // Open doorway
                } else {
                    '+' // Closed door (classic symbol usually preferred)
                }
            }
            CellType::IronBars => '≡',
            CellType::Stairs => '>', // Up/down handled by level data but Stairs is generic
            CellType::Pool | CellType::Moat | CellType::Water => '≈',
            CellType::Lava => '≈',
            CellType::Fountain => '¶',
            CellType::Throne => '≡',
            CellType::Sink => '#',
            CellType::Grave => '†',
            CellType::Altar => '_',
            _ => typ.symbol(),
        }
    }

    fn tile_char(&self, tile: &Tile) -> char {
        match tile {
            Tile::Dungeon(dt) => match dt {
                DungeonTile::StairsUp => '<',
                DungeonTile::StairsDown => '>',
                _ => tile.to_ascii(),
            },
            _ => tile.to_ascii(),
        }
    }
}

/// Detect if the terminal supports Unicode/UTF-8.
pub fn supports_unicode() -> bool {
    // Check LANG, LC_ALL, or LC_CTYPE for "UTF-8"
    let vars = ["LANG", "LC_ALL", "LC_CTYPE"];
    for var in vars {
        if let Ok(val) = std::env::var(var) {
            if val.to_uppercase().contains("UTF-8") || val.to_uppercase().contains("UTF8") {
                return true;
            }
        }
    }
    
    // On macOS/Linux, most modern terminals support UTF-8 by default.
    // If we're not sure, we can check TERM.
    if let Ok(term) = std::env::var("TERM") {
        if term == "xterm-256color" || term == "alacritty" || term == "kitty" || term == "iterm" {
            return true;
        }
    }

    false
}

/// Returns the best available glyph set for the current environment.
pub fn detect_glyph_set(mode: GraphicsMode) -> Box<dyn GlyphSet> {
    match mode {
        GraphicsMode::Classic => Box::new(ClassicGlyphs),
        GraphicsMode::Fancy => Box::new(FancyGlyphs),
        GraphicsMode::Auto => {
            if supports_unicode() {
                Box::new(FancyGlyphs)
            } else {
                Box::new(ClassicGlyphs)
            }
        }
    }
}
