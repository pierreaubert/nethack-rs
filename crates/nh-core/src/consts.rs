//! Core game constants from NetHack
//!
//! These are derived from include/config.h, include/global.h, and other headers.

/// Map dimensions
#[cfg(not(feature = "std"))]
use crate::compat::*;

pub const COLNO: usize = 80;
pub const ROWNO: usize = 21;

/// Maximum dungeon depth
pub const MAXDUNGEON: usize = 16;
pub const MAXLEVEL: usize = 32;

/// Maximum player level
pub const MAXULEV: usize = 30;

/// Maximum carrying capacity (from hack.h)
pub const MAX_CARR_CAP: i32 = 1000;

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

/// Special object type indices (gray stones)
pub const LUCKSTONE: i16 = 7970;
pub const LOADSTONE: i16 = 7971;
pub const TOUCHSTONE: i16 = 7972;
pub const FLINT: i16 = 7973;
pub const ROCK: i16 = 7974;
pub const BOULDER: i16 = 7975;
pub const STATUE: i16 = 7976;

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
    0,  // level 1
    20, // level 2
    40, 80, 160, 320, 640, 1280, 2560, 5120, // level 10
    10000, 20000, 40000, 80000, 160000, 320000, 640000, 1280000, 2560000, 5120000, // level 20
    10000000, 20000000, 40000000, 80000000, 160000000, 320000000, 640000000, 1280000000,
    2560000000, 5120000000, // level 30
];

// ============================================================================
// Utility functions (from hack.h and various sources)
// ============================================================================

/// Sign function: returns -1, 0, or 1
#[inline]
pub const fn sgn(x: i32) -> i32 {
    if x < 0 {
        -1
    } else if x > 0 {
        1
    } else {
        0
    }
}

/// Check if position is within map bounds
#[inline]
pub const fn isok(x: i8, y: i8) -> bool {
    x >= 0 && (x as usize) < COLNO && y >= 0 && (y as usize) < ROWNO
}

/// Distance squared between two points
#[inline]
pub const fn dist2(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    dx * dx + dy * dy
}

/// Chebyshev distance (maximum of abs differences)
#[inline]
pub const fn distmin(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    if dx > dy { dx } else { dy }
}

/// Manhattan distance
#[inline]
pub const fn mdistu(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x2 - x1).abs() + (y2 - y1).abs()
}

/// Check if two positions are adjacent (including diagonally)
#[inline]
pub const fn next_to(x1: i8, y1: i8, x2: i8, y2: i8) -> bool {
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = (y2 as i32 - y1 as i32).abs();
    dx <= 1 && dy <= 1
}

/// Direction from dx,dy to direction index (0-7, or -1 for invalid)
#[inline]
pub const fn xytod(dx: i32, dy: i32) -> i32 {
    // Direction indices:
    // 7 0 1
    // 6 . 2
    // 5 4 3
    match (sgn(dx), sgn(dy)) {
        (0, -1) => 0,  // up
        (1, -1) => 1,  // up-right
        (1, 0) => 2,   // right
        (1, 1) => 3,   // down-right
        (0, 1) => 4,   // down
        (-1, 1) => 5,  // down-left
        (-1, 0) => 6,  // left
        (-1, -1) => 7, // up-left
        _ => -1,       // no direction (dx=0, dy=0)
    }
}

/// Direction index to dx offset
pub const XDIR: [i8; 8] = [0, 1, 1, 1, 0, -1, -1, -1];

/// Direction index to dy offset
pub const YDIR: [i8; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];

/// Convert direction index to dx,dy
#[inline]
pub const fn dtoxy(dir: usize) -> (i8, i8) {
    if dir < 8 {
        (XDIR[dir], YDIR[dir])
    } else {
        (0, 0)
    }
}

