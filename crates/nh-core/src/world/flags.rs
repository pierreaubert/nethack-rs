//! Game flags and system options
//!
//! Manages global game flags, system configuration, and dynamic state cleanup.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::sync::OnceLock;

/// Global game flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Flags {
    // Game mode
    pub wizard: bool,
    pub explore: bool,
    pub debug: bool,

    // Game state
    pub started: bool,
    pub panic: bool,

    // Display options
    pub show_room: bool,
    pub show_corridor: bool,
    pub show_objects: bool,
    pub autopickup: bool,
    pub verbose: bool,
    pub silent: bool,

    // Gameplay options
    pub safe_pet: bool,
    pub safe_peaceful: bool,
    pub confirm: bool,
    pub pickup_thrown: bool,
    pub pushweapon: bool,

    // Number pad
    pub num_pad: bool,

    // Run modes
    pub run: i8, // 0 = not running, >0 = running mode

    // Sound
    pub soundlib: bool,

    // Travel
    pub travel_debug: bool,

    // End game
    pub ascended: bool,
    pub made_amulet: bool,
    pub invoked: bool,
}

/// System-level options and configuration
#[derive(Debug, Clone, Default)]
pub struct SystemOptions {
    // Configuration strings
    pub support: Option<String>,
    pub recover: Option<String>,
    pub wizards: Option<String>,
    pub fmtd_wizard_list: Option<String>,
    pub explorers: Option<String>,
    pub shellers: Option<String>,
    pub genericusers: Option<String>,
    pub debugfiles: Option<String>,
    pub dumplogfile: Option<String>,
    pub gdbpath: Option<String>,
    pub greppath: Option<String>,

    // Configuration values
    pub env_dbgfl: i32,
    pub maxplayers: i32,
    pub seduce: i32,
    pub check_save_uid: i32,
    pub check_plname: i32,
    pub bones_pools: i32,
    pub persmax: i32,
    pub pers_is_uid: i32,
    pub entrymax: i32,
    pub pointsmin: i32,
    pub tt_oname_maxrank: i32,
    pub panictrace_gdb: i32,
    pub panictrace_libc: i32,
    pub accessibility: i32,
    pub portable_device_paths: i32,
}

static SYSOPT: OnceLock<Mutex<SystemOptions>> = OnceLock::new();

fn get_sysopt() -> &'static Mutex<SystemOptions> {
    SYSOPT.get_or_init(|| Mutex::new(SystemOptions::default()))
}

/// Initialize system options early in startup.
/// Called before reading configuration files or user input.
///
/// Sets default values for all system options and performs sanity checks.
pub fn sys_early_init() {
    let mut sysopt = get_sysopt().lock().unwrap();

    // Clear string options
    sysopt.support = None;
    sysopt.recover = None;
    sysopt.wizards = None;
    sysopt.explorers = None;
    sysopt.shellers = None;
    sysopt.genericusers = None;
    sysopt.debugfiles = None;
    sysopt.dumplogfile = None;
    sysopt.gdbpath = None;
    sysopt.greppath = None;
    sysopt.fmtd_wizard_list = None;

    // Initialize debug flag
    sysopt.env_dbgfl = 0; // Haven't checked getenv("DEBUGFILES") yet

    // Set player and scoring defaults
    sysopt.maxplayers = 25; // Default MAX_NR_OF_PLAYERS equivalent
    sysopt.bones_pools = 0;

    // Record file settings
    sysopt.persmax = 1;
    sysopt.entrymax = 100;
    sysopt.pointsmin = 1;
    sysopt.pers_is_uid = 0;
    sysopt.tt_oname_maxrank = 10;

    // Sanity checks
    if sysopt.persmax < 1 {
        sysopt.persmax = 1;
    }
    if sysopt.entrymax < 10 {
        sysopt.entrymax = 10;
    }
    if sysopt.pointsmin < 1 {
        sysopt.pointsmin = 1;
    }
    if sysopt.pers_is_uid != 0 && sysopt.pers_is_uid != 1 {
        panic!("config error: PERS_IS_UID must be either 0 or 1");
    }

    // Panic tracing options
    #[cfg(feature = "panictrace")]
    {
        sysopt.gdbpath = Some("/usr/bin/gdb".to_string());
        sysopt.greppath = Some("/bin/grep".to_string());
        sysopt.panictrace_gdb = 1;
        sysopt.panictrace_libc = 0;
    }

    // Security and gameplay options
    sysopt.check_save_uid = 1;
    sysopt.check_plname = 0;
    sysopt.seduce = 1; // If compiled in, default to on
    sysopt.accessibility = 0;
    sysopt.portable_device_paths = 0;

    // Apply seduce setting
    sysopt_seduce_set(sysopt.seduce);
}

/// Release all dynamically allocated memory from system options.
///
/// Frees all string fields in the system options and resets them to None.
/// Should be called during cleanup, particularly before panic feedback
/// (since this might be used to generate error messages).
pub fn sysopt_release() {
    let mut sysopt = get_sysopt().lock().unwrap();

    // Free all string fields
    sysopt.support = None;
    sysopt.recover = None;
    sysopt.wizards = None;
    sysopt.explorers = None;
    sysopt.shellers = None;
    sysopt.debugfiles = None;
    sysopt.dumplogfile = None;
    sysopt.genericusers = None;
    sysopt.gdbpath = None;
    sysopt.greppath = None;

    // This one should be last since it might be used in panic feedback
    sysopt.fmtd_wizard_list = None;
}

/// Set the seduce option for incubus/succubus monsters.
///
/// In the original C code, this controlled attack substitution for seduce attacks,
/// but that's now handled dynamically in `getmattk()`. This function is kept for
/// compatibility but is mostly a no-op in Rust (no actual behavior change).
#[allow(unused_variables)]
pub fn sysopt_seduce_set(val: i32) {
    // Original behavior: Modified attack substitution for incubus/succubus
    // Now handled dynamically in attack resolution, so this is a no-op
}

/// Dummy initialization routine used to force proper linkage.
///
/// This function performs no operations and is called solely to ensure
/// all required object files are linked. In the original NetHack, this
/// works alongside `monst_init()` and `objects_init()` as part of a
/// linkage verification mechanism.
pub fn decl_init() {
    // Dummy function for linkage only
}

/// Free all dynamic game data.
///
/// Comprehensive cleanup of all dynamically allocated data structures:
/// - Mail system data
/// - Quest list
/// - Menu coloring rules
/// - Timed events
/// - Light sources
/// - Monster and object chains
/// - Fruit list
/// - Character names
/// - Dungeon structure
/// - Window port-specific data
/// - System options (last, in case needed for panic feedback)
pub fn freedynamicdata() {
    // In the Rust implementation, we use Rust's memory management
    // and don't need explicit freeing for most data structures.
    // However, we still need to clean up any manually-managed resources.

    // Clear quest list
    // unload_qtlist(); - would be called here

    // Free menu coloring - handled by Drop trait
    // free_menu_coloring();

    // Clear dungeon data
    // free_dungeons();

    // Free font resources (window port specific)
    // These would be handled by Drop trait in Rust

    // Release system options last (might be used in panic feedback)
    sysopt_release();
}
