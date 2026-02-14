//! Minimap overlay showing dungeon layout
//!
//! Provides:
//! - Corner minimap of level layout
//! - Player position marker
//! - Monster markers (hostile=red, pet=green, peaceful=yellow)
//! - Stairs and special location markers
//! - Toggle with 'M' key

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapSettings>().add_systems(
            Update,
            (toggle_minimap, render_minimap).run_if(in_state(AppState::Playing)),
        );
    }
}

/// Minimap display settings
#[derive(Resource)]
pub struct MinimapSettings {
    /// Whether minimap is visible
    pub visible: bool,
    /// Minimap size in pixels
    pub size: f32,
    /// Zoom level (1.0 = 1 tile per pixel, 2.0 = 2 pixels per tile)
    pub zoom: f32,
    /// Show only explored areas
    pub fog_enabled: bool,
    /// Background opacity
    pub background_opacity: u8,
}

impl Default for MinimapSettings {
    fn default() -> Self {
        Self {
            visible: true,
            size: 200.0,
            zoom: 2.5,
            fog_enabled: true,
            background_opacity: 180,
        }
    }
}

/// Toggle minimap with M key
fn toggle_minimap(input: Res<ButtonInput<KeyCode>>, mut settings: ResMut<MinimapSettings>) {
    if input.just_pressed(KeyCode::KeyM) {
        settings.visible = !settings.visible;
    }
}

/// Render the minimap
fn render_minimap(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    settings: Res<MinimapSettings>,
) {
    if !settings.visible {
        return;
    }

    let state = &game_state.0;
    let level = &state.current_level;
    let player_x = state.player.pos.x as usize;
    let player_y = state.player.pos.y as usize;

    // Calculate minimap dimensions
    let map_width = nh_core::COLNO;
    let map_height = nh_core::ROWNO;
    let tile_size = settings.zoom;

    egui::Window::new("Minimap")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 10.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .frame(
            egui::Frame::none().fill(egui::Color32::from_rgba_unmultiplied(
                0,
                0,
                0,
                settings.background_opacity,
            )),
        )
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_size(egui::vec2(settings.size, settings.size * 0.5));

            let (response, painter) = ui.allocate_painter(
                egui::vec2(settings.size, settings.size * 0.5),
                egui::Sense::hover(),
            );

            let rect = response.rect;
            let origin = rect.min;

            // Center the map in the minimap window
            let map_pixel_width = map_width as f32 * tile_size;
            let map_pixel_height = map_height as f32 * tile_size;
            let offset_x = (rect.width() - map_pixel_width) / 2.0;
            let offset_y = (rect.height() - map_pixel_height) / 2.0;

            // Draw terrain
            for y in 0..map_height {
                for x in 0..map_width {
                    let cell = level.cell(x, y);

                    // Skip unexplored cells if fog enabled
                    if settings.fog_enabled && !cell.explored {
                        continue;
                    }

                    let color = cell_to_minimap_color(&cell.typ, cell.explored, cell.lit);

                    if color.a() > 0 {
                        let px = origin.x + offset_x + x as f32 * tile_size;
                        let py = origin.y + offset_y + y as f32 * tile_size;

                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(px, py),
                                egui::vec2(tile_size, tile_size),
                            ),
                            egui::Rounding::ZERO,
                            color,
                        );
                    }
                }
            }

            // Draw stairs and ladders
            for y in 0..map_height {
                for x in 0..map_width {
                    let cell = level.cell(x, y);
                    if !settings.fog_enabled || cell.explored {
                        let stair_color = match cell.typ {
                            nh_core::dungeon::CellType::Stairs => {
                                // Cyan for stairs
                                Some(egui::Color32::from_rgb(100, 200, 255))
                            }
                            nh_core::dungeon::CellType::Ladder => {
                                // Yellow for ladders
                                Some(egui::Color32::from_rgb(200, 200, 100))
                            }
                            _ => None,
                        };

                        if let Some(color) = stair_color {
                            let px = origin.x + offset_x + x as f32 * tile_size + tile_size / 2.0;
                            let py = origin.y + offset_y + y as f32 * tile_size + tile_size / 2.0;
                            painter.circle_filled(egui::pos2(px, py), tile_size * 0.8, color);
                        }
                    }
                }
            }

            // Draw monsters
            for monster in &level.monsters {
                // Skip if in unexplored area and fog enabled
                let cell = level.cell(monster.x as usize, monster.y as usize);
                if settings.fog_enabled && !cell.explored {
                    continue;
                }

                let color = if monster.state.tame {
                    egui::Color32::from_rgb(100, 255, 100) // Green for pets
                } else if monster.state.peaceful {
                    egui::Color32::from_rgb(255, 255, 100) // Yellow for peaceful
                } else {
                    egui::Color32::from_rgb(255, 80, 80) // Red for hostile
                };

                let px = origin.x + offset_x + monster.x as f32 * tile_size + tile_size / 2.0;
                let py = origin.y + offset_y + monster.y as f32 * tile_size + tile_size / 2.0;
                painter.circle_filled(egui::pos2(px, py), tile_size * 0.6, color);
            }

            // Draw player (on top)
            let player_px = origin.x + offset_x + player_x as f32 * tile_size + tile_size / 2.0;
            let player_py = origin.y + offset_y + player_y as f32 * tile_size + tile_size / 2.0;

            // Player marker: white circle with border
            painter.circle_filled(
                egui::pos2(player_px, player_py),
                tile_size * 0.9,
                egui::Color32::WHITE,
            );
            painter.circle_stroke(
                egui::pos2(player_px, player_py),
                tile_size * 0.9,
                egui::Stroke::new(1.0, egui::Color32::BLACK),
            );

            // Legend at bottom
            let legend_y = rect.max.y - 12.0;
            painter.text(
                egui::pos2(rect.min.x + 5.0, legend_y),
                egui::Align2::LEFT_CENTER,
                "M: Toggle",
                egui::FontId::proportional(10.0),
                egui::Color32::GRAY,
            );
        });
}