/// Integer square root (floor)
#[inline]
pub const fn isqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Round division (round to nearest integer)
#[inline]
pub const fn rounddiv(x: i32, y: i32) -> i32 {
    if y == 0 {
        return 0;
    }
    if (x >= 0) == (y >= 0) {
        (x + y / 2) / y
    } else {
        (x - y / 2) / y
    }
}

/// Check if coordinates are lined up horizontally, vertically, or diagonally
#[inline]
pub const fn lined_up(ax: i32, ay: i32, bx: i32, by: i32) -> bool {
    let dx = (bx - ax).abs();
    let dy = (by - ay).abs();
    // Same row, same column, or same diagonal
    ax == bx || ay == by || dx == dy
}

/// Check if target is within throwing/missile range (simple straight line)
#[inline]
pub const fn linedup(ax: i32, ay: i32, bx: i32, by: i32, range: i32) -> bool {
    lined_up(ax, ay, bx, by) && dist2(ax, ay, bx, by) <= range * range
}

/// Coulomb distance (used for diagonal movement cost estimation)
#[inline]
pub const fn coulomb(x: i32, y: i32) -> i32 {
    let ax = x.abs();
    let ay = y.abs();
    if ax > ay { ax + ay / 2 } else { ay + ax / 2 }
}

/// Check if character is a digit
#[inline]
pub const fn digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

/// Check if character is a letter
#[inline]
pub const fn letter(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
}

/// Convert to lowercase
#[inline]
pub const fn lowc(c: char) -> char {
    if c >= 'A' && c <= 'Z' {
        ((c as u8) - b'A' + b'a') as char
    } else {
        c
    }
}

/// Convert to uppercase
#[inline]
pub const fn highc(c: char) -> char {
    if c >= 'a' && c <= 'z' {
        ((c as u8) - b'a' + b'A') as char
    } else {
        c
    }
}

/// Capitalize first letter of string
pub fn upstart(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if let Some(first) = chars.first_mut() {
        *first = highc(*first);
    }
    chars.into_iter().collect()
}

/// Convert string to lowercase
pub fn lcase(s: &str) -> String {
    s.to_lowercase()
}

/// Convert string to uppercase
pub fn ucase(s: &str) -> String {
    s.to_uppercase()
}

/// Clamp value to range
#[inline]
pub const fn bounded_increase(val: i32, inc: i32, max: i32) -> i32 {
    let result = val + inc;
    if result > max { max } else { result }
}

/// Calculate ordinal suffix (1st, 2nd, 3rd, etc.)
pub fn ordin(n: i32) -> &'static str {
    let abs_n = n.abs();
    if abs_n % 100 >= 11 && abs_n % 100 <= 13 {
        "th"
    } else {
        match abs_n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

/// Format number with ordinal suffix
pub fn sitoa(n: i32) -> String {
    format!("{}{}", n, ordin(n))
}

/// Plural suffix - returns "s" if count != 1, "" otherwise
#[inline]
pub const fn plur(count: i32) -> &'static str {
    if count == 1 { "" } else { "s" }
}

/// "y" -> "ies" or just "s" for plural
pub fn ies(word: &str) -> String {
    if word.ends_with('y') {
        format!("{}ies", &word[..word.len() - 1])
    } else {
        format!("{}s", word)
    }
}

// ============================================================================
// Body parts (from hack.c body_part function)
// ============================================================================

/// Body part indices (from hack.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BodyPart {
    Arm = 0,
    Eye = 1,
    Face = 2,
    Finger = 3,
    Fingertip = 4,
    Foot = 5,
    Hand = 6,
    HandedNess = 7, // "left handed" vs "right handed"
    Head = 8,
    Leg = 9,
    LightHeaded = 10, // adjective "light headed"
    Neck = 11,
    Spine = 12,
    Toe = 13,
    Hair = 14,
    Blood = 15,
    Lung = 16,
    Nose = 17,
    Stomach = 18,
}

impl BodyPart {
    pub const LAST: BodyPart = BodyPart::Stomach;
}

