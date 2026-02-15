//! Object creation (mkobj.c)
//!
//! Functions for creating and initializing objects.

#[cfg(not(feature = "std"))]
use crate::compat::*;

use crate::object::{BucStatus, ObjClassDef, Object, ObjectClass, ObjectId, ObjectLocation};
use crate::rng::GameRng;
use crate::world::{TimedEvent, TimedEventType, TimeoutManager};

// ============================================================================
// Object type constants (indices into OBJECTS array)
// These must match the nh_data::objects::ObjectType enum values
// ============================================================================

/// Corpse object type index
pub const CORPSE: i16 = 297; // Corpse in Food section
/// Statue object type index
pub const STATUE: i16 = 466; // Statue in Rocks section
/// Figurine object type index
pub const FIGURINE: i16 = 262; // Figurine in Tools section
/// Egg object type index
pub const EGG: i16 = 298; // Egg in Food section

// ============================================================================
// Corpse timing constants
// ============================================================================

/// Age when corpses become tainted/dangerous
const TAINT_AGE: i64 = 50;
/// Default age when corpses rot away completely
const ROT_AGE: i64 = 250;
/// Chance for troll to revive (1 in N per turn)
const TROLL_REVIVE_CHANCE: u32 = 37;

// ============================================================================
// Monster type constants for special corpses
// ============================================================================

/// Monster index for lizard (corpse doesn't rot)
pub const PM_LIZARD: i16 = 224;
/// Monster index for lichen (corpse doesn't rot)
pub const PM_LICHEN: i16 = 94;
/// Corpse creation flags
#[derive(Debug, Clone, Copy, Default)]
pub struct CorpstatFlags {
    /// Initialize the object normally
    pub init: bool,
    /// Don't start corpse timer
    pub no_timeout: bool,
}

impl CorpstatFlags {
    pub const INIT: Self = Self {
        init: true,
        no_timeout: false,
    };
    pub const NONE: Self = Self {
        init: false,
        no_timeout: false,
    };
}

// ============================================================================
// Class base indices - computed from OBJECTS array
// ============================================================================

/// Computed base indices for each object class in the OBJECTS array.
/// bases[class] = first index of that class in OBJECTS.
#[derive(Debug, Clone)]
pub struct ClassBases {
    bases: [usize; 18], // One for each ObjectClass variant
}

impl ClassBases {
    /// Compute class bases from the OBJECTS array.
    /// Returns bases where bases[class as usize] is the first index of that class.
    pub fn compute(objects: &[ObjClassDef]) -> Self {
        let mut bases = [0usize; 18];
        let mut current_class = ObjectClass::Random;

        for (i, obj) in objects.iter().enumerate() {
            if obj.class != current_class {
                bases[obj.class as usize] = i;
                current_class = obj.class;
            }
        }

        Self { bases }
    }

    /// Get the base index for a class
    pub fn get(&self, class: ObjectClass) -> usize {
        self.bases[class as usize]
    }
}

/// Select a random object type index from OBJECTS for the given class.
/// Uses the probability field in ObjClassDef for weighted selection.
///
/// This mirrors NetHack's mkobj() selection:
/// ```c
/// i = bases[(int) oclass];
/// while ((prob -= objects[i].oc_prob) > 0)
///     i++;
/// ```
pub fn select_object_type(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    rng: &mut GameRng,
    class: ObjectClass,
) -> Option<usize> {
    let base = bases.get(class);

    // Sum probabilities for this class
    let mut total_prob: i32 = 0;
    let mut count = 0;
    for obj in objects.iter().skip(base) {
        if obj.class != class {
            break;
        }
        if obj.probability > 0 {
            total_prob += obj.probability as i32;
            count += 1;
        }
    }

    if total_prob == 0 || count == 0 {
        return None;
    }

    // Roll random number from 1 to total_prob (NetHack uses rnd, not rn2)
    let mut prob = (rng.rn2(total_prob as u32) + 1) as i32;

    // Find the object
    for (i, obj) in objects.iter().enumerate().skip(base) {
        if obj.class != class {
            break;
        }
        if obj.probability > 0 {
            prob -= obj.probability as i32;
            if prob <= 0 {
                return Some(i);
            }
        }
    }

    // Fallback to first object of class
    Some(base)
}

/// Probability entry for random object class selection
struct ClassProb {
    prob: u8,
    class: ObjectClass,
}

/// Standard dungeon object class probabilities
const MKOBJ_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 10,
        class: ObjectClass::Weapon,
    },
    ClassProb {
        prob: 10,
        class: ObjectClass::Armor,
    },
    ClassProb {
        prob: 20,
        class: ObjectClass::Food,
    },
    ClassProb {
        prob: 8,
        class: ObjectClass::Tool,
    },
    ClassProb {
        prob: 8,
        class: ObjectClass::Gem,
    },
    ClassProb {
        prob: 16,
        class: ObjectClass::Potion,
    },
    ClassProb {
        prob: 16,
        class: ObjectClass::Scroll,
    },
    ClassProb {
        prob: 4,
        class: ObjectClass::Spellbook,
    },
    ClassProb {
        prob: 4,
        class: ObjectClass::Wand,
    },
    ClassProb {
        prob: 3,
        class: ObjectClass::Ring,
    },
    ClassProb {
        prob: 1,
        class: ObjectClass::Amulet,
    },
];

