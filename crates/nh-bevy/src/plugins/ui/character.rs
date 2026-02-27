//! Character sheet UI panel
//!
//! Displays detailed character information:
//! - Attributes (Str, Dex, Con, Int, Wis, Cha)
//! - Combat stats (AC, level, HP, experience)
//! - Resistances and intrinsics
//! - Toggle with '@' key

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use nh_core::player::{AlignmentType, Attribute, Property};

use crate::plugins::game::AppState;
use crate::resources::GameStateResource;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSheetState>()
            .add_systems(Update, toggle_character_sheet.run_if(in_state(AppState::Playing)))
            .add_systems(
                EguiPrimaryContextPass,
                render_character_sheet
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// Character sheet display state
#[derive(Resource, Default)]
pub struct CharacterSheetState {
    pub open: bool,
    pub tab: CharacterTab,
}

#[derive(Default, Clone, Copy, PartialEq)]
pub enum CharacterTab {
    #[default]
    Stats,
    Resistances,
    Skills,
}

/// Toggle character sheet with @ key
fn toggle_character_sheet(
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CharacterSheetState>,
) {
    // @ is Shift+2
    if (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight))
        && input.just_pressed(KeyCode::Digit2)
    {
        state.open = !state.open;
    }
    // Also 'C' for character
    if input.just_pressed(KeyCode::KeyC) && !input.pressed(KeyCode::ControlLeft) {
        state.open = !state.open;
    }
    // Close on Escape
    if input.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }
}

/// Render the character sheet
fn render_character_sheet(
    mut contexts: EguiContexts,
    game_state: Res<GameStateResource>,
    mut sheet_state: ResMut<CharacterSheetState>,
) {
    if !sheet_state.open {
        return ;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return; };
    let state = &game_state.0;
    let player = &state.player;

    egui::Window::new("Character Sheet")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(450.0)
        .show(ctx, |ui| {
            // Header with name and class
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(&player.name)
                        .size(24.0)
                        .strong()
                        .color(egui::Color32::GOLD),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Level {} {:?}",
                            player.exp_level, player.role
                        ))
                        .size(16.0)
                        .color(egui::Color32::LIGHT_GRAY),
                    );
                });
            });

            ui.separator();

            // Tab selection
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(sheet_state.tab == CharacterTab::Stats, "Stats")
                    .clicked()
                {
                    sheet_state.tab = CharacterTab::Stats;
                }
                if ui
                    .selectable_label(sheet_state.tab == CharacterTab::Resistances, "Resistances")
                    .clicked()
                {
                    sheet_state.tab = CharacterTab::Resistances;
                }
                if ui
                    .selectable_label(sheet_state.tab == CharacterTab::Skills, "Combat")
                    .clicked()
                {
                    sheet_state.tab = CharacterTab::Skills;
                }
            });

            ui.separator();
            ui.add_space(5.0);

            match sheet_state.tab {
                CharacterTab::Stats => render_stats_tab(ui, player),
                CharacterTab::Resistances => render_resistances_tab(ui, player),
                CharacterTab::Skills => render_skills_tab(ui, player),
            }

            ui.add_space(10.0);

            // Close button
            ui.vertical_centered(|ui| {
                if ui.button("Close (C or Esc)").clicked() {
                    sheet_state.open = false;
                }
            });
        });

    
}

