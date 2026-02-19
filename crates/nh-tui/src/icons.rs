//! Image-based tile rendering for terminals with inline image support.
//!
//! When `--icons` is passed, the map is rendered as a single composed image
//! using PNG sprites for objects and colored rectangles for terrain/monsters.

use std::collections::HashMap;
use std::path::PathBuf;

use image::{DynamicImage, Rgba, RgbaImage};

use nh_assets::registry::AssetRegistry;
use nh_core::dungeon::{CellType, Level};
use nh_core::player::You;
use nh_core::{COLNO, ROWNO};

/// Cache for loaded and pre-resized sprite images.
pub struct ImageTileCache {
    /// Raw loaded sprites keyed by bevy_sprite path.
    sprites: HashMap<String, DynamicImage>,
    /// Pre-resized sprites keyed by (sprite_path, tile_px).
    resized: HashMap<(String, u32), RgbaImage>,
    /// Base directory for sprite assets.
    assets_base: PathBuf,
}

impl ImageTileCache {
    pub fn new(assets_base: PathBuf) -> Self {
        Self {
            sprites: HashMap::new(),
            resized: HashMap::new(),
            assets_base,
        }
    }

    /// Get a sprite resized to tile_px × tile_px (cached).
    pub fn get_resized(&mut self, sprite_path: &str, tile_px: u32) -> Option<&RgbaImage> {
        let key = (sprite_path.to_string(), tile_px);
        if !self.resized.contains_key(&key) {
            // Load the raw sprite first
            let full_path = self.assets_base.join(sprite_path);
            if !self.sprites.contains_key(sprite_path) && let Ok(img) = image::open(&full_path) {
                self.sprites.insert(sprite_path.to_string(), img);
            }
            if let Some(raw) = self.sprites.get(sprite_path) {
                let resized = raw
                    .resize_exact(tile_px, tile_px, image::imageops::FilterType::Nearest)
                    .to_rgba8();
                self.resized.insert(key.clone(), resized);
            }
        }
        self.resized.get(&key)
    }

    /// Invalidate resized cache (e.g., when terminal resizes and tile_px changes).
    pub fn clear_resized(&mut self) {
        self.resized.clear();
    }
}

/// Locate the assets/items/ directory relative to the binary or current dir.
pub fn find_assets_base() -> PathBuf {
    // Try relative to CWD first
    let candidates = [
        PathBuf::from("assets/items"),
        PathBuf::from("crates/nh-tui/../../assets/items"),
    ];
    for c in &candidates {
        if c.is_dir() {
            return c.clone();
        }
    }
    // Fallback
    PathBuf::from("assets/items")
}

/// Compose the full dungeon map as a single RGBA image.
pub fn compose_map_image(
    level: &Level,
    player: &You,
    assets: &AssetRegistry,
    cache: &mut ImageTileCache,
    tile_px: u32,
) -> DynamicImage {
    let canvas_w = COLNO as u32 * tile_px;
    let canvas_h = ROWNO as u32 * tile_px;
    let mut canvas = RgbaImage::new(canvas_w, canvas_h);

    for y in 0..ROWNO {
        for x in 0..COLNO {
            let px_x = x as u32 * tile_px;
            let px_y = y as u32 * tile_px;
            let xi = x as i8;
            let yi = y as i8;

            let is_visible = level.is_visible(xi, yi);
            let is_explored = level.is_explored(xi, yi);

            if !is_explored {
                // Unexplored: black
                fill_rect(&mut canvas, px_x, px_y, tile_px, tile_px, Rgba([0, 0, 0, 255]));
                continue;
            }

            // Terrain background
            let terrain_color = terrain_rgba(&level.cells[x][y].typ, level.cells[x][y].lit, is_visible);
            fill_rect(&mut canvas, px_x, px_y, tile_px, tile_px, terrain_color);

            // Player (drawn on top of terrain, before visibility-gated items)
            if xi == player.pos.x && yi == player.pos.y {
                let inset = tile_px / 4;
                fill_rect(
                    &mut canvas,
                    px_x + inset,
                    px_y + inset,
                    tile_px - 2 * inset,
                    tile_px - 2 * inset,
                    Rgba([255, 255, 255, 255]),
                );
                // Draw @ in the center (as a small bright marker)
                let center_inset = tile_px * 3 / 8;
                fill_rect(
                    &mut canvas,
                    px_x + center_inset,
                    px_y + center_inset,
                    tile_px - 2 * center_inset,
                    tile_px - 2 * center_inset,
                    Rgba([255, 255, 0, 255]),
                );
                continue;
            }

            if !is_visible {
                continue;
            }

            // Monsters
            if let Some(monster) = level.monster_at(xi, yi) {
                let color = if monster.is_hostile() {
                    Rgba([220, 50, 50, 220])
                } else if monster.is_pet() {
                    Rgba([50, 220, 50, 220])
                } else {
                    Rgba([200, 200, 60, 220])
                };
                let inset = tile_px / 4;
                fill_rect(
                    &mut canvas,
                    px_x + inset,
                    px_y + inset,
                    tile_px - 2 * inset,
                    tile_px - 2 * inset,
                    color,
                );
                continue;
            }

            // Objects — try sprite, else colored rectangle
            let objects = level.objects_at(xi, yi);
            if let Some(obj) = objects.first() {
                let mut drawn = false;
                if let Some(sprite_path) = assets.get_sprite_path(obj)
                    && let Some(sprite) = cache.get_resized(sprite_path, tile_px)
                {
                    overlay_rgba(&mut canvas, sprite, px_x, px_y);
                    drawn = true;
                }
                if !drawn {
                    // Fallback: small colored square
                    let inset = tile_px / 3;
                    fill_rect(
                        &mut canvas,
                        px_x + inset,
                        px_y + inset,
                        tile_px - 2 * inset,
                        tile_px - 2 * inset,
                        Rgba([180, 180, 220, 200]),
                    );
                }
            }
        }
    }

    DynamicImage::ImageRgba8(canvas)
}

