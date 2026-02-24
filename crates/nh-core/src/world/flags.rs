//! Game flags and system options
//!
//! Manages global game flags, system configuration, and dynamic state cleanup.

use serde::{Deserialize, Serialize};

/// Global game flags
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Show legacy intro text (C: flags.legacy, default TRUE)
    #[serde(default = "default_true")]
    pub legacy: bool,

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

    // Real-world effects (C: flags.moonphase, flags.friday13)
    /// Phase of the moon at game start (0-7; 0=new, 4=full)
    #[serde(default)]
    pub moonphase: i32,
    /// Whether the game started on Friday the 13th
    #[serde(default)]
    pub friday13: bool,

    // End game
    pub ascended: bool,
    pub made_amulet: bool,
    pub invoked: bool,
}

fn default_true() -> bool {
    true
}

impl Default for Flags {
    /// Default flags matching C's initoptions() defaults.
    ///
    /// C: verbose=TRUE, confirm=TRUE, safe_pet=TRUE, pickup_thrown=TRUE,
    ///    legacy=TRUE, autopickup=TRUE (many players disable this).
    fn default() -> Self {
        Self {
            wizard: false,
            explore: false,
            debug: false,
            started: false,
            panic: false,
            show_room: false,
            show_corridor: false,
            show_objects: false,
            autopickup: true,
            verbose: true,
            silent: false,
            legacy: true,
            safe_pet: true,
            safe_peaceful: true,
            confirm: true,
            pickup_thrown: true,
            pushweapon: false,
            num_pad: false,
            run: 0,
            soundlib: false,
            travel_debug: false,
            moonphase: 0,
            friday13: false,
            ascended: false,
            made_amulet: false,
            invoked: false,
        }
    }
}

// --- System options (std-only, uses Mutex/OnceLock) ---

#[cfg(feature = "std")]
mod sys {
    use std::sync::Mutex;
    use std::sync::OnceLock;

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
    pub fn sys_early_init() {
        let mut sysopt = get_sysopt().lock().unwrap();

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

        sysopt.env_dbgfl = 0;
        sysopt.maxplayers = 25;
        sysopt.bones_pools = 0;

        sysopt.persmax = 1;
        sysopt.entrymax = 100;
        sysopt.pointsmin = 1;
        sysopt.pers_is_uid = 0;
        sysopt.tt_oname_maxrank = 10;

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

        #[cfg(feature = "panictrace")]
        {
            sysopt.gdbpath = Some("/usr/bin/gdb".to_string());
            sysopt.greppath = Some("/bin/grep".to_string());
            sysopt.panictrace_gdb = 1;
            sysopt.panictrace_libc = 0;
        }

        sysopt.check_save_uid = 1;
        sysopt.check_plname = 0;
        sysopt.seduce = 1;
        sysopt.accessibility = 0;
        sysopt.portable_device_paths = 0;

        sysopt_seduce_set(sysopt.seduce);
    }

    /// Release all dynamically allocated memory from system options.
    pub fn sysopt_release() {
        let mut sysopt = get_sysopt().lock().unwrap();

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
        sysopt.fmtd_wizard_list = None;
    }

    #[allow(unused_variables)]
    pub fn sysopt_seduce_set(val: i32) {
        // No-op in Rust — handled dynamically in attack resolution
    }

    pub fn decl_init() {
        // Dummy function for linkage only
    }

    pub fn freedynamicdata() {
        sysopt_release();
    }
}

#[cfg(feature = "std")]
pub use sys::*;
