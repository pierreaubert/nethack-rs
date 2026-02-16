//! Map display widget

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Widget};

use nh_assets::registry::AssetRegistry;
use nh_core::data::tile::{DungeonTile, Tile};
use nh_core::dungeon::{CellType, Level};
use nh_core::player::You;
use nh_core::{COLNO, ROWNO};

use crate::theme::Theme;

/// Widget for rendering the dungeon map
pub struct MapWidget<'a> {
    level: &'a Level,
    player: &'a You,
    assets: &'a AssetRegistry,
    theme: &'a Theme,
}

impl<'a> MapWidget<'a> {
    pub fn new(
        level: &'a Level,
        player: &'a You,
        assets: &'a AssetRegistry,
        theme: &'a Theme,
    ) -> Self {
        Self {
            level,
            player,
            assets,
            theme,
        }
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
            return ('@', Style::default().fg(self.theme.map_player).bold());
        }

        // Only show monsters and objects if currently visible
        if is_visible {
            // Monster at position
            if let Some(monster) = self.level.monster_at(xi, yi) {
                let tile = nh_core::data::tile::get_tile_for_monster(monster.permonst());
                let symbol = tile.to_ascii();
                let color = if monster.is_hostile() {
                    self.theme.map_hostile
                } else if monster.is_pet() {
                    self.theme.map_pet
                } else {
                    self.theme.map_peaceful
                };
                return (symbol, Style::default().fg(color));
            }

            // Objects at position - show top object's class symbol
            let objects = self.level.objects_at(xi, yi);
            if let Some(obj) = objects.first() {
                // Use the shared asset registry for item icons
                if let Ok(icon) = self.assets.get_icon(obj) {
                    let color = AssetRegistry::parse_color(&icon.tui_color)
                        .unwrap_or(self.theme.obj_default);
                    return (icon.tui_char, Style::default().fg(color));
                }

                // Fallback to core tile system if registry lookup fails
                let tile = nh_core::data::tile::get_tile_for_object(obj);
                let symbol = tile.to_ascii();
                let color = match obj.class {
                    nh_core::object::ObjectClass::Coin => self.theme.obj_coin,
                    nh_core::object::ObjectClass::Gem => self.theme.obj_gem,
                    nh_core::object::ObjectClass::Potion => self.theme.obj_potion,
                    nh_core::object::ObjectClass::Scroll => self.theme.obj_scroll,
                    nh_core::object::ObjectClass::Wand => self.theme.obj_wand,
                    nh_core::object::ObjectClass::Weapon => self.theme.obj_weapon,
                    nh_core::object::ObjectClass::Armor => self.theme.obj_weapon,
                    nh_core::object::ObjectClass::Food => self.theme.obj_food,
                    _ => self.theme.obj_default,
                };
                return (symbol, Style::default().fg(color));
            }
        }

        // Terrain (shown if explored, dimmed if not currently visible)
        let cell = &self.level.cells[x][y];
        let tile = Tile::Dungeon(DungeonTile::from(cell.typ));
        let symbol = tile.to_ascii();
        let t = self.theme;
        let base_color = match cell.typ {
            CellType::Stone => t.map_stone,
            CellType::Room | CellType::Corridor => {
                if cell.lit {
                    t.map_floor_lit
                } else {
                    t.map_floor_dark
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
            | CellType::TRWall => t.map_wall,
            CellType::Door => t.map_door,
            CellType::Stairs | CellType::Ladder => t.map_stairs,
            CellType::Pool | CellType::Moat | CellType::Water => t.map_water,
            CellType::Lava => t.map_lava,
            CellType::Fountain => t.map_fountain,
            CellType::Altar => t.map_altar,
            CellType::Throne => t.map_throne,
            CellType::Tree => t.map_tree,
            _ => t.map_default,
        };

        // Dim explored but not visible cells
        let color = if is_visible {
            base_color
        } else {
            t.map_explored
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
