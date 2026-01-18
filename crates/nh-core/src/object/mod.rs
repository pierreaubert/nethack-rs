//! Object system
//!
//! Contains object classes (templates) and instances.

pub mod inventory;
mod mkobj;
mod obj;
mod objclass;

pub use mkobj::{
    bless, bless_or_curse, buc_sign, can_merge, curse, merge_obj, mkgold, mkobj, mkobj_random,
    mksobj, split_obj, unbless, uncurse, LocationType, MkObjContext,
};
pub use obj::{BucStatus, Object, ObjectId, ObjectLocation};
pub use objclass::{ArmorCategory, DirectionType, Material, ObjectClass, ObjClassDef};
