//! Menu screens - main menu, pause menu, game over, settings
//!
//! Provides:
//! - Main menu with new game, load, settings, quit
//! - Pause menu with resume, save, settings, quit
//! - Game over screen with stats
//! - Settings panel
//! - Save/load browser

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .init_resource::<MenuState>()
            .init_resource::<SaveLoadState>()
            .add_systems(Update, render_main_menu.run_if(in_state(AppState::MainMenu)))
            .add_systems(Update, render_pause_menu.run_if(in_state(AppState::Paused)))
            .add_systems(
                Update,
                render_game_over_screen.run_if(in_state(AppState::GameOver)),
            );
    }
}

/// Tracks which submenu is open
#[derive(Resource, Default)]
pub struct MenuState {
    pub show_settings: bool,
    pub show_save_browser: bool,
    pub show_load_browser: bool,
    /// Where to return after closing settings
    pub return_to: ReturnState,
}

/// State for save/load browser
#[derive(Resource, Default)]
pub struct SaveLoadState {
    /// Cached list of save files
    pub saves: Vec<(std::path::PathBuf, nh_save::SaveHeader)>,
    /// Whether the save list needs refreshing
    pub needs_refresh: bool,
    /// Selected save slot index
    pub selected: Option<usize>,
    /// Status message to display
    pub status_message: Option<String>,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ReturnState {
    #[default]
    MainMenu,
    Paused,
}

/// Game settings that can be adjusted
#[derive(Resource)]
pub struct GameSettings {
    pub camera_sensitivity: f32,
    pub zoom_speed: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            camera_sensitivity: 1.0,
            zoom_speed: 1.0,
            music_volume: 0.7,
            sfx_volume: 1.0,
        }
    }
}

fn render_main_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut menu_state: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,
    mut save_state: ResMut<SaveLoadState>,
    mut game_state: ResMut<GameStateResource>,
) {
    // Full screen dark overlay
    egui::Area::new(egui::Id::new("main_menu_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 220),
            );
        });

    // Show settings if open
    if menu_state.show_settings {
        render_settings_panel(contexts.ctx_mut(), &mut menu_state, &mut settings);
        return;
    }

    // Show load browser if open
    if menu_state.show_load_browser {
        render_load_browser(
            contexts.ctx_mut(),
            &mut menu_state,
            &mut save_state,
            &mut game_state,
            &mut next_state,
        );
        return;
    }

    // Main menu window
    egui::Window::new("NetHack-RS")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(300.0);

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.label(
                    egui::RichText::new("NetHack-RS")
                        .size(48.0)
                        .color(egui::Color32::GOLD)
                        .strong(),
                );

                ui.label(
                    egui::RichText::new("A Rust NetHack in 3D")
                        .size(16.0)
                        .color(egui::Color32::GRAY)
                        .italics(),
                );

                ui.add_space(40.0);

                // Menu buttons
                let button_size = egui::vec2(200.0, 40.0);

                if ui
                    .add_sized(button_size, egui::Button::new("New Game"))
                    .clicked()
                {
                    // Reset to fresh game state
                    game_state.0 = nh_core::GameState::default();
                    next_state.set(AppState::Playing);
                }

                ui.add_space(10.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Load Game"))
                    .clicked()
                {
                    menu_state.show_load_browser = true;
                    save_state.needs_refresh = true;
                }

                ui.add_space(10.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Settings"))
                    .clicked()
                {
                    menu_state.show_settings = true;
                    menu_state.return_to = ReturnState::MainMenu;
                }

                ui.add_space(10.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Quit"))
                    .clicked()
                {
                    exit.send(AppExit::Success);
                }

                ui.add_space(20.0);

                // Footer
                ui.label(
                    egui::RichText::new("Press F1-F4 to change camera modes")
                        .size(12.0)
                        .color(egui::Color32::DARK_GRAY),
                );
            });
        });
}

