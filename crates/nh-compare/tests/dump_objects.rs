use nh_test::ffi::CGameEngineSubprocess as CGameEngine;
use serde_json::Value;
use serial_test::serial;

#[test]
#[serial]
fn dump_c_object_table() {
    let mut c_engine = CGameEngine::new();
    c_engine.init("Tourist", "Human", 0, 0).expect("C engine init failed");
    
    let json_str = c_engine.object_table_json();
    let table: Value = serde_json::from_str(&json_str).unwrap();
    
    println!("=== C Object Table ===");
    for obj in table.as_array().unwrap() {
        println!("{}: {}", obj["index"], obj["name"]);
    }
}
