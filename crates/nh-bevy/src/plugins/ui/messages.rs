//! Message log system

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use crate::resources::GameStateResource;
use crate::plugins::ui::UiState;
use crate::plugins::game::AppState;

pub struct MessagesPlugin;

impl Plugin for MessagesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MessageHistory>()
            .add_systems(Update, update_message_history)
            .add_systems(
                EguiPrimaryContextPass,
                render_messages
                    .run_if(in_state(UiState::Ready))
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Stores message history across turns
#[derive(Resource, Default)]
pub struct MessageHistory {
    pub messages: Vec<MessageEntry>,
    pub show_full_log: bool,
}

#[derive(Clone)]
pub struct MessageEntry {
    pub text: String,
    pub turn: u64,
    pub category: MessageCategory,
}

#[derive(Clone, Copy, Default)]
pub enum MessageCategory {
    #[default]
    Info,
    Combat,
    Item,
    Status,
}

impl MessageCategory {
    pub fn color(&self) -> egui::Color32 {
        match self {
            MessageCategory::Info => egui::Color32::WHITE,
            MessageCategory::Combat => egui::Color32::from_rgb(255, 100, 100),
            MessageCategory::Item => egui::Color32::from_rgb(255, 215, 0),
            MessageCategory::Status => egui::Color32::from_rgb(100, 200, 255),
        }
    }
}

fn categorize_message(text: &str) -> MessageCategory {
    let lower = text.to_lowercase();
    if lower.contains("hit")
        || lower.contains("miss")
        || lower.contains("kill")
        || lower.contains("attack")
        || lower.contains("damage")
    {
        MessageCategory::Combat
    } else if lower.contains("pick up")
        || lower.contains("drop")
        || lower.contains("eat")
        || lower.contains("drink")
        || lower.contains("wear")
        || lower.contains("wield")
    {
        MessageCategory::Item
    } else if lower.contains("feel")
        || lower.contains("hungry")
        || lower.contains("weak")
        || lower.contains("poison")
        || lower.contains("confused")
    {
        MessageCategory::Status
    } else {
        MessageCategory::Info
    }
}

fn update_message_history(game_state: Res<GameStateResource>, mut history: ResMut<MessageHistory>) {
    if !game_state.is_changed() {
        return;
    }

    let turn = game_state.0.turns;

    // Add new messages to history
    for msg in &game_state.0.messages {
        if !msg.is_empty() {
            // Avoid duplicates from same turn
            let already_added = history
                .messages
                .iter()
                .rev()
                .take(10)
                .any(|m| m.turn == turn && m.text == *msg);

            if !already_added {
                history.messages.push(MessageEntry {
                    text: msg.clone(),
                    turn,
                    category: categorize_message(msg),
                });
            }
        }
    }

    // Keep only last 100 messages
    if history.messages.len() > 100 {
        let excess = history.messages.len() - 100;
        history.messages.drain(0..excess);
    }
}

fn render_messages(
    mut contexts: EguiContexts,
    mut history: ResMut<MessageHistory>,
    _game_state: Res<GameStateResource>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return; };
    // Toggle full log with 'P' key or 'V' (standard NetHack)
    if input.just_pressed(KeyCode::KeyP)
        || (input.just_pressed(KeyCode::KeyV)
            && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)))
    {
        history.show_full_log = !history.show_full_log;
    }

    // Bottom message area
    egui::Area::new(egui::Id::new("message_log"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -40.0))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180))
                .inner_margin(egui::Margin::same(8))
                .corner_radius(egui::CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.set_min_width(500.0);
                    ui.set_max_width(600.0);

                    if history.show_full_log {
                        // Show scrollable full log
                        ui.label(
                            egui::RichText::new("Message History (P to close)")
                                .color(egui::Color32::GRAY)
                                .small(),
                        );
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .stick_to_bottom(true)
                            .show(ui, |ui| {
                                for entry in &history.messages {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("[{}]", entry.turn))
                                                .color(egui::Color32::DARK_GRAY)
                                                .small(),
                                        );
                                        ui.label(
                                            egui::RichText::new(&entry.text)
                                                .color(entry.category.color()),
                                        );
                                    });
                                }
                            });
                    } else {
                        // Show last 3 messages
                        let recent: Vec<_> = history.messages.iter().rev().take(3).collect();
                        for (i, entry) in recent.iter().rev().enumerate() {
                            // Fade older messages
                            let alpha = match i {
                                0 => 150,
                                1 => 200,
                                _ => 255,
                            };
                            let color = match entry.category {
                                MessageCategory::Info => {
                                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
                                }
                                MessageCategory::Combat => {
                                    egui::Color32::from_rgba_unmultiplied(255, 100, 100, alpha)
                                }
                                MessageCategory::Item => {
                                    egui::Color32::from_rgba_unmultiplied(255, 215, 0, alpha)
                                }
                                MessageCategory::Status => {
                                    egui::Color32::from_rgba_unmultiplied(100, 200, 255, alpha)
                                }
                            };
                            ui.label(egui::RichText::new(&entry.text).color(color));
                        }

                        if history.messages.is_empty() {
                            ui.label(
                                egui::RichText::new("Welcome to NetHack!")
                                    .color(egui::Color32::GRAY),
                            );
                        }
                    }
                });
        });

    
}
