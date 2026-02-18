//! Special system behavioral tests
//!
//! Tests for shopkeepers, priests, sounds, steeds, summoning,
//! vaults, pets, and other special systems from the special/ module.

use nh_core::monster::{Monster, MonsterFlags, MonsterId, MonsterSound};
use nh_core::player::{Attribute, Gender, Race, Role, You};
use nh_core::special::sounds::*;
use nh_core::special::steed::*;
use nh_core::special::shk::*;
use nh_core::special::{RoomType, ShopType, SummonResult};
use nh_core::GameRng;

// ============================================================================
// Helpers
// ============================================================================

fn make_monster_with_sound(name: &str, sound: MonsterSound) -> Monster {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.name = name.to_string();
    m.hp = 20;
    m.hp_max = 20;
    m
}

fn make_tame_monster(level: i32) -> Monster {
    let mut m = Monster::new(MonsterId::NONE, 0, 5, 5);
    m.name = "steed".to_string();
    m.hp = 30;
    m.hp_max = 30;
    m.level = level as u8;
    m.state.peaceful = true;
    m
}

fn make_player() -> You {
    let mut p = You::default();
    p.hp = 20;
    p.hp_max = 20;
    p.exp_level = 5;
    p.attr_current.set(Attribute::Strength, 12);
    p.attr_current.set(Attribute::Dexterity, 12);
    p
}

// ============================================================================
// Monster sounds: growl_sound
// ============================================================================

#[test]
fn test_growl_sound_bark() {
    assert_eq!(growl_sound(MonsterSound::Bark), "barks");
}

#[test]
fn test_growl_sound_mew() {
    assert_eq!(growl_sound(MonsterSound::Mew), "mews");
}

#[test]
fn test_growl_sound_roar() {
    assert_eq!(growl_sound(MonsterSound::Roar), "roars");
}

#[test]
fn test_growl_sound_growl() {
    assert_eq!(growl_sound(MonsterSound::Growl), "growls");
}

#[test]
fn test_growl_sound_sqeek() {
    assert_eq!(growl_sound(MonsterSound::Sqeek), "squeaks");
}

#[test]
fn test_growl_sound_sqawk() {
    assert_eq!(growl_sound(MonsterSound::Sqawk), "squawks");
}

#[test]
fn test_growl_sound_hiss() {
    assert_eq!(growl_sound(MonsterSound::Hiss), "hisses");
}

#[test]
fn test_growl_sound_buzz() {
    assert_eq!(growl_sound(MonsterSound::Buzz), "buzzes");
}

#[test]
fn test_growl_sound_grunt() {
    assert_eq!(growl_sound(MonsterSound::Grunt), "grunts");
}

#[test]
fn test_growl_sound_neigh() {
    assert_eq!(growl_sound(MonsterSound::Neigh), "neighs");
}

#[test]
fn test_growl_sound_wail() {
    assert_eq!(growl_sound(MonsterSound::Wail), "wails");
}

#[test]
fn test_growl_sound_gurgle() {
    assert_eq!(growl_sound(MonsterSound::Gurgle), "gurgles");
}

#[test]
fn test_growl_sound_burble() {
    assert_eq!(growl_sound(MonsterSound::Burble), "burbles");
}

#[test]
fn test_growl_sound_animal() {
    assert_eq!(growl_sound(MonsterSound::Animal), "growls");
}

#[test]
fn test_growl_sound_shriek() {
    assert_eq!(growl_sound(MonsterSound::Shriek), "shrieks");
}

#[test]
fn test_growl_sound_bones() {
    assert_eq!(growl_sound(MonsterSound::Bones), "rattles");
}

#[test]
fn test_growl_sound_laugh() {
    assert_eq!(growl_sound(MonsterSound::Laugh), "laughs");
}

#[test]
fn test_growl_sound_mumble() {
    assert_eq!(growl_sound(MonsterSound::Mumble), "mumbles");
}

#[test]
fn test_growl_sound_silent() {
    assert_eq!(growl_sound(MonsterSound::Silent), "is silent");
}

// ============================================================================
// Monster sounds: monster_growl
// ============================================================================

#[test]
fn test_monster_growl_active() {
    let m = make_monster_with_sound("wolf", MonsterSound::Bark);
    let result = monster_growl(&m, MonsterSound::Bark);
    assert!(result.is_some());
    assert!(result.unwrap().contains("barks"));
}

