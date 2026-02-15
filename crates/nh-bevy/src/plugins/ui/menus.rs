//! Menu screens - main menu, pause menu, game over, character creation, victory, settings
//!
//! Provides:
//! - Main menu with new game, load, settings, quit
//! - Character creation wizard (name, role, race, gender, alignment)
//! - Pause menu with resume, save, settings, quit
//! - Game over screen with full stats (attributes, conducts, inventory)
//! - Victory screen for ascension
//! - Settings panel
//! - Save/load browser

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::IntoEnumIterator;

use nh_core::player::{AlignmentType, Attribute, Gender, Race, Role};

use crate::plugins::game::AppState;
use crate::resources::{
    CharacterCreationState, CharacterCreationStep, GameOverInfo, GameStateResource,
};

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .init_resource::<MenuState>()
            .init_resource::<SaveLoadState>()
            .init_resource::<CharacterCreationState>()
            .init_resource::<GameOverInfo>()
            .add_systems(
                Update,
                render_main_menu.run_if(in_state(AppState::MainMenu)),
            )
            .add_systems(
                Update,
                render_character_creation.run_if(in_state(AppState::CharacterCreation)),
            )
            .add_systems(Update, render_pause_menu.run_if(in_state(AppState::Paused)))
            .add_systems(
                Update,
                render_game_over_screen.run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                render_victory_screen.run_if(in_state(AppState::Victory)),
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
    pub saves: Vec<(std::path::PathBuf, nh_core::save::SaveHeader)>,
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

#[allow(clippy::too_many_arguments)]
fn render_main_menu(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    mut menu_state: ResMut<MenuState>,
    mut settings: ResMut<GameSettings>,
    mut save_state: ResMut<SaveLoadState>,
    mut game_state: ResMut<GameStateResource>,
    mut cc_state: ResMut<CharacterCreationState>,
) {
    // Full screen dark overlay (non-interactable so clicks pass through to windows)
    egui::Area::new(egui::Id::new("main_menu_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .interactable(false)
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
                    cc_state.reset();
                    next_state.set(AppState::CharacterCreation);
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
        .interactable(false)
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
                    let path = nh_core::save::default_save_path(&game_state.0.player.name);
                    if let Err(e) = nh_core::save::save_game(&game_state.0, &path) {
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

fn render_character_creation(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut cc_state: ResMut<CharacterCreationState>,
    mut game_state: ResMut<GameStateResource>,
) {
    // Dark overlay
    egui::Area::new(egui::Id::new("cc_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .interactable(false)
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 230),
            );
        });

    egui::Window::new("Character Creation")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .min_width(400.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Create Your Character")
                        .size(28.0)
                        .color(egui::Color32::GOLD)
                        .strong(),
                );
                ui.add_space(15.0);
            });

            match cc_state.step {
                CharacterCreationStep::EnterName => {
                    ui.label(egui::RichText::new("What is your name?").size(16.0));
                    ui.add_space(8.0);
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut cc_state.name)
                            .desired_width(300.0)
                            .hint_text("Enter your name..."),
                    );
                    response.request_focus();
                    ui.add_space(10.0);
                    let name_valid = !cc_state.name.trim().is_empty();
                    if ui
                        .add_enabled(name_valid, egui::Button::new("Continue"))
                        .clicked()
                        || (response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && name_valid)
                    {
                        cc_state.name = cc_state.name.trim().to_string();
                        cc_state.step = CharacterCreationStep::AskRandom;
                    }
                }
                CharacterCreationStep::AskRandom => {
                    ui.label(
                        egui::RichText::new("Randomize your character?").size(16.0),
                    );
                    ui.add_space(8.0);
                    ui.label("A random character will be assigned a role, race, gender, and alignment.");
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_sized(egui::vec2(150.0, 35.0), egui::Button::new("Yes, random!"))
                            .clicked()
                        {
                            let roles: Vec<Role> = Role::iter().collect();
                            let races: Vec<Race> = Race::iter().collect();
                            let genders: Vec<Gender> =
                                Gender::iter().filter(|g| *g != Gender::Neuter).collect();
                            let aligns: Vec<AlignmentType> = AlignmentType::iter().collect();
                            cc_state.role =
                                Some(roles[fastrand::usize(..roles.len())]);
                            cc_state.race =
                                Some(races[fastrand::usize(..races.len())]);
                            cc_state.gender =
                                Some(genders[fastrand::usize(..genders.len())]);
                            cc_state.alignment =
                                Some(aligns[fastrand::usize(..aligns.len())]);
                            cc_state.step = CharacterCreationStep::Done;
                        }
                        if ui
                            .add_sized(egui::vec2(150.0, 35.0), egui::Button::new("No, I'll choose"))
                            .clicked()
                        {
                            cc_state.cursor = 0;
                            cc_state.step = CharacterCreationStep::SelectRole;
                        }
                    });
                }
                CharacterCreationStep::SelectRole => {
                    ui.label(egui::RichText::new("Choose your role:").size(16.0));
                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for role in Role::iter() {
                                let selected = cc_state.role == Some(role);
                                if ui
                                    .selectable_label(selected, format!("  {role}"))
                                    .clicked()
                                {
                                    cc_state.role = Some(role);
                                }
                            }
                        });
                    ui.add_space(10.0);
                    if ui
                        .add_enabled(cc_state.role.is_some(), egui::Button::new("Continue"))
                        .clicked()
                    {
                        cc_state.cursor = 0;
                        cc_state.step = CharacterCreationStep::SelectRace;
                    }
                }
                CharacterCreationStep::SelectRace => {
                    ui.label(egui::RichText::new("Choose your race:").size(16.0));
                    ui.add_space(8.0);
                    for race in Race::iter() {
                        let selected = cc_state.race == Some(race);
                        if ui
                            .selectable_label(selected, format!("  {race}"))
                            .clicked()
                        {
                            cc_state.race = Some(race);
                        }
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            cc_state.step = CharacterCreationStep::SelectRole;
                        }
                        if ui
                            .add_enabled(cc_state.race.is_some(), egui::Button::new("Continue"))
                            .clicked()
                        {
                            cc_state.step = CharacterCreationStep::SelectGender;
                        }
                    });
                }
                CharacterCreationStep::SelectGender => {
                    ui.label(egui::RichText::new("Choose your gender:").size(16.0));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        for gender in [Gender::Male, Gender::Female] {
                            let selected = cc_state.gender == Some(gender);
                            if ui
                                .selectable_label(selected, format!("  {gender}  "))
                                .clicked()
                            {
                                cc_state.gender = Some(gender);
                            }
                        }
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            cc_state.step = CharacterCreationStep::SelectRace;
                        }
                        if ui
                            .add_enabled(
                                cc_state.gender.is_some(),
                                egui::Button::new("Continue"),
                            )
                            .clicked()
                        {
                            cc_state.step = CharacterCreationStep::SelectAlignment;
                        }
                    });
                }
                CharacterCreationStep::SelectAlignment => {
                    ui.label(egui::RichText::new("Choose your alignment:").size(16.0));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        for align in AlignmentType::iter() {
                            let selected = cc_state.alignment == Some(align);
                            if ui
                                .selectable_label(selected, format!("  {align}  "))
                                .clicked()
                            {
                                cc_state.alignment = Some(align);
                            }
                        }
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            cc_state.step = CharacterCreationStep::SelectGender;
                        }
                        if ui
                            .add_enabled(
                                cc_state.alignment.is_some(),
                                egui::Button::new("Continue"),
                            )
                            .clicked()
                        {
                            cc_state.step = CharacterCreationStep::Done;
                        }
                    });
                }
                CharacterCreationStep::Done => {
                    let role = cc_state.role.unwrap_or_default();
                    let race = cc_state.race.unwrap_or_default();
                    let gender = cc_state.gender.unwrap_or_default();
                    let alignment = cc_state.alignment.unwrap_or_default();

                    ui.group(|ui| {
                        ui.set_min_width(350.0);
                        ui.label(egui::RichText::new("Character Summary").size(16.0).strong());
                        ui.separator();
                        egui::Grid::new("cc_summary")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Name:");
                                ui.label(
                                    egui::RichText::new(&cc_state.name)
                                        .strong()
                                        .color(egui::Color32::LIGHT_BLUE),
                                );
                                ui.end_row();
                                ui.label("Role:");
                                ui.label(format!("{role}"));
                                ui.end_row();
                                ui.label("Race:");
                                ui.label(format!("{race}"));
                                ui.end_row();
                                ui.label("Gender:");
                                ui.label(format!("{gender}"));
                                ui.end_row();
                                ui.label("Alignment:");
                                ui.label(format!("{alignment}"));
                                ui.end_row();
                            });
                    });

                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button("Back").clicked() {
                            cc_state.step = CharacterCreationStep::SelectAlignment;
                        }
                        if ui
                            .add_sized(
                                egui::vec2(200.0, 40.0),
                                egui::Button::new(
                                    egui::RichText::new("Start Adventure!").size(16.0).strong(),
                                ),
                            )
                            .clicked()
                        {
                            // Create properly initialized game state
                            let rng = nh_core::GameRng::from_entropy();
                            let mut state = nh_core::GameState::new_with_identity(
                                rng,
                                cc_state.name.clone(),
                                role,
                                race,
                                gender,
                            );
                            state.player.alignment =
                                nh_core::player::Alignment::new(alignment);

                            // Welcome messages
                            let title = state.player.rank_title();
                            state.message(format!(
                                "Welcome to NetHack, {} the {} {} {}!",
                                state.player.name, alignment, race, title,
                            ));
                            state.message(
                                "Be careful! You are about to enter the Dungeons of Doom...",
                            );

                            game_state.0 = state;
                            next_state.set(AppState::Playing);
                        }
                    });
                }
            }

            ui.add_space(10.0);
        });
}

