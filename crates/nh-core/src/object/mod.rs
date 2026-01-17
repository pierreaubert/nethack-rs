//! Object system
//!
//! Contains object classes (templates) and instances.

mod obj;
mod objclass;

pub use obj::{BucStatus, Object, ObjectId, ObjectLocation};
pub use objclass::{ArmorCategory, DirectionType, Material, ObjectClass, ObjClassDef};