#[test]
fn test_monster_growl_sleeping() {
    let mut m = make_monster_with_sound("wolf", MonsterSound::Bark);
    m.state.sleeping = true;
    let result = monster_growl(&m, MonsterSound::Bark);
    assert!(result.is_none(), "Sleeping monsters should not growl");
}

#[test]
fn test_monster_growl_silent() {
    let m = make_monster_with_sound("ghost", MonsterSound::Silent);
    let result = monster_growl(&m, MonsterSound::Silent);
    assert!(result.is_none(), "Silent monsters produce no growl");
}

// ============================================================================
// Monster sounds: monster_whimper
// ============================================================================

#[test]
fn test_monster_whimper_bark() {
    let m = make_monster_with_sound("dog", MonsterSound::Bark);
    let result = monster_whimper(&m, MonsterSound::Bark);
    assert!(result.is_some());
    assert!(result.unwrap().contains("whines"));
}

#[test]
fn test_monster_whimper_mew() {
    let m = make_monster_with_sound("cat", MonsterSound::Mew);
    let result = monster_whimper(&m, MonsterSound::Mew);
    assert!(result.is_some());
    assert!(result.unwrap().contains("yowls"));
}

#[test]
fn test_monster_whimper_roar() {
    let m = make_monster_with_sound("lion", MonsterSound::Roar);
    let result = monster_whimper(&m, MonsterSound::Roar);
    assert!(result.is_some());
    assert!(result.unwrap().contains("snarls"));
}

#[test]
fn test_monster_whimper_sleeping() {
    let mut m = make_monster_with_sound("dog", MonsterSound::Bark);
    m.state.sleeping = true;
    assert!(monster_whimper(&m, MonsterSound::Bark).is_none());
}

// ============================================================================
// Monster sounds: monster_yelp
// ============================================================================

#[test]
fn test_monster_yelp_bark() {
    let m = make_monster_with_sound("dog", MonsterSound::Bark);
    let result = monster_yelp(&m, MonsterSound::Bark);
    assert!(result.is_some());
}

#[test]
fn test_monster_yelp_mew() {
    let m = make_monster_with_sound("cat", MonsterSound::Mew);
    let result = monster_yelp(&m, MonsterSound::Mew);
    assert!(result.is_some());
}

#[test]
fn test_monster_yelp_sleeping() {
    let mut m = make_monster_with_sound("dog", MonsterSound::Bark);
    m.state.sleeping = true;
    assert!(monster_yelp(&m, MonsterSound::Bark).is_none());
}

// ============================================================================
// Can speak
// ============================================================================

#[test]
fn test_can_speak_humanoid() {
    assert!(can_speak(MonsterSound::Humanoid));
}

#[test]
fn test_cannot_speak_bark() {
    assert!(!can_speak(MonsterSound::Bark));
}

#[test]
fn test_cannot_speak_silent() {
    assert!(!can_speak(MonsterSound::Silent));
}

// ============================================================================
// Steed: can_ride
// ============================================================================

#[test]
fn test_can_ride_tame_large() {
    let m = make_tame_monster(5);
    let p = make_player();
    assert!(can_ride(&m, &p));
}

#[test]
fn test_cannot_ride_hostile() {
    let mut m = make_tame_monster(5);
    m.state.peaceful = false;
    let p = make_player();
    assert!(!can_ride(&m, &p));
}

#[test]
fn test_cannot_ride_small() {
    let m = make_tame_monster(1); // level 1 = too small
    let p = make_player();
    assert!(!can_ride(&m, &p));
}

#[test]
fn test_cannot_ride_while_polymorphed() {
    let m = make_tame_monster(5);
    let mut p = make_player();
    p.monster_num = Some(10);
    assert!(!can_ride(&m, &p));
}

#[test]
fn test_cannot_ride_while_swallowed() {
    let m = make_tame_monster(5);
    let mut p = make_player();
    p.swallowed = true;
    assert!(!can_ride(&m, &p));
}

// ============================================================================
// Steed: doride
// ============================================================================

#[test]
fn test_doride_success() {
    let m = make_tame_monster(5);
    let p = make_player();
    let result = doride(&p, &m);
    assert!(matches!(result, MountResult::Mounted(_)));
}

#[test]
fn test_doride_hostile() {
    let mut m = make_tame_monster(5);
    m.state.peaceful = false;
    let p = make_player();
    let result = doride(&p, &m);
    assert!(matches!(result, MountResult::CantMount(_)));
}

// ============================================================================
// Steed: dismount
// ============================================================================

