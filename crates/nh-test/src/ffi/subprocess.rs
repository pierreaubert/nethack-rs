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
    GetNutrition,
    GetAttributesJson,
    ExportLevel,
    EnableRngTracing,
    DisableRngTracing,
    GetRngTrace,
    ClearRngTrace,
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

impl CGameEngineSubprocess {
    pub fn new() -> Self {
        let mut exe_path = std::env::current_exe().expect("Failed to get current exe path");
        exe_path.pop();
        if exe_path.ends_with("deps") {
            exe_path.pop();
        }
        
        let worker_path = exe_path.join("nh-test-worker");
        
        let mut child = if worker_path.exists() {
            Command::new(worker_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn().expect("Failed to spawn worker")
        } else {
            Command::new("cargo")
                .args(&["run", "--bin", "nh-test-worker", "-q", "-p", "nh-test"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn().expect("Failed to spawn worker via cargo")
        };

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

    pub fn init(&mut self, role: &str, race: &str, gender: i32, align: i32) -> Result<(), String> {
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

    pub fn reset(&mut self, seed: u64) -> Result<(), String> {
        match self.send_command(CommandMsg::Reset { seed }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn reset_rng(&self, seed: u64) -> Result<(), String> {
        match self.send_command(CommandMsg::ResetRng { seed }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn generate_level(&self) -> Result<(), String> {
        match self.send_command(CommandMsg::GenerateLevel).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn generate_maze(&self) -> Result<(), String> {
        match self.send_command(CommandMsg::GenerateMaze).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn hp(&self) -> i32 {
        match self.send_command(CommandMsg::GetHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn max_hp(&self) -> i32 {
        match self.send_command(CommandMsg::GetMaxHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn energy(&self) -> i32 {
        match self.send_command(CommandMsg::GetEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn max_energy(&self) -> i32 {
        match self.send_command(CommandMsg::GetMaxEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn position(&self) -> (i32, i32) {
        match self.send_command(CommandMsg::GetPosition).unwrap() {
            ResponseMsg::Pos(x, y) => (x, y),
            _ => panic!("Unexpected response"),
        }
    }

    pub fn turn_count(&self) -> u64 {
        match self.send_command(CommandMsg::GetTurnCount).unwrap() {
            ResponseMsg::Long(t) => t,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn state_json(&self) -> String {
        match self.send_command(CommandMsg::GetStateJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn map_json(&self) -> String {
        match self.send_command(CommandMsg::GetMapJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn exec_cmd(&self, cmd: char) -> Result<(), String> {
        match self.send_command(CommandMsg::ExecCmd { cmd }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn exec_cmd_dir(&self, cmd: char, dx: i32, dy: i32) -> Result<(), String> {
        match self.send_command(CommandMsg::ExecCmdDir { cmd, dx, dy }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn set_dlevel(&self, dnum: i32, dlevel: i32) {
        let _ = self.send_command(CommandMsg::SetDLevel { dnum, dlevel });
    }

    pub fn set_state(&self, hp: i32, hpmax: i32, x: i32, y: i32, ac: i32, moves: i64) {
        let _ = self.send_command(CommandMsg::SetState { hp, hpmax, x, y, ac, moves });
    }

    pub fn armor_class(&self) -> i32 {
        match self.send_command(CommandMsg::GetArmorClass).unwrap() {
            ResponseMsg::Int(ac) => ac,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn gold(&self) -> i32 {
        match self.send_command(CommandMsg::GetGold).unwrap() {
            ResponseMsg::Int(g) => g,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn experience_level(&self) -> i32 {
        match self.send_command(CommandMsg::GetExperienceLevel).unwrap() {
            ResponseMsg::Int(l) => l,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn current_level(&self) -> i32 {
        match self.send_command(CommandMsg::GetCurrentLevel).unwrap() {
            ResponseMsg::Int(l) => l,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn dungeon_depth(&self) -> i32 {
        match self.send_command(CommandMsg::GetDungeonDepth).unwrap() {
            ResponseMsg::Int(d) => d,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn is_dead(&self) -> bool {
        match self.send_command(CommandMsg::IsDead).unwrap() {
            ResponseMsg::Bool(b) => b,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn last_message(&self) -> String {
        match self.send_command(CommandMsg::GetLastMessage).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn inventory_count(&self) -> i32 {
        match self.send_command(CommandMsg::GetInventoryCount).unwrap() {
            ResponseMsg::Int(c) => c,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn inventory_json(&self) -> String {
        match self.send_command(CommandMsg::GetInventoryJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn object_table_json(&self) -> String {
        match self.send_command(CommandMsg::GetObjectTableJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn monsters_json(&self) -> String {
        match self.send_command(CommandMsg::GetMonstersJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn set_wizard_mode(&self, enable: bool) {
        let _ = self.send_command(CommandMsg::SetWizardMode { enable });
    }

    pub fn add_item_to_inv(&self, item_id: i32, weight: i32) -> Result<(), String> {
        match self.send_command(CommandMsg::AddItemToInv { item_id, weight }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn carrying_weight(&self) -> i32 {
        match self.send_command(CommandMsg::GetCarryingWeight).unwrap() {
            ResponseMsg::Int(w) => w,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn monster_count(&self) -> i32 {
        match self.send_command(CommandMsg::GetMonsterCount).unwrap() {
            ResponseMsg::Int(c) => c,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn role(&self) -> String {
        match self.send_command(CommandMsg::GetRole).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn race(&self) -> String {
        match self.send_command(CommandMsg::GetRace).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unknown response"),
        }
    }

    pub fn gender_string(&self) -> String {
        match self.send_command(CommandMsg::GetGenderString).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn alignment_string(&self) -> String {
        match self.send_command(CommandMsg::GetAlignmentString).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn result_message(&self) -> String {
        match self.send_command(CommandMsg::GetResultMessage).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn rng_rn2(&self, limit: i32) -> i32 {
        match self.send_command(CommandMsg::RngRn2 { limit }).unwrap() {
            ResponseMsg::Int(i) => i,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn calc_base_damage(&self, weapon_id: i32, small_monster: bool) -> i32 {
        match self.send_command(CommandMsg::CalcBaseDamage { weapon_id, small_monster }).unwrap() {
            ResponseMsg::Int(d) => d,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn ac(&self) -> i32 {
        match self.send_command(CommandMsg::GetAc).unwrap() {
            ResponseMsg::Int(a) => a,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn test_setup_status(&self, hp: i32, max_hp: i32, level: i32, ac: i32) {
        let _ = self.send_command(CommandMsg::TestSetupStatus { hp, max_hp, level, ac });
    }

    pub fn wear_item(&self, item_id: i32) -> Result<(), String> {
        match self.send_command(CommandMsg::WearItem { item_id }).map_err(|e| e.to_string())? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(e),
            _ => Err("Unexpected response".to_string()),
        }
    }

    pub fn nutrition(&self) -> i32 {
        match self.send_command(CommandMsg::GetNutrition).unwrap() {
            ResponseMsg::Int(n) => n,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn attributes_json(&self) -> String {
        match self.send_command(CommandMsg::GetAttributesJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn export_level(&self) -> String {
        match self.send_command(CommandMsg::ExportLevel).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn enable_rng_tracing(&self) {
        let _ = self.send_command(CommandMsg::EnableRngTracing);
    }

    pub fn disable_rng_tracing(&self) {
        let _ = self.send_command(CommandMsg::DisableRngTracing);
    }

    pub fn rng_trace_json(&self) -> String {
        match self.send_command(CommandMsg::GetRngTrace).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn clear_rng_trace(&self) {
        let _ = self.send_command(CommandMsg::ClearRngTrace);
    }

    // ========================================================================
    // Function-level isolation testing (Phase 1)
    // ========================================================================

    pub fn test_finddpos(&self, xl: i32, yl: i32, xh: i32, yh: i32) -> (i32, i32) {
        match self.send_command(CommandMsg::TestFinddpos { xl, yl, xh, yh }).unwrap() {
            ResponseMsg::Pos(x, y) => (x, y),
            _ => panic!("Unexpected response"),
        }
    }

    pub fn test_dig_corridor(&self, sx: i32, sy: i32, dx: i32, dy: i32, nxcor: bool) -> bool {
        match self.send_command(CommandMsg::TestDigCorridor { sx, sy, dx, dy, nxcor }).unwrap() {
            ResponseMsg::Bool(b) => b,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn test_makecorridors(&self) {
        let _ = self.send_command(CommandMsg::TestMakecorridors);
    }

    pub fn test_join(&self, a: i32, b: i32, nxcor: bool) {
        let _ = self.send_command(CommandMsg::TestJoin { a, b, nxcor });
    }

    pub fn get_smeq(&self) -> String {
        match self.send_command(CommandMsg::GetSmeq).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn get_doorindex(&self) -> i32 {
        match self.send_command(CommandMsg::GetDoorindex).unwrap() {
            ResponseMsg::Int(i) => i,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn get_cell_region(&self, x1: i32, y1: i32, x2: i32, y2: i32) -> String {
        match self.send_command(CommandMsg::GetCellRegion { x1, y1, x2, y2 }).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn set_cell(&self, x: i32, y: i32, typ: i32) {
        let _ = self.send_command(CommandMsg::SetCell { x, y, typ });
    }

    pub fn clear_level(&self) {
        let _ = self.send_command(CommandMsg::ClearLevel);
    }

    pub fn add_room(&self, lx: i32, ly: i32, hx: i32, hy: i32, rtype: i32) -> i32 {
        match self.send_command(CommandMsg::AddRoom { lx, ly, hx, hy, rtype }).unwrap() {
            ResponseMsg::Int(i) => i,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn carve_room(&self, lx: i32, ly: i32, hx: i32, hy: i32) {
        let _ = self.send_command(CommandMsg::CarveRoom { lx, ly, hx, hy });
    }

    pub fn rect_json(&self) -> String {
        match self.send_command(CommandMsg::GetRectJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }
}

impl Drop for CGameEngineSubprocess {
    fn drop(&mut self) {
        let _ = self.send_command(CommandMsg::Exit);
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subprocess_init() {
        let mut engine = CGameEngineSubprocess::new();
        engine.init("Valkyrie", "Human", 0, 0).unwrap();
        assert_eq!(engine.hp(), 16);
    }

    #[test]
    fn test_subprocess_multi_init() {
        for _ in 0..2 {
            let mut engine = CGameEngineSubprocess::new();
            engine.init("Valkyrie", "Human", 0, 0).unwrap();
            assert_eq!(engine.hp(), 16);
        }
    }
}
