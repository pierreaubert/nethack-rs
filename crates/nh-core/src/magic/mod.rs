//! Magic system
//!
//! Implements wands, spells, potions, scrolls, and other magical effects.

pub mod detect;
pub mod potion;
pub mod scroll;
pub mod spell;
pub mod zap;

pub use potion::{quaff_potion, PotionResult, PotionType};
pub use scroll::{read_scroll, ScrollResult, ScrollType};
pub use spell::{cast_spell, KnownSpell, SpellResult, SpellSchool, SpellType};
pub use zap::{zap_wand, ZapDirection, ZapResult, ZapType, ZapVariant};
