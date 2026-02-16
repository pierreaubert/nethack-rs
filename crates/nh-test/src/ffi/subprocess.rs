use std::process::{Command, Child, Stdio};
use std::io::{Write, BufReader, BufRead};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow, Context};

#[derive(Serialize, Deserialize, Debug)]
enum CommandMsg {
    Init { role: String, race: String, gender: i32, align: i32 },
    Reset { seed: u64 },
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
    Exit,
}

#[derive(Serialize, Deserialize, Debug)]
enum ResponseMsg {
    Ok,
    Int(i32),
    Pos(i32, i32),
    Long(u64),
    String(String),
    Error(String),
}

pub struct CGameEngineSubprocess {
    child: Child,
    writer: std::io::BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
}

impl CGameEngineSubprocess {
    pub fn new() -> Result<Self> {
        // Find the worker binary. In cargo tests, it should be in the same dir as the test exe.
        let mut exe_path = std::env::current_exe()?;
        exe_path.pop(); // Remove filename
        if exe_path.ends_with("deps") {
            exe_path.pop();
        }
        
        let worker_path = exe_path.join("nh-test-worker");
        
        // If not found (e.g. during dev), try to use 'cargo run' as fallback (not ideal for perf)
        let mut child = if worker_path.exists() {
            Command::new(worker_path)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?
        } else {
            // Fallback for development if binary hasn't been built yet
            Command::new("cargo")
                .args(&["run", "--bin", "nh-test-worker", "-q", "-p", "nh-test"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?
        };

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to open stdout"))?;

        Ok(Self {
            child,
            writer: std::io::BufWriter::new(stdin),
            reader: BufReader::new(stdout),
        })
    }

    fn send_command(&mut self, cmd: CommandMsg) -> Result<ResponseMsg> {
        let json = serde_json::to_string(&cmd)?;
        self.writer.write_all(json.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;

        let mut line = String::new();
        self.reader.read_line(&mut line).context("Failed to read from worker")?;
        if line.is_empty() {
            return Err(anyhow!("Worker process exited unexpectedly"));
        }
        
        let resp: ResponseMsg = serde_json::from_str(&line).context(format!("Failed to parse worker response: {}", line))?;
        Ok(resp)
    }

    pub fn init(&mut self, role: &str, race: &str, gender: i32, align: i32) -> Result<()> {
        match self.send_command(CommandMsg::Init {
            role: role.to_string(),
            race: race.to_string(),
            gender,
            align,
        })? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from worker")),
        }
    }

    pub fn reset(&mut self, seed: u64) -> Result<()> {
        match self.send_command(CommandMsg::Reset { seed })? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from worker")),
        }
    }

    pub fn hp(&mut self) -> i32 {
        match self.send_command(CommandMsg::GetHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn max_hp(&mut self) -> i32 {
        match self.send_command(CommandMsg::GetMaxHp).unwrap() {
            ResponseMsg::Int(hp) => hp,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn energy(&mut self) -> i32 {
        match self.send_command(CommandMsg::GetEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn max_energy(&mut self) -> i32 {
        match self.send_command(CommandMsg::GetMaxEnergy).unwrap() {
            ResponseMsg::Int(e) => e,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn position(&mut self) -> (i32, i32) {
        match self.send_command(CommandMsg::GetPosition).unwrap() {
            ResponseMsg::Pos(x, y) => (x, y),
            _ => panic!("Unexpected response"),
        }
    }

    pub fn turn_count(&mut self) -> u64 {
        match self.send_command(CommandMsg::GetTurnCount).unwrap() {
            ResponseMsg::Long(t) => t,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn state_json(&mut self) -> String {
        match self.send_command(CommandMsg::GetStateJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn map_json(&mut self) -> String {
        match self.send_command(CommandMsg::GetMapJson).unwrap() {
            ResponseMsg::String(s) => s,
            _ => panic!("Unexpected response"),
        }
    }

    pub fn exec_cmd(&mut self, cmd: char) -> Result<()> {
        match self.send_command(CommandMsg::ExecCmd { cmd })? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from worker")),
        }
    }

    pub fn exec_cmd_dir(&mut self, cmd: char, dx: i32, dy: i32) -> Result<()> {
        match self.send_command(CommandMsg::ExecCmdDir { cmd, dx, dy })? {
            ResponseMsg::Ok => Ok(()),
            ResponseMsg::Error(e) => Err(anyhow!(e)),
            _ => Err(anyhow!("Unexpected response from worker")),
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
        let mut engine = CGameEngineSubprocess::new().unwrap();
        engine.init("Valkyrie", "Human", 0, 0).unwrap();
        
        // Valkyrie Human starts with 16 HP at seed 42
        assert_eq!(engine.hp(), 16);
        assert_eq!(engine.max_hp(), 16);
    }

    #[test]
    fn test_subprocess_multi_init() {
        // This used to SIGABRT with regular CGameEngine
        for _ in 0..3 {
            let mut engine = CGameEngineSubprocess::new().unwrap();
            engine.init("Valkyrie", "Human", 0, 0).unwrap();
            assert_eq!(engine.hp(), 16);
        }
    }
}