/// Body form types for different creature categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum BodyForm {
    /// Standard humanoid (default)
    #[default]
    Humanoid = 0,
    /// Quadruped (dogs, horses, etc.)
    Quadruped = 1,
    /// Snake-like (no limbs)
    Serpentine = 2,
    /// Spider (arachnid)
    Spider = 3,
    /// Bird form
    Avian = 4,
    /// Fish form
    Fish = 5,
    /// Fungus form
    Fungus = 6,
    /// Vortex/whirlpool
    Vortex = 7,
    /// Jelly/ooze
    Jelly = 8,
}

/// Standard body part names for humanoid forms
const HUMANOID_PARTS: [&str; 19] = [
    "arm",
    "eye",
    "face",
    "finger",
    "fingertip",
    "foot",
    "hand",
    "handed",
    "head",
    "leg",
    "light headed",
    "neck",
    "spine",
    "toe",
    "hair",
    "blood",
    "lung",
    "nose",
    "stomach",
];

/// Body part names for quadrupeds (dogs, cats, horses)
const QUADRUPED_PARTS: [&str; 19] = [
    "foreleg",
    "eye",
    "face",
    "paw",
    "paw pad",
    "rear paw",
    "paw",
    "pawed",
    "head",
    "rear leg",
    "light headed",
    "neck",
    "spine",
    "rear toe",
    "fur",
    "blood",
    "lung",
    "nose",
    "stomach",
];

/// Body part names for snakes/serpents
const SERPENTINE_PARTS: [&str; 19] = [
    "upper body",
    "eye",
    "face",
    "tip of tail",
    "tip of tail",
    "lower body",
    "coil",
    "coiled",
    "head",
    "lower body",
    "light headed",
    "neck",
    "length",
    "tail tip",
    "scales",
    "blood",
    "lung",
    "nose",
    "stomach",
];

/// Body part names for spiders
const SPIDER_PARTS: [&str; 19] = [
    "foreleg",
    "eye",
    "face",
    "leg tip",
    "leg tip",
    "rear leg",
    "leg",
    "legged",
    "head",
    "rear leg",
    "light headed",
    "neck",
    "abdomen",
    "rear leg tip",
    "hair",
    "blood",
    "book lung",
    "chelicera",
    "stomach",
];

/// Body part names for birds
const AVIAN_PARTS: [&str; 19] = [
    "wing",
    "eye",
    "face",
    "talon",
    "talon",
    "foot",
    "wing",
    "winged",
    "head",
    "leg",
    "light headed",
    "neck",
    "back",
    "claw",
    "feathers",
    "blood",
    "lung",
    "beak",
    "crop",
];

/// Body part names for fish
const FISH_PARTS: [&str; 19] = [
    "fin",
    "eye",
    "face",
    "fin tip",
    "fin tip",
    "tail",
    "fin",
    "finned",
    "head",
    "tail",
    "light headed",
    "gill",
    "spine",
    "tail tip",
    "scales",
    "blood",
    "gill",
    "snout",
    "stomach",
];

/// Body part names for fungi/plants
const FUNGUS_PARTS: [&str; 19] = [
    "mycelium",
    "eye",
    "surface",
    "hypha",
    "hypha",
    "root",
    "mycelium",
    "rooted",
    "cap",
    "stalk",
    "spore headed",
    "stalk",
    "stalk",
    "root tip",
    "spores",
    "juice",
    "gill",
    "cap",
    "interior",
];

/// Body part names for vortices
const VORTEX_PARTS: [&str; 19] = [
    "region",
    "eye",
    "front",
    "minor current",
    "minor current",
    "lower current",
    "current",
    "currented",
    "center",
    "lower current",
    "unstable",
    "center",
    "center",
    "edge",
    "particles",
    "essence",
    "center",
    "leading edge",
    "center",
];