/// Box/container content probabilities
const BOX_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 18,
        class: ObjectClass::Gem,
    },
    ClassProb {
        prob: 15,
        class: ObjectClass::Food,
    },
    ClassProb {
        prob: 18,
        class: ObjectClass::Potion,
    },
    ClassProb {
        prob: 18,
        class: ObjectClass::Scroll,
    },
    ClassProb {
        prob: 12,
        class: ObjectClass::Spellbook,
    },
    ClassProb {
        prob: 7,
        class: ObjectClass::Coin,
    },
    ClassProb {
        prob: 6,
        class: ObjectClass::Wand,
    },
    ClassProb {
        prob: 5,
        class: ObjectClass::Ring,
    },
    ClassProb {
        prob: 1,
        class: ObjectClass::Amulet,
    },
];

/// Rogue level object probabilities
const ROGUE_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 12,
        class: ObjectClass::Weapon,
    },
    ClassProb {
        prob: 12,
        class: ObjectClass::Armor,
    },
    ClassProb {
        prob: 22,
        class: ObjectClass::Food,
    },
    ClassProb {
        prob: 22,
        class: ObjectClass::Potion,
    },
    ClassProb {
        prob: 22,
        class: ObjectClass::Scroll,
    },
    ClassProb {
        prob: 5,
        class: ObjectClass::Wand,
    },
    ClassProb {
        prob: 5,
        class: ObjectClass::Ring,
    },
];

/// Hell/Gehennom object probabilities
const HELL_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 20,
        class: ObjectClass::Weapon,
    },
    ClassProb {
        prob: 20,
        class: ObjectClass::Armor,
    },
    ClassProb {
        prob: 16,
        class: ObjectClass::Food,
    },
    ClassProb {
        prob: 12,
        class: ObjectClass::Tool,
    },
    ClassProb {
        prob: 10,
        class: ObjectClass::Gem,
    },
    ClassProb {
        prob: 1,
        class: ObjectClass::Potion,
    },
    ClassProb {
        prob: 1,
        class: ObjectClass::Scroll,
    },
    ClassProb {
        prob: 8,
        class: ObjectClass::Wand,
    },
    ClassProb {
        prob: 8,
        class: ObjectClass::Ring,
    },
    ClassProb {
        prob: 4,
        class: ObjectClass::Amulet,
    },
];

/// Location type for probability selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationType {
    Normal,
    Rogue,
    Hell,
    Box,
}

/// Object creation context
pub struct MkObjContext {
    next_id: ObjectId,
    current_turn: i64,
}

impl MkObjContext {
    pub fn new() -> Self {
        Self {
            next_id: ObjectId(1),
            current_turn: 0,
        }
    }

    pub fn with_turn(turn: i64) -> Self {
        Self {
            next_id: ObjectId(1),
            current_turn: turn,
        }
    }

    pub fn set_turn(&mut self, turn: i64) {
        self.current_turn = turn;
    }

    /// Get next unique object ID
    pub fn next_id(&mut self) -> ObjectId {
        let id = self.next_id;
        self.next_id = ObjectId(self.next_id.0.wrapping_add(1));
        if self.next_id.0 == 0 {
            self.next_id = ObjectId(1); // Skip 0
        }
        id
    }
}

impl Default for MkObjContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Select a random object class based on location probabilities
pub fn random_class(rng: &mut GameRng, location: LocationType) -> ObjectClass {
    let probs = match location {
        LocationType::Normal => MKOBJ_PROBS,
        LocationType::Rogue => ROGUE_PROBS,
        LocationType::Hell => HELL_PROBS,
        LocationType::Box => BOX_PROBS,
    };

    let total: u32 = probs.iter().map(|p| p.prob as u32).sum();
    let mut roll = rng.rn2(total) as u8;

    for p in probs {
        if roll < p.prob {
            return p.class;
        }
        roll -= p.prob;
    }

    // Fallback (shouldn't happen)
    ObjectClass::Food
}

/// Create a random object of a specific class
pub fn mkobj(ctx: &mut MkObjContext, rng: &mut GameRng, class: ObjectClass, init: bool) -> Object {
    // For now, create a basic object of the class
    // In full implementation, this would select from OBJECTS array by probability
    let id = ctx.next_id();
    let mut obj = Object::new(id, 0, class);
    obj.age = ctx.current_turn;
    obj.location = ObjectLocation::Free;

    if init {
        init_object(&mut obj, rng);
    }

    obj
}

/// Create a random object of a specific class using the OBJECTS data.
/// This is the proper implementation that uses probability-based selection.
///
/// # Arguments
/// * `objects` - The OBJECTS array (from nh_data::objects::OBJECTS)
/// * `bases` - Pre-computed class base indices
/// * `ctx` - Object creation context
/// * `rng` - Random number generator
/// * `class` - The object class to create
/// * `init` - Whether to initialize the object with random properties
pub fn mkobj_with_data(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    class: ObjectClass,
    init: bool,
) -> Object {
    // Select object type by probability
    let object_type = select_object_type(objects, bases, rng, class).unwrap_or(0);

    mksobj_with_data(objects, ctx, rng, object_type as i16, init)
}

/// Create a random object, selecting class based on location, using OBJECTS data.
pub fn mkobj_random_with_data(
    objects: &[ObjClassDef],
    bases: &ClassBases,
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    location: LocationType,
    init: bool,
) -> Object {
    let class = random_class(rng, location);
    mkobj_with_data(objects, bases, ctx, rng, class, init)
}