fn render_pause_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    input: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,
    mut save_state: ResMut<SaveLoadState>,
    game_state: Res<GameStateResource>,
) {
    // Resume on ESC (only if no submenus open)
    if input.just_pressed(KeyCode::Escape)
        && !menu_state.show_settings
        && !menu_state.show_save_browser
    {
        next_state.set(AppState::Playing);
        return;
    }

    // Semi-transparent overlay
    egui::Area::new(egui::Id::new("pause_menu_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
            );
        });

    // Show settings if open
    if menu_state.show_settings {
        render_settings_panel(contexts.ctx_mut(), &mut menu_state, &mut settings);
        return;
    }

    // Show save browser if open
    if menu_state.show_save_browser {
        render_save_browser(
            contexts.ctx_mut(),
            &mut menu_state,
            &mut save_state,
            &game_state,
        );
        return;
    }

    // Pause menu window
    egui::Window::new("Paused")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(250.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                let button_size = egui::vec2(180.0, 35.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Resume"))
                    .clicked()
                {
                    next_state.set(AppState::Playing);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Save Game"))
                    .clicked()
                {
                    menu_state.show_save_browser = true;
                    save_state.needs_refresh = true;
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Settings"))
                    .clicked()
                {
                    menu_state.show_settings = true;
                    menu_state.return_to = ReturnState::Paused;
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Save & Quit"))
                    .clicked()
                {
                    // Quick save to default slot and quit
                    let path = nh_save::default_save_path(&game_state.0.player.name);
                    if let Err(e) = nh_save::save_game(&game_state.0, &path) {
                        eprintln!("Failed to save game: {}", e);
                    }
                    exit.send(AppExit::Success);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Quit Without Saving"))
                    .clicked()
                {
                    exit.send(AppExit::Success);
                }

                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("Press ESC to resume")
                        .size(12.0)
                        .color(egui::Color32::GRAY),
                );
            });
        });
}

fn render_game_over_screen(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut game_state: ResMut<GameStateResource>,
) {
    // Dark overlay
    egui::Area::new(egui::Id::new("game_over_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(50, 0, 0, 200),
            );
        });

    // Game over window
    egui::Window::new("Game Over")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(350.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("YOU DIED")
                        .size(36.0)
                        .color(egui::Color32::RED)
                        .strong(),
                );

                ui.add_space(20.0);

                // Character stats
                let state = &game_state.0;
                ui.group(|ui| {
                    ui.set_min_width(300.0);
                    ui.label(
                        egui::RichText::new("Character Summary")
                            .size(16.0)
                            .strong(),
                    );
                    ui.separator();

                    egui::Grid::new("stats_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Turns survived:");
                            ui.label(format!("{}", state.turns));
                            ui.end_row();

                            ui.label("Dungeon level:");
                            ui.label(format!("{}", state.current_level.dlevel.depth()));
                            ui.end_row();

                            ui.label("Gold collected:");
                            ui.label(format!("{}", state.player.gold));
                            ui.end_row();

                            ui.label("Experience level:");
                            ui.label(format!("{}", state.player.exp_level));
                            ui.end_row();

                            ui.label("Experience:");
                            ui.label(format!("{}", state.player.exp));
                            ui.end_row();
                        });
                });

                // Death message from messages
                if let Some(death_msg) = state.messages.last() {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(death_msg)
                            .color(egui::Color32::LIGHT_RED)
                            .italics(),
                    );
                }

                ui.add_space(20.0);

                let button_size = egui::vec2(150.0, 35.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Try Again"))
                    .clicked()
                {
                    // Reset game state
                    game_state.0 = nh_core::GameState::default();
                    next_state.set(AppState::Playing);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Main Menu"))
                    .clicked()
                {
                    // Reset game state
                    game_state.0 = nh_core::GameState::default();
                    next_state.set(AppState::MainMenu);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Quit"))
                    .clicked()
                {
                    exit.send(AppExit::Success);
                }

                ui.add_space(10.0);
            });
        });
}

