//! Status HUD - HP, energy, hunger, conditions, etc.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::resources::GameStateResource;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_hud);
    }
}

fn render_hud(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let player = &game_state.0.player;
    let state = &game_state.0;

    // Top-left status panel
    egui::Area::new(egui::Id::new("status_hud"))
        .fixed_pos(egui::pos2(10.0, 10.0))
        .show(contexts.ctx_mut().unwrap(), |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.set_min_width(200.0);

                    // FPS
                    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                        if let Some(value) = fps.smoothed() {
                            ui.label(
                                egui::RichText::new(format!("FPS: {:.0}", value))
                                    .color(if value < 30.0 {
                                        egui::Color32::RED
                                    } else {
                                        egui::Color32::GREEN
                                    })
                                    .small(),
                            );
                            ui.add_space(2.0);
                        }
                    }

                    // Player name and level
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(&player.name)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!("Lvl {}", player.exp_level))
                                .color(egui::Color32::YELLOW),
                        );
                    });

                    ui.add_space(4.0);

                    // HP Bar
                    let hp_ratio = player.hp as f32 / player.hp_max.max(1) as f32;
                    let hp_color = if hp_ratio > 0.5 {
                        egui::Color32::from_rgb(50, 205, 50) // Green
                    } else if hp_ratio > 0.25 {
                        egui::Color32::from_rgb(255, 165, 0) // Orange
                    } else {
                        egui::Color32::from_rgb(220, 20, 60) // Red
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("HP").color(egui::Color32::LIGHT_GRAY));
                        let bar_rect = ui.available_rect_before_wrap();
                        let bar_width = 120.0;
                        let bar_height = 14.0;
                        let bar_pos = egui::pos2(bar_rect.min.x + 30.0, bar_rect.min.y);

                        // Background
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(bar_pos, egui::vec2(bar_width, bar_height)),
                            2.0,
                            egui::Color32::from_rgb(40, 40, 40),
                        );
                        // Fill
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                bar_pos,
                                egui::vec2(bar_width * hp_ratio, bar_height),
                            ),
                            2.0,
                            hp_color,
                        );
                        // Text
                        ui.painter().text(
                            bar_pos + egui::vec2(bar_width / 2.0, bar_height / 2.0),
                            egui::Align2::CENTER_CENTER,
                            format!("{}/{}", player.hp, player.hp_max),
                            egui::FontId::proportional(11.0),
                            egui::Color32::WHITE,
                        );
                        ui.add_space(bar_width + 35.0);
                    });

                    ui.add_space(2.0);

                    // Energy/Mana Bar
                    let energy_ratio = player.energy as f32 / player.energy_max.max(1) as f32;
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Pw").color(egui::Color32::LIGHT_GRAY));
                        let bar_rect = ui.available_rect_before_wrap();
                        let bar_width = 120.0;
                        let bar_height = 14.0;
                        let bar_pos = egui::pos2(bar_rect.min.x + 30.0, bar_rect.min.y);

                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(bar_pos, egui::vec2(bar_width, bar_height)),
                            2.0,
                            egui::Color32::from_rgb(40, 40, 40),
                        );
                        ui.painter().rect_filled(
                            egui::Rect::from_min_size(
                                bar_pos,
                                egui::vec2(bar_width * energy_ratio, bar_height),
                            ),
                            2.0,
                            egui::Color32::from_rgb(65, 105, 225), // Royal blue
                        );
                        ui.painter().text(
                            bar_pos + egui::vec2(bar_width / 2.0, bar_height / 2.0),
                            egui::Align2::CENTER_CENTER,
                            format!("{}/{}", player.energy, player.energy_max),
                            egui::FontId::proportional(11.0),
                            egui::Color32::WHITE,
                        );
                        ui.add_space(bar_width + 35.0);
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Stats row
                    ui.horizontal(|ui| {
                        // Dungeon level
                        ui.label(
                            egui::RichText::new(format!("Dlvl:{}", player.level.depth()))
                                .color(egui::Color32::WHITE),
                        );
                        ui.separator();
                        // Gold
                        ui.label(
                            egui::RichText::new(format!("${}", player.gold))
                                .color(egui::Color32::GOLD),
                        );
                        ui.separator();
                        // AC
                        ui.label(
                            egui::RichText::new(format!("AC:{}", player.armor_class))
                                .color(egui::Color32::LIGHT_BLUE),
                        );
                    });

                    ui.add_space(4.0);

                    // Hunger status
                    let (hunger_text, hunger_color) = match player.hunger_state {
                        nh_core::player::HungerState::Satiated => {
                            ("Satiated", egui::Color32::GREEN)
                        }
                        nh_core::player::HungerState::NotHungry => ("", egui::Color32::WHITE),
                        nh_core::player::HungerState::Hungry => ("Hungry", egui::Color32::YELLOW),
                        nh_core::player::HungerState::Weak => ("Weak", egui::Color32::LIGHT_RED),
                        nh_core::player::HungerState::Fainting => ("Fainting", egui::Color32::RED),
                        nh_core::player::HungerState::Fainted => ("Fainted", egui::Color32::RED),
                        nh_core::player::HungerState::Starved => {
                            ("Starved", egui::Color32::DARK_RED)
                        }
                    };

                    if !hunger_text.is_empty() {
                        ui.label(egui::RichText::new(hunger_text).color(hunger_color));
                    }

                    // Status conditions
                    let mut conditions = Vec::new();
                    if player.confused_timeout > 0 {
                        conditions.push(("Conf", egui::Color32::from_rgb(255, 165, 0)));
                    }
                    if player.stunned_timeout > 0 {
                        conditions.push(("Stun", egui::Color32::from_rgb(255, 69, 0)));
                    }
                    if player.blinded_timeout > 0 {
                        conditions.push(("Blind", egui::Color32::from_rgb(100, 100, 100)));
                    }
                    if player.hallucinating_timeout > 0 {
                        conditions.push(("Hallu", egui::Color32::from_rgb(255, 0, 255)));
                    }
                    if player.paralyzed_timeout > 0 {
                        conditions.push(("Para", egui::Color32::from_rgb(128, 0, 128)));
                    }

                    if !conditions.is_empty() {
                        ui.horizontal(|ui| {
                            for (text, color) in conditions {
                                ui.label(egui::RichText::new(text).color(color).small());
                            }
                        });
                    }

                    ui.add_space(4.0);

                    // Turn counter
                    ui.label(
                        egui::RichText::new(format!("T:{}", state.turns))
                            .color(egui::Color32::GRAY)
                            .small(),
                    );
                });
        });

    // Bottom-right help hint
    egui::Area::new(egui::Id::new("help_hint"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .show(contexts.ctx_mut().unwrap(), |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 150))
                .inner_margin(egui::Margin::same(6))
                .corner_radius(egui::CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(
                            "hjklyubn:Move  i:Inventory  F1:Help  F2-F5:Camera  Home:Reset  Esc:Menu",
                        )
                        .color(egui::Color32::GRAY)
                        .small(),
                    );
                });
        });
}