/// Create a random object, selecting class based on location
pub fn mkobj_random(
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    location: LocationType,
    init: bool,
) -> Object {
    let class = random_class(rng, location);
    mkobj(ctx, rng, class, init)
}

/// Create a specific object by type index
pub fn mksobj(
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    object_type: i16,
    class: ObjectClass,
    init: bool,
) -> Object {
    let id = ctx.next_id();
    let mut obj = Object::new(id, object_type, class);
    obj.age = ctx.current_turn;
    obj.location = ObjectLocation::Free;

    if init {
        init_object(&mut obj, rng);
    }

    obj
}

/// Create a specific object by type index using the OBJECTS data.
/// This populates the object with data from ObjClassDef (weight, name, damage, etc.)
///
/// # Arguments
/// * `objects` - The OBJECTS array (from nh_data::objects::OBJECTS)
/// * `ctx` - Object creation context
/// * `rng` - Random number generator
/// * `object_type` - Index into OBJECTS array
/// * `init` - Whether to initialize with random properties (enchantment, BUC, etc.)
pub fn mksobj_with_data(
    objects: &[ObjClassDef],
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    object_type: i16,
    init: bool,
) -> Object {
    let idx = object_type as usize;
    let def = objects.get(idx).expect("Invalid object type index");

    let id = ctx.next_id();
    let mut obj = Object::new(id, object_type, def.class);
    obj.age = ctx.current_turn;
    obj.location = ObjectLocation::Free;

    // Copy static data from ObjClassDef
    obj.weight = def.weight as u32;
    obj.name = Some(def.name.to_string());
    obj.nutrition = def.nutrition;

    // Weapon damage/bonus
    if def.class == ObjectClass::Weapon {
        obj.damage_dice = 1; // dice count is usually 1
        obj.damage_sides = def.w_small_damage;
        obj.weapon_tohit = def.bonus;
    }

    // Armor AC
    if def.class == ObjectClass::Armor {
        obj.base_ac = def.bonus; // bonus field is AC for armor
    }

    // Multi-generation items (arrows, rocks, etc.) get quantity
    if def.merge
        && matches!(
            def.class,
            ObjectClass::Weapon | ObjectClass::Gem | ObjectClass::Rock
        )
    {
        // Projectiles and rocks get 6-11
        if def.skill < 0 || def.class == ObjectClass::Rock {
            obj.quantity = (rng.rn2(6) + 6) as i32;
        }
    }

    if init {
        init_object_with_data(&mut obj, def, rng);
    }

    obj
}

/// Initialize an object with random properties using ObjClassDef data.
/// This is more accurate than init_object() as it uses the actual object definition.
pub fn init_object_with_data(obj: &mut Object, def: &ObjClassDef, rng: &mut GameRng) {
    match def.class {
        ObjectClass::Weapon => init_weapon_with_data(obj, def, rng),
        ObjectClass::Armor => init_armor_with_data(obj, def, rng),
        ObjectClass::Food => init_food(obj, rng),
        ObjectClass::Tool => init_tool_with_data(obj, def, rng),
        ObjectClass::Gem => init_gem(obj, rng),
        ObjectClass::Potion => init_potion(obj, rng),
        ObjectClass::Scroll => init_scroll(obj, rng),
        ObjectClass::Spellbook => init_spellbook(obj, rng),
        ObjectClass::Wand => init_wand_with_data(obj, def, rng),
        ObjectClass::Ring => init_ring(obj, rng),
        ObjectClass::Amulet => init_amulet(obj, rng),
        ObjectClass::Coin => init_coin(obj, rng),
        ObjectClass::Rock => init_rock(obj, rng),
        _ => {}
    }
}

fn init_weapon_with_data(obj: &mut Object, def: &ObjClassDef, rng: &mut GameRng) {
    // Multi-gen weapons already have quantity set in mksobj_with_data
    if obj.quantity == 0 {
        obj.quantity = 1;
    }

    // 1/11 chance of being enchanted and blessed
    if rng.rn2(11) == 0 {
        obj.enchantment = rne(rng, 3) as i8;
        obj.buc = if rng.rn2(2) == 0 {
            BucStatus::Blessed
        } else {
            BucStatus::Uncursed
        };
    // 1/10 chance of being cursed with negative enchantment
    } else if rng.rn2(10) == 0 {
        curse(obj);
        obj.enchantment = -(rne(rng, 3) as i8);
    } else {
        bless_or_curse(obj, rng, 10);
    }

    // 1/100 chance of being poisoned (if skill is dart, spear, or dagger-like)
    // Negative skill means ammo/projectile
    let skill = def.skill.abs();
    if (1..=2).contains(&skill) || skill == 18 || skill == 24 || skill == 25 {
        // Dagger, knife, spear, dart, shuriken
        if rng.rn2(100) == 0 {
            obj.poisoned = true;
        }
    }
}

fn init_armor_with_data(obj: &mut Object, _def: &ObjClassDef, rng: &mut GameRng) {
    // Some armor types are usually cursed (elven mithril-coat, etc.)
    // For simplicity, use general logic for now
    if rng.rn2(10) == 0 && rng.rn2(11) == 0 {
        curse(obj);
        obj.enchantment = -(rne(rng, 3) as i8);
    } else if rng.rn2(10) == 0 {
        obj.buc = if rng.rn2(2) == 0 {
            BucStatus::Blessed
        } else {
            BucStatus::Uncursed
        };
        obj.enchantment = rne(rng, 3) as i8;
    } else {
        bless_or_curse(obj, rng, 10);
    }
}