/// Map terrain cell type to an RGBA color.
fn terrain_rgba(cell_type: &CellType, lit: bool, visible: bool) -> Rgba<u8> {
    let base = match cell_type {
        CellType::Stone => Rgba([30, 30, 30, 255]),
        CellType::Room | CellType::Corridor => {
            if lit {
                Rgba([70, 60, 45, 255])
            } else {
                Rgba([35, 30, 22, 255])
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
        | CellType::TRWall => Rgba([120, 120, 120, 255]),
        CellType::Door => Rgba([140, 100, 50, 255]),
        CellType::Stairs | CellType::Ladder => Rgba([200, 200, 200, 255]),
        CellType::Pool | CellType::Moat | CellType::Water => Rgba([20, 60, 160, 255]),
        CellType::Lava => Rgba([200, 60, 0, 255]),
        CellType::Fountain => Rgba([60, 120, 220, 255]),
        CellType::Altar => Rgba([180, 180, 220, 255]),
        CellType::Throne => Rgba([180, 160, 40, 255]),
        CellType::Tree => Rgba([30, 100, 30, 255]),
        _ => Rgba([40, 40, 40, 255]),
    };

    if visible {
        base
    } else {
        // Dim explored-but-not-visible cells
        Rgba([base[0] / 2, base[1] / 2, base[2] / 2, base[3]])
    }
}

/// Fill a rectangle on the canvas with a solid color.
fn fill_rect(canvas: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    let (cw, ch) = (canvas.width(), canvas.height());
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px < cw && py < ch {
                canvas.put_pixel(px, py, color);
            }
        }
    }
}

/// Overlay an RGBA image onto the canvas with alpha blending.
fn overlay_rgba(canvas: &mut RgbaImage, overlay: &RgbaImage, x: u32, y: u32) {
    let (cw, ch) = (canvas.width(), canvas.height());
    let (ow, oh) = (overlay.width(), overlay.height());
    for dy in 0..oh {
        for dx in 0..ow {
            let px = x + dx;
            let py = y + dy;
            if px < cw && py < ch {
                let src = overlay.get_pixel(dx, dy);
                if src[3] == 0 {
                    continue;
                }
                if src[3] == 255 {
                    canvas.put_pixel(px, py, *src);
                } else {
                    // Alpha blend
                    let dst = canvas.get_pixel(px, py);
                    let sa = src[3] as u16;
                    let da = 255 - sa;
                    let r = ((src[0] as u16 * sa + dst[0] as u16 * da) / 255) as u8;
                    let g = ((src[1] as u16 * sa + dst[1] as u16 * da) / 255) as u8;
                    let b = ((src[2] as u16 * sa + dst[2] as u16 * da) / 255) as u8;
                    canvas.put_pixel(px, py, Rgba([r, g, b, 255]));
                }
            }
        }
    }
}