/// Body part names for jellies/oozes
const JELLY_PARTS: [&str; 19] = [
    "pseudopod",
    "dark spot",
    "front",
    "pseudopod extension",
    "pseudopod extension",
    "lower pseudopod",
    "pseudopod",
    "pseudopoded",
    "top",
    "lower pseudopod",
    "unstable",
    "middle",
    "middle",
    "lower edge",
    "surface",
    "ooze",
    "interior",
    "leading edge",
    "interior",
];

/// Get the name of a body part for a given body form
pub fn body_part(form: BodyForm, part: BodyPart) -> &'static str {
    let idx = part as usize;
    match form {
        BodyForm::Humanoid => HUMANOID_PARTS[idx],
        BodyForm::Quadruped => QUADRUPED_PARTS[idx],
        BodyForm::Serpentine => SERPENTINE_PARTS[idx],
        BodyForm::Spider => SPIDER_PARTS[idx],
        BodyForm::Avian => AVIAN_PARTS[idx],
        BodyForm::Fish => FISH_PARTS[idx],
        BodyForm::Fungus => FUNGUS_PARTS[idx],
        BodyForm::Vortex => VORTEX_PARTS[idx],
        BodyForm::Jelly => JELLY_PARTS[idx],
    }
}

/// Get the body part name for a humanoid (convenience function)
pub fn humanoid_body_part(part: BodyPart) -> &'static str {
    body_part(BodyForm::Humanoid, part)
}

/// Get the body part name for a monster (mbodypart equivalent)
///
/// Returns the appropriate body part name based on the monster's body form.
pub fn mbodypart(body_form: BodyForm, part: BodyPart) -> &'static str {
    body_part(body_form, part)
}

// ============================================================================
// Misc string utilities (from hacklib.c)
// ============================================================================

/// Return "an" or "a" based on the word
pub fn an(word: &str) -> String {
    if word.is_empty() {
        return "a".to_string();
    }
    let first = word.chars().next().unwrap().to_ascii_lowercase();
    if "aeiou".contains(first) {
        // Exceptions for "u" words that sound like "you"
        let lower = word.to_lowercase();
        if first == 'u' && (lower.starts_with("uni") || lower.starts_with("use")) {
            format!("a {}", word)
        } else {
            format!("an {}", word)
        }
    } else {
        format!("a {}", word)
    }
}

/// Return "the" + word
pub fn the(word: &str) -> String {
    format!("the {}", word)
}

/// Get phrase for object position at player's location (at_your_feet equivalent)
///
/// Returns a phrase like "at your feet", "beneath you", etc. based on the terrain.
pub fn at_your_feet(surface_name: &str) -> String {
    match surface_name {
        "water" | "lava" => "beneath you".to_string(),
        "air" | "cloud" => "beneath you".to_string(),
        _ => "at your feet".to_string(),
    }
}

/// Get coordinate description (coord_desc equivalent)
///
/// Returns a human-readable description of the given coordinates.
/// If the coordinates match the player's position (px, py), returns "your current location".
/// Otherwise, returns the coordinate pair.
pub fn coord_desc(x: i8, y: i8, px: i8, py: i8) -> String {
    if x == px && y == py {
        "your current location".to_string()
    } else {
        format!("({},{})", x, y)
    }
}

/// Make a plural version of a noun
pub fn makeplural(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    // Handle special cases
    let lower = word.to_lowercase();
    match lower.as_str() {
        "mouse" => return format!("{}ice", &word[..word.len() - 4]),
        "tooth" => return format!("{}eeth", &word[..word.len() - 4]),
        "foot" => return format!("{}eet", &word[..word.len() - 3]),
        "goose" => return format!("{}eese", &word[..word.len() - 4]),
        "child" => return format!("{}ren", word),
        _ => {}
    }

    // Ends in -y preceded by consonant -> -ies
    if word.ends_with('y') {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() > 1 {
            let second_last = chars[chars.len() - 2].to_ascii_lowercase();
            if !"aeiou".contains(second_last) {
                return format!("{}ies", &word[..word.len() - 1]);
            }
        }
    }

    // Ends in -s, -x, -z, -ch, -sh -> add -es
    if word.ends_with('s')
        || word.ends_with('x')
        || word.ends_with('z')
        || word.ends_with("ch")
        || word.ends_with("sh")
    {
        return format!("{}es", word);
    }

    // Ends in -f or -fe -> -ves (with some exceptions)
    if word.ends_with('f') {
        return format!("{}ves", &word[..word.len() - 1]);
    }
    if word.ends_with("fe") {
        return format!("{}ves", &word[..word.len() - 2]);
    }

    // Default: add -s
    format!("{}s", word)
}

