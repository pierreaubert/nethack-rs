//! Magic system
//!
//! Implements wands, spells, potions, scrolls, and other magical effects.

pub mod advanced;
pub mod artifacts;
pub mod components;
pub mod cursed_items;
pub mod detect;
pub mod enchantment;
pub mod equipment;
pub mod genocide;
pub mod identification;
pub mod potion;
pub mod potions;
pub mod property_binding;
pub mod rings;
pub mod scroll;
pub mod special_items;
pub mod spell;
pub mod spell_advancement;
pub mod targeting;
pub mod zap;

pub use advanced::{
    InterruptEvent, SpellFailureReason, calculate_spell_failure_rate, calculate_spell_power,
    calculate_spell_resistance, can_start_spell, check_spell_failure, check_spell_interruption,
    check_spell_resistance, get_spell_amplification, monster_save_vs_magic, spell_affects_demons,
    spell_affects_undead, spell_failure_message,
};
pub use artifacts::{
    ArtifactAbility, ArtifactAttackType, ArtifactEffects, ArtifactProperty, InvokeResult,
    apply_artifact_effects, artifact_provides_protection, doinvoke, get_artifact_attack_bonus,
    get_artifact_defense_bonus, get_artifact_effects, get_artifact_warning,
    remove_artifact_effects, should_warn_of_monster,
};
pub use components::{
    ComponentInventory, ComponentType, can_cast_with_components, consume_spell_components,
    get_spell_components, missing_component_message,
};
pub use cursed_items::{
    CursedConsequence, CursedEffect, CursedItemTracker, apply_cursed_effect,
    calculate_curse_magnitude, check_fumble, cursed_effect_message, determine_cursed_effects,
};
pub use enchantment::{
    EnchantmentResult, can_enchant, damage_enchantment, describe_enchantment, disenchant,
    enchant_armor, enchant_weapon, enchantment_damage_chance, enchantment_to_ac_bonus,
    enchantment_to_damage_bonus, is_over_enchanted, recharge_wand,
};
pub use equipment::{
    apply_artifact_to_player, apply_cursed_item_effects, apply_luckstone_bonus,
    apply_poisoned_weapon_damage, can_drop_item as check_can_drop_item, check_artifact_warning,
    equip_item_with_effects, get_drop_failure_message, get_grease_erosion_resistance,
    reapply_equipment_properties, remove_artifact_from_player, reset_equipment_properties,
    tick_special_items, unequip_item_with_effects, use_grease_charge,
};
pub use genocide::{
    GenocideFlags, GenocideResult, MonsterVitals, do_class_genocide, do_genocide,
    do_reverse_genocide, is_unique_npc,
};
pub use identification::{
    IdentificationKnowledge, IdentificationLevel, IdentificationResult, ItemKnowledge,
    get_identification_progress, get_identified_description, identify_from_scroll,
    identify_from_use,
};
pub use potion::{
    PotionHitResult, PotionResult, PotionType, glow_color, glow_strength, glow_verb, p_glow1,
    p_glow2, potionhit, quaff_potion,
};
pub use potions::{
    ActivePotionEffect, PotionEffectTracker, PotionEffectType, PotionPotency, apply_potion_effect,
    check_potion_interaction, determine_potion_potency, get_effect_message,
};
pub use property_binding::{
    PropertyBinding, apply_all_equipment_properties, calculate_property_bonus,
    determine_item_properties, refresh_all_properties, remove_all_equipment_properties,
    should_apply_property, should_track_property_source,
};
pub use rings::{
    RingHand, RingPower, RingWear, WornRing, apply_ring_effects, calculate_ring_drain,
    check_power_feedback,
};
pub use scroll::{ScrollResult, ScrollType, read_scroll};
pub use special_items::{
    GreasedItem, Loadstone, Luckstone, PoisonedWeapon, SpecialItemTracker, SpecialItemType,
    can_drop_item, detect_loadstone_type, detect_luckstone_type, is_special_item,
    loadstone_stuck_message,
};
pub use spell::{KnownSpell, SpellResult, SpellSchool, SpellType, cast_spell};
pub use spell_advancement::{
    SpellStats, calculate_final_spell_mana_cost, get_spell_failure_reduction, get_spell_stats,
    get_total_spell_damage_bonus, record_critical_spell_hit, record_spell_cast,
    tick_spell_synergies,
};
pub use targeting::{
    TargetInfo, calculate_distance, find_monsters_in_range, find_nearest_monster,
    is_in_line_of_fire, monsters_in_direction,
};
pub use zap::{
    BhitResult, BreakingResult, BuzzResult, ExplosionResult, GolemCreationResult, MbhitEffect,
    ObjectMaterial, ObjectTransformResult, StoneTransformResult, ZapDirection, ZapResult, ZapType,
    ZapVariant, apply_wand_wear_penalty, bhit, breakmsg, breakobj, breaks, breaktest, buzz,
    calculate_wand_wear, can_recharge, check_wand_breakage, create_polymon,
    damage_type_to_zap_type, degrade_wand, direction_toward, do_osshock, elemental_clog, explode,
    explode_oil, get_wand_effectiveness, get_wand_status, hero_breaks, in_line_of_fire,
    max_wand_charges, mbhit_effect, monster_breath_weapon, muse_horn_to_zap_type,
    muse_wand_to_zap_type, poly_obj, splatter_burning_oil, stone_to_flesh_obj, valid_zap_direction,
    zap_damage, zap_wand, zapdir_to_glyph, zapnodir, zappable,
};
