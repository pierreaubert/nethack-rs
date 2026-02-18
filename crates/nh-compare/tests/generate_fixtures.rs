//! Fixture generation — export C-generated levels and save as JSON files.
//!
//! Run manually: `cargo test -p nh-compare --test generate_fixtures -- --ignored --nocapture`
//!
//! These fixtures are committed to data/fixtures/ and used by movement/combat/item
//! drift tests to ensure both engines operate on identical terrain.

use nh_core::dungeon::LevelFixture;
use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serial_test::serial;

const FIXTURE_DIR: &str = "data/fixtures";

/// Generate level fixtures for multiple seeds and roles.
/// This test is ignored by default — run manually to regenerate fixtures.
#[test]
#[serial]
#[ignore]
fn generate_level_fixtures() {
    let seeds = [1u64, 2, 3, 4, 5, 10, 42, 100, 12345, 99999];
    let roles = [
        ("Valkyrie", "Human"),
        ("Wizard", "Elf"),
        ("Rogue", "Human"),
    ];

    let fixture_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR);
    std::fs::create_dir_all(&fixture_dir).expect("Failed to create fixture directory");

    let mut generated = 0;

    for &seed in &seeds {
        for &(role, race) in &roles {
            let mut c_engine = CGameEngine::new();
            c_engine
                .init(role, race, 0, 0)
                .expect("C engine init failed");
            c_engine.reset(seed).expect("C engine reset failed");

            let level_json = c_engine.export_level();

            // Validate it parses as a LevelFixture
            match serde_json::from_str::<LevelFixture>(&level_json) {
                Ok(fixture) => {
                    let filename = format!(
                        "level_seed{}_{}.json",
                        seed,
                        role.to_lowercase()
                    );
                    let path = fixture_dir.join(&filename);
                    std::fs::write(&path, &level_json)
                        .expect("Failed to write fixture");
                    println!(
                        "Generated: {} ({}x{}, {} rooms, {} stairs, {} objects, {} monsters)",
                        filename,
                        fixture.width,
                        fixture.height,
                        fixture.rooms.len(),
                        fixture.stairs.len(),
                        fixture.objects.len(),
                        fixture.monsters.len(),
                    );
                    generated += 1;
                }
                Err(e) => {
                    eprintln!(
                        "WARNING: Failed to parse fixture for seed={} role={}: {}",
                        seed, role, e
                    );
                    eprintln!("Raw JSON (first 500 chars): {}", &level_json[..level_json.len().min(500)]);
                }
            }
        }
    }

    println!("\nGenerated {} fixtures in {}", generated, fixture_dir.display());
    assert!(generated > 0, "No fixtures were generated");
}

/// Verify that existing fixtures load correctly into Rust Level structs.
#[test]
fn test_load_fixtures() {
    let fixture_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR);

    if !fixture_dir.exists() {
        println!("No fixture directory found at {}. Run generate_level_fixtures first.", fixture_dir.display());
        return;
    }

    let mut loaded = 0;
    for entry in std::fs::read_dir(&fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let json = std::fs::read_to_string(&path).unwrap();
            let fixture: LevelFixture = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));

            // Import into Rust Level
            let level = nh_core::dungeon::Level::from_fixture(&fixture);

            // Basic validation
            assert_eq!(level.dlevel.dungeon_num, fixture.dnum);
            assert_eq!(level.dlevel.level_num, fixture.dlevel);
            assert_eq!(level.stairs.len(), fixture.stairs.len());

            println!(
                "Loaded: {} -> DLevel({},{}), {} stairs",
                path.file_name().unwrap().to_string_lossy(),
                level.dlevel.dungeon_num,
                level.dlevel.level_num,
                level.stairs.len(),
            );
            loaded += 1;
        }
    }

    if loaded == 0 {
        println!("No fixture files found. Run generate_level_fixtures first.");
    } else {
        println!("Successfully loaded {} fixtures", loaded);
    }
}
