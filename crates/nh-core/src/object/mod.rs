//! Object system
//!
//! Contains object classes (templates) and instances.

mod container;
pub mod inventory;
mod mkobj;
mod obj;
mod objclass;
mod objname;

pub use mkobj::{
    bless, bless_or_curse, buc_sign, can_merge, curse, merge_obj, mkgold, mkobj, mkobj_random,
    mkobj_random_with_data, mkobj_with_data, mksobj, mksobj_with_data, select_object_type,
    split_obj, unbless, uncurse, ClassBases, LocationType, MkObjContext,
    // Corpse creation
    mkcorpstat, mkcorpse, set_corpsenm, start_corpse_timeout, corpse_is_tainted, corpse_is_rotten,
    CorpstatFlags, CORPSE, STATUE, FIGURINE, EGG, PM_LIZARD, PM_LICHEN,
};
pub use obj::{BucStatus, Object, ObjectId, ObjectLocation};
pub use objclass::{ArmorCategory, DirectionType, Material, ObjectClass, ObjClassDef};
pub use objname::{
    an, base_object_name, full_object_name, makeplural, quantity_name, simple_object_name,
    simple_typename, the, ObjectKnowledge,
    // Corpse/special object naming
    corpse_xname, statue_xname, egg_xname, figurine_xname, killer_xname, killer_corpse_xname,
    makesingular,
};
pub use container::{
    can_put_in_container, container_weight, count_contents, count_total_items, empty_container,
    find_by_id, find_in_container, force_container, list_contents, lock_container, open_container,
    put_in_container, take_from_container, take_quantity_from_container, total_weight,
    unlock_container, ContainerResult, TrapType,
};