/// Add possessive 's or just ' for words ending in s
pub fn s_suffix(word: &str) -> String {
    if word.ends_with('s') || word.ends_with('x') || word.ends_with('z') {
        format!("{}'", word)
    } else {
        format!("{}'s", word)
    }
}

/// Check if string starts with a vowel sound
pub fn starts_with_vowel(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let first = s.chars().next().unwrap().to_ascii_lowercase();
    "aeiou".contains(first)
}

// ============================================================================
// Appearance and color functions (from do_name.c, apply.c, mondata.c)
// ============================================================================

/// Hallucination color strings
const HCOLORS: [&str; 33] = [
    "ultraviolet",
    "infrared",
    "bluish-orange",
    "reddish-green",
    "dark white",
    "light black",
    "sky blue-pink",
    "salty",
    "sweet",
    "sour",
    "bitter",
    "striped",
    "spiral",
    "swirly",
    "plaid",
    "checkered",
    "argyle",
    "paisley",
    "blotchy",
    "guernsey-spotted",
    "polka-dotted",
    "square",
    "round",
    "triangular",
    "cabernet",
    "sangria",
    "fuchsia",
    "wisteria",
    "lemon-lime",
    "strawberry-banana",
    "peppermint",
    "romantic",
    "incandescent",
];

/// Get a color description (hcolor equivalent)
///
/// Returns the given color, or a random hallucination color if
/// the player is hallucinating or no color is specified.
///
/// # Arguments
/// * `colorpref` - Optional preferred color; if None or hallucinating, a random color is returned
/// * `hallucinating` - Whether the player is hallucinating
/// * `rng` - Random number generator
pub fn hcolor(
    colorpref: Option<&str>,
    hallucinating: bool,
    rng: &mut crate::rng::GameRng,
) -> &'static str {
    if hallucinating || colorpref.is_none() {
        let idx = rng.rn2(HCOLORS.len() as u32) as usize;
        HCOLORS[idx]
    } else {
        // Return a static version of the color - caller provides color constants
        // Since we can't return the reference directly, use a match on common colors
        match colorpref.unwrap() {
            "black" => "black",
            "white" => "white",
            "red" => "red",
            "orange" => "orange",
            "yellow" => "yellow",
            "green" => "green",
            "blue" => "blue",
            "brown" => "brown",
            "cyan" => "cyan",
            "magenta" => "magenta",
            "gray" | "grey" => "gray",
            "amber" => "amber",
            "silver" => "silver",
            "golden" => "golden",
            "purple" => "purple",
            "violet" => "violet",
            "pink" => "pink",
            _ => "colorless",
        }
    }
}

/// Common color constants (NH_* from color.h)
pub mod colors {
    pub const NH_BLACK: &str = "black";
    pub const NH_WHITE: &str = "white";
    pub const NH_RED: &str = "red";
    pub const NH_ORANGE: &str = "orange";
    pub const NH_YELLOW: &str = "yellow";
    pub const NH_GREEN: &str = "green";
    pub const NH_BLUE: &str = "blue";
    pub const NH_BROWN: &str = "brown";
    pub const NH_CYAN: &str = "cyan";
    pub const NH_MAGENTA: &str = "magenta";
    pub const NH_GRAY: &str = "gray";
    pub const NH_AMBER: &str = "amber";
    pub const NH_SILVER: &str = "silver";
    pub const NH_GOLDEN: &str = "golden";
}

