//! Player system
//!
//! Contains the You struct and all player-related functionality.

mod alignment;
mod attributes;
mod conduct;
pub mod death;
mod hunger;
pub mod init;
pub mod polymorph;
mod properties;
mod role;
mod skills;
mod you;

pub use alignment::str2align;
pub use alignment::{Alignment, AlignmentType};
pub use alignment::{adjalign, noncoalignment};
pub use attributes::format_strength;
pub use attributes::{Attribute, Attributes};
pub use attributes::{attr2attrname, attrval};
pub use attributes::{exercise_message, record_exercise};
pub use conduct::{Conduct, doconduct, show_conduct};
pub use hunger::HungerState;
pub use properties::{Property, PropertyFlags, PropertySet};
pub use properties::{
    check_innate_source, float_down, float_up, floating_above, intrinsic_possible,
    levitation_dialogue, levitation_vs_flight, phasing_dialogue, toggle_blindness,
    toggle_displacement, toggle_stealth, update_monster_properties,
};
pub use role::{Gender, Race, Role};
pub use role::{
    RoleFilter, build_plselection_prompt, clearrolefilter, gotrolefilter, monster_gender, ok_align,
    ok_gend, ok_race, pet_type, pick_align, pick_gend, pick_race, pick_role, plnamesuffix,
    poly_gender, race_alignmentcount, rigid_role_checks, role_gendercount, role_init, role_init_rng,
    role_menu_extra, role_selection_prolog, root_plselection_prompt, setrolefilter, str2gend,
    str2race, str2role, validalign, validgend, validrace, validrole,
};
pub use skills::weapon_descr;
pub use skills::{
    Skill, SkillLevel, SkillSet, SkillType, add_weapon_skill, enhance_weapon_skill,
    lose_weapon_skill, peaked_skill, skill_level_name, skill_name, unrestrict_weapon_skill,
    use_skill,
};
pub use you::{
    Encumbrance, LUCKADD, Position, StatusEffect, TrapType as PlayerTrapType, You, stone_luck,
};
pub use you::{
    adjabil, check_level_gain, exp_percentage, experience, losehp, losexp, pluslvl, postadjabil,
    rndexp,
};
pub use you::{
    heal_legs, is_fainted, nomul, reset_faint, reset_utrap, set_utrap, set_wounded_legs, um_dist,
    unconscious, unfaint, unmul, wake_up,
};
pub use you::{u_on_dnstairs, u_on_newpos, u_on_rndspot, u_on_sstairs, u_on_upstairs};