fn render_game_over_screen(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    game_state: Res<GameStateResource>,
    game_over_info: Res<GameOverInfo>,
    mut cc_state: ResMut<CharacterCreationState>,
) {
    // Dark overlay
    egui::Area::new(egui::Id::new("game_over_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .interactable(false)
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(50, 0, 0, 200),
            );
        });

    egui::Window::new("Game Over")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(450.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("R.I.P.")
                        .size(42.0)
                        .color(egui::Color32::RED)
                        .strong(),
                );

                ui.add_space(5.0);

                let state = &game_state.0;
                let player = &state.player;

                // Player identity
                ui.label(
                    egui::RichText::new(format!(
                        "{} the {} {} {}",
                        player.name, player.alignment.typ, player.race, player.rank_title()
                    ))
                    .size(18.0)
                    .color(egui::Color32::LIGHT_GRAY),
                );

                // Cause of death
                if let Some(cause) = &game_over_info.cause_of_death {
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new(cause)
                            .color(egui::Color32::LIGHT_RED)
                            .italics(),
                    );
                }

                ui.add_space(15.0);

                // Stats
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Final Statistics").size(16.0).strong());
                    ui.separator();

                    egui::Grid::new("death_stats_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Turns:");
                            ui.label(format!("{}", state.turns));
                            ui.end_row();

                            ui.label("Dungeon level:");
                            ui.label(format!("{}", state.current_level.dlevel.depth()));
                            ui.end_row();

                            ui.label("Gold:");
                            ui.label(format!("{}", player.gold));
                            ui.end_row();

                            ui.label("Experience:");
                            ui.label(format!(
                                "Level {} ({} pts)",
                                player.exp_level, player.exp
                            ));
                            ui.end_row();

                            ui.label("HP:");
                            ui.label(format!("{}/{}", player.hp, player.hp_max));
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // Attributes
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Attributes").size(14.0).strong());
                    ui.separator();

                    egui::Grid::new("death_attrs_grid")
                        .num_columns(6)
                        .spacing([15.0, 4.0])
                        .show(ui, |ui| {
                            for attr in Attribute::ALL {
                                ui.label(
                                    egui::RichText::new(attr.short_name())
                                        .strong()
                                        .color(egui::Color32::LIGHT_BLUE),
                                );
                            }
                            ui.end_row();
                            for attr in Attribute::ALL {
                                ui.label(format!(
                                    "{}",
                                    player.attr_current.get(attr)
                                ));
                            }
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // Conducts
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Conducts").size(14.0).strong());
                    ui.separator();

                    let conduct = &player.conduct;
                    let checks: &[(&str, bool)] = &[
                        ("Foodless", conduct.is_foodless()),
                        ("Vegan", conduct.is_vegan()),
                        ("Vegetarian", conduct.is_vegetarian()),
                        ("Atheist", conduct.is_atheist()),
                        ("Weaponless", conduct.is_weaponless()),
                        ("Pacifist", conduct.is_pacifist()),
                        ("Illiterate", conduct.is_illiterate()),
                        ("Wishless", conduct.is_wishless()),
                        ("Genocideless", conduct.is_genocideless()),
                    ];

                    egui::Grid::new("death_conduct_grid")
                        .num_columns(2)
                        .spacing([20.0, 2.0])
                        .show(ui, |ui| {
                            for (name, maintained) in checks {
                                let (icon, color) = if *maintained {
                                    ("*", egui::Color32::GREEN)
                                } else {
                                    ("-", egui::Color32::DARK_GRAY)
                                };
                                ui.label(
                                    egui::RichText::new(icon).color(color).strong(),
                                );
                                ui.label(
                                    egui::RichText::new(*name).color(color),
                                );
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(8.0);

                // Inventory summary
                let inv_count = state.inventory.len();
                if inv_count > 0 {
                    ui.group(|ui| {
                        ui.set_min_width(400.0);
                        ui.label(
                            egui::RichText::new(format!("Inventory ({inv_count} items)"))
                                .size(14.0)
                                .strong(),
                        );
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(100.0)
                            .show(ui, |ui| {
                                for item in state.inventory.iter() {
                                    ui.label(format!("  {}", item.doname("")));
                                }
                            });
                    });
                }

                ui.add_space(15.0);

                let button_size = egui::vec2(150.0, 35.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Try Again"))
                    .clicked()
                {
                    cc_state.reset();
                    next_state.set(AppState::CharacterCreation);
                }

                ui.add_space(8.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Main Menu"))
                    .clicked()
                {
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

fn render_victory_screen(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: EventWriter<AppExit>,
    game_state: Res<GameStateResource>,
) {
    // Dark overlay with gold tint
    egui::Area::new(egui::Id::new("victory_bg"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
        .interactable(false)
        .show(contexts.ctx_mut(), |ui| {
            let screen_rect = ui.ctx().screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgba_unmultiplied(10, 10, 40, 220),
            );
        });

    egui::Window::new("Victory")
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .resizable(false)
        .collapsible(false)
        .show(contexts.ctx_mut(), |ui| {
            ui.set_min_width(450.0);

            ui.vertical_centered(|ui| {
                ui.add_space(10.0);

                ui.label(
                    egui::RichText::new("YOU ASCENDED!")
                        .size(42.0)
                        .color(egui::Color32::GOLD)
                        .strong(),
                );

                ui.add_space(5.0);

                let state = &game_state.0;
                let player = &state.player;

                ui.label(
                    egui::RichText::new(format!(
                        "{} the {} {} {}",
                        player.name, player.alignment.typ, player.race, player.rank_title()
                    ))
                    .size(18.0)
                    .color(egui::Color32::LIGHT_BLUE),
                );

                ui.label(
                    egui::RichText::new("achieved demigod-hood!")
                        .size(16.0)
                        .color(egui::Color32::GOLD)
                        .italics(),
                );

                ui.add_space(15.0);

                // Stats
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Final Statistics").size(16.0).strong());
                    ui.separator();

                    egui::Grid::new("victory_stats_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Turns:");
                            ui.label(format!("{}", state.turns));
                            ui.end_row();

                            ui.label("Deepest level:");
                            ui.label(format!("{}", state.current_level.dlevel.depth()));
                            ui.end_row();

                            ui.label("Gold:");
                            ui.label(format!("{}", player.gold));
                            ui.end_row();

                            ui.label("Experience:");
                            ui.label(format!(
                                "Level {} ({} pts)",
                                player.exp_level, player.exp
                            ));
                            ui.end_row();

                            ui.label("HP:");
                            ui.label(format!("{}/{}", player.hp, player.hp_max));
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // Attributes
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Attributes").size(14.0).strong());
                    ui.separator();

                    egui::Grid::new("victory_attrs_grid")
                        .num_columns(6)
                        .spacing([15.0, 4.0])
                        .show(ui, |ui| {
                            for attr in Attribute::ALL {
                                ui.label(
                                    egui::RichText::new(attr.short_name())
                                        .strong()
                                        .color(egui::Color32::GOLD),
                                );
                            }
                            ui.end_row();
                            for attr in Attribute::ALL {
                                ui.label(format!(
                                    "{}",
                                    player.attr_current.get(attr)
                                ));
                            }
                            ui.end_row();
                        });
                });

                ui.add_space(8.0);

                // Conducts
                ui.group(|ui| {
                    ui.set_min_width(400.0);
                    ui.label(egui::RichText::new("Conducts").size(14.0).strong());
                    ui.separator();

                    let conduct = &player.conduct;
                    let checks: &[(&str, bool)] = &[
                        ("Foodless", conduct.is_foodless()),
                        ("Vegan", conduct.is_vegan()),
                        ("Vegetarian", conduct.is_vegetarian()),
                        ("Atheist", conduct.is_atheist()),
                        ("Weaponless", conduct.is_weaponless()),
                        ("Pacifist", conduct.is_pacifist()),
                        ("Illiterate", conduct.is_illiterate()),
                        ("Wishless", conduct.is_wishless()),
                        ("Genocideless", conduct.is_genocideless()),
                    ];

                    egui::Grid::new("victory_conduct_grid")
                        .num_columns(2)
                        .spacing([20.0, 2.0])
                        .show(ui, |ui| {
                            for (name, maintained) in checks {
                                let (icon, color) = if *maintained {
                                    ("*", egui::Color32::GOLD)
                                } else {
                                    ("-", egui::Color32::DARK_GRAY)
                                };
                                ui.label(
                                    egui::RichText::new(icon).color(color).strong(),
                                );
                                ui.label(
                                    egui::RichText::new(*name).color(color),
                                );
                                ui.end_row();
                            }
                        });
                });

                ui.add_space(15.0);

                let button_size = egui::vec2(150.0, 35.0);

                if ui
                    .add_sized(button_size, egui::Button::new("Main Menu"))
                    .clicked()
                {
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
                    ui.add(egui::Slider::new(&mut settings.zoom_speed, 0.1..=3.0).show_value(true));
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

                        ui.label("Help:");
                        ui.label("F1 or ?");
                        ui.end_row();

                        ui.label("Camera Mode:");
                        ui.label("F2-F5");
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
        save_state.saves = nh_core::save::list_saves().unwrap_or_default();
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
                        nh_core::save::default_save_path(&game_state.0.player.name)
                    };

                    match nh_core::save::save_game(&game_state.0, &path) {
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
                        if nh_core::save::delete_save(path).is_ok() {
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
        save_state.saves = nh_core::save::list_saves().unwrap_or_default();
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

                            let ago = chrono_lite_format(header.timestamp);
                            let label = format!(
                                "{} - {} (Turn {}, {})",
                                header.player_name, header.dlevel, header.turns, ago
                            );

                            if ui.selectable_label(selected, &label).clicked() {
                                save_state.selected = Some(i);
                            }

                            ui.add_space(2.0);
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
                        match nh_core::save::load_game(path) {
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
                        if nh_core::save::delete_save(path).is_ok() {
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
