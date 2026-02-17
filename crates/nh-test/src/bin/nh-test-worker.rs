use std::io::{self, BufRead, Write};
use serde::{Serialize, Deserialize};
use nh_test::ffi::CGameEngine;

#[derive(Serialize, Deserialize)]
enum Command {
    Init { role: String, race: String, gender: i32, align: i32 },
    Reset { seed: u64 },
    ResetRng { seed: u64 },
    GenerateLevel,
    GenerateMaze,
    GetHp,
    GetMaxHp,
    GetEnergy,
    GetMaxEnergy,
    GetPosition,
    GetTurnCount,
    GetStateJson,
    GetMapJson,
    ExecCmd { cmd: char },
    ExecCmdDir { cmd: char, dx: i32, dy: i32 },
    SetDLevel { dnum: i32, dlevel: i32 },
    SetState { hp: i32, hpmax: i32, x: i32, y: i32, ac: i32, moves: i64 },
    GetArmorClass,
    GetGold,
    GetExperienceLevel,
    GetCurrentLevel,
    GetDungeonDepth,
    IsDead,
    GetLastMessage,
    GetInventoryCount,
    GetInventoryJson,
    GetObjectTableJson,
    GetMonstersJson,
    SetWizardMode { enable: bool },
    AddItemToInv { item_id: i32, weight: i32 },
    GetCarryingWeight,
    GetMonsterCount,
    GetRole,
    GetRace,
    GetGenderString,
    GetAlignmentString,
    GetResultMessage,
    RngRn2 { limit: i32 },
    CalcBaseDamage { weapon_id: i32, small_monster: bool },
    GetAc,
    TestSetupStatus { hp: i32, max_hp: i32, level: i32, ac: i32 },
    WearItem { item_id: i32 },
    Exit,
}

#[derive(Serialize, Deserialize)]
enum Response {
    Ok,
    Int(i32),
    Pos(i32, i32),
    Long(u64),
    String(String),
    Bool(bool),
    Error(String),
}

fn main() {
    let mut engine = CGameEngine::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() { continue; }

        let cmd: Command = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let resp = Response::Error(format!("Invalid command: {}", e));
                println!("JSON:{}", serde_json::to_string(&resp).unwrap());
                continue;
            }
        };

        let resp = match cmd {
            Command::Init { role, race, gender, align } => {
                match engine.init(&role, &race, gender, align) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::Reset { seed } => {
                match engine.reset(seed) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::ResetRng { seed } => {
                match engine.reset_rng(seed) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::GenerateLevel => {
                match engine.generate_level() {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::GenerateMaze => {
                match engine.generate_maze() {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::GetHp => Response::Int(engine.hp()),
            Command::GetMaxHp => Response::Int(engine.max_hp()),
            Command::GetEnergy => Response::Int(engine.energy()),
            Command::GetMaxEnergy => Response::Int(engine.max_energy()),
            Command::GetPosition => {
                let (x, y) = engine.position();
                Response::Pos(x, y)
            }
            Command::GetTurnCount => Response::Long(engine.turn_count()),
            Command::GetStateJson => Response::String(engine.state_json()),
            Command::GetMapJson => Response::String(engine.map_json()),
            Command::ExecCmd { cmd } => {
                match engine.exec_cmd(cmd) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::ExecCmdDir { cmd, dx, dy } => {
                match engine.exec_cmd_dir(cmd, dx, dy) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::SetDLevel { dnum, dlevel } => {
                engine.set_dlevel(dnum, dlevel);
                Response::Ok
            }
            Command::SetState { hp, hpmax, x, y, ac, moves } => {
                engine.set_state(hp, hpmax, x, y, ac, moves);
                Response::Ok
            }
            Command::GetArmorClass => Response::Int(engine.armor_class()),
            Command::GetGold => Response::Int(engine.gold()),
            Command::GetExperienceLevel => Response::Int(engine.experience_level()),
            Command::GetCurrentLevel => Response::Int(engine.current_level()),
            Command::GetDungeonDepth => Response::Int(engine.dungeon_depth()),
            Command::IsDead => Response::Bool(engine.is_dead()),
            Command::GetLastMessage => Response::String(engine.last_message()),
            Command::GetInventoryCount => Response::Int(engine.inventory_count()),
            Command::GetInventoryJson => Response::String(engine.inventory_json()),
            Command::GetObjectTableJson => Response::String(engine.object_table_json()),
            Command::GetMonstersJson => Response::String(engine.monsters_json()),
            Command::SetWizardMode { enable } => {
                engine.set_wizard_mode(enable);
                Response::Ok
            }
            Command::AddItemToInv { item_id, weight } => {
                match engine.add_item_to_inv(item_id, weight) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::GetCarryingWeight => Response::Int(engine.carrying_weight()),
            Command::GetMonsterCount => Response::Int(engine.monster_count()),
            Command::GetRole => Response::String(engine.role()),
            Command::GetRace => Response::String(engine.race()),
            Command::GetGenderString => Response::String(engine.gender_string()),
            Command::GetAlignmentString => Response::String(engine.alignment_string()),
            Command::GetResultMessage => Response::String(engine.result_message()),
            Command::RngRn2 { limit } => Response::Int(engine.rng_rn2(limit)),
            Command::CalcBaseDamage { weapon_id, small_monster } => Response::Int(engine.calc_base_damage(weapon_id, small_monster)),
            Command::GetAc => Response::Int(engine.ac()),
            Command::TestSetupStatus { hp, max_hp, level, ac } => {
                engine.test_setup_status(hp, max_hp, level, ac);
                Response::Ok
            }
            Command::WearItem { item_id } => {
                match engine.wear_item(item_id) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(format!("{}", e)),
                }
            }
            Command::Exit => break,
        };

        println!("JSON:{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }
}
