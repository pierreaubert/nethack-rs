//! NetHack clone in Rust
//!
//! Main entry point for the game.

use std::io;
use std::time::Duration;

use crossterm::{
    event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use nh_core::dungeon::Level;
use nh_core::player::{Gender, Race, Role, You};
use nh_core::{GameLoopResult, GameRng, GameState};
use nh_data::monsters;
use nh_save::{save_game, load_game, default_save_path, delete_save};
use nh_ui::App;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Try to load existing save, or create new game
    let player_name = "Player"; // Default player name
    let save_path = default_save_path(player_name);
    
    let state = if save_path.exists() {
        match load_game(&save_path) {
            Ok(loaded_state) => {
                eprintln!("Loaded saved game for {}", loaded_state.player.name);
                loaded_state
            }
            Err(e) => {
                eprintln!("Failed to load save: {}, starting new game", e);
                create_new_game()
            }
        }
    } else {
        create_new_game()
    };

    // Create app
    let mut app = App::new(state);

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
                    GameLoopResult::PlayerDied(msg) => {
                        // Delete save file on death (permadeath)
                        let save_path = default_save_path(&app.state().player.name);
                        let _ = delete_save(&save_path);
                        
                        // Show death message
                        app.state_mut().message(format!("You died: {}", msg));
                        terminal.draw(|frame| app.render(frame))?;
                        std::thread::sleep(Duration::from_secs(2));
                        break;
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
                        app.state_mut().message("Congratulations! You have ascended!");
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

/// Create a new game with initial state
fn create_new_game() -> GameState {
    let rng = GameRng::from_entropy();

    // Create player
    let mut player = You::new(
        "Player".to_string(),
        Role::Valkyrie,
        Race::Human,
        Gender::Female,
    );

    // Initialize player stats
    player.hp = 16;
    player.hp_max = 16;
    player.energy = 1;
    player.energy_max = 1;
    player.armor_class = 10;

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

    state.message("Welcome to NetHack!  You are a Valkyrie.");

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
            let max_type = monsters::num_monsters().min(20) as i16; // For now, use first 20 monsters
            let monster_type = rng.rn2(max_type as u32) as i16;

            // Get monster template
            if let Some(permonst) = monsters::get_monster(monster_type) {
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

