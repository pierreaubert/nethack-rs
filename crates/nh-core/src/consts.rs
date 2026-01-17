//! Core game constants from NetHack
//!
//! These are derived from include/config.h, include/global.h, and other headers.

/// Map dimensions
pub const COLNO: usize = 80;
pub const ROWNO: usize = 21;

/// Maximum dungeon depth
pub const MAXDUNGEON: usize = 16;
pub const MAXLEVEL: usize = 32;

/// Maximum player level
pub const MAXULEV: usize = 30;

/// Movement speed constants (from permonst.h)
pub const NORMAL_SPEED: i16 = 12;
pub const FAST_SPEED: i16 = 18;
pub const SLOW_SPEED: i16 = 6;

/// Maximum number of attacks per monster
pub const NATTK: usize = 6;

/// Maximum number of skills
pub const P_NUM_SKILLS: usize = 42;

/// Room limits
pub const MAXNROFROOMS: usize = 40;
pub const MAX_SUBROOMS: usize = 24;

/// Object limits
pub const NUM_OBJECTS: usize = 500; // approximate, will be exact from objects.c

/// Monster limits
pub const NUMMONS: usize = 400; // approximate, will be exact from monst.c

/// Inventory letters
pub const GOLD_SYM: char = '$';
pub const WEAPON_SYM: char = ')';
pub const ARMOR_SYM: char = '[';
pub const RING_SYM: char = '=';
pub const AMULET_SYM: char = '"';
pub const TOOL_SYM: char = '(';
pub const FOOD_SYM: char = '%';
pub const POTION_SYM: char = '!';
pub const SCROLL_SYM: char = '?';
pub const SPBOOK_SYM: char = '+';
pub const WAND_SYM: char = '/';
pub const GEM_SYM: char = '*';
pub const ROCK_SYM: char = '`';
pub const BALL_SYM: char = '0';
pub const CHAIN_SYM: char = '_';
pub const VENOM_SYM: char = '.';

/// Map symbols
pub const S_STONE: char = ' ';
pub const S_VWALL: char = '|';
pub const S_HWALL: char = '-';
pub const S_TLCORN: char = '-';
pub const S_TRCORN: char = '-';
pub const S_BLCORN: char = '-';
pub const S_BRCORN: char = '-';
pub const S_CRWALL: char = '-';
pub const S_ROOM: char = '.';
pub const S_CORR: char = '#';
pub const S_LITCORR: char = '#';
pub const S_UPSTAIR: char = '<';
pub const S_DNSTAIR: char = '>';
pub const S_UPLADDER: char = '<';
pub const S_DNLADDER: char = '>';
pub const S_ALTAR: char = '_';
pub const S_GRAVE: char = '|';
pub const S_THRONE: char = '\\';
pub const S_SINK: char = '#';
pub const S_FOUNTAIN: char = '{';
pub const S_POOL: char = '}';
pub const S_ICE: char = '.';
pub const S_LAVA: char = '}';
pub const S_VODOOR: char = '|';
pub const S_HODOOR: char = '-';
pub const S_VCDOOR: char = '+';
pub const S_HCDOOR: char = '+';
pub const S_BARS: char = '#';
pub const S_TREE: char = '#';
pub const S_ARROW_TRAP: char = '^';
pub const S_TELEPORTATION_TRAP: char = '^';
pub const S_WEB: char = '"';

/// Attribute indices
pub const A_STR: usize = 0;
pub const A_INT: usize = 1;
pub const A_WIS: usize = 2;
pub const A_DEX: usize = 3;
pub const A_CON: usize = 4;
pub const A_CHA: usize = 5;
pub const NUM_ATTRS: usize = 6;

/// Alignment values
pub const A_LAWFUL: i8 = 1;
pub const A_NEUTRAL: i8 = 0;
pub const A_CHAOTIC: i8 = -1;

/// Hunger thresholds
pub const SATIATED: i32 = 0;
pub const NOT_HUNGRY: i32 = 1;
pub const HUNGRY: i32 = 2;
pub const WEAK: i32 = 3;
pub const FAINTING: i32 = 4;
pub const FAINTED: i32 = 5;
pub const STARVED: i32 = 6;

/// Nutrition values
pub const HUNGER_MAX: i32 = 2000;
pub const HUNGER_DECREMENT: i32 = 1; // per turn

/// Base armor class (no armor)
pub const BASE_AC: i8 = 10;

/// Experience level thresholds
pub const EXP_THRESHOLDS: [u64; MAXULEV] = [
    0,          // level 1
    20,         // level 2
    40,
    80,
    160,
    320,
    640,
    1280,
    2560,
    5120,       // level 10
    10000,
    20000,
    40000,
    80000,
    160000,
    320000,
    640000,
    1280000,
    2560000,
    5120000,    // level 20
    10000000,
    20000000,
    40000000,
    80000000,
    160000000,
    320000000,
    640000000,
    1280000000,
    2560000000,
    5120000000, // level 30
];
