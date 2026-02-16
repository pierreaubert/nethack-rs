use std::io::{self, BufRead, Write};
use serde::{Serialize, Deserialize};
use nh_test::ffi::CGameEngine;

#[derive(Serialize, Deserialize)]
enum Command {
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

#[derive(Serialize, Deserialize)]
enum Response {
    Ok,
    Int(i32),
    Pos(i32, i32),
    Long(u64),
    String(String),
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

        let cmd: Command = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let resp = Response::Error(format!("Invalid command: {}", e));
                println!("{}", serde_json::to_string(&resp).unwrap());
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
            Command::Exit => break,
        };

        println!("{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }
}