fn init_tool_with_data(obj: &mut Object, def: &ObjClassDef, rng: &mut GameRng) {
    bless_or_curse(obj, rng, 5);

    // Charged tools (lamps, horns, etc.) get charges
    // This is indicated by the 'magical' field for most tools
    if def.magical {
        // Horn of plenty, magic lamp, etc.
        obj.enchantment = (rng.rn2(5) + 4) as i8;
    }
}

fn init_wand_with_data(obj: &mut Object, def: &ObjClassDef, rng: &mut GameRng) {
    // Wands get charges based on direction type
    // Ray wands: 6-10, beam wands: 4-8, non-directional: higher
    let base_charges = match def.direction {
        crate::object::DirectionType::Ray => 6,
        crate::object::DirectionType::Immediate => 4,
        _ => 4,
    };
    obj.enchantment = (rng.rn2(5) + base_charges) as i8;
    obj.recharged = 0;
    bless_or_curse(obj, rng, 17);
}

/// Initialize an object with random properties based on its class
pub fn init_object(obj: &mut Object, rng: &mut GameRng) {
    match obj.class {
        ObjectClass::Weapon => init_weapon(obj, rng),
        ObjectClass::Armor => init_armor(obj, rng),
        ObjectClass::Food => init_food(obj, rng),
        ObjectClass::Tool => init_tool(obj, rng),
        ObjectClass::Gem => init_gem(obj, rng),
        ObjectClass::Potion => init_potion(obj, rng),
        ObjectClass::Scroll => init_scroll(obj, rng),
        ObjectClass::Spellbook => init_spellbook(obj, rng),
        ObjectClass::Wand => init_wand(obj, rng),
        ObjectClass::Ring => init_ring(obj, rng),
        ObjectClass::Amulet => init_amulet(obj, rng),
        ObjectClass::Coin => init_coin(obj, rng),
        ObjectClass::Rock => init_rock(obj, rng),
        _ => {}
    }
}

fn init_weapon(obj: &mut Object, rng: &mut GameRng) {
    // Multi-gen weapons (arrows, etc.) get 6-11 quantity
    // For now, assume single quantity unless specified
    obj.quantity = 1;

    // 1/11 chance of being enchanted and blessed
    if rng.rn2(11) == 0 {
        obj.enchantment = rne(rng, 3) as i8;
        obj.buc = if rng.rn2(2) == 0 {
            BucStatus::Blessed
        } else {
            BucStatus::Uncursed
        };
    // 1/10 chance of being cursed with negative enchantment
    } else if rng.rn2(10) == 0 {
        curse(obj);
        obj.enchantment = -(rne(rng, 3) as i8);
    } else {
        bless_or_curse(obj, rng, 10);
    }

    // 1/100 chance of being poisoned (if poisonable)
    if rng.rn2(100) == 0 {
        obj.poisoned = true;
    }
}

fn init_armor(obj: &mut Object, rng: &mut GameRng) {
    // Some armor types are usually cursed
    // For simplicity, use general logic
    if rng.rn2(10) == 0 && rng.rn2(11) == 0 {
        curse(obj);
        obj.enchantment = -(rne(rng, 3) as i8);
    } else if rng.rn2(10) == 0 {
        obj.buc = if rng.rn2(2) == 0 {
            BucStatus::Blessed
        } else {
            BucStatus::Uncursed
        };
        obj.enchantment = rne(rng, 3) as i8;
    } else {
        bless_or_curse(obj, rng, 10);
    }
}

fn init_food(obj: &mut Object, rng: &mut GameRng) {
    // Most food doesn't stack much
    if rng.rn2(6) == 0 {
        obj.quantity = 2;
    } else {
        obj.quantity = 1;
    }
}

fn init_tool(obj: &mut Object, rng: &mut GameRng) {
    // Tools have various special initializations
    // Candles, lamps, containers, etc.
    bless_or_curse(obj, rng, 5);
}

fn init_gem(obj: &mut Object, rng: &mut GameRng) {
    // Rocks get 6-11 quantity, gems usually 1-2
    if rng.rn2(6) == 0 {
        obj.quantity = 2;
    } else {
        obj.quantity = 1;
    }
}

fn init_potion(obj: &mut Object, rng: &mut GameRng) {
    bless_or_curse(obj, rng, 4);
}

fn init_scroll(obj: &mut Object, rng: &mut GameRng) {
    bless_or_curse(obj, rng, 4);
}

fn init_spellbook(obj: &mut Object, rng: &mut GameRng) {
    bless_or_curse(obj, rng, 17);
}

fn init_wand(obj: &mut Object, rng: &mut GameRng) {
    // Wands get 4-8 or 11-15 charges depending on type
    obj.enchantment = (rng.rn2(5) + 4) as i8;
    obj.recharged = 0;
    bless_or_curse(obj, rng, 17);
}

fn init_ring(obj: &mut Object, rng: &mut GameRng) {
    // Charged rings get enchantment
    if rng.rn2(10) != 0 {
        if rng.rn2(10) != 0 {
            let sign = if rng.rn2(2) == 0 { 1 } else { -1 };
            obj.enchantment = sign * rne(rng, 3) as i8;
        } else {
            obj.enchantment = (rng.rn2(2) as i8) - (rng.rn2(3) as i8);
        }
        // Make useless +0 rings less common
        if obj.enchantment == 0 {
            obj.enchantment = (rng.rn2(4) as i8) - (rng.rn2(3) as i8);
        }
        // Negative rings are usually cursed
        if obj.enchantment < 0 && rng.rn2(5) != 0 {
            curse(obj);
        }
    }
    bless_or_curse(obj, rng, 3);
}