/// Render the settings panel (used from both main menu and pause menu)
fn render_settings_panel(
    ctx: &mut egui::Context,
    menu_state: &mut MenuState,
    settings: &mut GameSettings,
) {
    egui::Window::new("Settings")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(ctx, |ui| {
            ui.set_min_width(350.0);

            ui.add_space(10.0);

            // Camera settings
            ui.group(|ui| {
                ui.label(egui::RichText::new("Camera").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Sensitivity:");
                    ui.add(
                        egui::Slider::new(&mut settings.camera_sensitivity, 0.1..=3.0)
                            .show_value(true),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Zoom Speed:");
                    ui.add(
                        egui::Slider::new(&mut settings.zoom_speed, 0.1..=3.0).show_value(true),
                    );
                });
            });

            ui.add_space(10.0);

            // Audio settings
            ui.group(|ui| {
                ui.label(egui::RichText::new("Audio").strong());
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Music Volume:");
                    ui.add(
                        egui::Slider::new(&mut settings.music_volume, 0.0..=1.0)
                            .show_value(true)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("SFX Volume:");
                    ui.add(
                        egui::Slider::new(&mut settings.sfx_volume, 0.0..=1.0)
                            .show_value(true)
                            .custom_formatter(|v, _| format!("{:.0}%", v * 100.0)),
                    );
                });
            });

            ui.add_space(10.0);

            // Key bindings display
            ui.group(|ui| {
                ui.label(egui::RichText::new("Controls").strong());
                ui.separator();

                egui::Grid::new("keybinds")
                    .num_columns(2)
                    .spacing([20.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Movement:");
                        ui.label("h/j/k/l or Arrow keys");
                        ui.end_row();

                        ui.label("Diagonal:");
                        ui.label("y/u/b/n");
                        ui.end_row();

                        ui.label("Inventory:");
                        ui.label("i");
                        ui.end_row();

                        ui.label("Pickup:");
                        ui.label(",");
                        ui.end_row();

                        ui.label("Search:");
                        ui.label("s");
                        ui.end_row();

                        ui.label("Open/Close:");
                        ui.label("o / c");
                        ui.end_row();

                        ui.label("Stairs:");
                        ui.label("< / >");
                        ui.end_row();

                        ui.label("Camera Mode:");
                        ui.label("F1-F4");
                        ui.end_row();

                        ui.label("Zoom:");
                        ui.label("Mouse wheel");
                        ui.end_row();

                        ui.label("Pan:");
                        ui.label("Right-click drag");
                        ui.end_row();

                        ui.label("Message Log:");
                        ui.label("P");
                        ui.end_row();
                    });
            });

            ui.add_space(20.0);

            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(egui::vec2(120.0, 30.0), egui::Button::new("Back"))
                    .clicked()
                {
                    menu_state.show_settings = false;
                }
            });

            ui.add_space(10.0);
        });
}

/// Render save game browser
fn render_save_browser(
    ctx: &mut egui::Context,
    menu_state: &mut MenuState,
    save_state: &mut SaveLoadState,
    game_state: &GameStateResource,
) {
    // Refresh save list if needed
    if save_state.needs_refresh {
        save_state.saves = nh_save::list_saves().unwrap_or_default();
        save_state.needs_refresh = false;
        save_state.selected = None;
    }

    egui::Window::new("Save Game")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.add_space(10.0);

            // Current game info
            ui.group(|ui| {
                ui.label(egui::RichText::new("Current Game").strong());
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Player:");
                    ui.label(&game_state.0.player.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    ui.label(format!("{}", game_state.0.current_level.dlevel.depth()));
                });
                ui.horizontal(|ui| {
                    ui.label("Turns:");
                    ui.label(format!("{}", game_state.0.turns));
                });
            });

            ui.add_space(10.0);

            // Save slot list
            ui.label(egui::RichText::new("Save Slots").strong());
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    // New save slot
                    let new_slot_selected = save_state.selected.is_none();
                    if ui
                        .selectable_label(new_slot_selected, "  [New Save Slot]")
                        .clicked()
                    {
                        save_state.selected = None;
                    }

                    // Existing saves
                    for (i, (_path, header)) in save_state.saves.iter().enumerate() {
                        let selected = save_state.selected == Some(i);
                        let label = format!(
                            "  {} - {} (Turn {})",
                            header.player_name, header.dlevel, header.turns
                        );
                        if ui.selectable_label(selected, &label).clicked() {
                            save_state.selected = Some(i);
                        }
                    }
                });

            ui.add_space(10.0);

            // Status message
            if let Some(msg) = &save_state.status_message {
                ui.label(egui::RichText::new(msg).color(egui::Color32::GREEN));
                ui.add_space(5.0);
            }

            // Action buttons
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let path = if let Some(idx) = save_state.selected {
                        save_state.saves[idx].0.clone()
                    } else {
                        nh_save::default_save_path(&game_state.0.player.name)
                    };

                    match nh_save::save_game(&game_state.0, &path) {
                        Ok(()) => {
                            save_state.status_message = Some("Game saved!".to_string());
                            save_state.needs_refresh = true;
                        }
                        Err(e) => {
                            save_state.status_message = Some(format!("Save failed: {}", e));
                        }
                    }
                }

                if save_state.selected.is_some() && ui.button("Delete").clicked() {
                    if let Some(idx) = save_state.selected {
                        let path = &save_state.saves[idx].0;
                        if nh_save::delete_save(path).is_ok() {
                            save_state.status_message = Some("Save deleted.".to_string());
                            save_state.needs_refresh = true;
                        }
                    }
                }

                if ui.button("Back").clicked() {
                    menu_state.show_save_browser = false;
                    save_state.status_message = None;
                }
            });

            ui.add_space(10.0);
        });
}

