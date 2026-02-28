//! NetHack clone in Rust
//!
//! Main entry point for the game.

use std::io;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event, execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use nh_core::player::{AlignmentType, Gender, Race, Role};
use nh_core::save::{default_save_path, delete_save, load_game, save_game};
use nh_core::{GameLoopResult, GameRng, GameState};
use nh_tui::{App, AppEvent, Theme, GraphicsMode, StartMenuAction};

/// NetHack clone in Rust
#[derive(Parser, Debug)]
#[command(name = "nethack")]
#[command(author, version, about = "NetHack - Explore the dungeon!", long_about = None)]
struct Args {
    /// Player name
    #[arg(short = 'u', long = "name")]
    name: Option<String>,

    /// Role/profession (e.g., Valkyrie, Wizard, Rogue)
    #[arg(short = 'p', long = "role")]
    role: Option<String>,

    /// Race (e.g., Human, Elf, Dwarf)
    #[arg(short = 'r', long = "race")]
    race: Option<String>,

    /// Gender (male/female)
    #[arg(short = 'g', long = "gender")]
    gender: Option<String>,

    /// Alignment (lawful/neutral/chaotic)
    #[arg(short = 'a', long = "align")]
    alignment: Option<String>,

    /// Random character (pick all options randomly)
    #[arg(short = '@', long = "random")]
    random: bool,

    /// Wizard (debug) mode
    #[arg(short = 'D', long = "wizard")]
    wizard: bool,

    /// Discovery (explore) mode
    #[arg(short = 'X', long = "discover")]
    discover: bool,

    /// View high scores
    #[arg(short = 's', long = "scores")]
    scores: bool,

    /// Recovery mode - recover from interrupted game
    #[arg(long = "recovery")]
    recovery: bool,

    /// Playground directory (for saving games)
    #[arg(long = "playground")]
    playground: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Use light background color theme (auto-detected from COLORFGBG/NH_LIGHT_BG)
    #[arg(long = "light")]
    light: bool,

    /// Graphics mode for the map (classic, fancy, auto)
    #[arg(short = 'G', long = "graphics", default_value = "auto")]
    graphics: GraphicsMode,
}