fn init_amulet(obj: &mut Object, rng: &mut GameRng) {
    bless_or_curse(obj, rng, 10);
}

fn init_coin(obj: &mut Object, _rng: &mut GameRng) {
    // Gold quantity is usually set by caller
    if obj.quantity == 0 {
        obj.quantity = 1;
    }
}

fn init_rock(obj: &mut Object, rng: &mut GameRng) {
    // Rocks get 6-11 quantity
    obj.quantity = (rng.rn2(6) + 6) as i32;
}

/// Random number for enchantment (exponential distribution)
/// Returns 1 most often, higher values less likely
/// C-compatible rne(x): generates 1..utmp with 1/x probability per step.
/// utmp = (player_level < 15) ? 5 : player_level/3
/// During object creation, player_level is typically 1 so utmp = 5.
fn rne(rng: &mut GameRng, x: u32) -> u32 {
    rne_at_level(rng, x, 1)
}

/// rne with explicit player level (for testing and contexts where level is known)
fn rne_at_level(rng: &mut GameRng, x: u32, player_level: u32) -> u32 {
    let utmp = if player_level < 15 {
        5
    } else {
        player_level / 3
    };
    let mut tmp = 1;
    while tmp < utmp && rng.rn2(x) == 0 {
        tmp += 1;
    }
    tmp
}

/// Randomly bless or curse an object
pub fn bless_or_curse(obj: &mut Object, rng: &mut GameRng, chance: u32) {
    if rng.rn2(chance) == 0 {
        if rng.rn2(2) == 0 {
            curse(obj);
        } else {
            bless(obj);
        }
    }
}

/// Bless an object
pub fn bless(obj: &mut Object) {
    if obj.class == ObjectClass::Coin {
        return;
    }
    obj.buc = BucStatus::Blessed;
}

/// Curse an object
pub fn curse(obj: &mut Object) {
    if obj.class == ObjectClass::Coin {
        return;
    }
    obj.buc = BucStatus::Cursed;
}

/// Remove blessing from object
pub fn unbless(obj: &mut Object) {
    if matches!(obj.buc, BucStatus::Blessed) {
        obj.buc = BucStatus::Uncursed;
    }
}

/// Remove curse from object
pub fn uncurse(obj: &mut Object) {
    if matches!(obj.buc, BucStatus::Cursed) {
        obj.buc = BucStatus::Uncursed;
    }
}

/// Get the BUC sign: +1 for blessed, -1 for cursed, 0 for uncursed
pub fn buc_sign(obj: &Object) -> i8 {
    match obj.buc {
        BucStatus::Blessed => 1,
        BucStatus::Cursed => -1,
        BucStatus::Uncursed => 0,
    }
}

/// Create gold coins
pub fn mkgold(ctx: &mut MkObjContext, amount: i32) -> Object {
    let id = ctx.next_id();
    let mut obj = Object::new(id, 0, ObjectClass::Coin);
    obj.quantity = amount.max(1);
    obj.name = Some("gold piece".to_string());
    obj
}

/// Split a stack of objects, returning the split-off portion
pub fn split_obj(obj: &mut Object, ctx: &mut MkObjContext, num: i32) -> Option<Object> {
    if num <= 0 || obj.quantity <= num || !obj.contents.is_empty() {
        return None;
    }

    let mut new_obj = obj.clone();
    new_obj.id = ctx.next_id();
    new_obj.quantity = num;
    new_obj.worn_mask = 0; // New object isn't worn
    new_obj.contents.clear();

    obj.quantity -= num;

    Some(new_obj)
}

/// Check if two objects can be merged
pub fn can_merge(obj1: &Object, obj2: &Object) -> bool {
    // Can't merge containers with contents
    if !obj1.contents.is_empty() || !obj2.contents.is_empty() {
        return false;
    }

    // Must be same type
    if obj1.object_type != obj2.object_type {
        return false;
    }

    // Must have same BUC status
    if obj1.buc != obj2.buc {
        return false;
    }

    // Must have same enchantment
    if obj1.enchantment != obj2.enchantment {
        return false;
    }

    // Must have same erosion
    if obj1.erosion1 != obj2.erosion1 || obj1.erosion2 != obj2.erosion2 {
        return false;
    }

    // Must have same erosion-proof status
    if obj1.erosion_proof != obj2.erosion_proof {
        return false;
    }

    // Must have same name (or both unnamed)
    if obj1.name != obj2.name {
        return false;
    }

    // Can't merge if either is worn/wielded
    if obj1.worn_mask != 0 || obj2.worn_mask != 0 {
        return false;
    }

    true
}

/// Merge two objects, adding obj2's quantity to obj1
/// Returns true if merge was successful
pub fn merge_obj(obj1: &mut Object, obj2: &Object) -> bool {
    if !can_merge(obj1, obj2) {
        return false;
    }

    obj1.quantity += obj2.quantity;
    true
}

// ============================================================================
// Corpse/Statue creation (mkcorpstat from mkobj.c)
// ============================================================================

