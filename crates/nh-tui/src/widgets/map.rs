//! Map display widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Widget};

use nh_core::data::tile::{DungeonTile, Tile};
use nh_core::dungeon::{CellType, Level};
use nh_core::player::You;
use nh_core::{COLNO, ROWNO};

/// Widget for rendering the dungeon map
pub struct MapWidget<'a> {
    level: &'a Level,
    player: &'a You,
}

impl<'a> MapWidget<'a> {
    pub fn new(level: &'a Level, player: &'a You) -> Self {
        Self { level, player }
    }

    fn cell_display(&self, x: usize, y: usize) -> (char, Style) {
        let xi = x as i8;
        let yi = y as i8;
        let is_visible = self.level.is_visible(xi, yi);
        let is_explored = self.level.is_explored(xi, yi);

        // If not explored, show nothing (space)
        if !is_explored {
            return (' ', Style::default());
        }

        // Player position (always visible)
        if xi == self.player.pos.x && yi == self.player.pos.y {
            return ('@', Style::default().fg(Color::White).bold());
        }

        // Only show monsters and objects if currently visible
        if is_visible {
            // Monster at position
            if let Some(monster) = self.level.monster_at(xi, yi) {
                let tile = nh_core::data::tile::get_tile_for_monster(monster.permonst());
                let symbol = tile.to_ascii();
                let color = if monster.is_hostile() {
                    Color::Red
                } else if monster.is_pet() {
                    Color::White
                } else {
                    Color::Yellow
                };
                return (symbol, Style::default().fg(color));
            }

            // Objects at position - show top object's class symbol
            let objects = self.level.objects_at(xi, yi);
            if let Some(obj) = objects.first() {
                let tile = nh_core::data::tile::get_tile_for_object(obj);
                let symbol = tile.to_ascii();
                let color = match obj.class {
                    nh_core::object::ObjectClass::Coin => Color::Yellow,
                    nh_core::object::ObjectClass::Gem => Color::Cyan,
                    nh_core::object::ObjectClass::Potion => Color::Magenta,
                    nh_core::object::ObjectClass::Scroll => Color::White,
                    nh_core::object::ObjectClass::Wand => Color::LightBlue,
                    nh_core::object::ObjectClass::Weapon => Color::Gray,
                    nh_core::object::ObjectClass::Armor => Color::Gray,
                    nh_core::object::ObjectClass::Food => Color::LightRed,
                    _ => Color::Yellow,
                };
                return (symbol, Style::default().fg(color));
            }
        }

        // Terrain (shown if explored, dimmed if not currently visible)
        let cell = &self.level.cells[x][y];
        let tile = Tile::Dungeon(DungeonTile::from(cell.typ));
        let symbol = tile.to_ascii();
        let base_color = match cell.typ {
            CellType::Stone => Color::Black,
            CellType::Room | CellType::Corridor => {
                if cell.lit {
                    Color::White
                } else {
                    Color::DarkGray
                }
            }
            CellType::VWall
            | CellType::HWall
            | CellType::TLCorner
            | CellType::TRCorner
            | CellType::BLCorner
            | CellType::BRCorner
            | CellType::CrossWall
            | CellType::TUWall
            | CellType::TDWall
            | CellType::TLWall
            | CellType::TRWall => Color::Gray,
            CellType::Door => Color::Yellow,
            CellType::Stairs | CellType::Ladder => Color::White,
            CellType::Pool | CellType::Moat | CellType::Water => Color::Blue,
            CellType::Lava => Color::Red,
            CellType::Fountain => Color::Cyan,
            CellType::Altar => Color::Gray,
            CellType::Throne => Color::Yellow,
            CellType::Tree => Color::Green,
            _ => Color::White,
        };

        // Dim explored but not visible cells
        let color = if is_visible {
            base_color
        } else {
            Color::DarkGray
        };

        (symbol, Style::default().fg(color))
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title("NetHack");

        let inner = block.inner(area);
        block.render(area, buf);

        for y in 0..ROWNO.min(inner.height as usize) {
            for x in 0..COLNO.min(inner.width as usize) {
                let (ch, style) = self.cell_display(x, y);
                if let Some(cell) =
                    buf.cell_mut(Position::new(inner.x + x as u16, inner.y + y as u16))
                {
                    cell.set_char(ch);
                    cell.set_style(style);
                }
            }
        }
    }
}