/// Get a "beautiful" or "handsome" or "ugly" description based on charisma (beautiful equivalent)
///
/// # Arguments
/// * `charisma` - The character's charisma score
/// * `is_female` - Whether the character is female
pub fn beautiful(charisma: i8, is_female: bool) -> &'static str {
    if charisma > 14 {
        if is_female { "beautiful" } else { "handsome" }
    } else {
        "ugly"
    }
}

/// Monster type categories for fire effect descriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireEffectCategory {
    /// Already on fire (fire elemental, flaming sphere, etc.)
    AlreadyOnFire,
    /// Water-based (water elemental, steam vortex)
    WaterBased,
    /// Ice-based or melts (ice vortex, glass golem)
    Melting,
    /// Heats up (stone/clay golems, elementals, vortices)
    HeatingUp,
    /// Standard creature
    Normal,
}

/// Get the fire effect description for a monster (on_fire equivalent)
///
/// Returns a phrase describing the effect of fire on the given creature type.
///
/// # Arguments
/// * `category` - The monster's category for fire effects
/// * `is_hug_attack` - Whether this is a bear hug style attack
pub fn on_fire(category: FireEffectCategory, is_hug_attack: bool) -> &'static str {
    match category {
        FireEffectCategory::AlreadyOnFire => "already on fire",
        FireEffectCategory::WaterBased => "boiling",
        FireEffectCategory::Melting => "melting",
        FireEffectCategory::HeatingUp => "heating up",
        FireEffectCategory::Normal => {
            if is_hug_attack {
                "being roasted"
            } else {
                "on fire"
            }
        }
    }
}

/// Determine the fire effect category for a monster type
///
/// # Arguments
/// * `monster_type` - The monster type index
pub fn fire_effect_category(monster_type: i16) -> FireEffectCategory {
    // PM_* constants - these should match the monster type indices in data/monsters.rs
    // For now, use approximate type ranges
    match monster_type {
        // Fire creatures
        mt if mt >= 300 && mt < 310 => FireEffectCategory::AlreadyOnFire,
        // Water creatures
        mt if mt >= 310 && mt < 320 => FireEffectCategory::WaterBased,
        // Ice/glass creatures
        mt if mt >= 320 && mt < 330 => FireEffectCategory::Melting,
        // Stone/earth creatures
        mt if mt >= 330 && mt < 350 => FireEffectCategory::HeatingUp,
        // Normal creatures
        _ => FireEffectCategory::Normal,
    }
}

/// Hallucination liquid strings
const HLIQUIDS: [&str; 33] = [
    "yoghurt",
    "oobleck",
    "clotted blood",
    "diluted water",
    "purified water",
    "instant coffee",
    "tea",
    "herbal infusion",
    "liquid rainbow",
    "creamy foam",
    "mulled wine",
    "bouillon",
    "nectar",
    "grog",
    "flubber",
    "ketchup",
    "slow light",
    "oil",
    "vinaigrette",
    "liquid crystal",
    "honey",
    "caramel sauce",
    "ink",
    "aqueous humour",
    "milk substitute",
    "fruit juice",
    "glowing lava",
    "gastric acid",
    "mineral water",
    "cough syrup",
    "quicksilver",
    "sweet vitriol",
    "grey goo",
];

/// Get a liquid description (hliquid equivalent)
///
/// Returns the given liquid, or a random hallucination liquid if
/// the player is hallucinating or no liquid is specified.
///
/// # Arguments
/// * `liquidpref` - Optional preferred liquid name
/// * `hallucinating` - Whether the player is hallucinating
/// * `rng` - Random number generator
pub fn hliquid(
    liquidpref: Option<&str>,
    hallucinating: bool,
    rng: &mut crate::rng::GameRng,
) -> &'static str {
    if hallucinating || liquidpref.is_none() {
        let idx = rng.rn2(HLIQUIDS.len() as u32) as usize;
        HLIQUIDS[idx]
    } else {
        match liquidpref.unwrap() {
            "water" => "water",
            "blood" => "blood",
            "oil" => "oil",
            "acid" => "acid",
            "poison" => "poison",
            "wine" => "wine",
            "potion" => "potion",
            _ => "liquid",
        }
    }
}