#[test]
fn test_dismount_not_riding() {
    let p = make_player();
    let result = dismount(&p);
    assert!(matches!(result, DismountResult::NotRiding));
}

// ============================================================================
// Steed: rider_speed_bonus
// ============================================================================

#[test]
fn test_rider_speed_bonus_fast() {
    let bonus = rider_speed_bonus(18);
    assert!(bonus > 0, "Fast steed should give speed bonus");
}

#[test]
fn test_rider_speed_bonus_slow() {
    let bonus = rider_speed_bonus(6);
    assert!(bonus <= rider_speed_bonus(18));
}

// ============================================================================
// Steed: riding_ac_bonus
// ============================================================================

#[test]
fn test_riding_ac_bonus_positive_skill() {
    let bonus = riding_ac_bonus(5);
    assert!(bonus != 0, "Skilled rider should get AC bonus");
}

#[test]
fn test_riding_ac_bonus_zero_skill() {
    let bonus = riding_ac_bonus(0);
    let _ = bonus; // May be 0
}

// ============================================================================
// ShopType variants
// ============================================================================

#[test]
fn test_shop_type_general() {
    let _ = ShopType::General;
}

#[test]
fn test_shop_type_armor() {
    let _ = ShopType::Armor;
}

#[test]
fn test_shop_type_weapon() {
    let _ = ShopType::Weapon;
}

#[test]
fn test_shop_type_food() {
    let _ = ShopType::Food;
}

#[test]
fn test_shop_type_scroll() {
    let _ = ShopType::Scroll;
}

#[test]
fn test_shop_type_potion() {
    let _ = ShopType::Potion;
}

#[test]
fn test_shop_type_wand() {
    let _ = ShopType::Wand;
}

#[test]
fn test_shop_type_book() {
    let _ = ShopType::Book;
}

// ============================================================================
// RoomType variants
// ============================================================================

#[test]
fn test_room_type_ordinary() {
    let rt = RoomType::Ordinary;
    assert_eq!(rt, RoomType::Ordinary);
}

#[test]
fn test_room_type_shop() {
    let rt = RoomType::Shop(ShopType::General);
    assert!(matches!(rt, RoomType::Shop(_)));
}

#[test]
fn test_room_type_vault() {
    let rt = RoomType::Vault;
    assert_eq!(rt, RoomType::Vault);
}

#[test]
fn test_room_type_temple() {
    let rt = RoomType::Temple;
    assert_eq!(rt, RoomType::Temple);
}

#[test]
fn test_room_type_zoo() {
    let rt = RoomType::Zoo;
    assert_eq!(rt, RoomType::Zoo);
}

#[test]
fn test_room_type_morgue() {
    let rt = RoomType::Morgue;
    assert_eq!(rt, RoomType::Morgue);
}

#[test]
fn test_room_type_beehive() {
    let rt = RoomType::Beehive;
    assert_eq!(rt, RoomType::Beehive);
}

#[test]
fn test_room_type_barracks() {
    let rt = RoomType::Barracks;
    assert_eq!(rt, RoomType::Barracks);
}

#[test]
fn test_room_type_court() {
    let rt = RoomType::Court;
    assert_eq!(rt, RoomType::Court);
}

#[test]
fn test_room_type_swamp() {
    let rt = RoomType::Swamp;
    assert_eq!(rt, RoomType::Swamp);
}

// ============================================================================
// ShopkeeperExtension
// ============================================================================

#[test]
fn test_shopkeeper_extension_new() {
    let ext = ShopkeeperExtension {
        robbed: 0,
        credit: 0,
        debit: 0,
        loan: 0,
        shop_type: ShopType::General,
        shop_room: 0,
        following: false,
        surcharge: false,
        dismiss_kops: false,
        shop_pos: (10, 10),
        door_pos: (10, 15),
        shop_level: 5,
        bill_count: 0,
        bills: Vec::new(),
        visit_count: 0,
        customer_name: String::new(),
        shop_name: "Strstrk's shop".to_string(),
    };
    assert_eq!(ext.robbed, 0);
    assert_eq!(ext.shop_type, ShopType::General);
}

#[test]
fn test_bill_entry_creation() {
    let entry = BillEntry {
        object_id: 42,
        used_up: false,
        price: 100,
        quantity: 1,
    };
    assert_eq!(entry.price, 100);
    assert!(!entry.used_up);
}

#[test]
fn test_shop_damage_creation() {
    let dmg = ShopDamage {
        x: 5,
        y: 10,
        cost: 200,
    };
    assert_eq!(dmg.cost, 200);
}
