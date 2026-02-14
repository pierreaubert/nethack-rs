//! Discoveries UI panel - shows identified items

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use nh_core::object::ObjectClass;

use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct DiscoveriesPlugin;

impl Plugin for DiscoveriesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiscoveriesState>().add_systems(
            Update,
            render_discoveries.run_if(in_state(AppState::Playing)),
        );
    }
}

#[derive(Resource, Default)]
pub struct DiscoveriesState {
    pub open: bool,
}

fn render_discoveries(
    mut contexts: EguiContexts,
    mut state: ResMut<DiscoveriesState>,
    game_state: Res<GameStateResource>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // Toggle with '\' key
    if input.just_pressed(KeyCode::Backslash) {
        state.open = !state.open;
    }

    if !state.open {
        return;
    }

    egui::Window::new("Discoveries")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(true)
        .default_width(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.label(egui::RichText::new("Identified Items").strong());
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    let categories = [
                        ObjectClass::Weapon,
                        ObjectClass::Armor,
                        ObjectClass::Ring,
                        ObjectClass::Amulet,
                        ObjectClass::Tool,
                        ObjectClass::Food,
                        ObjectClass::Potion,
                        ObjectClass::Scroll,
                        ObjectClass::Spellbook,
                        ObjectClass::Wand,
                        ObjectClass::Gem,
                    ];

                    for cat in categories {
                        render_category(ui, cat, &game_state.0);
                    }
                });

            ui.separator();
            if ui.button("Close").clicked() {
                state.open = false;
            }
        });
}

fn render_category(ui: &mut egui::Ui, class: ObjectClass, _game_state: &nh_core::GameState) {
    // In a real NetHack engine, we'd check game_state.identified_objects
    // For this prototype, we'll show all objects of this class from nh_data
    // but marked as "Known" or "Unknown" (appearance only).

    let objects = nh_core::data::objects::OBJECTS;
    let mut found = false;

    // Filter objects by class
    let cat_objects: Vec<_> = objects
        .iter()
        .enumerate()
        .filter(|(_, obj)| obj.class == class && !obj.name.is_empty())
        .collect();

    if cat_objects.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("{:?}s", class))
        .default_open(false)
        .show(ui, |ui| {
            found = true;
            for (_, obj) in cat_objects {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(obj.name).color(egui::Color32::WHITE));
                    if !obj.description.is_empty() {
                        ui.label(
                            egui::RichText::new(format!("({})", obj.description))
                                .color(egui::Color32::GRAY)
                                .small(),
                        );
                    }
                });
            }
        });
}