/// Create a corpse or statue.
///
/// This function creates either a corpse or statue object for a monster.
/// The corpse_type field is set to the monster index, and appropriate
/// timers are started for corpse rot or revival.
///
/// # Arguments
/// * `ctx` - Object creation context
/// * `rng` - Random number generator
/// * `objtype` - Either CORPSE or STATUE constant
/// * `monster_type` - Monster index for the corpse/statue
/// * `flags` - Creation flags
///
/// # Returns
/// The created corpse or statue object
pub fn mkcorpstat(
    ctx: &mut MkObjContext,
    _rng: &mut GameRng,
    objtype: i16,
    monster_type: i16,
    flags: CorpstatFlags,
) -> Object {
    assert!(
        objtype == CORPSE || objtype == STATUE,
        "mkcorpstat: invalid objtype {}",
        objtype
    );

    let id = ctx.next_id();
    let class = if objtype == CORPSE {
        ObjectClass::Food
    } else {
        ObjectClass::Rock // Statues are in Rock/Gem class
    };

    let mut obj = Object::new(id, objtype, class);
    obj.age = ctx.current_turn;
    obj.location = ObjectLocation::Free;

    // Set the monster type
    obj.corpse_type = monster_type;

    // Set weight based on monster (simplified - would need monster data)
    // Default corpse weight; in full implementation this uses monster weight
    obj.weight = if objtype == CORPSE { 50 } else { 2500 };

    if flags.init {
        // Corpses and statues don't have enchantment
        obj.buc = BucStatus::Uncursed;
    }

    // Name based on type
    obj.name = if objtype == CORPSE {
        Some("corpse".to_string())
    } else {
        Some("statue".to_string())
    };

    obj
}

/// Set the monster type for a corpse/statue/egg/figurine and start appropriate timers.
///
/// This function updates the corpse_type field and starts any necessary
/// decay or revival timers.
///
/// # Arguments
/// * `obj` - The corpse/statue/egg/figurine object
/// * `monster_type` - The monster index to set
/// * `timeout_manager` - Timer manager for scheduling decay/revival
/// * `rng` - Random number generator
pub fn set_corpsenm(
    obj: &mut Object,
    monster_type: i16,
    timeout_manager: &mut TimeoutManager,
    rng: &mut GameRng,
) {
    // Cancel any existing timers for this object
    timeout_manager.cancel_object_events(obj.id);

    obj.corpse_type = monster_type;

    match obj.object_type {
        t if t == CORPSE => {
            start_corpse_timeout(obj, timeout_manager, rng, false);
            // Update weight based on monster (simplified)
            obj.weight = 50; // Would use monster weight in full implementation
        }
        t if t == FIGURINE => {
            // Figurines may animate later
            if monster_type >= 0 {
                // Schedule figurine animation (random delay)
                let delay = (rng.rn2(100) + 50) as u64;
                let event = TimedEvent::new(
                    obj.age as u64 + delay,
                    TimedEventType::FigurineAnimate(obj.id),
                );
                timeout_manager.schedule(event);
            }
            obj.weight = 50;
        }
        t if t == EGG => {
            // Eggs may hatch
            if monster_type >= 0 {
                let delay = (rng.rn2(200) + 150) as u64;
                let event =
                    TimedEvent::new(obj.age as u64 + delay, TimedEventType::EggHatch(obj.id));
                timeout_manager.schedule(event);
            }
        }
        _ => {
            // Tin or other object
            obj.weight = 10;
        }
    }
}

/// Start a corpse decay or revive timer.
///
/// This function schedules when a corpse will rot away or potentially revive
/// (for trolls, riders, etc.). Takes the corpse age into account.
///
/// # Arguments
/// * `corpse` - The corpse object
/// * `timeout_manager` - Timer manager for scheduling
/// * `rng` - Random number generator
/// * `in_mklev` - True if being created during level generation (gives more variation)
pub fn start_corpse_timeout(
    corpse: &mut Object,
    timeout_manager: &mut TimeoutManager,
    rng: &mut GameRng,
    in_mklev: bool,
) {
    // Lizards and lichens don't rot
    if corpse.corpse_type == PM_LIZARD || corpse.corpse_type == PM_LICHEN {
        return;
    }

    let rot_adjust: i64 = if in_mklev { 25 } else { 10 };
    let corpse_age = corpse.age.max(0);

    // Calculate when corpse will rot
    let base_when = if corpse_age > ROT_AGE {
        rot_adjust
    } else {
        ROT_AGE - corpse_age
    };

    // Add some random variation
    let variation = rng.rnz(rot_adjust as u32) as i64 - rot_adjust;
    let when = (base_when + variation).max(1) as u64;

    // Check for special monsters that revive
    let will_revive = is_reviver(corpse.corpse_type, rng);

    let event_type = if will_revive {
        // This corpse will revive instead of rot
        // For trolls, check each turn from age 2 to TAINT_AGE for revival
        // For simplicity, we schedule the revival time directly
        TimedEventType::CorpseRot(corpse.id) // Will be handled as revival in event processing
    } else {
        TimedEventType::CorpseRot(corpse.id)
    };

    // Mark if this corpse can revive
    corpse.norevive = !will_revive;

    let event = TimedEvent::new(when, event_type);
    timeout_manager.schedule(event);
}

