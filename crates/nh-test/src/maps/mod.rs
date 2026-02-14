//! Map/dungeon generation comparison module
//!
//! Compares dungeon generation algorithms between C and Rust.
//!
//! ## C Dungeon Generation Overview
//!
//! NetHack 3.6.7 dungeon generation uses:
//! - `makerooms()` in mklev.c - Creates up to 40 rooms per level
//! - `makecorridors()` - Connects rooms in 4 phases
//! - `mkroom()` in mkroom.c - Creates special room types
//! - Rectangle system (rect.c) - Tracks available space for placement
//!
//! ## Room Types
//!
//! 25 room types from OROOM (0) to CANDLESHOP (25):
//! - Special rooms: Court, Swamp, Vault, Beehive, Morgue, etc.
//! - Shops: General, Armor, Scroll, Potion, Weapon, etc.

pub mod generation;
pub mod room_types;
pub mod rooms;

pub use room_types::{CRoomType, c_door_constants, c_room_constants};
