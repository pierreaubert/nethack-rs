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
use strum::IntoEnumIterator;

use nh_core::dungeon::Level;
use nh_core::player::{AlignmentType, Gender, Race, Role, You};
use nh_core::save::{default_save_path, delete_save, load_game, save_game};
use nh_core::{GameLoopResult, GameRng, GameState};
use nh_tui::App;

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
}

fn main() -> io::Result<()> {
    // Parse command-line arguments before terminal setup
    let args = Args::parse();

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
        // No name provided - TUI will ask for it
        run_character_creation(&mut terminal, &args)?
    };

    // Load asset mapping
    let assets_path = "crates/nh-assets/initial_mapping.json";
    let assets = nh_assets::registry::AssetRegistry::load_from_file(assets_path)
        .unwrap_or_else(|_| {
            nh_assets::registry::AssetRegistry::new(nh_assets::mapping::AssetMapping::default())
        });

    // Create app
    let mut app = App::new(state, assets);

    // Main loop
    loop {
        // Draw
        terminal.draw(|frame| app.render(frame))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;

            if let Some(command) = app.handle_event(event) {
                let result = app.execute(command);

                match result {
                    GameLoopResult::PlayerDied(_msg) => {
                        // Delete save file on death (permadeath)
                        let save_path = default_save_path(&app.state().player.name);
                        let _ = delete_save(&save_path);
                        // Death screen is shown via UiMode::DeathScreen (set in execute())
                        // Loop continues so the user can view the screen and press Enter
                    }
                    GameLoopResult::PlayerQuit => break,
                    GameLoopResult::SaveAndQuit => {
                        // Save game
                        let save_path = default_save_path(&app.state().player.name);
                        if let Err(e) = save_game(app.state(), &save_path) {
                            eprintln!("Failed to save game: {}", e);
                        }
                        break;
                    }
                    GameLoopResult::PlayerWon => {
                        // Delete save file on victory
                        let save_path = default_save_path(&app.state().player.name);
                        let _ = delete_save(&save_path);

                        // Show victory message
                        app.state_mut()
                            .message("Congratulations! You have ascended!");
                        terminal.draw(|frame| app.render(frame))?;
                        std::thread::sleep(Duration::from_secs(3));
                        break;
                    }
                    GameLoopResult::Continue => {}
                }
            }

            if app.should_quit() {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
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
    let assets = nh_assets::registry::AssetRegistry::new(nh_assets::mapping::AssetMapping::default());
    let mut app = App::new(temp_state, assets);

    // Start character creation - with name if provided via CLI
    if let Some(ref name) = args.name {
        app.start_character_creation_with_name(name.clone());
    } else {
        app.start_character_creation();
    }

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

    // Parse race from args or pick random
    let race = if args.random {
        random_race(&mut rng)
    } else if let Some(ref race_str) = args.race {
        parse_race(race_str).unwrap_or_else(|| random_race(&mut rng))
    } else {
        random_race(&mut rng)
    };

    // Parse gender from args or pick random
    let gender = if args.random {
        random_gender(&mut rng)
    } else if let Some(ref gender_str) = args.gender {
        parse_gender(gender_str).unwrap_or_else(|| random_gender(&mut rng))
    } else {
        random_gender(&mut rng)
    };

    // Parse alignment from args or pick random
    let alignment = if args.random {
        random_alignment(&mut rng)
    } else if let Some(ref align_str) = args.alignment {
        parse_alignment(align_str).unwrap_or_else(|| random_alignment(&mut rng))
    } else {
        random_alignment(&mut rng)
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

/// Parse role from string
fn parse_role(s: &str) -> Option<Role> {
    let s = s.to_lowercase();
    for role in Role::iter() {
        if role.to_string().to_lowercase().starts_with(&s) {
            return Some(role);
        }
    }
    None
}

/// Parse race from string
fn parse_race(s: &str) -> Option<Race> {
    let s = s.to_lowercase();
    for race in Race::iter() {
        if race.to_string().to_lowercase().starts_with(&s) {
            return Some(race);
        }
    }
    None
}

/// Parse gender from string
fn parse_gender(s: &str) -> Option<Gender> {
    let s = s.to_lowercase();
    if s.starts_with('m') {
        Some(Gender::Male)
    } else if s.starts_with('f') {
        Some(Gender::Female)
    } else {
        None
    }
}

/// Parse alignment from string
fn parse_alignment(s: &str) -> Option<AlignmentType> {
    let s = s.to_lowercase();
    if s.starts_with('l') {
        Some(AlignmentType::Lawful)
    } else if s.starts_with('n') {
        Some(AlignmentType::Neutral)
    } else if s.starts_with('c') {
        Some(AlignmentType::Chaotic)
    } else {
        None
    }
}

/// Random role selection
fn random_role(rng: &mut GameRng) -> Role {
    let roles: Vec<Role> = Role::iter().collect();
    roles[rng.rn2(roles.len() as u32) as usize]
}

/// Random race selection
fn random_race(rng: &mut GameRng) -> Race {
    let races: Vec<Race> = Race::iter().collect();
    races[rng.rn2(races.len() as u32) as usize]
}

/// Random gender selection
fn random_gender(rng: &mut GameRng) -> Gender {
    if rng.one_in(2) {
        Gender::Male
    } else {
        Gender::Female
    }
}

/// Random alignment selection
fn random_alignment(rng: &mut GameRng) -> AlignmentType {
    let aligns: Vec<AlignmentType> = AlignmentType::iter().collect();
    aligns[rng.rn2(aligns.len() as u32) as usize]
}

/// Create a new game with specified choices
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

    // Create player
    let mut player = You::new(name.to_string(), role, race, gender);
    player.alignment.typ = alignment;
    player.original_alignment = alignment;

    // Initialize player stats based on role
    player.hp = 16;
    player.hp_max = 16;
    player.energy = 1;
    player.energy_max = 1;
    player.armor_class = 10;

    // Set game modes - wizard mode gives extra powers for debugging
    if wizard_mode {
        player.hp = 100;
        player.hp_max = 100;
        player.energy = 100;
        player.energy_max = 100;
    }

    let _ = discover_mode; // TODO: implement discover mode effects

    // Set initial attributes (average values)
    use nh_core::player::Attribute;
    player.attr_current.set(Attribute::Strength, 18);
    player.attr_current.set(Attribute::Dexterity, 13);
    player.attr_current.set(Attribute::Constitution, 14);
    player.attr_current.set(Attribute::Intelligence, 7);
    player.attr_current.set(Attribute::Wisdom, 7);
    player.attr_current.set(Attribute::Charisma, 10);
    player.attr_max = player.attr_current;

    // Create game state (includes generated level with basic monsters)
    let mut state = GameState::new(rng);

    // Preserve the spawn position from level generation
    let spawn_pos = state.player.pos;
    let spawn_prev = state.player.prev_pos;
    state.player = player;
    state.player.pos = spawn_pos;
    state.player.prev_pos = spawn_prev;

    // Populate monsters with actual data from nh-data
    populate_monster_data(&mut state.current_level, &mut state.rng);

    state.flags.started = true;

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

/// Populate monsters with actual data from nh-data
fn populate_monster_data(level: &mut Level, rng: &mut GameRng) {
    // Get list of monster IDs to update
    let monster_ids: Vec<_> = level.monsters.iter().map(|m| m.id).collect();

    for monster_id in monster_ids {
        if let Some(monster) = level.monster_mut(monster_id) {
            // Pick a random monster type from the available 380+ monsters
            // Use depth-based selection for variety
            let max_type = nh_core::data::monsters::num_monsters().min(20) as i16; // For now, use first 20 monsters
            let monster_type = rng.rn2(max_type as u32) as i16;

            // Get monster template
            if let Some(permonst) = nh_core::data::monsters::get_monster(monster_type) {
                monster.monster_type = monster_type;
                monster.original_type = monster_type;
                monster.name = permonst.name.to_string();
                monster.attacks = permonst.attacks;
                monster.level = permonst.level as u8;
                monster.alignment = permonst.alignment;

                // Set HP based on level
                let base_hp = permonst.level as i32 + 1;
                let hp = base_hp + rng.rnd(base_hp as u32) as i32;
                monster.hp = hp;
                monster.hp_max = hp;

                // 20% chance to be peaceful
                if rng.one_in(5) {
                    monster.state.peaceful = true;
                }

                // 10% chance to be sleeping
                if rng.one_in(10) {
                    monster.state.sleeping = true;
                }
            }
        }
    }
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
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("(none specified)")
        );
        println!("\nTo start a new game, run: nethack");
    }

    Ok(())
}

/// Setup playground directory for saves
fn setup_playground(playground: Option<&str>) -> io::Result<()> {
    if let Some(path) = playground {
        use std::path::Path;
        let playground_path = Path::new(path);

        if !playground_path.exists() {
            std::fs::create_dir_all(playground_path)?;
            println!("Created playground directory: {}", path);
        }

        println!("Using playground: {}", path);
    }

    Ok(())
}
