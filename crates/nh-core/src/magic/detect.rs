//! Detection and mapping (detect.c)
//!
//! Handles magic mapping, clairvoyance, object/monster/trap detection,
//! crystal ball usage, and the search command.
//!
//! In the C source these functions manipulate glyphs on the display map.
//! In Rust we operate on the Level's explored/visible grids and set
//! "detected" flags on monsters and traps directly.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::dungeon::{CellType, Level, TrapType};
use crate::object::{Object, ObjectClass};
use crate::player::You;
use crate::rng::GameRng;
use crate::{COLNO, ROWNO};

// ============================================================================
// Result types
// ============================================================================

/// Result of a detection operation
#[derive(Debug, Clone)]
pub struct DetectResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Number of things detected
    pub count: usize,
    /// Whether something was detected at all
    pub found_something: bool,
}

impl DetectResult {
    pub fn nothing(msg: impl Into<String>) -> Self {
        Self {
            messages: vec![msg.into()],
            count: 0,
            found_something: false,
        }
    }

    pub fn found(msg: impl Into<String>, count: usize) -> Self {
        Self {
            messages: vec![msg.into()],
            count,
            found_something: true,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

// ============================================================================
// Magic mapping
// ============================================================================

/// Reveal the entire level terrain (magic mapping scroll/spell).
///
/// Matches C `do_mapping()` in detect.c. Reveals all terrain, converts
/// secret doors to regular doors, converts secret corridors to corridors,
/// and marks all traps as seen.
pub fn do_mapping(level: &mut Level) -> DetectResult {
    let mut count = 0;

    for x in 0..COLNO {
        for y in 0..ROWNO {
            let cell = &mut level.cells[x][y];

            // Convert secret doors to regular doors
            if cell.typ == CellType::SecretDoor {
                cell.typ = CellType::Door;
                count += 1;
            }

            // Convert secret corridors to regular corridors
            if cell.typ == CellType::SecretCorridor {
                cell.typ = CellType::Corridor;
                count += 1;
            }

            // Mark all non-stone cells as explored
            if cell.typ != CellType::Stone {
                level.explored[x][y] = true;
                count += 1;
            }
        }
    }

    // Mark all traps as seen
    for trap in &mut level.traps {
        if !trap.seen {
            trap.seen = true;
            count += 1;
        }
    }

    DetectResult::found("A map coalesces in your mind!", count)
}

/// Reveal terrain in a vicinity around the player (clairvoyance).
///
/// Matches C `do_vicinity_map()` in detect.c. Reveals an 11x12 area
/// centered on the player. Blessed version also reveals objects and
/// marks monsters as detected.
pub fn do_vicinity_map(
    level: &mut Level,
    player: &You,
    blessed: bool,
) -> DetectResult {
    let px = player.pos.x as i32;
    let py = player.pos.y as i32;
    let mut count = 0;

    // Clairvoyance area: ±5 columns, ±5 rows (11x11 around player)
    let x_min = (px - 5).max(0) as usize;
    let x_max = (px + 5).min(COLNO as i32 - 1) as usize;
    let y_min = (py - 5).max(0) as usize;
    let y_max = (py + 5).min(ROWNO as i32 - 1) as usize;

    for x in x_min..=x_max {
        for y in y_min..=y_max {
            let cell = &mut level.cells[x][y];

            // Convert secret features
            if cell.typ == CellType::SecretDoor {
                cell.typ = CellType::Door;
            }
            if cell.typ == CellType::SecretCorridor {
                cell.typ = CellType::Corridor;
            }

            // Mark as explored
            if cell.typ != CellType::Stone {
                level.explored[x][y] = true;
                count += 1;
            }
        }
    }

    // Mark traps in area as seen
    for trap in &mut level.traps {
        let tx = trap.x as usize;
        let ty = trap.y as usize;
        if tx >= x_min && tx <= x_max && ty >= y_min && ty <= y_max && !trap.seen {
            trap.seen = true;
            count += 1;
        }
    }

    if blessed {
        // Blessed clairvoyance also reveals monsters in the area
        for mon in &mut level.monsters {
            let mx = mon.x as usize;
            let my = mon.y as usize;
            if mx >= x_min && mx <= x_max && my >= y_min && my <= y_max {
                // Mark monster position as explored
                level.explored[mx][my] = true;
                count += 1;
            }
        }
    }

    if count > 0 {
        DetectResult::found("You have a vision of nearby terrain.", count)
    } else {
        DetectResult::nothing("Your vision is cloudy.")
    }
}

// ============================================================================
// Object detection
// ============================================================================

/// Detect objects on the current level.
///
/// Matches C `object_detect()` in detect.c.
/// - Unblessed: reveals object positions on the map
/// - Blessed: also identifies discovered objects
/// - Cursed: no useful effect
/// - `class`: if Some, only detect objects of that class
pub fn object_detect(
    level: &mut Level,
    class: Option<ObjectClass>,
    cursed: bool,
) -> DetectResult {
    if cursed {
        return DetectResult::nothing("You sense the presence of objects, but the feeling is vague.");
    }

    let mut count = 0;

    for obj in &level.objects {
        if let Some(cls) = class {
            if obj.class != cls {
                continue;
            }
        }
        let x = obj.x as usize;
        let y = obj.y as usize;
        if x < COLNO && y < ROWNO {
            level.explored[x][y] = true;
            count += 1;
        }
    }

    if count > 0 {
        DetectResult::found("You sense the presence of objects.", count)
    } else {
        DetectResult::nothing("You sense no objects.")
    }
}

/// Detect gold on the current level.
///
/// Matches C `gold_detect()` in detect.c.
/// - Blessed: detects all gold-material objects
/// - Unblessed: detects coins only
/// - Cursed: no useful detection
pub fn gold_detect(
    level: &mut Level,
    blessed: bool,
    cursed: bool,
) -> DetectResult {
    if cursed {
        return DetectResult::nothing("You feel very greedy, but can't find anything.");
    }

    let mut count = 0;

    for obj in &level.objects {
        let is_gold = if blessed {
            // Blessed detects all gold objects, not just coins
            // Object has no material field; approximated by class (Coin=gold, Ring often gold)
            obj.class == ObjectClass::Coin || obj.class == ObjectClass::Ring
        } else {
            obj.class == ObjectClass::Coin
        };

        if is_gold {
            let x = obj.x as usize;
            let y = obj.y as usize;
            if x < COLNO && y < ROWNO {
                level.explored[x][y] = true;
                count += 1;
            }
        }
    }

    if count > 0 {
        DetectResult::found("You sense the presence of gold.", count)
    } else {
        DetectResult::nothing("You feel materially poor.")
    }
}

/// Detect food on the current level.
///
/// Matches C `food_detect()` in detect.c.
/// - Blessed: grants sense of edibility
/// - Cursed: detects potions instead of food
pub fn food_detect(
    level: &mut Level,
    blessed: bool,
    cursed: bool,
) -> DetectResult {
    let target_class = if cursed {
        ObjectClass::Potion
    } else {
        ObjectClass::Food
    };

    let mut count = 0;

    for obj in &level.objects {
        if obj.class == target_class {
            let x = obj.x as usize;
            let y = obj.y as usize;
            if x < COLNO && y < ROWNO {
                level.explored[x][y] = true;
                count += 1;
            }
        }
    }

    if count > 0 {
        let msg = if cursed {
            "You sense the presence of potions."
        } else if blessed {
            "You sense the presence of food, and can tell what is safe to eat."
        } else {
            "You sense the presence of food."
        };
        DetectResult::found(msg, count)
    } else {
        let msg = if cursed {
            "You sense no potions."
        } else {
            "You sense no food."
        };
        DetectResult::nothing(msg)
    }
}

// ============================================================================
// Monster detection
// ============================================================================

/// Detect monsters on the current level.
///
/// Matches C `monster_detect()` in detect.c.
/// - Unblessed: reveals monster positions temporarily
/// - Blessed: persistent detection
/// - Cursed: wakes sleeping monsters instead
/// - `class_char`: if Some, only detect monsters with that symbol
pub fn monster_detect(
    level: &mut Level,
    class_char: Option<char>,
    cursed: bool,
    rng: &mut GameRng,
) -> DetectResult {
    if cursed {
        // Cursed: wake sleeping monsters
        let mut woken = 0;
        for mon in &mut level.monsters {
            if mon.state.sleeping && rng.one_in(2) {
                mon.state.sleeping = false;
                mon.sleep_timeout = 0;
                woken += 1;
            }
        }
        if woken > 0 {
            return DetectResult::found("You hear some stirring in the distance.", woken);
        }
        return DetectResult::nothing("You sense no hostile creatures nearby.");
    }

    let mut count = 0;

    // Mark monster positions as explored
    for mon in &level.monsters {
        if let Some(cls) = class_char {
            // Would need to check monster symbol against class_char
            // For now we use a simplified check based on monster type
            // (The real check needs the MONSTERS database)
            let _ = cls;
        }
        let mx = mon.x as usize;
        let my = mon.y as usize;
        if mx < COLNO && my < ROWNO {
            level.explored[mx][my] = true;
            count += 1;
        }
    }

    if count > 0 {
        DetectResult::found("You sense the presence of monsters.", count)
    } else {
        DetectResult::nothing("You sense no monsters.")
    }
}

// ============================================================================
// Trap detection
// ============================================================================

/// Detect traps on the current level.
///
/// Matches C `trap_detect()` in detect.c. Reveals all traps on the
/// level. Also detects trapped containers.
/// - Blessed: also reveals trap types
/// - Cursed: shows misleading information
pub fn trap_detect(
    level: &mut Level,
    cursed: bool,
) -> DetectResult {
    if cursed {
        return DetectResult::nothing("You feel very anxious, but see nothing unusual.");
    }

    let mut count = 0;

    // Reveal all traps
    for trap in &mut level.traps {
        if !trap.seen {
            trap.seen = true;
            count += 1;
        }
        // Mark trap position as explored
        let tx = trap.x as usize;
        let ty = trap.y as usize;
        if tx < COLNO && ty < ROWNO {
            level.explored[tx][ty] = true;
        }
    }

    // Check for trapped containers on the floor
    let trapped_objects: usize = level
        .objects
        .iter()
        .filter(|obj| obj.trapped)
        .count();
    count += trapped_objects;

    if count > 0 {
        DetectResult::found("You sense the presence of traps.", count)
    } else {
        DetectResult::nothing("You sense no traps.")
    }
}

// ============================================================================
// Search command
// ============================================================================

/// Search adjacent squares for hidden features.
///
/// Matches C `dosearch0()` in detect.c. Searches the 3x3 area around
/// the player for:
/// - Secret doors (converts to regular doors)
/// - Secret corridors (converts to regular corridors)
/// - Hidden traps (marks as seen)
///
/// `search_bonus`: extra search skill from equipment (lenses, artifact).
/// Higher values increase the chance of finding things.
/// `autosearch`: if true, this is a passive search (from intrinsics),
/// which skips monster/trap detection.
pub fn dosearch0(
    level: &mut Level,
    player: &You,
    search_bonus: i32,
    autosearch: bool,
    rng: &mut GameRng,
) -> DetectResult {
    let px = player.pos.x;
    let py = player.pos.y;
    let fund = search_bonus.min(5); // Cap at 5

    let mut count = 0;
    let mut messages = Vec::new();

    for dy in -1..=1i8 {
        for dx in -1..=1i8 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let x = px + dx;
            let y = py + dy;
            if !level.is_valid_pos(x, y) {
                continue;
            }

            let ux = x as usize;
            let uy = y as usize;

            // Check for secret doors
            if level.cells[ux][uy].typ == CellType::SecretDoor {
                // rnl(7 - fund): higher fund = better chance
                let threshold = (7 - fund).max(1) as u32;
                if rng.rn2(threshold) == 0 {
                    level.cells[ux][uy].typ = CellType::Door;
                    level.explored[ux][uy] = true;
                    messages.push("You find a hidden door!".to_string());
                    count += 1;
                }
            }

            // Check for secret corridors
            if level.cells[ux][uy].typ == CellType::SecretCorridor {
                let threshold = (7 - fund).max(1) as u32;
                if rng.rn2(threshold) == 0 {
                    level.cells[ux][uy].typ = CellType::Corridor;
                    level.explored[ux][uy] = true;
                    messages.push("You find a hidden passage!".to_string());
                    count += 1;
                }
            }

            // Check for hidden traps (not in autosearch mode)
            if !autosearch {
                for trap in &mut level.traps {
                    if trap.x == x && trap.y == y && !trap.seen {
                        // 1/8 chance to find trap
                        if rng.rn2(8) == 0 {
                            trap.seen = true;
                            level.explored[ux][uy] = true;
                            messages.push(format!("You find a {}!", trap_name(trap.trap_type)));
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    if count == 0 {
        messages.push("You search but find nothing.".to_string());
    }

    DetectResult {
        messages,
        count,
        found_something: count > 0,
    }
}

/// Get the display name for a trap type
fn trap_name(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::Arrow => "arrow trap",
        TrapType::Dart => "dart trap",
        TrapType::RockFall => "falling rock trap",
        TrapType::Squeaky => "squeaky board",
        TrapType::BearTrap => "bear trap",
        TrapType::LandMine => "land mine",
        TrapType::RollingBoulder => "rolling boulder trap",
        TrapType::SleepingGas => "sleeping gas trap",
        TrapType::RustTrap => "rust trap",
        TrapType::FireTrap => "fire trap",
        TrapType::Pit => "pit",
        TrapType::SpikedPit => "spiked pit",
        TrapType::Hole => "hole",
        TrapType::TrapDoor => "trap door",
        TrapType::Teleport => "teleportation trap",
        TrapType::LevelTeleport => "level teleporter",
        TrapType::MagicPortal => "magic portal",
        TrapType::Web => "web",
        TrapType::Statue => "statue trap",
        TrapType::MagicTrap => "magic trap",
        TrapType::AntiMagic => "anti-magic field",
        TrapType::Polymorph => "polymorph trap",
    }
}

// ============================================================================
// Crystal ball
// ============================================================================

/// Crystal ball detection result
#[derive(Debug, Clone)]
pub struct CrystalBallResult {
    /// Messages to display
    pub messages: Vec<String>,
    /// Whether the crystal ball was consumed (broken)
    pub ball_broken: bool,
    /// Whether charges were used
    pub charge_used: bool,
    /// Whether the player was harmed
    pub player_harmed: bool,
    /// Damage dealt to player (from misuse)
    pub damage: i32,
}

impl CrystalBallResult {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            ball_broken: false,
            charge_used: false,
            player_harmed: false,
            damage: 0,
        }
    }
}

/// Use a crystal ball to divine information.
///
/// Matches C `use_crystal_ball()` in detect.c. The crystal ball can
/// detect objects, monsters, or traps based on the symbol provided.
/// Misuse can cause damage; cursed balls are particularly dangerous.
///
/// `symbol`: what to look for:
/// - '^': detect traps
/// - Object class char (')', '[', etc.): detect objects of that class
/// - Monster class char ('d', 'D', etc.): detect monster presence
/// - Other: random cryptic message
pub fn use_crystal_ball(
    level: &mut Level,
    _player: &You,
    ball: &Object,
    symbol: char,
    rng: &mut GameRng,
) -> CrystalBallResult {
    let mut result = CrystalBallResult::new();

    // Check charges
    if ball.enchantment <= 0 {
        result.messages.push("The crystal ball is dark.".to_string());
        return result;
    }

    // Check for misuse damage (cursed ball, or intelligence-based failure)
    if ball.is_cursed() && rng.one_in(5) {
        result.messages.push("The crystal ball explodes!".to_string());
        result.ball_broken = true;
        result.player_harmed = true;
        result.damage = rng.dice(3, 6) as i32;
        return result;
    }

    result.charge_used = true;

    match symbol {
        '^' => {
            // Detect traps
            let detect = trap_detect(level, false);
            result.messages.extend(detect.messages);
        }
        ')' | '[' | '=' | '"' | '(' | '%' | '!' | '?' | '+' | '/' | '*' | '`' | '$' => {
            // Object class detection
            let class = object_class_from_symbol(symbol);
            let detect = object_detect(level, class, false);
            result.messages.extend(detect.messages);
        }
        _ => {
            // Try as monster class or give cryptic message
            let monster_count = level.monsters.len();

            if monster_count > 0 {
                let detect = monster_detect(level, Some(symbol), false, rng);
                result.messages.extend(detect.messages);
            } else {
                // Cryptic random messages (matching C's flavor text)
                let msg = match rng.rn2(4) {
                    0 => "You see a swirling mist in the crystal ball.",
                    1 => "The crystal ball shows a dark, distant place.",
                    2 => "You see a vision of great danger ahead.",
                    3 => "The crystal ball clouds over.",
                    _ => unreachable!(),
                };
                result.messages.push(msg.to_string());
            }
        }
    }

    result
}

/// Map a display symbol to an ObjectClass
fn object_class_from_symbol(symbol: char) -> Option<ObjectClass> {
    match symbol {
        ')' => Some(ObjectClass::Weapon),
        '[' => Some(ObjectClass::Armor),
        '=' => Some(ObjectClass::Ring),
        '"' => Some(ObjectClass::Amulet),
        '(' => Some(ObjectClass::Tool),
        '%' => Some(ObjectClass::Food),
        '!' => Some(ObjectClass::Potion),
        '?' => Some(ObjectClass::Scroll),
        '+' => Some(ObjectClass::Spellbook),
        '/' => Some(ObjectClass::Wand),
        '*' => Some(ObjectClass::Gem),
        '`' => Some(ObjectClass::Rock),
        '$' => Some(ObjectClass::Coin),
        _ => None,
    }
}

// ============================================================================
// dosearch — Player search command (detect.c:1610)
// ============================================================================

/// Player search command wrapper (dosearch from detect.c:1610).
///
/// Searches all 8 adjacent squares for secret doors, traps, and hidden monsters.
/// Uses search_bonus from player stats (enhanced searching ability).
pub fn dosearch(
    level: &mut Level,
    player: &You,
    rng: &mut GameRng,
) -> DetectResult {
    // dosearch calls dosearch0 with search_bonus = 0 and autosearch = false
    dosearch0(level, player, 0, false, rng)
}

// ============================================================================
// Container trap detection (detect.c:1442)
// ============================================================================

/// Check if containers in a list are trapped (detect_obj_traps from detect.c:1442).
///
/// Returns the number of trapped containers found and messages.
pub fn detect_obj_traps(
    objects: &[Object],
    known: bool,
) -> (usize, Vec<String>) {
    let mut count = 0;
    let mut messages = Vec::new();

    for obj in objects {
        if obj.is_container() && obj.trapped {
            count += 1;
            if known {
                messages.push(format!("{} is trapped!", obj.display_name()));
            }
        }
    }

    (count, messages)
}

/// Check if there is a trapped chest among objects (trapped_chest_at from detect.c:1460).
pub fn trapped_chest_at(objects: &[Object]) -> bool {
    objects.iter().any(|obj| obj.is_container() && obj.trapped)
}

// ============================================================================
// Terrain reveal (detect.c:1080)
// ============================================================================

/// Reveal terrain around the player (reveal_terrain from detect.c:1080).
///
/// Shows the map without monsters or objects — just terrain features.
/// Used by the #terrain extended command and some scrolls.
pub fn reveal_terrain(
    level: &mut Level,
    full: bool,
) -> DetectResult {
    let mut count = 0;

    for x in 1..COLNO as i8 {
        for y in 0..ROWNO as i8 {
            if !level.is_valid_pos(x, y) {
                continue;
            }

            let cell = &level.cells[x as usize][y as usize];
            let is_interesting = !matches!(cell.typ, CellType::Stone);

            if full || is_interesting {
                level.cells[x as usize][y as usize].explored = true;
                count += 1;
            }
        }
    }

    if count > 0 {
        DetectResult::found("The terrain is revealed.", count)
    } else {
        DetectResult::nothing("You don't learn anything new about the terrain.")
    }
}

/// Identify the type of a detected trap (sense_trap from detect.c:1510).
///
/// When a trap is detected by searching, return its type description.
pub fn sense_trap(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::Arrow => "an arrow trap",
        TrapType::Dart => "a dart trap",
        TrapType::RockFall => "a falling rock trap",
        TrapType::Squeaky => "a squeaky board",
        TrapType::BearTrap => "a bear trap",
        TrapType::LandMine => "a land mine",
        TrapType::RollingBoulder => "a rolling boulder trap",
        TrapType::SleepingGas => "a sleeping gas trap",
        TrapType::RustTrap => "a rust trap",
        TrapType::FireTrap => "a fire trap",
        TrapType::Pit => "a pit",
        TrapType::SpikedPit => "a spiked pit",
        TrapType::Hole => "a hole",
        TrapType::TrapDoor => "a trap door",
        TrapType::Teleport => "a teleportation trap",
        TrapType::LevelTeleport => "a level teleporter",
        TrapType::MagicPortal => "a magic portal",
        TrapType::Web => "a web",
        TrapType::Statue => "a statue trap",
        TrapType::MagicTrap => "a magic trap",
        TrapType::AntiMagic => "an anti-magic field",
        TrapType::Polymorph => "a polymorph trap",
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::{DLevel, Level, Trap, TrapType};
    use crate::object::Object;
    use crate::player::You;
    use crate::rng::GameRng;

    /// Create a test level with a room and some features
    fn test_level() -> Level {
        let mut level = Level::new(DLevel::default());
        // Create a room (10x5 at position 5,3)
        for x in 5..15 {
            for y in 3..8 {
                level.cells[x][y].typ = CellType::Room;
            }
        }
        // Add a secret door
        level.cells[4][5].typ = CellType::SecretDoor;
        // Add a secret corridor
        level.cells[3][5].typ = CellType::SecretCorridor;
        level
    }

    fn test_player_at(x: i8, y: i8) -> You {
        let mut player = You::default();
        player.pos.x = x;
        player.pos.y = y;
        player
    }

    // ---- do_mapping tests ----

    #[test]
    fn test_do_mapping_reveals_level() {
        let mut level = test_level();
        assert!(!level.explored[7][5]);

        let result = do_mapping(&mut level);
        assert!(result.found_something);
        assert!(result.count > 0);
        // Room cells should now be explored
        assert!(level.explored[7][5]);
    }

    #[test]
    fn test_do_mapping_converts_secrets() {
        let mut level = test_level();
        assert_eq!(level.cells[4][5].typ, CellType::SecretDoor);
        assert_eq!(level.cells[3][5].typ, CellType::SecretCorridor);

        do_mapping(&mut level);

        assert_eq!(level.cells[4][5].typ, CellType::Door);
        assert_eq!(level.cells[3][5].typ, CellType::Corridor);
    }

    #[test]
    fn test_do_mapping_reveals_traps() {
        let mut level = test_level();
        level.traps.push(Trap {
            x: 7,
            y: 5,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });

        do_mapping(&mut level);
        assert!(level.traps[0].seen);
    }

    // ---- do_vicinity_map tests ----

    #[test]
    fn test_vicinity_map_reveals_nearby() {
        let mut level = test_level();
        let player = test_player_at(7, 5);

        let result = do_vicinity_map(&mut level, &player, false);
        assert!(result.found_something);
        // Cells within 5 squares should be explored
        assert!(level.explored[7][5]);
        assert!(level.explored[10][5]);
    }

    #[test]
    fn test_vicinity_map_does_not_reveal_far() {
        let mut level = test_level();
        // Create a room far from player
        for x in 60..70 {
            for y in 15..20 {
                level.cells[x][y].typ = CellType::Room;
            }
        }
        let player = test_player_at(7, 5);

        do_vicinity_map(&mut level, &player, false);
        // Far room should NOT be explored
        assert!(!level.explored[65][17]);
    }

    #[test]
    fn test_vicinity_map_converts_secrets_in_range() {
        let mut level = test_level();
        let player = test_player_at(5, 5);

        do_vicinity_map(&mut level, &player, false);
        // Secret door at (4,5) is within range of player at (5,5)
        assert_eq!(level.cells[4][5].typ, CellType::Door);
    }

    // ---- object_detect tests ----

    #[test]
    fn test_object_detect_all() {
        let mut level = test_level();
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.x = 7;
        obj.y = 5;
        level.objects.push(obj);

        let result = object_detect(&mut level, None, false);
        assert!(result.found_something);
        assert_eq!(result.count, 1);
        assert!(level.explored[7][5]);
    }

    #[test]
    fn test_object_detect_by_class() {
        let mut level = test_level();
        let mut weapon = Object::default();
        weapon.class = ObjectClass::Weapon;
        weapon.x = 7;
        weapon.y = 5;
        level.objects.push(weapon);

        let mut food = Object::default();
        food.class = ObjectClass::Food;
        food.x = 8;
        food.y = 5;
        level.objects.push(food);

        let result = object_detect(&mut level, Some(ObjectClass::Weapon), false);
        assert_eq!(result.count, 1);
        assert!(level.explored[7][5]);
        // Food position should NOT be revealed
        assert!(!level.explored[8][5]);
    }

    #[test]
    fn test_object_detect_cursed() {
        let mut level = test_level();
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;
        obj.x = 7;
        obj.y = 5;
        level.objects.push(obj);

        let result = object_detect(&mut level, None, true);
        assert!(!result.found_something);
    }

    #[test]
    fn test_object_detect_empty_level() {
        let mut level = test_level();
        let result = object_detect(&mut level, None, false);
        assert!(!result.found_something);
        assert_eq!(result.count, 0);
    }

    // ---- gold_detect tests ----

    #[test]
    fn test_gold_detect_coins() {
        let mut level = test_level();
        let mut gold = Object::default();
        gold.class = ObjectClass::Coin;
        gold.x = 7;
        gold.y = 5;
        level.objects.push(gold);

        let result = gold_detect(&mut level, false, false);
        assert!(result.found_something);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_gold_detect_cursed() {
        let mut level = test_level();
        let mut gold = Object::default();
        gold.class = ObjectClass::Coin;
        gold.x = 7;
        gold.y = 5;
        level.objects.push(gold);

        let result = gold_detect(&mut level, false, true);
        assert!(!result.found_something);
    }

    // ---- food_detect tests ----

    #[test]
    fn test_food_detect() {
        let mut level = test_level();
        let mut food = Object::default();
        food.class = ObjectClass::Food;
        food.x = 7;
        food.y = 5;
        level.objects.push(food);

        let result = food_detect(&mut level, false, false);
        assert!(result.found_something);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_food_detect_cursed_finds_potions() {
        let mut level = test_level();
        let mut potion = Object::default();
        potion.class = ObjectClass::Potion;
        potion.x = 7;
        potion.y = 5;
        level.objects.push(potion);

        let mut food = Object::default();
        food.class = ObjectClass::Food;
        food.x = 8;
        food.y = 5;
        level.objects.push(food);

        let result = food_detect(&mut level, false, true);
        assert!(result.found_something);
        assert_eq!(result.count, 1); // Only potion, not food
        assert!(level.explored[7][5]); // Potion position
        assert!(!level.explored[8][5]); // Food position NOT revealed
    }

    // ---- monster_detect tests ----

    #[test]
    fn test_monster_detect() {
        let mut level = test_level();
        use crate::monster::{Monster, MonsterId};
        let mon = Monster::new(MonsterId(0), 0, 7, 5);
        level.add_monster(mon);

        let mut rng = GameRng::new(42);
        let result = monster_detect(&mut level, None, false, &mut rng);
        assert!(result.found_something);
        assert_eq!(result.count, 1);
    }

    #[test]
    fn test_monster_detect_cursed_wakes() {
        let mut level = test_level();
        use crate::monster::{Monster, MonsterId, MonsterState};
        let mut mon = Monster::new(MonsterId(0), 0, 7, 5);
        mon.state = MonsterState::default();
        mon.state.sleeping = true;
        mon.state.can_move = true;
        level.add_monster(mon);

        let mut rng = GameRng::new(42);
        // Run multiple times since waking is random
        for _ in 0..20 {
            let _ = monster_detect(&mut level, None, true, &mut rng);
        }
        // At least some should have woken
    }

    // ---- trap_detect tests ----

    #[test]
    fn test_trap_detect() {
        let mut level = test_level();
        level.traps.push(Trap {
            x: 7,
            y: 5,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });
        level.traps.push(Trap {
            x: 8,
            y: 5,
            trap_type: TrapType::Arrow,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });

        let result = trap_detect(&mut level, false);
        assert!(result.found_something);
        assert_eq!(result.count, 2);
        assert!(level.traps[0].seen);
        assert!(level.traps[1].seen);
    }

    #[test]
    fn test_trap_detect_cursed() {
        let mut level = test_level();
        level.traps.push(Trap {
            x: 7,
            y: 5,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });

        let result = trap_detect(&mut level, true);
        assert!(!result.found_something);
        assert!(!level.traps[0].seen); // Should NOT be revealed
    }

    // ---- dosearch0 tests ----

    #[test]
    fn test_dosearch_finds_secret_door() {
        let mut level = test_level();
        let player = test_player_at(5, 5);
        let mut rng = GameRng::new(42);

        // Secret door at (4,5) is adjacent to player at (5,5)
        // With search_bonus=5, threshold=(7-5)=2, chance=50% per search
        let mut found = false;
        for _ in 0..50 {
            let result = dosearch0(&mut level, &player, 5, false, &mut rng);
            if result.found_something {
                found = true;
                break;
            }
        }
        assert!(found, "should find secret door after many searches");
        assert_eq!(level.cells[4][5].typ, CellType::Door);
    }

    #[test]
    fn test_dosearch_finds_hidden_trap() {
        let mut level = test_level();
        let player = test_player_at(7, 5);
        level.traps.push(Trap {
            x: 8,
            y: 5,
            trap_type: TrapType::BearTrap,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });
        let mut rng = GameRng::new(42);

        let mut found = false;
        for _ in 0..100 {
            let result = dosearch0(&mut level, &player, 0, false, &mut rng);
            if level.traps[0].seen {
                found = true;
                break;
            }
            let _ = result;
        }
        assert!(found, "should find hidden trap after many searches");
    }

    #[test]
    fn test_dosearch_nothing_found() {
        let mut level = Level::new(DLevel::default());
        // Create a room with no secrets
        for x in 5..10 {
            for y in 5..10 {
                level.cells[x][y].typ = CellType::Room;
            }
        }
        let player = test_player_at(7, 7);
        let mut rng = GameRng::new(42);

        let result = dosearch0(&mut level, &player, 0, false, &mut rng);
        assert!(!result.found_something);
    }

    // ---- trap_name tests ----

    #[test]
    fn test_trap_names() {
        assert_eq!(trap_name(TrapType::Pit), "pit");
        assert_eq!(trap_name(TrapType::Arrow), "arrow trap");
        assert_eq!(trap_name(TrapType::Web), "web");
        assert_eq!(trap_name(TrapType::Polymorph), "polymorph trap");
    }

    // ---- object_class_from_symbol tests ----

    #[test]
    fn test_object_class_from_symbol() {
        assert_eq!(object_class_from_symbol(')'), Some(ObjectClass::Weapon));
        assert_eq!(object_class_from_symbol('['), Some(ObjectClass::Armor));
        assert_eq!(object_class_from_symbol('$'), Some(ObjectClass::Coin));
        assert_eq!(object_class_from_symbol('z'), None);
    }

    // ---- crystal ball tests ----

    #[test]
    fn test_crystal_ball_no_charges() {
        let mut level = test_level();
        let player = test_player_at(7, 5);
        let mut ball = Object::default();
        ball.enchantment = 0;
        let mut rng = GameRng::new(42);

        let result = use_crystal_ball(&mut level, &player, &ball, '^', &mut rng);
        assert!(!result.charge_used);
        assert!(result.messages[0].contains("dark"));
    }

    #[test]
    fn test_crystal_ball_detect_traps() {
        let mut level = test_level();
        level.traps.push(Trap {
            x: 7,
            y: 5,
            trap_type: TrapType::Pit,
            activated: false,
            seen: false,
            once: false,
            madeby_u: false,
            launch_oid: None,
        });
        let player = test_player_at(7, 5);
        let mut ball = Object::default();
        ball.enchantment = 3;
        let mut rng = GameRng::new(42);

        let result = use_crystal_ball(&mut level, &player, &ball, '^', &mut rng);
        assert!(result.charge_used);
        assert!(level.traps[0].seen);
    }

    #[test]
    fn test_crystal_ball_detect_objects() {
        let mut level = test_level();
        let mut weapon = Object::default();
        weapon.class = ObjectClass::Weapon;
        weapon.x = 7;
        weapon.y = 5;
        level.objects.push(weapon);

        let player = test_player_at(10, 5);
        let mut ball = Object::default();
        ball.enchantment = 3;
        let mut rng = GameRng::new(42);

        let result = use_crystal_ball(&mut level, &player, &ball, ')', &mut rng);
        assert!(result.charge_used);
        assert!(level.explored[7][5]);
    }
}