/// Get a random real color (rndcolor equivalent)
///
/// Returns a random color from the standard color palette, unless hallucinating.
///
/// # Arguments
/// * `hallucinating` - Whether the player is hallucinating
/// * `rng` - Random number generator
pub fn rndcolor(hallucinating: bool, rng: &mut crate::rng::GameRng) -> &'static str {
    const OBJ_COLORS: [&str; 16] = [
        "black",
        "red",
        "green",
        "brown",
        "blue",
        "magenta",
        "cyan",
        "gray",
        "orange",
        "bright green",
        "yellow",
        "bright blue",
        "bright magenta",
        "bright cyan",
        "white",
        "colorless",
    ];

    if hallucinating {
        hcolor(None, true, rng)
    } else {
        let idx = rng.rn2(OBJ_COLORS.len() as u32) as usize;
        OBJ_COLORS[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sgn() {
        assert_eq!(sgn(-5), -1);
        assert_eq!(sgn(0), 0);
        assert_eq!(sgn(5), 1);
    }

    #[test]
    fn test_isok() {
        assert!(isok(0, 0));
        assert!(isok(79, 20));
        assert!(!isok(-1, 0));
        assert!(!isok(0, -1));
        assert!(!isok(80, 0));
        assert!(!isok(0, 21));
    }

    #[test]
    fn test_dist2() {
        assert_eq!(dist2(0, 0, 3, 4), 25); // 3-4-5 triangle
        assert_eq!(dist2(0, 0, 0, 0), 0);
        assert_eq!(dist2(1, 1, 4, 5), 25);
    }

    #[test]
    fn test_distmin() {
        assert_eq!(distmin(0, 0, 3, 4), 4);
        assert_eq!(distmin(0, 0, 5, 3), 5);
        assert_eq!(distmin(0, 0, 0, 0), 0);
    }

    #[test]
    fn test_isqrt() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(4), 2);
        assert_eq!(isqrt(9), 3);
        assert_eq!(isqrt(10), 3);
        assert_eq!(isqrt(25), 5);
    }

    #[test]
    fn test_ordin() {
        assert_eq!(ordin(1), "st");
        assert_eq!(ordin(2), "nd");
        assert_eq!(ordin(3), "rd");
        assert_eq!(ordin(4), "th");
        assert_eq!(ordin(11), "th");
        assert_eq!(ordin(12), "th");
        assert_eq!(ordin(13), "th");
        assert_eq!(ordin(21), "st");
        assert_eq!(ordin(22), "nd");
        assert_eq!(ordin(23), "rd");
    }

    #[test]
    fn test_upstart() {
        assert_eq!(upstart("hello"), "Hello");
        assert_eq!(upstart("HELLO"), "HELLO");
        assert_eq!(upstart(""), "");
    }

    #[test]
    fn test_lined_up() {
        assert!(lined_up(0, 0, 5, 0)); // horizontal
        assert!(lined_up(0, 0, 0, 5)); // vertical
        assert!(lined_up(0, 0, 5, 5)); // diagonal
        assert!(!lined_up(0, 0, 3, 5)); // not lined up
    }

    #[test]
    fn test_body_part() {
        assert_eq!(body_part(BodyForm::Humanoid, BodyPart::Hand), "hand");
        assert_eq!(body_part(BodyForm::Humanoid, BodyPart::Foot), "foot");
        assert_eq!(body_part(BodyForm::Quadruped, BodyPart::Hand), "paw");
        assert_eq!(body_part(BodyForm::Serpentine, BodyPart::Hand), "coil");
        assert_eq!(body_part(BodyForm::Avian, BodyPart::Arm), "wing");
    }

    #[test]
    fn test_an() {
        assert_eq!(an("apple"), "an apple");
        assert_eq!(an("banana"), "a banana");
        assert_eq!(an("umbrella"), "an umbrella");
        assert_eq!(an("unicorn"), "a unicorn"); // sounds like "you"
        assert_eq!(an(""), "a");
    }

    #[test]
    fn test_makeplural() {
        assert_eq!(makeplural("cat"), "cats");
        assert_eq!(makeplural("box"), "boxes");
        assert_eq!(makeplural("fly"), "flies");
        assert_eq!(makeplural("key"), "keys"); // vowel + y
        assert_eq!(makeplural("knife"), "knives");
        assert_eq!(makeplural(""), "");
    }

    #[test]
    fn test_s_suffix() {
        assert_eq!(s_suffix("player"), "player's");
        assert_eq!(s_suffix("boss"), "boss'");
        assert_eq!(s_suffix("fox"), "fox'");
    }

    #[test]
    fn test_beautiful() {
        // High charisma female
        assert_eq!(beautiful(16, true), "beautiful");
        // High charisma male
        assert_eq!(beautiful(16, false), "handsome");
        // Low charisma
        assert_eq!(beautiful(10, true), "ugly");
        assert_eq!(beautiful(10, false), "ugly");
        // Edge case - exactly 14 is ugly
        assert_eq!(beautiful(14, true), "ugly");
        // 15 is beautiful/handsome
        assert_eq!(beautiful(15, true), "beautiful");
    }

    #[test]
    fn test_on_fire() {
        assert_eq!(
            on_fire(FireEffectCategory::AlreadyOnFire, false),
            "already on fire"
        );
        assert_eq!(on_fire(FireEffectCategory::WaterBased, false), "boiling");
        assert_eq!(on_fire(FireEffectCategory::Melting, false), "melting");
        assert_eq!(on_fire(FireEffectCategory::HeatingUp, false), "heating up");
        assert_eq!(on_fire(FireEffectCategory::Normal, false), "on fire");
        assert_eq!(on_fire(FireEffectCategory::Normal, true), "being roasted");
    }

    #[test]
    fn test_hcolor() {
        let mut rng = crate::rng::GameRng::new(42);

        // Non-hallucinating with specific color
        assert_eq!(hcolor(Some("black"), false, &mut rng), "black");
        assert_eq!(hcolor(Some("amber"), false, &mut rng), "amber");

        // Hallucinating or no preference returns a valid hallucination color
        let h_color = hcolor(None, true, &mut rng);
        assert!(HCOLORS.contains(&h_color));
    }

    #[test]
    fn test_hliquid() {
        let mut rng = crate::rng::GameRng::new(42);

        // Non-hallucinating with specific liquid
        assert_eq!(hliquid(Some("water"), false, &mut rng), "water");
        assert_eq!(hliquid(Some("blood"), false, &mut rng), "blood");

        // Hallucinating returns a valid hallucination liquid
        let h_liquid = hliquid(None, true, &mut rng);
        assert!(HLIQUIDS.contains(&h_liquid));
    }

    #[test]
    fn test_rndcolor() {
        let mut rng = crate::rng::GameRng::new(42);

        // Non-hallucinating returns a valid color
        let color = rndcolor(false, &mut rng);
        let valid_colors = [
            "black",
            "red",
            "green",
            "brown",
            "blue",
            "magenta",
            "cyan",
            "gray",
            "orange",
            "bright green",
            "yellow",
            "bright blue",
            "bright magenta",
            "bright cyan",
            "white",
            "colorless",
        ];
        assert!(valid_colors.contains(&color));

        // Hallucinating returns a hallucination color
        let h_color = rndcolor(true, &mut rng);
        assert!(HCOLORS.contains(&h_color));
    }
}