/// Render the stats tab
fn render_stats_tab(ui: &mut egui::Ui, player: &nh_core::player::You) {
    ui.columns(2, |columns| {
        // Left column: Attributes
        columns[0].group(|ui| {
            ui.label(egui::RichText::new("Attributes").strong());
            ui.separator();

            egui::Grid::new("attributes_grid")
                .num_columns(3)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    let attrs = [
                        ("Strength", player.attr_current.get(Attribute::Strength)),
                        ("Dexterity", player.attr_current.get(Attribute::Dexterity)),
                        (
                            "Constitution",
                            player.attr_current.get(Attribute::Constitution),
                        ),
                        (
                            "Intelligence",
                            player.attr_current.get(Attribute::Intelligence),
                        ),
                        ("Wisdom", player.attr_current.get(Attribute::Wisdom)),
                        ("Charisma", player.attr_current.get(Attribute::Charisma)),
                    ];

                    for (name, value) in attrs {
                        ui.label(name);
                        ui.label(format!("{}", value));

                        // Color indicator
                        let color = if value >= 18 {
                            egui::Color32::GREEN
                        } else if value >= 14 {
                            egui::Color32::LIGHT_GREEN
                        } else if value >= 10 {
                            egui::Color32::YELLOW
                        } else if value >= 6 {
                            egui::Color32::from_rgb(255, 165, 0)
                        } else {
                            egui::Color32::RED
                        };
                        ui.colored_label(color, "●");
                        ui.end_row();
                    }
                });
        });

        // Right column: Vitals
        columns[1].group(|ui| {
            ui.label(egui::RichText::new("Vitals").strong());
            ui.separator();

            egui::Grid::new("vitals_grid")
                .num_columns(2)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label("Hit Points:");
                    let hp_color = if player.hp > player.hp_max / 2 {
                        egui::Color32::GREEN
                    } else if player.hp > player.hp_max / 4 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(hp_color, format!("{}/{}", player.hp, player.hp_max));
                    ui.end_row();

                    ui.label("Energy:");
                    ui.label(format!("{}/{}", player.energy, player.energy_max));
                    ui.end_row();

                    ui.label("Armor Class:");
                    let ac = player.armor_class;
                    let ac_color = if ac <= 0 {
                        egui::Color32::GREEN
                    } else if ac <= 5 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::from_rgb(255, 165, 0)
                    };
                    ui.colored_label(ac_color, format!("{}", ac));
                    ui.end_row();

                    ui.label("Experience:");
                    ui.label(format!("{}", player.exp));
                    ui.end_row();

                    ui.label("Gold:");
                    ui.colored_label(egui::Color32::GOLD, format!("{}", player.gold));
                    ui.end_row();

                    ui.label("Alignment:");
                    let align_str = match player.alignment.typ {
                        AlignmentType::Lawful => "Lawful",
                        AlignmentType::Neutral => "Neutral",
                        AlignmentType::Chaotic => "Chaotic",
                    };
                    ui.label(align_str);
                    ui.end_row();
                });
        });
    });

    ui.add_space(10.0);

    // Hunger status
    ui.horizontal(|ui| {
        ui.label("Hunger:");
        let (hunger_text, hunger_color) = match player.hunger_state {
            nh_core::player::HungerState::Satiated => ("Satiated", egui::Color32::LIGHT_BLUE),
            nh_core::player::HungerState::NotHungry => ("Not Hungry", egui::Color32::GREEN),
            nh_core::player::HungerState::Hungry => ("Hungry", egui::Color32::YELLOW),
            nh_core::player::HungerState::Weak => ("Weak", egui::Color32::from_rgb(255, 165, 0)),
            nh_core::player::HungerState::Fainting => ("Fainting", egui::Color32::RED),
            nh_core::player::HungerState::Fainted => ("Fainted", egui::Color32::DARK_RED),
            nh_core::player::HungerState::Starved => {
                ("Starved", egui::Color32::from_rgb(128, 0, 0))
            }
        };
        ui.colored_label(hunger_color, hunger_text);
    });
}

