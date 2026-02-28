use std::io::{self, BufRead, Write};
use serde::{Serialize, Deserialize};
use nh_test::ffi::CGameEngine;
use nh_core::CGameEngineTrait;

#[derive(Serialize, Deserialize)]
enum Command {
    Init { role: String, race: String, gender: i32, align: i32 },
    Reset { seed: u64 },
    ResetRng { seed: u64 },
    GenerateLevel,
    GenerateAndPlace,
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
    GetRngCallCount,
    SetSkipMovemon { skip: bool },
    RngRn2 { limit: i32 },
    CalcBaseDamage { weapon_id: i32, small_monster: bool },
    GetAc,
    TestSetupStatus { hp: i32, max_hp: i32, level: i32, ac: i32 },
    WearItem { item_id: i32 },
    GetNutrition,
    GetAttributesJson,
    ExportLevel,
    EnableRngTracing,
    DisableRngTracing,
    GetRngTrace,
    ClearRngTrace,
    GetVisibility,
    GetCouldsee,
    // Function-level isolation testing (Phase 1)
    TestFinddpos { xl: i32, yl: i32, xh: i32, yh: i32 },
    TestDigCorridor { sx: i32, sy: i32, dx: i32, dy: i32, nxcor: bool },
    TestMakecorridors,
    TestJoin { a: i32, b: i32, nxcor: bool },
    GetSmeq,
    GetDoorindex,
    GetCellRegion { x1: i32, y1: i32, x2: i32, y2: i32 },
    SetCell { x: i32, y: i32, typ: i32 },
    ClearLevel,
    AddRoom { lx: i32, ly: i32, hx: i32, hy: i32, rtype: i32 },
    CarveRoom { lx: i32, ly: i32, hx: i32, hy: i32 },
    GetRectJson,
    DebugCell { x: i32, y: i32 },
    DebugMfndpos { mon_index: i32 },
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
                match CGameEngineTrait::init(&mut engine, &role, &race, gender, align) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::Reset { seed } => {
                match CGameEngineTrait::reset(&mut engine, seed) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::ResetRng { seed } => {
                match engine.reset_rng(seed) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GenerateLevel => {
                match engine.generate_level() {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GenerateAndPlace => {
                match CGameEngineTrait::generate_and_place(&engine) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GenerateMaze => {
                match engine.generate_maze() {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GetHp => Response::Int(CGameEngineTrait::hp(&engine)),
            Command::GetMaxHp => Response::Int(CGameEngineTrait::max_hp(&engine)),
            Command::GetEnergy => Response::Int(CGameEngineTrait::energy(&engine)),
            Command::GetMaxEnergy => Response::Int(CGameEngineTrait::max_energy(&engine)),
            Command::GetPosition => {
                let (x, y) = CGameEngineTrait::position(&engine);
                Response::Pos(x, y)
            }
            Command::GetTurnCount => Response::Long(CGameEngineTrait::turn_count(&engine)),
            Command::GetStateJson => Response::String(CGameEngineTrait::state_json(&engine)),
            Command::GetMapJson => Response::String(engine.map_json()),
            Command::ExecCmd { cmd } => {
                match CGameEngineTrait::exec_cmd(&engine, cmd) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::ExecCmdDir { cmd, dx, dy } => {
                match CGameEngineTrait::exec_cmd_dir(&engine, cmd, dx, dy) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::SetDLevel { dnum, dlevel } => {
                engine.set_dlevel(dnum, dlevel);
                Response::Ok
            }
            Command::SetState { hp, hpmax, x, y, ac, moves } => {
                CGameEngineTrait::set_state(&engine, hp, hpmax, x, y, ac, moves);
                Response::Ok
            }
            Command::GetArmorClass => Response::Int(CGameEngineTrait::armor_class(&engine)),
            Command::GetGold => Response::Int(CGameEngineTrait::gold(&engine)),
            Command::GetExperienceLevel => Response::Int(CGameEngineTrait::experience_level(&engine)),
            Command::GetCurrentLevel => Response::Int(CGameEngineTrait::current_level(&engine)),
            Command::GetDungeonDepth => Response::Int(CGameEngineTrait::dungeon_depth(&engine)),
            Command::IsDead => Response::Bool(CGameEngineTrait::is_dead(&engine)),
            Command::GetLastMessage => Response::String(CGameEngineTrait::last_message(&engine)),
            Command::GetInventoryCount => Response::Int(engine.inventory_count()),
            Command::GetInventoryJson => Response::String(CGameEngineTrait::inventory_json(&engine)),
            Command::GetObjectTableJson => Response::String(engine.object_table_json()),
            Command::GetMonstersJson => Response::String(CGameEngineTrait::monsters_json(&engine)),
            Command::SetWizardMode { enable } => {
                engine.set_wizard_mode(enable);
                Response::Ok
            }
            Command::AddItemToInv { item_id, weight } => {
                match engine.add_item_to_inv(item_id, weight) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GetCarryingWeight => Response::Int(engine.carrying_weight()),
            Command::GetMonsterCount => Response::Int(engine.monster_count()),
            Command::GetRole => Response::String(CGameEngineTrait::role(&engine)),
            Command::GetRace => Response::String(CGameEngineTrait::race(&engine)),
            Command::GetGenderString => Response::String(CGameEngineTrait::gender_string(&engine)),
            Command::GetAlignmentString => Response::String(CGameEngineTrait::alignment_string(&engine)),
            Command::GetResultMessage => Response::String(engine.result_message()),
            Command::GetRngCallCount => Response::Int(engine.rng_call_count() as i32),
            Command::SetSkipMovemon { skip } => {
                engine.set_skip_movemon(skip);
                Response::Ok
            }
            Command::RngRn2 { limit } => Response::Int(engine.rng_rn2(limit)),
            Command::CalcBaseDamage { weapon_id, small_monster } => {
                Response::Int(engine.calc_base_damage(weapon_id, small_monster))
            }
            Command::GetAc => Response::Int(engine.ac()),
            Command::TestSetupStatus { hp, max_hp, level, ac } => {
                engine.test_setup_status(hp, max_hp, level, ac);
                Response::Ok
            }
            Command::WearItem { item_id } => {
                match engine.wear_item(item_id) {
                    Ok(_) => Response::Ok,
                    Err(e) => Response::Error(e),
                }
            }
            Command::GetNutrition => Response::Int(engine.nutrition()),
            Command::GetAttributesJson => Response::String(engine.attributes_json()),
            Command::ExportLevel => Response::String(CGameEngineTrait::export_level(&engine)),
            Command::EnableRngTracing => {
                engine.enable_rng_tracing();
                Response::Ok
            }
            Command::DisableRngTracing => {
                engine.disable_rng_tracing();
                Response::Ok
            }
            Command::GetRngTrace => Response::String(engine.rng_trace_json()),
            Command::ClearRngTrace => {
                engine.clear_rng_trace();
                Response::Ok
            }
            Command::GetVisibility => Response::String(serde_json::to_string(&engine.get_visibility()).unwrap()),
            Command::GetCouldsee => Response::String(serde_json::to_string(&engine.get_couldsee()).unwrap()),
            Command::TestFinddpos { xl, yl, xh, yh } => {
                let (x, y) = engine.test_finddpos(xl, yl, xh, yh);
                Response::Pos(x, y)
            }
            Command::TestDigCorridor { sx, sy, dx, dy, nxcor } => {
                Response::Bool(engine.test_dig_corridor(sx, sy, dx, dy, nxcor))
            }
            Command::TestMakecorridors => {
                engine.test_makecorridors();
                Response::Ok
            }
            Command::TestJoin { a, b, nxcor } => {
                engine.test_join(a, b, nxcor);
                Response::Ok
            }
            Command::GetSmeq => Response::String(engine.get_smeq()),
            Command::GetDoorindex => Response::Int(engine.get_doorindex()),
            Command::GetCellRegion { x1, y1, x2, y2 } => Response::String(engine.get_cell_region(x1, y1, x2, y2)),
            Command::SetCell { x, y, typ } => {
                engine.set_cell(x, y, typ);
                Response::Ok
            }
            Command::ClearLevel => {
                engine.clear_level();
                Response::Ok
            }
            Command::AddRoom { lx, ly, hx, hy, rtype } => Response::Int(engine.add_room(lx, ly, hx, hy, rtype)),
            Command::CarveRoom { lx, ly, hx, hy } => {
                engine.carve_room(lx, ly, hx, hy);
                Response::Ok
            }
            Command::GetRectJson => Response::String(engine.rect_json()),
            Command::DebugCell { x, y } => Response::String(engine.debug_cell(x, y)),
            Command::DebugMfndpos { mon_index } => Response::String(engine.debug_mfndpos(mon_index)),
            Command::Exit => break,
        };

        println!("JSON:{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }
}