/// Check if a monster type's corpse can revive.
///
/// Returns true for trolls (chance-based) and riders (always revive).
fn is_reviver(monster_type: i16, rng: &mut GameRng) -> bool {
    // Check if this is a troll (simplified - would check monster letter in full impl)
    // Trolls have a chance to revive
    let is_troll = (200..=210).contains(&monster_type); // Approximate troll range

    // Riders always revive (Death, Pestilence, Famine)
    let is_rider = (350..=353).contains(&monster_type); // Approximate rider range

    if is_rider {
        true
    } else if is_troll {
        // Check if troll revives (1/37 chance per turn from age 2 to 50)
        for _ in 2..=TAINT_AGE {
            if rng.rn2(TROLL_REVIVE_CHANCE) == 0 {
                return true;
            }
        }
        false
    } else {
        false
    }
}

/// Create a corpse for a specific monster.
///
/// Convenience function that creates a corpse and sets up timers.
///
/// # Arguments
/// * `ctx` - Object creation context
/// * `rng` - Random number generator
/// * `monster_type` - Monster index
/// * `timeout_manager` - Timer manager
pub fn mkcorpse(
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    monster_type: i16,
    timeout_manager: &mut TimeoutManager,
) -> Object {
    let mut corpse = mkcorpstat(ctx, rng, CORPSE, monster_type, CorpstatFlags::INIT);
    start_corpse_timeout(&mut corpse, timeout_manager, rng, false);
    corpse
}

/// Check if a corpse is old enough to be dangerous (tainted).
///
/// Corpses become tainted after TAINT_AGE turns (50 turns).
pub fn corpse_is_tainted(corpse: &Object, current_turn: i64) -> bool {
    if corpse.object_type != CORPSE {
        return false;
    }

    // Lizard and lichen corpses never taint
    if corpse.corpse_type == PM_LIZARD || corpse.corpse_type == PM_LICHEN {
        return false;
    }

    let age = current_turn - corpse.age;
    age > TAINT_AGE
}

