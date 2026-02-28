use std::process::{Command, Child, Stdio};
use std::io::{Write, BufReader, BufRead, BufWriter};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};
use std::cell::RefCell;

#[derive(Serialize, Deserialize, Debug)]
enum CommandMsg {
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

#[derive(Serialize, Deserialize, Debug)]
enum ResponseMsg {
    Ok,
    Int(i32),
    Pos(i32, i32),
    Long(u64),
    String(String),
    Bool(bool),
    Error(String),
}

pub struct CGameEngineSubprocess {
    child: Child,
    writer: RefCell<BufWriter<std::process::ChildStdin>>,
    reader: RefCell<BufReader<std::process::ChildStdout>>,
}

impl nh_core::CGameEngineTrait for CGameEngineSubprocess {
    fn init(&mut self, role: &str, race: &str, gender: i32, align: i32) -> Result<(), String> {
        match self.send_command(CommandMsg::Init {
            role: role.to_string(),
            race: race.to_string(),
            gender,
            align,
        }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    fn reset(&mut self, seed: u64) -> Result<(), String> {
        match self.send_command(CommandMsg::Reset { seed }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    fn generate_and_place(&self) -> Result<(), String> {
        match self.send_command(CommandMsg::GenerateAndPlace).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    fn export_level(&self) -> String {
        match self.send_command(CommandMsg::ExportLevel).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn exec_cmd(&self, cmd: char) -> Result<(), String> {
        match self.send_command(CommandMsg::ExecCmd { cmd }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    fn exec_cmd_dir(&self, cmd: char, dx: i32, dy: i32) -> Result<(), String> {
        match self.send_command(CommandMsg::ExecCmdDir { cmd, dx, dy }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    fn hp(&self) -> i32 {
        match self.send_command(CommandMsg::GetHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    fn max_hp(&self) -> i32 {
        match self.send_command(CommandMsg::GetMaxHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    fn energy(&self) -> i32 {
        match self.send_command(CommandMsg::GetEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    fn max_energy(&self) -> i32 {
        match self.send_command(CommandMsg::GetMaxEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    fn gold(&self) -> i32 {
        match self.send_command(CommandMsg::GetGold).unwrap() {
            ResponseMsg::Int(g) => g,
            _ => panic!("Unexpected response"),
        }
    }

    fn position(&self) -> (i32, i32) {
        match self.send_command(CommandMsg::GetPosition).unwrap() {
            ResponseMsg::Pos(x, y) => (x, y),
            _ => panic!("Unexpected response"),
        }
    }

    fn set_state(&self, hp: i32, hpmax: i32, x: i32, y: i32, ac: i32, moves: i64) {
        let _ = self.send_command(CommandMsg::SetState { hp, hpmax, x, y, ac, moves });
    }

    fn armor_class(&self) -> i32 {
        match self.send_command(CommandMsg::GetArmorClass).unwrap() {
            ResponseMsg::Int(ac) => ac,
            _ => panic!("Unexpected response"),
        }
    }

    fn experience_level(&self) -> i32 {
        match self.send_command(CommandMsg::GetExperienceLevel).unwrap() {
            ResponseMsg::Int(l) => l,
            _ => panic!("Unexpected response"),
        }
    }

    fn current_level(&self) -> i32 {
        match self.send_command(CommandMsg::GetCurrentLevel).unwrap() {
            ResponseMsg::Int(l) => l,
            _ => panic!("Unexpected response"),
        }
    }

    fn dungeon_depth(&self) -> i32 {
        match self.send_command(CommandMsg::GetDungeonDepth).unwrap() {
            ResponseMsg::Int(d) => d,
            _ => panic!("Unexpected response"),
        }
    }

    fn turn_count(&self) -> u64 {
        match self.send_command(CommandMsg::GetTurnCount).unwrap() {
            ResponseMsg::Long(t) => t,
            _ => panic!("Unexpected response"),
        }
    }

    fn is_dead(&self) -> bool {
        match self.send_command(CommandMsg::IsDead).unwrap() {
            ResponseMsg::Bool(b) => b,
            _ => panic!("Unexpected response"),
        }
    }

    fn is_game_over(&self) -> bool {
        self.is_dead()
    }

    fn is_won(&self) -> bool {
        false
    }

    fn state_json(&self) -> String {
        match self.send_command(CommandMsg::GetStateJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn last_message(&self) -> String {
        match self.send_command(CommandMsg::GetLastMessage).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn inventory_json(&self) -> String {
        match self.send_command(CommandMsg::GetInventoryJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn monsters_json(&self) -> String {
        match self.send_command(CommandMsg::GetMonstersJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn role(&self) -> String {
        match self.send_command(CommandMsg::GetRole).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn race(&self) -> String {
        match self.send_command(CommandMsg::GetRace).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unknown response"),
        }
    }

    fn gender_string(&self) -> String {
        match self.send_command(CommandMsg::GetGenderString).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    fn alignment_string(&self) -> String {
        match self.send_command(CommandMsg::GetAlignmentString).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }
}

impl CGameEngineSubprocess {
    pub fn new() -> Self {
        let mut child = Command::new("cargo")
            .args(&["run", "--bin", "nh-test-worker", "-q", "-p", "nh-test"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn().expect("Failed to spawn worker via cargo");

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        Self {
            child,
            writer: RefCell::new(BufWriter::new(stdin)),
            reader: RefCell::new(BufReader::new(stdout)),
        }
    }

    fn send_command(&self, cmd: CommandMsg) -> Result<ResponseMsg> {
        let json = serde_json::to_string(&cmd)?;
        let mut writer = self.writer.borrow_mut();
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        let mut reader = self.reader.borrow_mut();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).context("Failed to read from worker")?;
            if line.is_empty() {
                return Err(anyhow!("Worker process exited unexpectedly"));
            }
            if let Some(json_content) = line.trim().strip_prefix("JSON:") {
                let resp: ResponseMsg = serde_json::from_str(json_content).context(format!("Failed to parse worker response: {}", json_content))?;
                return Ok(resp);
            }
        }
    }
}