/// Render the resistances tab
fn render_resistances_tab(ui: &mut egui::Ui, player: &nh_core::player::You) {
    ui.columns(2, |columns| {
        // Left column: Resistances
        columns[0].group(|ui| {
            ui.label(egui::RichText::new("Resistances").strong());
            ui.separator();

            let resistances = [
                ("Fire", player.properties.has(Property::FireResistance)),
                ("Cold", player.properties.has(Property::ColdResistance)),
                ("Sleep", player.properties.has(Property::SleepResistance)),
                (
                    "Disintegrate",
                    player.properties.has(Property::DisintResistance),
                ),
                ("Shock", player.properties.has(Property::ShockResistance)),
                ("Poison", player.properties.has(Property::PoisonResistance)),
                ("Acid", player.properties.has(Property::AcidResistance)),
                ("Stone", player.properties.has(Property::StoneResistance)),
                ("Drain", player.properties.has(Property::DrainResistance)),
                ("Magic", player.properties.has(Property::MagicResistance)),
            ];

            for (name, has) in resistances {
                ui.horizontal(|ui| {
                    let (symbol, color) = if has {
                        ("✓", egui::Color32::GREEN)
                    } else {
                        ("✗", egui::Color32::DARK_GRAY)
                    };
                    ui.colored_label(color, symbol);
                    ui.label(name);
                });
            }
        });

        // Right column: Intrinsics
        columns[1].group(|ui| {
            ui.label(egui::RichText::new("Intrinsics").strong());
            ui.separator();

            let intrinsics = [
                (
                    "See Invisible",
                    player.properties.has(Property::SeeInvisible),
                ),
                ("Telepathy", player.properties.has(Property::Telepathy)),
                ("Warning", player.properties.has(Property::Warning)),
                ("Searching", player.properties.has(Property::Searching)),
                ("Infravision", player.properties.has(Property::Infravision)),
                ("Stealth", player.properties.has(Property::Stealth)),
                ("Speed", player.properties.has(Property::Speed)),
                (
                    "Regeneration",
                    player.properties.has(Property::Regeneration),
                ),
                ("Reflection", player.properties.has(Property::Reflection)),
                ("Free Action", player.properties.has(Property::FreeAction)),
            ];

            for (name, has) in intrinsics {
                ui.horizontal(|ui| {
                    let (symbol, color) = if has {
                        ("✓", egui::Color32::GREEN)
                    } else {
                        ("✗", egui::Color32::DARK_GRAY)
                    };
                    ui.colored_label(color, symbol);
                    ui.label(name);
                });
            }
        });
    });
}

/// Render the skills/combat tab
fn render_skills_tab(ui: &mut egui::Ui, player: &nh_core::player::You) {
    ui.group(|ui| {
        ui.label(egui::RichText::new("Combat Information").strong());
        ui.separator();

        egui::Grid::new("combat_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show(ui, |ui| {
                ui.label("Base to-hit bonus:");
                ui.label(format!("+{}", player.exp_level / 2));
                ui.end_row();

                ui.label("Damage bonus:");
                // Get strength bonus
                let str_val = player.attr_current.get(Attribute::Strength);
                let str_bonus = if str_val >= 18 {
                    6
                } else if str_val >= 16 {
                    3
                } else if str_val >= 14 {
                    2
                } else if str_val >= 10 {
                    1
                } else {
                    0
                };
                ui.label(format!("+{} (from Str)", str_bonus));
                ui.end_row();

                ui.label("Movement speed:");
                let speed = if player.properties.has(Property::VeryFast) {
                    "Very Fast"
                } else if player.properties.has(Property::Speed) {
                    "Fast"
                } else {
                    "Normal"
                };
                ui.label(speed);
                ui.end_row();
            });
    });

    ui.add_space(10.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("Status Effects").strong());
        ui.separator();

        let mut statuses = Vec::new();

        // Check status effect timeouts
        if player.confused_timeout > 0 {
            statuses.push(("Confused", egui::Color32::YELLOW));
        }
        if player.stunned_timeout > 0 {
            statuses.push(("Stunned", egui::Color32::from_rgb(255, 165, 0)));
        }
        if player.hallucinating_timeout > 0 {
            statuses.push(("Hallucinating", egui::Color32::from_rgb(255, 0, 255)));
        }
        if player.blinded_timeout > 0 {
            statuses.push(("Blind", egui::Color32::DARK_GRAY));
        }
        if player.sleeping_timeout > 0 {
            statuses.push(("Sleeping", egui::Color32::LIGHT_BLUE));
        }
        if player.paralyzed_timeout > 0 {
            statuses.push(("Paralyzed", egui::Color32::from_rgb(128, 128, 128)));
        }
        if player.properties.has(Property::Levitation) {
            statuses.push(("Levitating", egui::Color32::LIGHT_BLUE));
        }
        if player.properties.has(Property::Flying) {
            statuses.push(("Flying", egui::Color32::from_rgb(135, 206, 250)));
        }

        if statuses.is_empty() {
            ui.label(
                egui::RichText::new("No active status effects")
                    .color(egui::Color32::GRAY)
                    .italics(),
            );
        } else {
            for (status, color) in statuses {
                ui.colored_label(color, format!("• {}", status));
            }
        }
    });
}