/// Check if a corpse has completely rotted away.
pub fn corpse_is_rotten(corpse: &Object, current_turn: i64) -> bool {
    if corpse.object_type != CORPSE {
        return false;
    }

    // Lizard and lichen corpses never rot
    if corpse.corpse_type == PM_LIZARD || corpse.corpse_type == PM_LICHEN {
        return false;
    }

    let age = current_turn - corpse.age;
    age > ROT_AGE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mkobj_context() {
        let mut ctx = MkObjContext::new();
        let id1 = ctx.next_id();
        let id2 = ctx.next_id();
        assert_eq!(id1.0, 1);
        assert_eq!(id2.0, 2);
    }

    #[test]
    fn test_random_class() {
        let mut rng = GameRng::new(42);
        // Just verify it returns valid classes
        for _ in 0..100 {
            let class = random_class(&mut rng, LocationType::Normal);
            assert!(matches!(
                class,
                ObjectClass::Weapon
                    | ObjectClass::Armor
                    | ObjectClass::Food
                    | ObjectClass::Tool
                    | ObjectClass::Gem
                    | ObjectClass::Potion
                    | ObjectClass::Scroll
                    | ObjectClass::Spellbook
                    | ObjectClass::Wand
                    | ObjectClass::Ring
                    | ObjectClass::Amulet
            ));
        }
    }

    #[test]
    fn test_mkobj() {
        let mut ctx = MkObjContext::new();
        let mut rng = GameRng::new(42);

        let obj = mkobj(&mut ctx, &mut rng, ObjectClass::Weapon, true);
        assert_eq!(obj.class, ObjectClass::Weapon);
        assert_eq!(obj.id.0, 1);
    }

    #[test]
    fn test_mkgold() {
        let mut ctx = MkObjContext::new();
        let gold = mkgold(&mut ctx, 100);
        assert_eq!(gold.class, ObjectClass::Coin);
        assert_eq!(gold.quantity, 100);
    }

    #[test]
    fn test_bless_curse() {
        let mut obj = Object::default();
        obj.class = ObjectClass::Weapon;

        bless(&mut obj);
        assert!(matches!(obj.buc, BucStatus::Blessed));

        unbless(&mut obj);
        assert!(matches!(obj.buc, BucStatus::Uncursed));

        curse(&mut obj);
        assert!(matches!(obj.buc, BucStatus::Cursed));

        uncurse(&mut obj);
        assert!(matches!(obj.buc, BucStatus::Uncursed));
    }

    #[test]
    fn test_buc_sign() {
        let mut obj = Object::default();

        obj.buc = BucStatus::Blessed;
        assert_eq!(buc_sign(&obj), 1);

        obj.buc = BucStatus::Cursed;
        assert_eq!(buc_sign(&obj), -1);

        obj.buc = BucStatus::Uncursed;
        assert_eq!(buc_sign(&obj), 0);
    }

    #[test]
    fn test_split_obj() {
        let mut ctx = MkObjContext::new();
        let mut obj = Object::default();
        obj.id = ctx.next_id();
        obj.quantity = 10;
        obj.class = ObjectClass::Weapon;

        let split = split_obj(&mut obj, &mut ctx, 3);
        assert!(split.is_some());
        let split = split.unwrap();
        assert_eq!(obj.quantity, 7);
        assert_eq!(split.quantity, 3);
        assert_ne!(obj.id, split.id);
    }

    #[test]
    fn test_can_merge() {
        let mut obj1 = Object::default();
        obj1.object_type = 1;
        obj1.quantity = 5;

        let mut obj2 = Object::default();
        obj2.object_type = 1;
        obj2.quantity = 3;

        assert!(can_merge(&obj1, &obj2));

        // Different types can't merge
        obj2.object_type = 2;
        assert!(!can_merge(&obj1, &obj2));
    }

    #[test]
    fn test_merge_obj() {
        let mut obj1 = Object::default();
        obj1.object_type = 1;
        obj1.quantity = 5;

        let obj2 = Object::default();
        let mut obj2 = obj2;
        obj2.object_type = 1;
        obj2.quantity = 3;

        assert!(merge_obj(&mut obj1, &obj2));
        assert_eq!(obj1.quantity, 8);
    }

    #[test]
    fn test_rne_distribution() {
        let mut rng = GameRng::new(42);
        let mut counts = [0u32; 5];

        for _ in 0..1000 {
            let n = rne(&mut rng, 4) as usize;
            if n < 5 {
                counts[n] += 1;
            }
        }

        // rne should return 1 most often
        assert!(counts[1] > counts[2]);
        assert!(counts[2] > counts[3]);
    }

    #[test]
    fn test_class_bases() {
        use crate::object::{DirectionType, Material};

        // Create mock OBJECTS array with a few classes
        let mock_objects = vec![
            // Index 0: dummy
            ObjClassDef {
                name: "strange object",
                description: "",
                class: ObjectClass::IllObj,
                material: Material::default(),
                weight: 0,
                cost: 0,
                probability: 0,
                nutrition: 0,
                w_small_damage: 0,
                w_large_damage: 0,
                bonus: 0,
                skill: 0,
                delay: 0,
                color: 0,
                magical: false,
                merge: false,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
            // Index 1: First weapon
            ObjClassDef {
                name: "dagger",
                description: "",
                class: ObjectClass::Weapon,
                material: Material::Iron,
                weight: 10,
                cost: 4,
                probability: 30,
                nutrition: 0,
                w_small_damage: 4,
                w_large_damage: 3,
                bonus: 2,
                skill: 1,
                delay: 0,
                color: 0,
                magical: false,
                merge: true,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
            // Index 2: Second weapon
            ObjClassDef {
                name: "sword",
                description: "",
                class: ObjectClass::Weapon,
                material: Material::Iron,
                weight: 40,
                cost: 10,
                probability: 50,
                nutrition: 0,
                w_small_damage: 8,
                w_large_damage: 12,
                bonus: 0,
                skill: 7,
                delay: 0,
                color: 0,
                magical: false,
                merge: false,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
            // Index 3: First armor
            ObjClassDef {
                name: "leather armor",
                description: "",
                class: ObjectClass::Armor,
                material: Material::Leather,
                weight: 150,
                cost: 5,
                probability: 82,
                nutrition: 0,
                w_small_damage: 0,
                w_large_damage: 0,
                bonus: 2,
                skill: 0,
                delay: 0,
                color: 0,
                magical: false,
                merge: false,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
        ];

        let bases = ClassBases::compute(&mock_objects);

        // IllObj starts at 0
        assert_eq!(bases.get(ObjectClass::IllObj), 0);
        // Weapon starts at 1
        assert_eq!(bases.get(ObjectClass::Weapon), 1);
        // Armor starts at 3
        assert_eq!(bases.get(ObjectClass::Armor), 3);
    }

    #[test]
    fn test_select_object_type() {
        use crate::object::{DirectionType, Material};

        // Create mock weapons with different probabilities
        let mock_objects = vec![
            ObjClassDef {
                name: "strange object",
                description: "",
                class: ObjectClass::IllObj,
                material: Material::default(),
                weight: 0,
                cost: 0,
                probability: 0,
                nutrition: 0,
                w_small_damage: 0,
                w_large_damage: 0,
                bonus: 0,
                skill: 0,
                delay: 0,
                color: 0,
                magical: false,
                merge: false,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
            ObjClassDef {
                name: "dagger",
                description: "",
                class: ObjectClass::Weapon,
                material: Material::Iron,
                weight: 10,
                cost: 4,
                probability: 30,
                nutrition: 0,
                w_small_damage: 4,
                w_large_damage: 3,
                bonus: 2,
                skill: 1,
                delay: 0,
                color: 0,
                magical: false,
                merge: true,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
            ObjClassDef {
                name: "sword",
                description: "",
                class: ObjectClass::Weapon,
                material: Material::Iron,
                weight: 40,
                cost: 10,
                probability: 70,
                nutrition: 0,
                w_small_damage: 8,
                w_large_damage: 12,
                bonus: 0,
                skill: 7,
                delay: 0,
                color: 0,
                magical: false,
                merge: false,
                unique: false,
                no_wish: false,
                big: false,
                direction: DirectionType::None,
                armor_category: None,
                property: 0,
            },
        ];

        let bases = ClassBases::compute(&mock_objects);
        let mut rng = GameRng::new(42);

        // Count selections over many iterations
        let mut dagger_count = 0;
        let mut sword_count = 0;

        for _ in 0..1000 {
            let idx = select_object_type(&mock_objects, &bases, &mut rng, ObjectClass::Weapon);
            match idx {
                Some(1) => dagger_count += 1,
                Some(2) => sword_count += 1,
                _ => panic!("unexpected selection"),
            }
        }

        // Sword should be selected more often (70% vs 30%)
        assert!(
            sword_count > dagger_count,
            "sword={} dagger={}",
            sword_count,
            dagger_count
        );
        // Rough check: sword should be about 2.3x more common
        assert!(
            sword_count > dagger_count * 2,
            "sword should be much more common"
        );
    }
}
