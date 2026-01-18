//! Object creation (mkobj.c)
//!
//! Functions for creating and initializing objects.

use crate::object::{BucStatus, Object, ObjectClass, ObjectId, ObjectLocation};
use crate::rng::GameRng;

/// Probability entry for random object class selection
struct ClassProb {
    prob: u8,
    class: ObjectClass,
}

/// Standard dungeon object class probabilities
const MKOBJ_PROBS: &[ClassProb] = &[
    ClassProb { prob: 10, class: ObjectClass::Weapon },
    ClassProb { prob: 10, class: ObjectClass::Armor },
    ClassProb { prob: 20, class: ObjectClass::Food },
    ClassProb { prob: 8, class: ObjectClass::Tool },
    ClassProb { prob: 8, class: ObjectClass::Gem },
    ClassProb { prob: 16, class: ObjectClass::Potion },
    ClassProb { prob: 16, class: ObjectClass::Scroll },
    ClassProb { prob: 4, class: ObjectClass::Spellbook },
    ClassProb { prob: 4, class: ObjectClass::Wand },
    ClassProb { prob: 3, class: ObjectClass::Ring },
    ClassProb { prob: 1, class: ObjectClass::Amulet },
];

/// Box/container content probabilities
const BOX_PROBS: &[ClassProb] = &[
    ClassProb { prob: 18, class: ObjectClass::Gem },
    ClassProb { prob: 15, class: ObjectClass::Food },
    ClassProb { prob: 18, class: ObjectClass::Potion },
    ClassProb { prob: 18, class: ObjectClass::Scroll },
    ClassProb { prob: 12, class: ObjectClass::Spellbook },
    ClassProb { prob: 7, class: ObjectClass::Coin },
    ClassProb { prob: 6, class: ObjectClass::Wand },
    ClassProb { prob: 5, class: ObjectClass::Ring },
    ClassProb { prob: 1, class: ObjectClass::Amulet },
];

/// Rogue level object probabilities
const ROGUE_PROBS: &[ClassProb] = &[
    ClassProb { prob: 12, class: ObjectClass::Weapon },
    ClassProb { prob: 12, class: ObjectClass::Armor },
    ClassProb { prob: 22, class: ObjectClass::Food },
    ClassProb { prob: 22, class: ObjectClass::Potion },
    ClassProb { prob: 22, class: ObjectClass::Scroll },
    ClassProb { prob: 5, class: ObjectClass::Wand },
    ClassProb { prob: 5, class: ObjectClass::Ring },
];

/// Hell/Gehennom object probabilities
const HELL_PROBS: &[ClassProb] = &[
    ClassProb { prob: 20, class: ObjectClass::Weapon },
    ClassProb { prob: 20, class: ObjectClass::Armor },
    ClassProb { prob: 16, class: ObjectClass::Food },
    ClassProb { prob: 12, class: ObjectClass::Tool },
    ClassProb { prob: 10, class: ObjectClass::Gem },
    ClassProb { prob: 1, class: ObjectClass::Potion },
    ClassProb { prob: 1, class: ObjectClass::Scroll },
    ClassProb { prob: 8, class: ObjectClass::Wand },
    ClassProb { prob: 8, class: ObjectClass::Ring },
    ClassProb { prob: 4, class: ObjectClass::Amulet },
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
pub fn mkobj(
    ctx: &mut MkObjContext,
    rng: &mut GameRng,
    class: ObjectClass,
    init: bool,
) -> Object {
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
fn rne(rng: &mut GameRng, x: u32) -> u32 {
    let mut n = 1;
    while n < x && rng.rn2(4) == 0 {
        n += 1;
    }
    n
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
}