/// Render load game browser
fn render_load_browser(
    ctx: &mut egui::Context,
    menu_state: &mut MenuState,
    save_state: &mut SaveLoadState,
    game_state: &mut GameStateResource,
    next_state: &mut NextState<AppState>,
) {
    // Refresh save list if needed
    if save_state.needs_refresh {
        save_state.saves = nh_save::list_saves().unwrap_or_default();
        save_state.needs_refresh = false;
        save_state.selected = None;
    }

    egui::Window::new("Load Game")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(400.0)
        .show(ctx, |ui| {
            ui.add_space(10.0);

            if save_state.saves.is_empty() {
                ui.label(
                    egui::RichText::new("No saved games found.")
                        .color(egui::Color32::GRAY)
                        .italics(),
                );
                ui.add_space(20.0);
            } else {
                ui.label(egui::RichText::new("Select a save file:").strong());
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for (i, (_path, header)) in save_state.saves.iter().enumerate() {
                            let selected = save_state.selected == Some(i);

                            ui.group(|ui| {
                                ui.set_min_width(350.0);
                                let response = ui.selectable_label(selected, "");

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&header.player_name)
                                            .strong()
                                            .size(16.0),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(&header.dlevel)
                                                    .color(egui::Color32::LIGHT_BLUE),
                                            );
                                        },
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label(format!("Turn {}", header.turns));

                                    // Format timestamp
                                    let datetime =
                                        chrono_lite_format(header.timestamp);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(datetime)
                                                    .color(egui::Color32::GRAY)
                                                    .small(),
                                            );
                                        },
                                    );
                                });

                                if response.clicked() {
                                    save_state.selected = Some(i);
                                }
                            });

                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(10.0);
            }

            // Status message
            if let Some(msg) = &save_state.status_message {
                ui.label(egui::RichText::new(msg).color(egui::Color32::RED));
                ui.add_space(5.0);
            }

            // Action buttons
            ui.horizontal(|ui| {
                let can_load = save_state.selected.is_some();

                if ui
                    .add_enabled(can_load, egui::Button::new("Load"))
                    .clicked()
                {
                    if let Some(idx) = save_state.selected {
                        let path = &save_state.saves[idx].0;
                        match nh_save::load_game(path) {
                            Ok(loaded_state) => {
                                game_state.0 = loaded_state;
                                menu_state.show_load_browser = false;
                                save_state.status_message = None;
                                next_state.set(AppState::Playing);
                            }
                            Err(e) => {
                                save_state.status_message = Some(format!("Load failed: {}", e));
                            }
                        }
                    }
                }

                if can_load && ui.button("Delete").clicked() {
                    if let Some(idx) = save_state.selected {
                        let path = &save_state.saves[idx].0;
                        if nh_save::delete_save(path).is_ok() {
                            save_state.needs_refresh = true;
                            save_state.selected = None;
                        }
                    }
                }

                if ui.button("Back").clicked() {
                    menu_state.show_load_browser = false;
                    save_state.status_message = None;
                }
            });

            ui.add_space(10.0);
        });
}

/// Format a Unix timestamp without requiring chrono crate
fn chrono_lite_format(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    if let Ok(duration) = datetime.duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        // Simple relative time
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(secs);
        let ago = now.saturating_sub(secs);

        if ago < 60 {
            "Just now".to_string()
        } else if ago < 3600 {
            format!("{} min ago", ago / 60)
        } else if ago < 86400 {
            format!("{} hours ago", ago / 3600)
        } else {
            format!("{} days ago", ago / 86400)
        }
    } else {
        "Unknown".to_string()
    }
}