/// Convert cell type to minimap color
fn cell_to_minimap_color(
    cell_type: &nh_core::dungeon::CellType,
    _explored: bool,
    lit: bool,
) -> egui::Color32 {
    use nh_core::dungeon::CellType;

    // Dim color for explored but unlit areas
    let brightness = if lit { 1.0 } else { 0.6 };
    let b = |v: u8| (v as f32 * brightness) as u8;

    match cell_type {
        // Empty/void
        CellType::Stone => egui::Color32::TRANSPARENT,

        // Walls
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
        | CellType::TRWall
        | CellType::DBWall
        | CellType::Wall => egui::Color32::from_rgb(b(100), b(100), b(120)),

        // Floors
        CellType::Room | CellType::Corridor | CellType::Vault => {
            egui::Color32::from_rgb(b(60), b(50), b(40))
        }

        // Doors
        CellType::Door | CellType::SecretDoor | CellType::SecretCorridor => {
            egui::Color32::from_rgb(b(139), b(90), b(43))
        }

        // Water features
        CellType::Pool | CellType::Moat | CellType::Water => {
            egui::Color32::from_rgb(b(30), b(80), b(150))
        }

        // Lava
        CellType::Lava => egui::Color32::from_rgb(b(255), b(100), b(30)),

        // Ice
        CellType::Ice => egui::Color32::from_rgb(b(180), b(220), b(255)),

        // Special locations
        CellType::Altar => egui::Color32::from_rgb(b(200), b(200), b(255)),
        CellType::Grave => egui::Color32::from_rgb(b(100), b(100), b(100)),
        CellType::Throne => egui::Color32::from_rgb(b(255), b(215), b(0)),
        CellType::Sink => egui::Color32::from_rgb(b(150), b(150), b(180)),
        CellType::Fountain => egui::Color32::from_rgb(b(100), b(150), b(255)),

        // Nature
        CellType::Tree => egui::Color32::from_rgb(b(34), b(100), b(34)),

        // Clouds/air
        CellType::Cloud | CellType::Air => {
            egui::Color32::from_rgba_unmultiplied(b(200), b(200), b(255), 80)
        }

        // Iron bars
        CellType::IronBars => egui::Color32::from_rgb(b(80), b(80), b(80)),

        // Drawbridge
        CellType::DrawbridgeUp | CellType::DrawbridgeDown => {
            egui::Color32::from_rgb(b(100), b(80), b(60))
        }

        // Stairs and ladder (floor color, markers drawn separately)
        CellType::Stairs | CellType::Ladder => egui::Color32::from_rgb(b(60), b(50), b(40)),
    }
}