fn main() -> io::Result<()> {
    // Parse command-line arguments before terminal setup
    let args = Args::parse();

    // Early system initialization (C: sys_early_init + decl_init)
    nh_core::world::sys_early_init();
    nh_core::world::decl_init();

    // Handle special modes that don't require full game setup

    // View high scores
    if args.scores {
        display_high_scores(&args)?;
        return Ok(());
    }

    // Show version info
    if args.verbose {
        println!("NetHack {}", env!("CARGO_PKG_VERSION"));
        println!("Rust implementation of the classic roguelike");
        println!("Built with Bevy game engine");
        return Ok(());
    }

    // Handle recovery mode
    if args.recovery {
        handle_recovery_mode(&args)?;
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // If name provided via CLI, check for existing save
    let state = if let Some(ref player_name) = args.name {
        let save_path = default_save_path(player_name);
        if save_path.exists() {
            match load_game(&save_path) {
                Ok(loaded_state) => loaded_state,
                Err(_) => {
                    let _ = delete_save(&save_path);
                    run_character_creation(&mut terminal, &args)?
                }
            }
        } else {
            run_character_creation(&mut terminal, &args)?
        }
    } else {
        // No name provided - TUI will show startup menu
        run_startup_menu(&mut terminal, &args)?
    };

    // Detect or override color theme
    let theme = if args.light {
        Theme::light()
    } else {
        Theme::detect()
    };

    // Create app
    let mut app = App::new(state, theme, args.graphics);

    // Main loop — drain all queued events before each render to stay responsive.
    loop {
        // Draw
        terminal.draw(|frame| app.render(frame))?;

        // Wait for at least one event (with timeout for idle redraws)
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        // Drain all already-queued events before the next render
        while event::poll(Duration::ZERO)? {
            let ev = event::read()?;

            if let Some(event) = app.handle_event(ev) {
                match event {
                    AppEvent::Command(command) => {
                        let result = app.execute(command);
                        if process_result(&mut app, &mut terminal, result)? {
                            break;
                        }
                    }
                    AppEvent::StartMenu(_) => {
                        // Should not happen in main loop unless we re-enter start menu
                    }
                }
            }

            if app.should_quit() {
                break;
            }
        }

        if app.should_quit() {
            break;
        }
    }

    // Cleanup and restore terminal
    nh_core::world::freedynamicdata();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

/// Process a game loop result. Returns true if the main loop should break.
fn process_result(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    result: GameLoopResult,
) -> io::Result<bool> {
    match result {
        GameLoopResult::PlayerDied(_msg) => {
            // Delete save file on death (permadeath)
            let save_path = default_save_path(&app.state().player.name);
            let _ = delete_save(&save_path);
            // Death screen is shown via UiMode::DeathScreen (set in execute())
            // Loop continues so the user can view the screen and press Enter
            Ok(false)
        }
        GameLoopResult::PlayerQuit => {
            app.set_should_quit();
            Ok(true)
        }
        GameLoopResult::SaveAndQuit => {
            let save_path = default_save_path(&app.state().player.name);
            if let Err(e) = save_game(app.state(), &save_path) {
                eprintln!("Failed to save game: {}", e);
            }
            app.set_should_quit();
            Ok(true)
        }
        GameLoopResult::PlayerWon => {
            let save_path = default_save_path(&app.state().player.name);
            let _ = delete_save(&save_path);
            app.state_mut()
                .message("Congratulations! You have ascended!");
            terminal.draw(|frame| app.render(frame))?;
            std::thread::sleep(Duration::from_secs(3));
            app.set_should_quit();
            Ok(true)
        }
        GameLoopResult::Continue => Ok(false),
    }
}

/// Run TUI startup menu and return the game state
fn run_startup_menu(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    args: &Args,
) -> io::Result<GameState> {
    let temp_state = GameState::new(GameRng::from_entropy());
    let theme = if args.light {
        Theme::light()
    } else {
        Theme::detect()
    };
    let mut app = App::new(temp_state, theme, args.graphics);
    app.set_startup_menu();

    loop {
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(100))? {
            let ev = event::read()?;
            if let Some(app_event) = app.handle_event(ev) {
                if let AppEvent::StartMenu(action) = app_event {
                    match action {
                        StartMenuAction::NewGame => {
                            return run_character_creation(terminal, args);
                        }
                        StartMenuAction::LoadGame => {
                            // For now, load default if exists, or show error?
                            // Better: prompt for name if not provided?
                            // For simplicity, let's look for a save if name was provided, 
                            // or ask for name if not.
                            let player_name = args.name.clone().unwrap_or_else(|| "Player".to_string());
                            let save_path = default_save_path(&player_name);
                            if save_path.exists() {
                                match load_game(&save_path) {
                                    Ok(loaded_state) => return Ok(loaded_state),
                                    Err(_) => {
                                        // TODO: show error message in UI
                                    }
                                }
                            } else {
                                // TODO: show "No save found" message in UI
                            }
                        }
                        StartMenuAction::Quit => {
                            disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                            terminal.show_cursor()?;
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
    }
}

/// Run TUI character creation and return the new game state
fn run_character_creation(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    args: &Args,
) -> io::Result<GameState> {
    // If command-line args specify everything including name, skip TUI entirely
    if args.name.is_some() && args.random {
        let name = args.name.clone().unwrap();
        return create_new_game_with_args(&name, args);
    }
    if args.name.is_some()
        && args.role.is_some()
        && args.race.is_some()
        && args.gender.is_some()
        && args.alignment.is_some()
    {
        let name = args.name.clone().unwrap();
        return create_new_game_with_args(&name, args);
    }

    // Create a temporary game state for the character creation UI
    let temp_state = GameState::new(GameRng::from_entropy());
    let theme = if args.light {
        Theme::light()
    } else {
        Theme::detect()
    };
    let mut app = App::new(temp_state, theme, args.graphics);

    // Parse what we can from CLI args
    let role = args.role.as_ref().and_then(|s| parse_role(s));
    let race = args.race.as_ref().and_then(|s| parse_race(s));
    let gender = args.gender.as_ref().and_then(|s| parse_gender(s));
    let alignment = args.alignment.as_ref().and_then(|s| parse_alignment(s));

    // Start character creation - with any provided CLI options
    app.start_character_creation_with_choices(
        args.name.clone(),
        role,
        race,
        gender,
        alignment,
    );

    // Character creation loop
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(100))? {
            let evt = event::read()?;
            app.handle_event(evt);

            // Check if character creation is complete
            if let Some(choices) = app.get_character_choices() {
                app.finish_character_creation();

                // Create the actual game with the chosen options
                return Ok(create_new_game_with_choices(
                    &choices.name,
                    choices.role,
                    choices.race,
                    choices.gender,
                    choices.alignment,
                    args.wizard,
                    args.discover,
                ));
            }

            // Check if user quit during character creation
            if app.should_quit() {
                // Restore terminal before exiting
                disable_raw_mode()?;
                execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                terminal.show_cursor()?;
                std::process::exit(0);
            }
        }
    }
}

/// Create new game using command-line args (only called when all args provided)
fn create_new_game_with_args(player_name: &str, args: &Args) -> io::Result<GameState> {
    let mut rng = GameRng::from_entropy();

    // Parse role from args or pick random
    let role = if args.random {
        random_role(&mut rng)
    } else if let Some(ref role_str) = args.role {
        parse_role(role_str).unwrap_or_else(|| random_role(&mut rng))
    } else {
        random_role(&mut rng)
    };

    // Parse race from args or pick compatible random
    let race = if args.random {
        random_race(role, &mut rng)
    } else if let Some(ref race_str) = args.race {
        parse_race(race_str).unwrap_or_else(|| random_race(role, &mut rng))
    } else {
        random_race(role, &mut rng)
    };

    // Parse gender from args or pick compatible random
    let gender = if args.random {
        random_gender(role, race, &mut rng)
    } else if let Some(ref gender_str) = args.gender {
        parse_gender(gender_str).unwrap_or_else(|| random_gender(role, race, &mut rng))
    } else {
        random_gender(role, race, &mut rng)
    };

    // Parse alignment from args or use role default
    let alignment = if args.random {
        random_alignment(role)
    } else if let Some(ref align_str) = args.alignment {
        parse_alignment(align_str).unwrap_or_else(|| random_alignment(role))
    } else {
        random_alignment(role)
    };

    Ok(create_new_game_with_choices(
        player_name,
        role,
        race,
        gender,
        alignment,
        args.wizard,
        args.discover,
    ))
}

/// Parse role from string - delegates to nh_core's str2role
fn parse_role(s: &str) -> Option<Role> {
    nh_core::player::str2role(s)
}

/// Parse race from string - delegates to nh_core's str2race
fn parse_race(s: &str) -> Option<Race> {
    nh_core::player::str2race(s)
}

/// Parse gender from string - delegates to nh_core's str2gend
fn parse_gender(s: &str) -> Option<Gender> {
    nh_core::player::str2gend(s)
}

/// Parse alignment from string - delegates to nh_core's str2align
fn parse_alignment(s: &str) -> Option<AlignmentType> {
    nh_core::player::str2align(s)
}

/// Random role selection using pick_role with no constraints
fn random_role(_rng: &mut GameRng) -> Role {
    let filter = nh_core::player::RoleFilter::new();
    nh_core::player::pick_role(None, None, None, &filter).unwrap_or(Role::Valkyrie)
}

/// Random race selection using pick_race
fn random_race(role: Role, _rng: &mut GameRng) -> Race {
    let filter = nh_core::player::RoleFilter::new();
    nh_core::player::pick_race(Some(role), None, None, &filter).unwrap_or(Race::Human)
}

/// Random gender selection using pick_gend
fn random_gender(role: Role, race: Race, _rng: &mut GameRng) -> Gender {
    let filter = nh_core::player::RoleFilter::new();
    nh_core::player::pick_gend(Some(role), race, None, &filter).unwrap_or(Gender::Male)
}

/// Random alignment selection using pick_align
fn random_alignment(role: Role) -> AlignmentType {
    nh_core::player::pick_align(role).unwrap_or(AlignmentType::Neutral)
}

/// Create a new game with specified choices
///
/// Uses `GameState::new_with_identity()` which calls `u_init()` internally,
/// properly initializing HP/energy/attributes/skills/inventory based on role.
fn create_new_game_with_choices(
    name: &str,
    role: Role,
    race: Race,
    gender: Gender,
    alignment: AlignmentType,
    wizard_mode: bool,
    discover_mode: bool,
) -> GameState {
    let rng = GameRng::from_entropy();

    // Create game state with proper initialization via u_init()
    let mut state = GameState::new_with_identity(
        rng,
        name.to_string(),
        role,
        race,
        gender,
        alignment,
    );

    // Post-creation setup matching C's newgame() in allmain.c
    state.spawn_starting_pet();          // C: makedog()
    state.player.next_attrib_check = 600; // C: context.next_attrib_check = 600L

    state.flags.started = true;

    // Wizard mode overrides for debugging
    if wizard_mode {
        state.player.hp = 100;
        state.player.hp_max = 100;
        state.player.energy = 100;
        state.player.energy_max = 100;
    }

    // Discover mode (C: flags.explore = TRUE in enter_explore_mode)
    if discover_mode {
        state.flags.explore = true;
        state.message("You are in explore mode.");
    }

    // Add welcome and intro messages
    let rank = role.rank_title(1, gender);
    state.message(format!(
        "Welcome to NetHack! You are a {} {} {} {}.",
        alignment, race, gender, role
    ));
    state.message(format!("You are a {}.", rank));

    // Wizard mode notification
    if wizard_mode {
        state.message("You are in wizard mode - debugging powers enabled!");
    }

    // Add role-specific intro
    match role {
        Role::Valkyrie => state.message("You must prove yourself worthy to enter Valhalla."),
        Role::Wizard => state.message("You seek the secrets of the Mazes of Menace."),
        Role::Archeologist => state.message("You seek ancient treasures and lost artifacts."),
        Role::Barbarian => state.message("You seek glory through conquest and battle."),
        Role::Caveman => state.message("You seek to survive in this hostile world."),
        Role::Healer => state.message("You seek to cure the sick and aid the wounded."),
        Role::Knight => state.message("You seek to uphold honor and chivalry."),
        Role::Monk => state.message("You seek enlightenment through discipline."),
        Role::Priest => state.message("You seek to spread the faith of your deity."),
        Role::Ranger => state.message("You seek to protect the wilderness."),
        Role::Rogue => state.message("You seek fortune through cunning and stealth."),
        Role::Samurai => state.message("You seek to restore honor to your family."),
        Role::Tourist => state.message("You seek adventure and souvenirs."),
    }

    state.message("Be careful! The dungeon is full of monsters.");

    state
}

/// Display the high scores screen
fn display_high_scores(args: &Args) -> io::Result<()> {
    println!("\n=== NetHack High Scores ===\n");
    println!(
        "{:<4} {:<20} {:<15} {:<10}",
        "Rank", "Player", "Score", "Level"
    );
    println!("{:-<50}", "");

    // Placeholder - would load from score file
    println!("{:<4} {:<20} {:<15} {:<10}", "1", "Hero", "100000", "20");
    println!(
        "{:<4} {:<20} {:<15} {:<10}",
        "2", "Adventurer", "50000", "15"
    );
    println!("{:<4} {:<20} {:<15} {:<10}", "3", "Wanderer", "25000", "10");

    if args.verbose {
        println!("\n(Score file: ~/.nethackrc)");
        println!("To reset scores, delete the score file.");
    }

    Ok(())
}

/// Handle recovery mode - restore interrupted games
fn handle_recovery_mode(args: &Args) -> io::Result<()> {
    println!("\n=== NetHack Recovery Mode ===\n");
    println!("Available saved games for recovery:\n");

    // Look for any save files that might need recovery
    let player_name = args.name.clone().unwrap_or_else(|| "unknown".to_string());
    let save_path = default_save_path(&player_name);

    if save_path.exists() {
        println!("Found save for player: {}", player_name);
        println!("Path: {}", save_path.display());

        // Attempt to load and show recovery info
        match load_game(&save_path) {
            Ok(game_state) => {
                println!("\nGame State:");
                println!("  Player: {}", game_state.player.name);
                println!(
                    "  HP: {}/{}",
                    game_state.player.hp, game_state.player.hp_max
                );
                println!("  Level: {}", game_state.player.level);
                println!("  Turn: {}", game_state.turns);
                println!("\nTo recover this game, run: nethack -u {}", player_name);
            }
            Err(e) => {
                eprintln!("Error reading save file: {}", e);
                println!("Save file may be corrupted. Delete and start a new game.");
            }
        }
    } else {
        println!(
            "No save file found for player: {}",
            args.name
                .as_deref()
                .unwrap_or("(none specified)")
        );
        println!("\nTo start a new game, run: nethack");
    }

    Ok(())
}
