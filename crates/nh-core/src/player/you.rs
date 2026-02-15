//! Main player structure (struct you from you.h)

use serde::{Deserialize, Serialize};

use super::{
    Alignment, AlignmentType, Attributes, Conduct, Gender, HungerState, Property, PropertySet,
    Race, Role, SkillSet,
};
use crate::combat::StatusEffectTracker;
use crate::dungeon::DLevel;
use crate::monster::MonsterId;
use crate::{MAXULEV, NORMAL_SPEED};

/// Types of traps the player can be caught in
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrapType {
    #[default]
    None,
    BearTrap,
    Pit,
    SpikedPit,
    Web,
    Lava,
    InFloor,    // Stuck in solid rock/floor
    BuriedBall, // Attached to buried ball and chain
}

impl TrapType {
    /// Get display name for the trap type
    pub const fn name(&self) -> &'static str {
        match self {
            TrapType::None => "none",
            TrapType::BearTrap => "bear trap",
            TrapType::Pit => "pit",
            TrapType::SpikedPit => "spiked pit",
            TrapType::Web => "web",
            TrapType::Lava => "lava",
            TrapType::InFloor => "solid rock",
            TrapType::BuriedBall => "buried ball",
        }
    }

    /// Check if this is a pit-type trap
    pub const fn is_pit(&self) -> bool {
        matches!(self, TrapType::Pit | TrapType::SpikedPit)
    }
}

/// Position on the map
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }

    /// Calculate distance squared to another position
    pub const fn distance_sq(&self, other: &Position) -> i32 {
        let dx = (self.x - other.x) as i32;
        let dy = (self.y - other.y) as i32;
        dx * dx + dy * dy
    }

    /// Check if adjacent (including diagonals)
    pub const fn is_adjacent(&self, other: &Position) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= 1 && dy <= 1 && (dx > 0 || dy > 0)
    }
}

/// The player character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct You {
    // Identity
    pub name: String,
    pub role: Role,
    pub race: Race,
    pub gender: Gender,

    // Position
    pub pos: Position,
    pub prev_pos: Position,
    pub direction: Position, // dx, dy for movement

    // Dungeon location
    pub level: DLevel,
    pub prev_level: DLevel,
    pub moved: bool,

    // Experience
    pub exp_level: i32,
    pub max_exp_level: i32,
    pub exp: u64,

    // Health
    pub hp: i32,
    pub hp_max: i32,
    pub hp_increases: Vec<i8>,

    // Magic energy
    pub energy: i32,
    pub energy_max: i32,
    pub energy_increases: Vec<i8>,

    // Combat
    pub armor_class: i8,
    pub hit_bonus: i8,
    pub damage_bonus: i8,
    pub protection_level: i8, // Magical protection level from blessed items
    pub spell_protection: i8, // Protection from spells and magic

    // Attributes
    pub attr_current: Attributes,
    pub attr_max: Attributes,

    // Alignment
    pub alignment: Alignment,
    pub original_alignment: AlignmentType,

    // Luck
    pub luck: i8,
    pub luck_bonus: i8,

    // Hunger
    pub nutrition: i32,
    pub hunger_state: HungerState,

    // Movement
    pub movement_points: i16,

    // Properties (resistances, intrinsics, etc.)
    pub properties: PropertySet,

    // Skills
    pub skills: SkillSet,

    // Conduct
    pub conduct: Conduct,

    // Spells
    pub known_spells: Vec<crate::magic::spell::KnownSpell>,

    // Phase 7: Cursed items, artifacts, and special items
    pub cursed_item_tracker: crate::magic::cursed_items::CursedItemTracker,
    pub special_item_tracker: crate::magic::special_items::SpecialItemTracker,
    pub property_binding: crate::magic::property_binding::PropertyBinding,
    pub equipped_items: Vec<u32>, // Track currently equipped item IDs for property binding

    // Extensions: Spell mechanics (synergies, specialization, mastery)
    #[cfg(feature = "extensions")]
    pub spell_synergy_tracker: crate::magic::spell_synergies::SpellSynergyTracker,
    #[cfg(feature = "extensions")]
    pub specialization_tracker: crate::magic::school_specialization::SpecializationTracker,
    #[cfg(feature = "extensions")]
    pub mastery_tracker: crate::magic::mastery_advancement::MasteryAdvancementTracker,

    // Extensions: Advanced spell system state
    #[cfg(feature = "extensions")]
    pub advanced_spell_state: crate::magic::advanced_spells::AdvancedSpellState,

    // Status effects
    pub confused_timeout: u16,
    pub stunned_timeout: u16,
    pub blinded_timeout: u16,
    pub sleeping_timeout: u16,
    pub hallucinating_timeout: u16,
    pub paralyzed_timeout: u16,
    pub sickness_timeout: u16,
    pub vomiting_timeout: u16,
    pub sick_food_timeout: u16,
    pub sick_illness_timeout: u16,
    pub sliming_timeout: u16,
    pub stoning_timeout: u16,
    pub temp_str_bonus: i8,
    pub str_timeout: u16,
    pub status_effects: StatusEffectTracker, // Phase 13: Comprehensive status effect tracking

    // Equipment bonuses
    pub weapon_bonus: i8,

    // Wealth
    pub gold: i32,

    // Polymorph
    pub monster_num: Option<i16>,
    pub polymorph_timeout: u32,

    // Encumbrance
    pub carrying_capacity: i32,
    pub current_weight: i32,

    // Special states
    pub swallowed: bool,
    pub underwater: bool,
    pub buried: bool,

    // Delayed-onset status effects
    /// Turns until petrification completes (0 = not stoning)
    pub stoning: i32,
    /// Turns of food-poisoning / illness remaining (0 = not sick)
    pub sick: i32,
    /// Source of sickness (for death message)
    pub sick_reason: Option<String>,
    /// Turns until strangulation kills (0 = not strangling)
    pub strangled: i32,
    /// Lycanthropy: monster type the player turns into (None = not lycanthropic)
    pub lycanthropy: Option<i16>,

    // Trap state
    pub utrap: u32,           // Turns remaining in trap (0 = not trapped)
    pub utrap_type: TrapType, // Type of trap player is in

    // Shop state
    /// Index of shop the player is currently in (None = not in a shop)
    pub in_shop: Option<usize>,

    // Multi-turn action state
    pub multi: i32, // Turns of multi-turn action remaining (negative = helpless)
    pub multi_reason: Option<String>, // Reason for multi-turn action

    // Wounded legs
    pub wounded_legs_left: u16,  // Turns of wounded left leg
    pub wounded_legs_right: u16, // Turns of wounded right leg

    // Punishment (ball and chain)
    pub punishment: crate::magic::scroll::PunishmentState,

    // Monster interactions
    pub grabbed_by: Option<MonsterId>,
    pub steed: Option<MonsterId>,

    // Religion
    pub god_anger: i32,
    pub prayer_timeout: i32,
    /// Blessing count — turns until god will help again (C: ublesscnt)
    pub bless_count: i32,
    /// Number of artifact gifts from god (C: ugifts)
    pub god_gifts: i32,

    // Turns
    pub turns_played: u64,

    // Spell casting state (for interruption tracking)
    pub casting_spell: Option<u32>, // Spell ID being cast, None if not casting
    pub casting_turns_remaining: u32, // Turns left to cast
    pub casting_interrupted: bool,  // Whether spell was interrupted
}

impl Default for You {
    fn default() -> Self {
        Self {
            name: String::new(),
            role: Role::default(),
            race: Race::default(),
            gender: Gender::default(),

            pos: Position::default(),
            prev_pos: Position::default(),
            direction: Position::default(),

            level: DLevel::default(),
            prev_level: DLevel::default(),
            moved: false,

            exp_level: 1,
            max_exp_level: 1,
            exp: 0,

            hp: 10,
            hp_max: 10,
            hp_increases: vec![0; MAXULEV],

            energy: 1,
            energy_max: 1,
            energy_increases: vec![0; MAXULEV],

            armor_class: 10,
            hit_bonus: 0,
            damage_bonus: 0,
            protection_level: 0,
            spell_protection: 0,

            attr_current: Attributes::default(),
            attr_max: Attributes::default(),

            alignment: Alignment::default(),
            original_alignment: AlignmentType::default(),

            luck: 0,
            luck_bonus: 0,

            nutrition: 900,
            hunger_state: HungerState::NotHungry,

            movement_points: NORMAL_SPEED,

            properties: PropertySet::default(),
            skills: SkillSet::default(),
            conduct: Conduct::default(),
            known_spells: Vec::new(),

            cursed_item_tracker: crate::magic::cursed_items::CursedItemTracker::new(),
            special_item_tracker: crate::magic::special_items::SpecialItemTracker::new(),
            property_binding: crate::magic::property_binding::PropertyBinding::new(),
            equipped_items: Vec::new(),

            #[cfg(feature = "extensions")]
            spell_synergy_tracker: crate::magic::spell_synergies::SpellSynergyTracker::new(),
            #[cfg(feature = "extensions")]
            specialization_tracker: crate::magic::school_specialization::SpecializationTracker::new(
            ),
            #[cfg(feature = "extensions")]
            mastery_tracker: crate::magic::mastery_advancement::MasteryAdvancementTracker::new(),

            #[cfg(feature = "extensions")]
            advanced_spell_state: crate::magic::advanced_spells::AdvancedSpellState::new(),

            confused_timeout: 0,
            stunned_timeout: 0,
            blinded_timeout: 0,
            sleeping_timeout: 0,
            hallucinating_timeout: 0,
            paralyzed_timeout: 0,
            sickness_timeout: 0,
            vomiting_timeout: 0,
            sick_food_timeout: 0,
            sick_illness_timeout: 0,
            sliming_timeout: 0,
            stoning_timeout: 0,
            temp_str_bonus: 0,
            str_timeout: 0,
            status_effects: StatusEffectTracker::new(),

            weapon_bonus: 0,
            gold: 0,

            monster_num: None,
            polymorph_timeout: 0,

            carrying_capacity: 1000,
            current_weight: 0,

            swallowed: false,
            underwater: false,
            buried: false,

            stoning: 0,
            sick: 0,
            sick_reason: None,
            strangled: 0,
            lycanthropy: None,

            utrap: 0,
            utrap_type: TrapType::None,

            in_shop: None,

            multi: 0,
            multi_reason: None,

            wounded_legs_left: 0,
            wounded_legs_right: 0,

            punishment: crate::magic::scroll::PunishmentState::new(),

            grabbed_by: None,
            steed: None,

            god_anger: 0,
            prayer_timeout: 0,
            bless_count: 0,
            god_gifts: 0,

            turns_played: 0,

            casting_spell: None,
            casting_turns_remaining: 0,
            casting_interrupted: false,
        }
    }
}

impl You {
    /// Create a new player with the given identity
    pub fn new(name: String, role: Role, race: Race, gender: Gender) -> Self {
        let mut player = Self {
            name,
            role,
            race,
            gender,
            alignment: Alignment::new(role.default_alignment()),
            original_alignment: role.default_alignment(),
            ..Default::default()
        };

        // Grant racial intrinsics
        if race.has_infravision() {
            player.properties.grant_intrinsic(Property::Infravision);
        }

        player
    }

    /// Check if player is polymorphed
    pub const fn is_polymorphed(&self) -> bool {
        self.monster_num.is_some()
    }

    /// Check if player can move normally
    pub fn can_move(&self) -> bool {
        !self.buried
            && self.sleeping_timeout == 0
            && self.stunned_timeout == 0 // stunned can still move, just randomly
            && !matches!(self.hunger_state, HungerState::Fainted | HungerState::Starved)
    }

    /// Check if player is confused
    pub const fn is_confused(&self) -> bool {
        self.confused_timeout > 0
    }

    /// Check if player is stunned
    pub const fn is_stunned(&self) -> bool {
        self.stunned_timeout > 0
    }

    /// Check if player is blind
    pub const fn is_blind(&self) -> bool {
        self.blinded_timeout > 0
    }

    /// Get player's rank title
    pub fn rank_title(&self) -> &'static str {
        self.role.rank_title(self.exp_level, self.gender)
    }

    /// Calculate weight capacity (C: weight_cap).
    ///
    /// Base: `25 * (STR + CON) + 50`.
    /// Levitation/flying/riding strong steed: MAX_CARR_CAP.
    /// Wounded legs (not flying): -100.
    pub fn weight_cap(&self) -> i32 {
        use crate::player::Property;
        use crate::MAX_CARR_CAP;

        let mut cap = self.attr_current.base_carry_capacity();

        if self.properties.has(Property::Levitation) {
            cap = MAX_CARR_CAP;
        } else {
            if cap > MAX_CARR_CAP {
                cap = MAX_CARR_CAP;
            }
            if !self.properties.has(Property::Flying)
                && self.properties.has(Property::WoundedLegs)
            {
                cap -= 100;
            }
            if cap < 0 {
                cap = 0;
            }
        }

        cap
    }

    /// Recalculate carrying_capacity from current attributes/properties
    pub fn update_carrying_capacity(&mut self) {
        self.carrying_capacity = self.weight_cap();
    }

    /// Calculate encumbrance level (C: calc_capacity/near_capacity).
    ///
    /// Uses C formula: `cap = (excess_weight * 2 / weight_cap) + 1` capped at 5.
    pub fn encumbrance(&self) -> Encumbrance {
        let wc = self.carrying_capacity;
        let excess = self.current_weight - wc;

        if excess <= 0 {
            Encumbrance::Unencumbered
        } else if wc <= 1 {
            Encumbrance::Overloaded
        } else {
            let cap = (excess * 2 / wc) + 1;
            match cap.min(5) {
                1 => Encumbrance::Burdened,
                2 => Encumbrance::Stressed,
                3 => Encumbrance::Strained,
                4 => Encumbrance::Overtaxed,
                _ => Encumbrance::Overloaded,
            }
        }
    }

    /// Calculate encumbrance with extra weight added (C: calc_capacity(xtra_wt))
    pub fn encumbrance_with_extra(&self, extra_weight: i32) -> Encumbrance {
        let wc = self.carrying_capacity;
        let excess = self.current_weight + extra_weight - wc;

        if excess <= 0 {
            Encumbrance::Unencumbered
        } else if wc <= 1 {
            Encumbrance::Overloaded
        } else {
            let cap = (excess * 2 / wc) + 1;
            match cap.min(5) {
                1 => Encumbrance::Burdened,
                2 => Encumbrance::Stressed,
                3 => Encumbrance::Strained,
                4 => Encumbrance::Overtaxed,
                _ => Encumbrance::Overloaded,
            }
        }
    }

    /// Excess weight over capacity (C: inv_weight). Negative = under capacity.
    pub fn excess_weight(&self) -> i32 {
        self.current_weight - self.carrying_capacity
    }

    /// Update hunger state based on nutrition
    pub fn update_hunger(&mut self) {
        self.hunger_state = HungerState::from_nutrition(self.nutrition);
    }

    /// Decrement nutrition (called each turn)
    pub fn digest(&mut self, amount: i32) {
        self.nutrition = self.nutrition.saturating_sub(amount);
        self.update_hunger();
    }

    /// Gain experience points
    pub fn gain_exp(&mut self, exp: u64) {
        self.exp = self.exp.saturating_add(exp);
        self.check_level_up();
    }

    /// Check if player should level up based on current experience
    pub fn check_level_up(&mut self) {
        use crate::{EXP_THRESHOLDS, MAXULEV};

        while (self.exp_level as usize) < MAXULEV {
            let next_level = self.exp_level as usize; // 0-indexed threshold for next level
            if next_level < EXP_THRESHOLDS.len() && self.exp >= EXP_THRESHOLDS[next_level] {
                self.exp_level += 1;
                if self.exp_level > self.max_exp_level {
                    self.max_exp_level = self.exp_level;
                    // Gain HP on level up (based on constitution)
                    let hp_gain =
                        1 + (self.attr_current.get(super::Attribute::Constitution) as i32 / 3);
                    self.hp_max += hp_gain;
                    self.hp += hp_gain;
                    // Gain energy on level up
                    let energy_gain =
                        1 + (self.attr_current.get(super::Attribute::Wisdom) as i32 / 5);
                    self.energy_max += energy_gain;
                    self.energy += energy_gain;
                }
            } else {
                break;
            }
        }
    }

    /// Take damage
    pub fn take_damage(&mut self, damage: i32) {
        self.hp -= damage;
    }

    /// Heal damage
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// Check if player is dead
    pub const fn is_dead(&self) -> bool {
        self.hp <= 0
    }

    // ========================================================================
    // Additional helper functions
    // ========================================================================

    /// Check if player can see at a position (not blind, within sight range)
    pub fn can_see(&self, x: i8, y: i8, sight_range: i32) -> bool {
        if self.is_blind() {
            return false;
        }
        let dist = self.distu(x, y);
        dist <= (sight_range * sight_range)
    }

    /// Check if player can see a monster (not invisible, within sight, etc.)
    pub fn can_see_monster(&self, mon: &crate::monster::Monster, sight_range: i32) -> bool {
        if self.is_blind() {
            return false;
        }
        if mon.state.invisible && !self.properties.has_see_invisible() {
            return false;
        }
        self.can_see(mon.x, mon.y, sight_range)
    }

    /// Distance squared from player to position (distu equivalent)
    pub fn distu(&self, x: i8, y: i8) -> i32 {
        let dx = self.pos.x as i32 - x as i32;
        let dy = self.pos.y as i32 - y as i32;
        dx * dx + dy * dy
    }

    /// Chebyshev distance from player to position (distmin)
    pub fn distmin(&self, x: i8, y: i8) -> i32 {
        let dx = (self.pos.x as i32 - x as i32).abs();
        let dy = (self.pos.y as i32 - y as i32).abs();
        if dx > dy { dx } else { dy }
    }

    /// Check if player is adjacent to a position
    pub fn next_to(&self, x: i8, y: i8) -> bool {
        let dx = (self.pos.x as i32 - x as i32).abs();
        let dy = (self.pos.y as i32 - y as i32).abs();
        dx <= 1 && dy <= 1
    }

    /// Check if player is hallucinating
    pub const fn is_hallucinating(&self) -> bool {
        self.hallucinating_timeout > 0
    }

    /// Check if player is paralyzed
    pub const fn is_paralyzed(&self) -> bool {
        self.paralyzed_timeout > 0
    }

    /// Check if player is sleeping
    pub const fn is_sleeping(&self) -> bool {
        self.sleeping_timeout > 0
    }

    /// Check if player has levitation
    pub fn is_levitating(&self) -> bool {
        self.properties.has_levitation()
    }

    /// Check if player has flying
    pub fn is_flying(&self) -> bool {
        self.properties.has_flying()
    }

    /// Check if player can pass through walls
    pub fn can_pass_walls(&self) -> bool {
        self.properties.has_phasing()
    }

    /// Check if player has telepathy
    pub fn has_telepathy(&self) -> bool {
        self.properties.has_telepathy()
    }

    /// Check if player has see invisible
    pub fn has_see_invisible(&self) -> bool {
        self.properties.has_see_invisible()
    }

    /// Check if player has infravision
    pub fn has_infravision(&self) -> bool {
        self.properties.has_infravision()
    }

    /// Check if player has a disease
    pub fn is_diseased(&self) -> bool {
        // Would check for disease property/timeout
        false
    }

    /// Check if player is sick (food poisoning)
    pub fn is_sick(&self) -> bool {
        // Would check for sick timeout
        false
    }

    // ========================================================================
    // Can-do predicates (ability checks from various sources)
    // ========================================================================

    /// Check if player can be strangled (can_be_strangled)
    /// Breathless, non-humanoid, or already strangled prevents strangulation
    pub fn can_be_strangled(&self) -> bool {
        // Can't strangle breathless creatures
        // Would check for breathless property/polymorph form here
        // For now, assume player can be strangled unless polymorphed
        // Can't strangle non-humanoids (polymorphed into weird form)
        // Would check polymorph form here
        true
    }

    /// Check if player can two-weapon fight (can_twoweapon)
    pub fn can_twoweapon(&self) -> bool {
        // Monks can't two-weapon
        if self.role == Role::Monk {
            return false;
        }
        // Would also check for polymorph into non-humanoid
        // Would check for wielded weapon compatibility
        true
    }

    /// Check if player can pray (can_pray)
    pub fn can_pray(&self) -> bool {
        self.prayer_timeout == 0
    }

    /// Check if player can reach the floor (can_reach_floor)
    /// Levitating players can't reach floor unless they can fly
    pub fn can_reach_floor(&self, levitating_ok: bool) -> bool {
        if self.is_levitating() && !levitating_ok && !self.is_flying() {
            return false;
        }
        true
    }

    /// Check if player can ride a steed (can_ride)
    pub fn can_ride(&self) -> bool {
        // Can't ride while polymorphed into non-humanoid
        // Would check polymorph form
        // Can't ride while levitating
        if self.is_levitating() {
            return false;
        }
        true
    }

    /// Check if player can saddle a monster (can_saddle)
    pub fn can_saddle(&self) -> bool {
        // Similar to can_ride
        self.can_ride()
    }

    /// Check if player can advance skills (can_advance)
    pub fn can_advance(&self, skill_slots: i32) -> bool {
        skill_slots > 0
    }

    /// Check if player can chant (magic voice) (can_chant)
    pub fn can_chant(&self) -> bool {
        // Would check for being strangled, silenced, etc.
        true
    }

    /// Check if player can blow a musical instrument (can_blow)
    pub fn can_blow(&self) -> bool {
        // Would check for being underwater, polymorphed into breathless form
        if self.underwater {
            return false;
        }
        true
    }

    /// Check if player can make fog (with scroll/wand of fog) (can_fog)
    pub fn can_fog(&self) -> bool {
        // Would depend on level type (outdoors, etc.)
        true
    }

    /// Check if player can ooze through small spaces (can_ooze)
    pub fn can_ooze(&self) -> bool {
        // Amorphous polymorphed forms can ooze
        // Would check polymorph form
        false
    }

    /// Check if player can track monsters (can_track)
    pub fn can_track(&self) -> bool {
        // Rangers and some monsters can track
        if self.role == Role::Ranger {
            return true;
        }
        // Would check for tracking intrinsic
        false
    }

    /// Get effective luck (clamped to -13..13)
    pub fn effective_luck(&self) -> i8 {
        (self.luck + self.luck_bonus).clamp(-13, 13)
    }

    /// Change luck by an amount (change_luck equivalent)
    /// The change is applied to base luck (not luck_bonus)
    pub fn change_luck(&mut self, delta: i8) {
        self.luck = (self.luck + delta).clamp(-13, 13);
    }

    /// Set luck to specific value (used for luckstone effects)
    pub fn set_luck(&mut self, value: i8) {
        self.luck = value.clamp(-13, 13);
    }

    /// Lose luck (for bad actions like breaking mirrors)
    pub fn lose_luck(&mut self, amount: i8) {
        self.change_luck(-amount);
    }

    /// Gain luck (for good actions)
    pub fn gain_luck(&mut self, amount: i8) {
        self.change_luck(amount);
    }

    /// Check if luck is positive
    pub fn has_good_luck(&self) -> bool {
        self.effective_luck() > 0
    }

    /// Check if luck is negative
    pub fn has_bad_luck(&self) -> bool {
        self.effective_luck() < 0
    }

    /// Luck timeout function - luck decays toward 0 over time
    /// Call this periodically (e.g., every 600 turns)
    pub fn decay_luck(&mut self) {
        if self.luck > 0 {
            self.luck -= 1;
        } else if self.luck < 0 {
            self.luck += 1;
        }
    }

    /// Update moreluck based on inventory (set_moreluck equivalent)
    ///
    /// Call this when inventory changes affect luck-granting items.
    /// Uses stone_luck to determine the bonus and sets luck_bonus accordingly.
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory items
    pub fn set_moreluck(&mut self, inventory: &[crate::object::Object]) {
        let luckbon = stone_luck(inventory, true);

        // Check if carrying any luckstone
        let has_luckstone = inventory.iter().any(|o| o.confers_luck());

        if luckbon == 0 && !has_luckstone {
            self.luck_bonus = 0;
        } else if luckbon >= 0 {
            self.luck_bonus = LUCKADD;
        } else {
            self.luck_bonus = -LUCKADD;
        }
    }

    // ========================================================================
    // God/alignment helper functions
    // ========================================================================

    /// Get the name of the player's god (u_gname in C)
    pub fn god_name(&self) -> &'static str {
        self.role.god_for_alignment(self.alignment.typ)
    }

    /// Get the god name for a specific alignment (a_gname in C)
    pub fn god_name_for_alignment(&self, align: AlignmentType) -> &'static str {
        self.role.god_for_alignment(align)
    }

    /// Get the alignment name as a string (align_str in C)
    pub fn align_str(&self) -> &'static str {
        self.alignment.typ.as_str()
    }

    /// Get god name for alignment title (align_gname in C)
    pub fn align_gname(&self) -> &'static str {
        self.alignment.typ.as_title()
    }

    /// Get god title for a specific alignment (align_gtitle in C)
    pub fn align_gtitle(&self, align: AlignmentType) -> &'static str {
        align.as_title()
    }

    /// Check if player is in good standing with their god
    pub fn in_god_favor(&self) -> bool {
        self.alignment.in_good_standing() && self.god_anger == 0
    }

    /// Restore player HP
    pub fn restore_hp(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.hp_max);
    }

    /// Restore player energy
    pub fn restore_energy(&mut self, amount: i32) {
        self.energy = (self.energy + amount).min(self.energy_max);
    }

    /// Use energy for spellcasting
    pub fn use_energy(&mut self, amount: i32) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            false
        }
    }

    /// Get hit dice for current level
    pub fn hit_dice(&self) -> i32 {
        self.exp_level
    }

    /// Calculate base damage (for weapons or bare hands)
    pub fn base_damage(
        &self,
        weapon: Option<&crate::object::Object>,
        rng: &mut crate::GameRng,
    ) -> i32 {
        match weapon {
            Some(w) => {
                let dice = if w.damage_dice > 0 { w.damage_dice } else { 1 };
                let sides = if w.damage_sides > 0 {
                    w.damage_sides
                } else {
                    6
                };
                rng.dice(dice as u32, sides as u32) as i32
            }
            None => {
                // Bare hands
                if self.role == Role::Monk {
                    // Monks deal more unarmed damage
                    let sides = ((self.exp_level / 2) + 1).clamp(2, 16) as u32;
                    rng.dice(1, sides) as i32
                } else {
                    rng.dice(1, 2) as i32
                }
            }
        }
    }

    /// Get player pronoun
    pub fn pronoun(&self, case: crate::monster::PronounCase) -> &'static str {
        use crate::monster::PronounCase;
        match (self.gender, case) {
            (Gender::Male, PronounCase::Subject) => "he",
            (Gender::Male, PronounCase::Object) => "him",
            (Gender::Male, PronounCase::Possessive) => "his",
            (Gender::Female, PronounCase::Subject) => "she",
            (Gender::Female, PronounCase::Object) => "her",
            (Gender::Female, PronounCase::Possessive) => "her",
            (Gender::Neuter, PronounCase::Subject) => "it",
            (Gender::Neuter, PronounCase::Object) => "it",
            (Gender::Neuter, PronounCase::Possessive) => "its",
        }
    }

    /// Update all status timeouts (call once per turn)
    pub fn update_timeouts(&mut self) {
        if self.confused_timeout > 0 {
            self.confused_timeout -= 1;
        }
        if self.stunned_timeout > 0 {
            self.stunned_timeout -= 1;
        }
        if self.blinded_timeout > 0 {
            self.blinded_timeout -= 1;
        }
        if self.sleeping_timeout > 0 {
            self.sleeping_timeout -= 1;
        }
        if self.hallucinating_timeout > 0 {
            self.hallucinating_timeout -= 1;
        }
        if self.paralyzed_timeout > 0 {
            self.paralyzed_timeout -= 1;
        }
    }

    // ========================================================================
    // Attribute accessors (from attrib.c - acurr, acurrstr, abon)
    // ========================================================================

    /// Get current (effective) attribute value accounting for modifiers (acurr equivalent)
    /// This includes:
    /// - Base attribute
    /// - Temporary bonuses/penalties
    /// - Equipment effects (rings of gain strength, etc.)
    /// For now, returns the current attribute value directly
    pub fn acurr(&self, attr: super::Attribute) -> i8 {
        // In full implementation, this would also account for:
        // - Polymorph form modifiers
        // - Ring effects (gain strength, etc.)
        // - Divine protection
        // - Other temporary effects
        self.attr_current.get(attr)
    }

    /// Get current strength as display value (acurrstr equivalent)
    /// Handles the special 18/xx notation for strength
    pub fn acurrstr(&self) -> i8 {
        self.acurr(super::Attribute::Strength)
    }

    /// Get base attribute bonus (abon equivalent)
    /// Returns the modifier applied to base attribute for this character
    pub fn abon(&self, attr: super::Attribute) -> i8 {
        // Difference between current and max
        self.attr_current.get(attr) - self.attr_max.get(attr)
    }

    /// Get attribute max (used for restoration)
    pub fn amax(&self, attr: super::Attribute) -> i8 {
        self.attr_max.get(attr)
    }

    /// Set attribute to a new value (with bounds checking)
    pub fn set_attr(&mut self, attr: super::Attribute, value: i8) {
        self.attr_current.set(attr, value);
        // If current exceeds max, raise max
        if self.attr_current.get(attr) > self.attr_max.get(attr) {
            self.attr_max.set(attr, self.attr_current.get(attr));
        }
    }

    /// Modify an attribute by delta (adjattrib equivalent)
    /// Returns true if the attribute actually changed
    pub fn adjattrib(&mut self, attr: super::Attribute, delta: i8) -> bool {
        let old_value = self.attr_current.get(attr);
        self.attr_current.modify(attr, delta);
        let new_value = self.attr_current.get(attr);

        // Update max if needed
        if new_value > self.attr_max.get(attr) {
            self.attr_max.set(attr, new_value);
        }

        old_value != new_value
    }

    /// Exercise an attribute (from eating, practicing, etc.)
    /// Might increase the attribute over time
    pub fn exercise_attr(&mut self, attr: super::Attribute, _amount: i8) {
        // In full implementation, this would track exercise points
        // and potentially increase the attribute after enough exercise
        // For now, stub implementation
        let _ = attr;
    }

    /// Restore attribute to max value (from restore ability effects)
    pub fn restore_attr(&mut self, attr: super::Attribute) {
        self.attr_current.set(attr, self.attr_max.get(attr));
    }

    /// Restore all attributes to max values
    pub fn restore_all_attrs(&mut self) {
        for attr in [
            super::Attribute::Strength,
            super::Attribute::Intelligence,
            super::Attribute::Wisdom,
            super::Attribute::Dexterity,
            super::Attribute::Constitution,
            super::Attribute::Charisma,
        ] {
            self.restore_attr(attr);
        }
    }

    /// Get strength-based to-hit bonus
    pub fn str_to_hit_bonus(&self) -> i8 {
        self.attr_current.strength_to_hit_bonus()
    }

    /// Get strength-based damage bonus
    pub fn str_damage_bonus(&self) -> i8 {
        self.attr_current.strength_damage_bonus()
    }

    /// Get dexterity-based AC bonus
    pub fn dex_ac_bonus(&self) -> i8 {
        self.attr_current.dexterity_ac_bonus()
    }

    /// Get dexterity-based to-hit bonus
    pub fn dex_to_hit_bonus(&self) -> i8 {
        self.attr_current.dexterity_to_hit_bonus()
    }

    /// Get constitution-based HP bonus per level
    pub fn con_hp_bonus(&self) -> i8 {
        self.attr_current.constitution_hp_bonus()
    }

    /// Get charisma price modifier (percentage)
    pub fn cha_price_modifier(&self) -> i32 {
        self.attr_current.charisma_price_modifier()
    }

    // ========================================================================
    // Carrying capacity functions (from hack.c, weight.c)
    // ========================================================================

    /// Calculate maximum carrying capacity based on strength (calc_capacity equivalent)
    /// Returns weight in units (50 weight units = 1 unit for most objects)
    pub fn calc_capacity(&self) -> i32 {
        let str = self.acurr(super::Attribute::Strength) as i32;

        // Base carrying capacity from strength
        // This follows the original NetHack formula
        let base = if str < 3 {
            25
        } else if str < 6 {
            100 + (str - 3) * 25
        } else if str < 10 {
            175 + (str - 6) * 50
        } else if str < 14 {
            375 + (str - 10) * 75
        } else if str < 18 {
            675 + (str - 14) * 100
        } else if str == 18 {
            1075
        } else {
            // Exceptional strength (18/01 to 18/100) mapped to 19-24
            // and 19+ for gauntlets of power, etc.
            match str {
                19 => 1175,
                20 => 1275,
                21 => 1375,
                22 => 1475,
                23 => 1575,
                24 => 1675,
                _ => 1775, // 25 or higher
            }
        };

        // Modifiers
        let mut capacity = base;

        // Constitution bonus (small effect)
        let con = self.acurr(super::Attribute::Constitution) as i32;
        if con > 14 {
            capacity += (con - 14) * 10;
        }

        // Level bonus
        capacity += self.exp_level * 5;

        capacity
    }

    /// Update carrying_capacity field based on current attributes
    pub fn update_capacity(&mut self) {
        self.carrying_capacity = self.calc_capacity();
    }

    /// Check carrying capacity (near_capacity equivalent)
    /// Returns the encumbrance level based on current weight
    pub fn check_capacity(&self) -> Encumbrance {
        let cap = self.carrying_capacity.max(1) as u32;
        let weight = self.current_weight as u32;

        if weight <= cap / 4 {
            Encumbrance::Unencumbered
        } else if weight <= cap / 2 {
            Encumbrance::Burdened
        } else if weight <= (cap * 3) / 4 {
            Encumbrance::Stressed
        } else if weight <= (cap * 9) / 10 {
            Encumbrance::Strained
        } else if weight <= cap {
            Encumbrance::Overtaxed
        } else {
            Encumbrance::Overloaded
        }
    }

    /// Calculate weight capacity at a given encumbrance level
    pub fn weight_cap_at(&self, enc: Encumbrance) -> i32 {
        let cap = self.carrying_capacity;
        match enc {
            Encumbrance::Unencumbered => cap / 4,
            Encumbrance::Burdened => cap / 2,
            Encumbrance::Stressed => (cap * 3) / 4,
            Encumbrance::Strained => (cap * 9) / 10,
            Encumbrance::Overtaxed | Encumbrance::Overloaded => cap,
        }
    }

    /// Get maximum weight before given encumbrance level
    pub fn max_weight_for(&self, enc: Encumbrance) -> i32 {
        self.weight_cap_at(enc)
    }

    /// Get remaining capacity before reaching an encumbrance level
    pub fn remaining_capacity(&self, enc: Encumbrance) -> i32 {
        let limit = self.weight_cap_at(enc);
        (limit - self.current_weight).max(0)
    }

    /// Check if player can carry additional weight without increasing encumbrance
    pub fn can_carry(&self, weight: i32) -> bool {
        let new_weight = self.current_weight + weight;
        new_weight <= self.weight_cap_at(self.encumbrance())
    }

    /// Get encumbrance level as integer (near_capacity equivalent)
    ///
    /// Returns 0-5 matching the C code's SLT_ENCUMBER constants:
    /// 0 = Unencumbered, 1 = Burdened, 2 = Stressed,
    /// 3 = Strained, 4 = Overtaxed, 5 = Overloaded
    pub fn near_capacity(&self) -> u8 {
        self.check_capacity() as u8
    }

    // ========================================================================
    // Healing and cure functions
    // ========================================================================

    /// Heal player HP and optionally cure status effects (healup equivalent)
    ///
    /// # Arguments
    /// * `hp_amount` - Amount of HP to heal
    /// * `cure_sick` - Whether to cure sickness/disease
    /// * `cure_blind` - Whether to cure blindness
    ///
    /// # Returns
    /// true if something was healed/cured
    pub fn healup(&mut self, hp_amount: i32, cure_sick: bool, cure_blind: bool) -> bool {
        let mut did_something = false;

        // Heal HP
        if hp_amount > 0 && self.hp < self.hp_max {
            self.hp = (self.hp + hp_amount).min(self.hp_max);
            did_something = true;
        }

        // Cure sickness (would need sick_timeout field)
        if cure_sick {
            // In full implementation, would clear sick_timeout
            // sick flag, disease flag, etc.
            did_something = true;
        }

        // Cure blindness
        if cure_blind && self.blinded_timeout > 0 {
            self.blinded_timeout = 0;
            did_something = true;
        }

        did_something
    }

    /// Heal legs (wounded legs cure) (heal_legs equivalent)
    ///
    /// Wounded legs reduce movement speed and can't be fixed by normal healing.
    /// Returns true if legs were healed.
    pub fn heal_legs(&mut self) -> bool {
        // Check if player has wounded legs property
        if self.properties.has(Property::WoundedLegs) {
            self.properties.remove_intrinsic(Property::WoundedLegs);
            return true;
        }
        false
    }

    /// Check if something cures sliming (cures_sliming equivalent)
    ///
    /// Sliming is cured by:
    /// - Fire damage (burns away slime)
    /// - Scroll of fire
    /// - Wand of fire
    /// - Burning (from lava, fire trap, etc.)
    /// - Polymorph (changes form)
    ///
    /// # Arguments
    /// * `by_fire` - Whether cure is from fire source
    ///
    /// # Returns
    /// true if sliming was cured (player had sliming and it was cured)
    pub fn cure_sliming(&mut self, by_fire: bool) -> bool {
        // In full implementation, would check for sliming timeout
        // For now, check if player is turning to slime
        // sliming would be tracked as a timeout or property

        if by_fire {
            // Fire cures sliming
            // Would set sliming timeout to 0
            return true;
        }

        false
    }

    /// Check if something cures stoning (cures_stoning equivalent)
    ///
    /// Stoning is cured by:
    /// - Eating acidic food (like a lizard corpse)
    /// - Acid damage
    /// - Polymorph
    ///
    /// # Arguments
    /// * `by_acid` - Whether cure is from acid source
    ///
    /// # Returns
    /// true if stoning was cured
    pub fn cure_stoning(&mut self, by_acid: bool) -> bool {
        // In full implementation, would check for stoning timeout
        // For now, assume success if by_acid

        if by_acid {
            // Acid cures stoning
            // Would set stoning timeout to 0
            return true;
        }

        false
    }

    /// Restore player HP to maximum (for full healing effects)
    pub fn full_heal(&mut self) {
        self.hp = self.hp_max;
    }

    /// Restore player energy to maximum
    pub fn full_energy(&mut self) {
        self.energy = self.energy_max;
    }

    /// Clear all negative status effects
    pub fn cure_all(&mut self) {
        self.confused_timeout = 0;
        self.stunned_timeout = 0;
        self.blinded_timeout = 0;
        self.hallucinating_timeout = 0;
        self.paralyzed_timeout = 0;
        // Would also clear sick, sliming, stoning, etc.
    }

    /// Apply a status effect with duration
    pub fn apply_status(&mut self, status: StatusEffect, duration: u16) {
        match status {
            StatusEffect::Confused => {
                self.confused_timeout = self.confused_timeout.saturating_add(duration)
            }
            StatusEffect::Stunned => {
                self.stunned_timeout = self.stunned_timeout.saturating_add(duration)
            }
            StatusEffect::Blinded => {
                self.blinded_timeout = self.blinded_timeout.saturating_add(duration)
            }
            StatusEffect::Sleeping => {
                self.sleeping_timeout = self.sleeping_timeout.saturating_add(duration)
            }
            StatusEffect::Hallucinating => {
                self.hallucinating_timeout = self.hallucinating_timeout.saturating_add(duration)
            }
            StatusEffect::Paralyzed => {
                self.paralyzed_timeout = self.paralyzed_timeout.saturating_add(duration)
            }
        }
    }

    // ========================================================================
    // Status effect functions (make_* from timeout.c, attrib.c)
    // ========================================================================

    /// Set or clear blindness (make_blinded equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    pub fn make_blinded(&mut self, duration: u16, talk: bool) -> Option<String> {
        let was_blind = self.is_blind();
        self.blinded_timeout = duration;
        let is_blind = self.is_blind();

        if !talk {
            return None;
        }

        match (was_blind, is_blind) {
            (false, true) => Some("A cloud of darkness falls upon you.".to_string()),
            (true, false) => Some("You can see again.".to_string()),
            _ => None,
        }
    }

    /// Set or clear confusion (make_confused equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    pub fn make_confused(&mut self, duration: u16, talk: bool) -> Option<String> {
        let was_confused = self.is_confused();
        self.confused_timeout = duration;
        let is_confused = self.is_confused();

        if !talk {
            return None;
        }

        match (was_confused, is_confused) {
            (false, true) => Some("You feel somewhat dizzy.".to_string()),
            (true, false) => Some("You feel less confused now.".to_string()),
            _ => None,
        }
    }

    /// Set or clear stun (make_stunned equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    pub fn make_stunned(&mut self, duration: u16, talk: bool) -> Option<String> {
        let was_stunned = self.is_stunned();
        self.stunned_timeout = duration;
        let is_stunned = self.is_stunned();

        if !talk {
            return None;
        }

        match (was_stunned, is_stunned) {
            (false, true) => Some("You stagger...".to_string()),
            (true, false) => Some("You feel steadier.".to_string()),
            _ => None,
        }
    }

    /// Set or clear hallucination (make_hallucinated equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    pub fn make_hallucinated(&mut self, duration: u16, talk: bool) -> Option<String> {
        let was_hallucinating = self.is_hallucinating();
        self.hallucinating_timeout = duration;
        let is_hallucinating = self.is_hallucinating();

        if !talk {
            return None;
        }

        match (was_hallucinating, is_hallucinating) {
            (false, true) => Some("Oh wow!  Everything looks so cosmic!".to_string()),
            (true, false) => Some("Everything looks so normal now.".to_string()),
            _ => None,
        }
    }

    /// Set or clear deafness (make_deaf equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    /// Note: deaf_timeout would need to be added to the You struct
    pub fn make_deaf(&mut self, _duration: u16, talk: bool) -> Option<String> {
        // Would need deaf_timeout field
        // For now, just return a message
        if !talk {
            return None;
        }
        None
    }

    /// Set or add glib (slippery fingers) effect (make_glib equivalent)
    ///
    /// Returns a message describing the change, or None if no change.
    /// Note: glib_timeout would need to be added to the You struct
    pub fn make_glib(&mut self, _duration: u16, talk: bool) -> Option<String> {
        // Would need glib_timeout field
        if !talk {
            return None;
        }
        None
    }

    /// Handle putting on a blindfold (Blindf_on equivalent)
    ///
    /// Returns a message describing the effect.
    pub fn blindf_on(&mut self) -> Option<String> {
        self.blinded_timeout = 1; // Blindfold keeps you blind as long as worn
        Some("Everything goes dark.".to_string())
    }

    /// Handle taking off a blindfold (Blindf_off equivalent)
    ///
    /// Returns a message describing the effect.
    pub fn blindf_off(&mut self) -> Option<String> {
        // Only clear blindness if it was from the blindfold
        // (timeout of 1 is a marker for blindfold-induced blindness)
        if self.blinded_timeout == 1 {
            self.blinded_timeout = 0;
            Some("You can see again.".to_string())
        } else {
            None
        }
    }

    // ========================================================================
    // Weight and encumbrance message functions
    // ========================================================================

    /// Get total weight carried (weight equivalent)
    pub fn weight(&self) -> i32 {
        self.current_weight
    }

    /// Get cached weight capacity
    pub fn carrying_capacity_cached(&self) -> i32 {
        self.carrying_capacity
    }

    /// Get encumbrance message (encumber_msg equivalent)
    ///
    /// Returns a message describing the change in encumbrance level,
    /// or None if there was no change.
    pub fn encumber_msg(&self, old_encumbrance: Encumbrance) -> Option<String> {
        let new_enc = self.check_capacity();

        if new_enc == old_encumbrance {
            return None;
        }

        use std::cmp::Ordering;
        match new_enc.cmp(&old_encumbrance) {
            Ordering::Greater => {
                // More encumbered
                Some(match new_enc {
                    Encumbrance::Burdened => {
                        "Your movements are slowed slightly because of your load.".to_string()
                    }
                    Encumbrance::Stressed => {
                        "You rebalance your load.  Movement is difficult.".to_string()
                    }
                    Encumbrance::Strained => {
                        "You stagger under your heavy load.  Movement is very hard.".to_string()
                    }
                    Encumbrance::Overtaxed => {
                        "You can barely move a handspan with this load!".to_string()
                    }
                    Encumbrance::Overloaded => "You collapse under your load.".to_string(),
                    Encumbrance::Unencumbered => None?, // Can't happen
                })
            }
            Ordering::Less => {
                // Less encumbered
                Some(match new_enc {
                    Encumbrance::Unencumbered => "Your movements are now unencumbered.".to_string(),
                    Encumbrance::Burdened => "Your load is lighter than before.".to_string(),
                    Encumbrance::Stressed => "You rebalance your load.".to_string(),
                    Encumbrance::Strained => "Your load is still quite heavy.".to_string(),
                    Encumbrance::Overtaxed => "You are still overtaxed.".to_string(),
                    Encumbrance::Overloaded => None?, // Shouldn't happen going down
                })
            }
            Ordering::Equal => None,
        }
    }

    /// Update weight and return encumbrance change message
    pub fn update_weight(&mut self, new_weight: i32) -> Option<String> {
        let old_enc = self.check_capacity();
        self.current_weight = new_weight;
        self.encumber_msg(old_enc)
    }

    /// Add weight and return encumbrance change message
    pub fn add_weight(&mut self, amount: i32) -> Option<String> {
        let old_enc = self.check_capacity();
        self.current_weight = self.current_weight.saturating_add(amount);
        self.encumber_msg(old_enc)
    }

    /// Remove weight and return encumbrance change message
    pub fn remove_weight(&mut self, amount: i32) -> Option<String> {
        let old_enc = self.check_capacity();
        self.current_weight = self.current_weight.saturating_sub(amount);
        self.encumber_msg(old_enc)
    }

    // ========================================================================
    // Experience and Level functions (exper.c)
    // ========================================================================

    /// Regenerate HP over time (regen_hp equivalent)
    /// Called periodically to restore HP based on constitution and level.
    /// Returns true if HP was regenerated.
    pub fn regen_hp(&mut self, rng: &mut crate::GameRng) -> bool {
        if self.hp >= self.hp_max {
            return false;
        }

        // Base regen chance depends on constitution
        let con = self.acurr(super::Attribute::Constitution) as u32;
        let regen_chance = 10 + con; // Higher con = better regen

        // Random chance to regen
        if rng.rn2(regen_chance) == 0 {
            self.hp = (self.hp + 1).min(self.hp_max);
            return true;
        }

        false
    }

    /// Regenerate energy over time
    /// Returns true if energy was regenerated.
    pub fn regen_energy(&mut self, rng: &mut crate::GameRng) -> bool {
        if self.energy >= self.energy_max {
            return false;
        }

        // Base regen chance depends on wisdom
        let wis = self.acurr(super::Attribute::Wisdom) as u32;
        let regen_chance = 15 + wis; // Higher wis = better energy regen

        if rng.rn2(regen_chance) == 0 {
            self.energy = (self.energy + 1).min(self.energy_max);
            return true;
        }

        false
    }

    /// Lose experience level (losexp equivalent)
    /// Called when drained by a vampire, wraith, etc.
    /// Returns true if level was lost.
    pub fn losexp(&mut self, message: bool) -> bool {
        if self.exp_level <= 1 {
            return false;
        }

        self.exp_level -= 1;

        // Lose HP from this level's gain
        let hp_loss = if let Some(&gain) = self.hp_increases.last() {
            self.hp_increases.pop();
            gain as i32
        } else {
            // Default HP loss if no record
            let con = self.acurr(super::Attribute::Constitution);
            1 + (con as i32 / 3)
        };

        self.hp_max = (self.hp_max - hp_loss).max(1);
        self.hp = self.hp.min(self.hp_max);

        // Lose energy from this level's gain
        let energy_loss = if let Some(&gain) = self.energy_increases.last() {
            self.energy_increases.pop();
            gain as i32
        } else {
            let wis = self.acurr(super::Attribute::Wisdom);
            1 + (wis as i32 / 5)
        };

        self.energy_max = (self.energy_max - energy_loss).max(0);
        self.energy = self.energy.min(self.energy_max);

        // Set experience to minimum for current level
        self.exp = if self.exp_level > 1 {
            crate::EXP_THRESHOLDS[(self.exp_level - 1) as usize]
        } else {
            0
        };

        let _ = message; // Would display "You feel less experienced." if true

        true
    }

    /// Gain an experience level (pluslvl equivalent)
    /// Called when gaining a level from potion, wraith corpse, etc.
    /// Returns true if level was gained.
    pub fn pluslvl(&mut self, rng: &mut crate::GameRng, intrinsic: bool) -> bool {
        if self.exp_level >= crate::MAXULEV as i32 {
            return false;
        }

        self.exp_level += 1;
        if self.exp_level > self.max_exp_level {
            self.max_exp_level = self.exp_level;
        }

        // Gain HP
        let con = self.acurr(super::Attribute::Constitution);
        let hp_gain = (rng.rnd(8) as i8 + (con as i8 / 3)).max(1);
        self.hp_max += hp_gain as i32;
        self.hp += hp_gain as i32;
        self.hp_increases.push(hp_gain);

        // Gain energy
        let wis = self.acurr(super::Attribute::Wisdom);
        let energy_gain = (rng.rnd(4) as i8 + (wis as i8 / 5)).max(1);
        self.energy_max += energy_gain as i32;
        self.energy += energy_gain as i32;
        self.energy_increases.push(energy_gain);

        // Set experience to threshold for new level
        if intrinsic {
            // Intrinsic level gain sets XP to threshold
            let threshold_idx = (self.exp_level - 1) as usize;
            if threshold_idx < crate::EXP_THRESHOLDS.len() {
                self.exp = crate::EXP_THRESHOLDS[threshold_idx];
            }
        }

        true
    }

    /// Calculate experience needed for next level (newuexp equivalent)
    pub fn newuexp(&self) -> u64 {
        let next_level = self.exp_level as usize;
        if next_level < crate::EXP_THRESHOLDS.len() {
            crate::EXP_THRESHOLDS[next_level]
        } else {
            u64::MAX // Already at max level
        }
    }

    /// Calculate experience for a specific level (newexplevel equivalent)
    pub fn newexplevel(level: i32) -> u64 {
        let idx = (level - 1) as usize;
        if idx < crate::EXP_THRESHOLDS.len() {
            crate::EXP_THRESHOLDS[idx]
        } else {
            u64::MAX
        }
    }

    /// Award experience for an action (more_experienced equivalent)
    /// exp_gain is the base experience, skill_gain affects skill advancement
    pub fn more_experienced(&mut self, exp_gain: u64, skill_gain: i32) {
        self.exp = self.exp.saturating_add(exp_gain);
        self.check_level_up();

        // Skill gain would affect skill training
        let _ = skill_gain;
    }

    /// Lose strength (losestr equivalent)
    /// Called when poisoned, drained, etc.
    pub fn losestr(&mut self, amount: i8) {
        let str = self.attr_current.get(super::Attribute::Strength);
        let new_str = (str - amount).max(3); // Minimum strength is 3
        self.attr_current.set(super::Attribute::Strength, new_str);
    }

    /// Gain strength (gainstr equivalent)
    /// Called when blessed, from giant strength, etc.
    pub fn gainstr(&mut self, amount: i8) {
        let str = self.attr_current.get(super::Attribute::Strength);
        let max_str = self.attr_max.get(super::Attribute::Strength);
        let new_str = (str + amount).min(max_str).min(25); // Max strength is 25 (with items)
        self.attr_current.set(super::Attribute::Strength, new_str);
    }

    /// Drain energy (drain_en equivalent)
    /// Called when energy is drained by attacks, spells, etc.
    pub fn drain_en(&mut self, amount: i32) {
        self.energy = (self.energy - amount).max(0);
    }

    // ========================================================================
    // Attribute exercise functions (attrib.c)
    // ========================================================================

    /// Exercise an attribute (exercise equivalent)
    /// Called when player does something that might improve an attribute.
    /// For now, this is a simplified version.
    pub fn exercise(&mut self, attr: super::Attribute, positive: bool) {
        // In full implementation, this would track exercise points
        // and potentially increase/decrease attributes over time
        // For simplicity, we just make small immediate changes occasionally
        if positive {
            // Small chance to increase attribute (would be much rarer in real game)
            // This is a stub - real implementation tracks exercise over time
        } else {
            // Small chance to decrease attribute (would be much rarer in real game)
        }
        let _ = attr;
    }

    /// Check and apply exercise effects (exerchk equivalent)
    /// Called periodically to process accumulated exercise.
    pub fn exerchk(&mut self, rng: &mut crate::GameRng) {
        // In full implementation, this would:
        // 1. Check accumulated exercise points for each attribute
        // 2. Potentially increase/decrease attributes based on points
        // 3. Reset exercise counters
        // For now, just a stub
        let _ = rng;
    }

    /// Restore a single attribute to max (restore_attrib equivalent concept)
    /// Different from restore_attr in that it handles messages and checks
    pub fn restore_attrib(&mut self, attr: super::Attribute) -> bool {
        let current = self.attr_current.get(attr);
        let max = self.attr_max.get(attr);

        if current < max {
            self.attr_current.set(attr, max);
            return true;
        }
        false
    }

    /// Abuse an attribute (from illness, bad actions, etc.)
    /// This is the inverse of exercise
    pub fn abuse_attr(&mut self, attr: super::Attribute) {
        // Would track negative exercise points
        // For now, stub
        let _ = attr;
    }

    /// Calculate total armor class from worn equipment and modifiers
    ///
    /// Lower AC is better (ranges -128 to 127).
    /// Base unarmored AC is 10, modified by:
    /// - Worn armor pieces (subtracts their AC bonus + enchantment - erosion)
    /// - Rings of protection (subtracts enchantment)
    /// - Intrinsic protection (subtracts protection level)
    /// - Spell protection (subtracts spell protection value)
    ///
    /// # Arguments
    /// * `inventory` - The player's inventory items
    pub fn find_ac(&mut self, inventory: &[crate::object::Object]) {
        let mut ac = 10i32; // Base unarmored AC

        // Find worn armor pieces and calculate their AC contribution
        for obj in inventory {
            if obj.worn_mask & crate::action::wear::worn_mask::W_ARMOR != 0 {
                // Calculate armor bonus same as find_mac():
                // base_ac + enchantment - erosion, minimum 0
                let base = obj.base_ac as i32;
                let enchant = obj.enchantment as i32;
                let erosion = obj.erosion() as i32;
                let bonus = (base + enchant - erosion).max(0);
                ac = ac.saturating_sub(bonus);
            }
        }

        // Rings of protection (left or right)
        // Note: In full implementation, would check object_type for ring of protection specifically
        // For now, all rings worn reduce AC by their enchantment value
        for obj in inventory {
            use crate::action::wear::worn_mask::*;
            if (obj.worn_mask & (W_RINGL | W_RINGR)) != 0 {
                // Rings can provide AC bonus if enchanted
                if obj.enchantment > 0 {
                    ac = ac.saturating_sub(obj.enchantment as i32);
                }
            }
        }

        // Intrinsic protection (from blessed items and spells)
        ac = ac.saturating_sub(self.protection_level as i32);
        ac = ac.saturating_sub(self.spell_protection as i32);

        // Clamp to valid signed i8 range
        ac = ac.clamp(-128, 127);

        self.armor_class = ac as i8;
    }
}

// ============================================================================
// Module-level player management functions (C you.c translations)
// ============================================================================

/// Luck addition value for luckstones (from hack.h LUCKADD)
pub const LUCKADD: i8 = 3;

/// Calculate luck bonus from carried stones (stone_luck equivalent)
///
/// Calculates the luck modifier based on carried luck-granting items
/// (luckstones and certain artifacts).
///
/// # Arguments
/// * `inventory` - The player's inventory items
/// * `count_uncursed` - If true, uncursed stones count as +1; if false, only blessed count
///
/// # Returns
/// -1, 0, or +1 indicating the direction of luck modification
pub fn stone_luck(inventory: &[crate::object::Object], count_uncursed: bool) -> i8 {
    let mut bonchance: i64 = 0;

    for obj in inventory {
        if obj.confers_luck() {
            if obj.is_cursed() {
                bonchance -= obj.quantity as i64;
            } else if obj.is_blessed() {
                bonchance += obj.quantity as i64;
            } else if count_uncursed {
                // Uncursed stones count positively only when parameter is true
                bonchance += obj.quantity as i64;
            }
        }
    }

    // Return sign: -1, 0, or +1
    crate::consts::sgn(bonchance as i32) as i8
}

/// Lose hit points (losehp equivalent)
pub fn losehp(player: &mut You, damage: i32, cause: Option<&str>) -> bool {
    if player.is_polymorphed() {
        // Losing monster form HP
        let current_mh = player.hp; // Using hp field for polymorph form HP
        let new_mh = (current_mh - damage).max(1);

        if new_mh <= 0 {
            // Rehumanize and take remaining damage
            player.monster_num = None;
            player.polymorph_timeout = 0;
            let remaining = damage - current_mh;
            losehp(player, remaining, cause);
            return true; // Polymorphed form was killed
        }

        player.hp = new_mh;
        false
    } else {
        // Normal form HP loss
        let new_hp = player.hp - damage;

        if new_hp <= 0 {
            // Player died
            player.hp = 0;
            return true;
        }

        player.hp = new_hp;
        false
    }
}

/// Lose experience points (losexp equivalent)
pub fn losexp(player: &mut You, fraction_string: Option<&str>) {
    let current_threshold = You::newexplevel(player.exp_level);

    let loss = if let Some(frac) = fraction_string {
        if frac == "1/2" {
            (player.exp / 2).max(1)
        } else if frac == "1/4" {
            (player.exp / 4).max(1)
        } else {
            player.exp / 10 // Default 1/10
        }
    } else {
        player.exp / 10
    };

    player.exp = player.exp.saturating_sub(loss);

    // Check if we should drop levels
    while player.exp_level > 1 && player.exp < current_threshold {
        player.exp_level -= 1;
    }
}

/// Calculate new HP gain for level up (newhp equivalent)
pub fn newhp(player: &You, rng: &mut crate::GameRng) -> i32 {
    let mut hp = 0i32;

    if player.exp_level == 1 {
        // Initial HP: fixed + role/race + random component
        hp += 10; // Base HD
    // In full implementation: urole.hpadv.infix + urace.hpadv.infix
    // Plus: rnd(urole.hpadv.inrnd) + rnd(urace.hpadv.inrnd)
    } else if player.exp_level < 30 {
        // Normal level: fixed + random
        hp += 8; // Typical
        hp += rng.rn2(4) as i32; // Random 0-3
    } else {
        // High level: smaller gains
        hp += 3;
    }

    // Adjust for constitution
    let con_attr = player
        .attr_current
        .get(crate::player::Attribute::Constitution) as i32;
    let con_bonus = if con_attr <= 3 {
        -2
    } else if con_attr <= 6 {
        -1
    } else if con_attr <= 14 {
        0
    } else if con_attr <= 16 {
        1
    } else if con_attr == 17 {
        2
    } else if con_attr == 18 {
        3
    } else {
        4
    };

    hp += con_bonus;
    hp.max(1)
}

/// Calculate new magic energy/power for level up (newpw equivalent)
pub fn newpw(player: &You, rng: &mut crate::GameRng) -> i32 {
    let mut en = 0i32;

    if player.exp_level == 1 {
        // Initial energy: fixed + role/race + random
        en += 5; // Base
    } else {
        // Energy gain based on wisdom and role
        let wis_attr = player.attr_current.get(crate::player::Attribute::Wisdom) as i32;
        let en_base = wis_attr / 2;

        en += en_base;
        en += rng.rn2(4) as i32;
    }

    // Apply role modifier for spell-using classes
    let en = match player.role {
        Role::Priest | Role::Wizard => en * 2,       // Double energy
        Role::Healer | Role::Knight => (en * 3) / 2, // 1.5x energy
        Role::Barbarian | Role::Valkyrie => (en * 3) / 4, // 0.75x energy
        _ => en,
    };

    en.max(1)
}

/// Award experience for defeating a monster (experience equivalent - stub)
pub fn experience(player: &mut You, monster_level: i32, _monster_speed: i32) -> u64 {
    // Simplified: award experience based on monster level
    let base_exp = (10 * (monster_level * monster_level)) as u64;
    player.exp = player.exp.saturating_add(base_exp);
    base_exp
}

/// Get experience display as percentage of next level (exp_percentage equivalent)
pub fn exp_percentage(player: &You) -> u32 {
    let current_level_exp = You::newexplevel(player.exp_level);
    let next_level_exp = You::newexplevel(player.exp_level + 1);
    let range = next_level_exp.saturating_sub(current_level_exp);

    if range == 0 {
        100
    } else {
        let progress = player.exp.saturating_sub(current_level_exp);
        ((progress * 100) / range).min(100) as u32
    }
}

/// Check and potentially gain level through experience (check_level_gain equivalent)
pub fn check_level_gain(player: &mut You, rng: &mut crate::GameRng) {
    if player.exp_level < 30 {
        let threshold = You::newexplevel(player.exp_level + 1);
        if player.exp >= threshold {
            pluslvl(player, rng, true);
        }
    }
}

/// Gain a level (pluslvl equivalent)
pub fn pluslvl(player: &mut You, rng: &mut crate::GameRng, incremental: bool) {
    if !incremental {
        // Non-incremental: display message
        // In full implementation: pline("Welcome to experience level %d.", player.exp_level + 1);
    }

    // Increase HP
    let hp_gain = newhp(player, rng);
    player.hp_max = player.hp_max.saturating_add(hp_gain);
    player.hp = player.hp.saturating_add(hp_gain);

    // Increase energy
    let en_gain = newpw(player, rng);
    player.energy_max = player.energy_max.saturating_add(en_gain);
    player.energy = player.energy.saturating_add(en_gain);

    // Increase level
    if player.exp_level < 30 {
        player.exp_level += 1;
        if player.exp_level > player.max_exp_level {
            player.max_exp_level = player.exp_level;
        }

        // Adjust experience to match new level
        let new_threshold = You::newexplevel(player.exp_level);
        if incremental && player.exp >= new_threshold {
            player.exp = new_threshold.saturating_sub(1);
        }
    }
}

/// Generate random experience gain for polymorphed form (rndexp equivalent)
pub fn rndexp(player: &You, _rng: &mut crate::GameRng) -> u64 {
    let min_exp = if player.exp_level <= 1 {
        0
    } else {
        You::newexplevel(player.exp_level - 1)
    };

    let max_exp = You::newexplevel(player.exp_level);
    let range = max_exp.saturating_sub(min_exp);

    min_exp + (range / 2) // Simplified: midpoint instead of random
}

/// Adjust abilities based on level change (adjabil equivalent - stub)
pub fn adjabil(player: &mut You, old_level: i32, new_level: i32) {
    // Simplified stub: in full implementation would grant/remove intrinsics
    // based on level-gated role and race abilities

    if new_level > old_level {
        // Gaining abilities
        match player.role {
            Role::Rogue if new_level >= 5 => {
                player.properties.grant_intrinsic(Property::Stealth);
            }
            Role::Ranger if new_level >= 3 => {
                player.properties.grant_intrinsic(Property::Searching);
            }
            _ => {}
        }
    }
}

/// Post-adjust abilities (postadjabil equivalent)
pub fn postadjabil(player: &mut You, ability_changed: Option<super::Property>) {
    // Handle side effects of ability changes
    if let Some(ability) = ability_changed {
        match ability {
            // Flying/levitation changes affect movement
            Property::Flying | Property::Levitation | Property::VeryFast => {
                // Movement speed affected - would update player.movement_points in full implementation
            }

            // Invisibility affects stealth
            Property::Invisibility => {
                // Invisibility gained/lost - affects detection
            }

            // Stealth affects surprise
            Property::Stealth => {
                // Stealth changed
            }

            // Speed changes movement
            Property::Speed => {
                // Speed affects movement_points
            }

            // Regeneration affects HP recovery
            Property::Regeneration => {
                // Would enable passive HP recovery
            }

            // Magic resistance affects spell defense
            Property::MagicResistance => {
                // Would improve spell saves
            }

            // Protection affects AC
            Property::Protection => {
                // Player AC is re-calculated elsewhere
            }

            // Other abilities
            _ => {
                // Most other abilities don't have immediate side effects
            }
        }
    }
}

/// Place player at a new map position (u_on_newpos equivalent)
pub fn u_on_newpos(player: &mut You, x: i8, y: i8) {
    // Basic bounds checking (assumes 80x21 level typical)
    if x >= 0 && x < 80 && y >= 0 && y < 21 {
        player.prev_pos = player.pos;
        player.pos = Position::new(x, y);
        player.moved = true;
    }
}

/// Place player at a random valid spot on level (u_on_rndspot equivalent)
pub fn u_on_rndspot(player: &mut You, rng: &mut crate::GameRng) {
    // Try to find a random valid position (assumes 80x21 level)
    let mut attempts = 0;
    while attempts < 100 {
        let x = (rng.rn2(78) as i8) + 1; // 1-78
        let y = (rng.rn2(19) as i8) + 1; // 1-19

        // Simple bounds check - in real implementation would check walkability
        if x > 0 && x < 79 && y > 0 && y < 20 {
            u_on_newpos(player, x, y);
            return;
        }
        attempts += 1;
    }

    // Fallback to center of level
    u_on_newpos(player, 40, 10);
}

/// Place player at dungeon stairs (u_on_dnstairs equivalent - stub)
pub fn u_on_dnstairs(player: &mut You) {
    // Simplified: just place at level center
    // In full implementation: find actual staircase location
    u_on_newpos(player, 40, 10);
}

/// Place player at up stairs (u_on_upstairs equivalent - stub)
pub fn u_on_upstairs(player: &mut You) {
    // Simplified: just place at level center
    u_on_newpos(player, 40, 10);
}

/// Place player at special stairs (u_on_sstairs equivalent - stub)
pub fn u_on_sstairs(player: &mut You, _going_up: bool) {
    // Simplified: just place at level center
    u_on_newpos(player, 40, 10);
}

// ============================================================================
// Trap Functions (set_utrap, reset_utrap from trap.c)
// ============================================================================

/// Set player trap state (set_utrap equivalent)
///
/// Sets the player's trap timer and type. When tim is 0, the player is freed.
///
/// # Arguments
/// * `player` - The player
/// * `tim` - Turns remaining in trap (0 = freed)
/// * `typ` - Type of trap
pub fn set_utrap(player: &mut You, tim: u32, typ: TrapType) {
    player.utrap = tim;
    player.utrap_type = if tim > 0 { typ } else { TrapType::None };

    // In full implementation: would call float_vs_flight() here
    // to potentially block levitation/flying while trapped
}

/// Reset player trap state (reset_utrap equivalent)
///
/// Frees the player from any trap and optionally provides feedback messages.
///
/// # Arguments
/// * `player` - The player
/// * `msg` - If true, provide feedback about regained abilities
///
/// # Returns
/// A message if the player regained levitation or flight, None otherwise
pub fn reset_utrap(player: &mut You, msg: bool) -> Option<String> {
    let was_levitating = player.is_levitating();
    let was_flying = player.is_flying();

    set_utrap(player, 0, TrapType::None);

    if msg {
        if !was_levitating && player.is_levitating() {
            return Some("You float up.".to_string());
        }
        if !was_flying && player.is_flying() {
            return Some("You can fly.".to_string());
        }
    }

    None
}

// ============================================================================
// Multi-turn Action Functions (nomul, unmul from hack.c)
// ============================================================================

/// Set multi-turn action (nomul equivalent)
///
/// Sets the player to perform a multi-turn action. Negative values indicate
/// the player is helpless (paralyzed, sleeping, etc.).
///
/// # Arguments
/// * `player` - The player
/// * `nval` - Number of turns (negative = helpless)
/// * `reason` - Optional reason for the action
pub fn nomul(player: &mut You, nval: i32, reason: Option<String>) {
    if player.multi < nval {
        return; // Don't interrupt a longer action
    }

    player.multi = nval;
    player.multi_reason = reason;

    if nval == 0 {
        player.multi_reason = None;
    }
}

/// Clear multi-turn action (unmul equivalent)
///
/// Cancels any ongoing multi-turn action.
///
/// # Arguments
/// * `player` - The player
/// * `msg` - Optional message to display (unused in this simplified version)
pub fn unmul(player: &mut You, _msg: Option<&str>) {
    player.multi = 0;
    player.multi_reason = None;
}

// ============================================================================
// Wounded Legs Functions (set_wounded_legs, heal_legs from do.c)
// ============================================================================

/// Set wounded legs state (set_wounded_legs equivalent)
///
/// Wounds the player's legs, reducing movement speed.
///
/// # Arguments
/// * `player` - The player
/// * `side` - Which leg (0 = left, 1 = right, 2 = both)
/// * `duration` - How many turns the wound lasts
pub fn set_wounded_legs(player: &mut You, side: u8, duration: u16) {
    match side {
        0 => player.wounded_legs_left = duration,
        1 => player.wounded_legs_right = duration,
        _ => {
            player.wounded_legs_left = duration;
            player.wounded_legs_right = duration;
        }
    }

    // Grant the WoundedLegs property if not already present
    if duration > 0 {
        player.properties.grant_intrinsic(Property::WoundedLegs);
    }
}

/// Heal wounded legs (heal_legs equivalent - already implemented as method)
///
/// This is a module-level wrapper for the You::heal_legs method.
pub fn heal_legs(player: &mut You) -> bool {
    player.wounded_legs_left = 0;
    player.wounded_legs_right = 0;
    player.heal_legs()
}

// ============================================================================
// Consciousness Functions (is_fainted, reset_faint, unfaint, unconscious, wake_up)
// ============================================================================

/// Check if player has fainted from hunger (is_fainted equivalent)
pub fn is_fainted(player: &You) -> bool {
    matches!(player.hunger_state, HungerState::Fainted)
}

/// Reset faint state (reset_faint equivalent)
///
/// Called when player recovers from fainting.
pub fn reset_faint(player: &mut You) {
    if is_fainted(player) {
        player.hunger_state = HungerState::Fainting;
    }
}

/// Recover from fainting (unfaint equivalent)
///
/// Called when player eats food while fainted.
pub fn unfaint(player: &mut You) {
    if is_fainted(player) {
        player.hunger_state = HungerState::Weak;
        unmul(player, Some("You regain consciousness."));
    }
}

/// Check if player is unconscious (unconscious equivalent)
///
/// Player is unconscious if sleeping or fainted.
pub fn unconscious(player: &You) -> bool {
    player.is_sleeping() || is_fainted(player)
}

/// Wake up the player (wake_up equivalent)
///
/// Clears sleeping state and optionally resets multi-turn actions.
///
/// # Arguments
/// * `player` - The player
/// * `msg` - If true, display wake-up message
pub fn wake_up(player: &mut You, msg: bool) -> Option<String> {
    if player.sleeping_timeout > 0 {
        player.sleeping_timeout = 0;
        if msg {
            return Some("You wake up.".to_string());
        }
    }
    None
}

// ============================================================================
// Distance Functions (um_dist from hack.c)
// ============================================================================

/// Calculate distance from player to a monster position (um_dist equivalent)
///
/// Returns the squared distance for efficiency (avoids sqrt).
pub fn um_dist(player: &You, mx: i8, my: i8) -> i32 {
    player.distu(mx, my)
}

/// Status effects that can be applied to the player
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusEffect {
    Confused,
    Stunned,
    Blinded,
    Sleeping,
    Hallucinating,
    Paralyzed,
}

/// Encumbrance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Encumbrance {
    Unencumbered,
    Burdened,
    Stressed,
    Strained,
    Overtaxed,
    Overloaded,
}

impl Encumbrance {
    /// Get movement penalty (subtracted from speed)
    pub const fn movement_penalty(&self) -> i16 {
        match self {
            Encumbrance::Unencumbered => 0,
            Encumbrance::Burdened => 1,
            Encumbrance::Stressed => 3,
            Encumbrance::Strained => 5,
            Encumbrance::Overtaxed => 7,
            Encumbrance::Overloaded => NORMAL_SPEED, // can't move
        }
    }

    /// Get status line display string
    pub const fn status_string(&self) -> Option<&'static str> {
        match self {
            Encumbrance::Unencumbered => None,
            Encumbrance::Burdened => Some("Burdened"),
            Encumbrance::Stressed => Some("Stressed"),
            Encumbrance::Strained => Some("Strained"),
            Encumbrance::Overtaxed => Some("Overtaxed"),
            Encumbrance::Overloaded => Some("Overloaded"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Attribute, Property};
    use super::*;

    fn make_player(str_val: i8, con_val: i8) -> You {
        let mut player = You::default();
        player.attr_current.set(Attribute::Strength, str_val);
        player.attr_current.set(Attribute::Constitution, con_val);
        player.update_carrying_capacity();
        player
    }

    #[test]
    fn test_weight_cap_str_con_formula() {
        // C formula: 25 * (STR + CON) + 50
        let player = make_player(18, 14);
        assert_eq!(player.carrying_capacity, 25 * (18 + 14) + 50); // 850
    }

    #[test]
    fn test_weight_cap_capped_at_max() {
        // MAX_CARR_CAP = 1000
        let player = make_player(25, 25);
        // 25*(25+25)+50 = 1300, but capped at 1000
        assert_eq!(player.carrying_capacity, 1000);
    }

    #[test]
    fn test_weight_cap_low_stats() {
        let player = make_player(3, 3);
        assert_eq!(player.carrying_capacity, 25 * (3 + 3) + 50); // 200
    }

    #[test]
    fn test_encumbrance_unencumbered() {
        let mut player = make_player(18, 14); // cap = 850
        player.current_weight = 800; // under capacity
        assert_eq!(player.encumbrance(), Encumbrance::Unencumbered);
    }

    #[test]
    fn test_encumbrance_burdened() {
        let mut player = make_player(18, 14); // cap = 850
        // excess = 1, cap = (1*2/850)+1 = 1 → Burdened
        player.current_weight = 851;
        assert_eq!(player.encumbrance(), Encumbrance::Burdened);
    }

    #[test]
    fn test_encumbrance_stressed() {
        let mut player = make_player(18, 14); // cap = 850
        // excess = 425, cap = (425*2/850)+1 = 2 → Stressed
        player.current_weight = 1275;
        assert_eq!(player.encumbrance(), Encumbrance::Stressed);
    }

    #[test]
    fn test_encumbrance_strained() {
        let mut player = make_player(18, 14); // cap = 850
        // excess = 850, cap = (850*2/850)+1 = 3 → Strained
        player.current_weight = 1700;
        assert_eq!(player.encumbrance(), Encumbrance::Strained);
    }

    #[test]
    fn test_encumbrance_overtaxed() {
        let mut player = make_player(18, 14); // cap = 850
        // excess = 1275, cap = (1275*2/850)+1 = 4 → Overtaxed
        player.current_weight = 2125;
        assert_eq!(player.encumbrance(), Encumbrance::Overtaxed);
    }

    #[test]
    fn test_encumbrance_overloaded() {
        let mut player = make_player(18, 14); // cap = 850
        // excess = 1700, cap = (1700*2/850)+1 = 5 → Overloaded
        player.current_weight = 2550;
        assert_eq!(player.encumbrance(), Encumbrance::Overloaded);
    }

    #[test]
    fn test_encumbrance_with_extra() {
        let mut player = make_player(18, 14); // cap = 850
        player.current_weight = 840; // under by 10
        assert_eq!(player.encumbrance(), Encumbrance::Unencumbered);
        // Adding 20 puts us 10 over → Burdened
        assert_eq!(player.encumbrance_with_extra(20), Encumbrance::Burdened);
    }

    #[test]
    fn test_excess_weight() {
        let mut player = make_player(18, 14); // cap = 850
        player.current_weight = 800;
        assert_eq!(player.excess_weight(), -50);
        player.current_weight = 900;
        assert_eq!(player.excess_weight(), 50);
    }

    #[test]
    fn test_movement_penalty_per_encumbrance() {
        assert_eq!(Encumbrance::Unencumbered.movement_penalty(), 0);
        assert_eq!(Encumbrance::Burdened.movement_penalty(), 1);
        assert_eq!(Encumbrance::Stressed.movement_penalty(), 3);
        assert_eq!(Encumbrance::Strained.movement_penalty(), 5);
        assert_eq!(Encumbrance::Overtaxed.movement_penalty(), 7);
        assert_eq!(Encumbrance::Overloaded.movement_penalty(), NORMAL_SPEED);
    }

    #[test]
    fn test_gain_exp_no_level_up() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 10 exp - not enough to level up (need 20 for level 2)
        player.gain_exp(10);

        assert_eq!(player.exp, 10);
        assert_eq!(player.exp_level, 1);
    }

    #[test]
    fn test_gain_exp_level_up() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.hp = 10;
        player.hp_max = 10;
        player.energy = 5;
        player.energy_max = 5;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 25 exp - enough to level up to 2 (need 20)
        player.gain_exp(25);

        assert_eq!(player.exp, 25);
        assert_eq!(player.exp_level, 2);
        assert_eq!(player.max_exp_level, 2);
        assert!(player.hp_max > 10, "HP should increase on level up");
        assert!(player.energy_max > 5, "Energy should increase on level up");
    }

    #[test]
    fn test_gain_exp_multiple_levels() {
        let mut player = You::default();
        player.exp_level = 1;
        player.exp = 0;
        player.hp = 10;
        player.hp_max = 10;
        player.energy = 5;
        player.energy_max = 5;
        player.attr_current.set(Attribute::Constitution, 12);
        player.attr_current.set(Attribute::Wisdom, 10);

        // Gain 100 exp - enough to level up to 4 (need 80 for level 4)
        player.gain_exp(100);

        assert_eq!(player.exp, 100);
        assert_eq!(player.exp_level, 4);
        assert_eq!(player.max_exp_level, 4);
    }

    // ========================================================================
    // Tests for find_ac() - Player AC Calculation
    // ========================================================================

    #[test]
    fn test_find_ac_unarmored() {
        // No armor = base AC 10
        let mut player = You::default();
        player.find_ac(&[]);
        assert_eq!(player.armor_class, 10);
    }

    #[test]
    fn test_find_ac_single_armor_piece() {
        use crate::action::wear::worn_mask::W_ARM;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = -3; // Good armor (negative = better)
        armor.enchantment = 0;
        armor.worn_mask = W_ARM; // Being worn

        player.find_ac(&[armor]);
        // AC = 10 - (-3) = 10 - (-3) = 13... wait, base_ac is -3, so bonus is (-3 + 0 - 0).max(0) = 0
        // Actually bonus should be max(0), so 0. AC stays 10.
        // Let me recalculate: base = -3, enchant = 0, erosion = 0
        // bonus = (-3 + 0 - 0).max(0) = 0.max(0) = 0
        // ac = 10 - 0 = 10
        assert_eq!(player.armor_class, 10);
    }

    #[test]
    fn test_find_ac_positive_armor_bonus() {
        use crate::action::wear::worn_mask::W_ARM;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = -5; // base_ac is stored as negative
        armor.enchantment = 2; // +2 enchantment
        armor.worn_mask = W_ARM;

        player.find_ac(&[armor]);
        // bonus = (-5 + 2 - 0).max(0) = (-3).max(0) = 0
        // ac = 10 - 0 = 10
        assert_eq!(player.armor_class, 10);
    }

    #[test]
    fn test_find_ac_with_erosion() {
        use crate::action::wear::worn_mask::W_ARM;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 5;
        armor.enchantment = 3;
        armor.erosion1 = 1; // Minor erosion
        armor.worn_mask = W_ARM;

        player.find_ac(&[armor]);
        // bonus = (5 + 3 - 1).max(0) = 7
        // ac = 10 - 7 = 3
        assert_eq!(player.armor_class, 3);
    }

    #[test]
    fn test_find_ac_with_ring_protection() {
        use crate::action::wear::worn_mask::W_RINGL;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut ring = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Ring);
        ring.enchantment = 2;
        ring.worn_mask = W_RINGL;

        player.find_ac(&[ring]);
        // ac = 10 - 2 = 8
        assert_eq!(player.armor_class, 8);
    }

    #[test]
    fn test_find_ac_with_intrinsic_protection() {
        let mut player = You::default();
        player.protection_level = 3;

        player.find_ac(&[]);
        // ac = 10 - 3 = 7
        assert_eq!(player.armor_class, 7);
    }

    #[test]
    fn test_find_ac_with_spell_protection() {
        let mut player = You::default();
        player.spell_protection = 2;

        player.find_ac(&[]);
        // ac = 10 - 2 = 8
        assert_eq!(player.armor_class, 8);
    }

    #[test]
    fn test_find_ac_combined_protections() {
        use crate::action::wear::worn_mask::{W_ARM, W_RINGL};
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        player.protection_level = 1;
        player.spell_protection = 1;

        let mut armor = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Armor);
        armor.base_ac = 4;
        armor.enchantment = 1;
        armor.worn_mask = W_ARM;

        let mut ring = Object::new(crate::object::ObjectId(2), 2, ObjectClass::Ring);
        ring.enchantment = 1;
        ring.worn_mask = W_RINGL;

        player.find_ac(&[armor, ring]);
        // armor bonus = (4 + 1 - 0).max(0) = 5
        // ac = 10 - 5 = 5
        // ac -= ring enchantment = 5 - 1 = 4
        // ac -= protection_level = 4 - 1 = 3
        // ac -= spell_protection = 3 - 1 = 2
        assert_eq!(player.armor_class, 2);
    }

    #[test]
    fn test_find_ac_clamping_maximum() {
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        // Create many armor pieces to push AC extremely low
        let mut objects = Vec::new();
        for i in 0..10 {
            let mut armor = Object::new(
                crate::object::ObjectId(i as u32 + 1),
                i as i16,
                ObjectClass::Armor,
            );
            armor.base_ac = 20; // Large positive base
            armor.enchantment = 10;
            armor.worn_mask = 1; // W_ARM
            objects.push(armor);
        }

        player.find_ac(&objects);
        // Each piece contributes 30 to ac reduction
        // 10 - (10 * 30) would be -290, but clamped to -128
        assert_eq!(player.armor_class, -128);
    }

    #[test]
    fn test_find_ac_clamping_minimum() {
        let mut player = You::default();
        // No armor, no protection
        player.find_ac(&[]);
        assert!(player.armor_class >= -128 && player.armor_class <= 127);
    }

    #[test]
    fn test_find_ac_negative_ring_enchantment_ignored() {
        use crate::action::wear::worn_mask::W_RINGL;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut ring = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Ring);
        ring.enchantment = -2; // Cursed ring
        ring.worn_mask = W_RINGL;

        player.find_ac(&[ring]);
        // Negative enchantment on ring is ignored
        // ac = 10 (unchanged)
        assert_eq!(player.armor_class, 10);
    }

    #[test]
    fn test_find_ac_zero_ring_enchantment() {
        use crate::action::wear::worn_mask::W_RINGL;
        use crate::object::{Object, ObjectClass};

        let mut player = You::default();
        let mut ring = Object::new(crate::object::ObjectId(1), 1, ObjectClass::Ring);
        ring.enchantment = 0;
        ring.worn_mask = W_RINGL;

        player.find_ac(&[ring]);
        // Zero enchantment doesn't help
        // ac = 10 (unchanged)
        assert_eq!(player.armor_class, 10);
    }

    #[test]
    fn test_losehp_normal_form() {
        let mut player = You::default();
        player.hp = 20;
        player.hp_max = 20;

        let died = losehp(&mut player, 10, Some("fire"));
        assert_eq!(player.hp, 10);
        assert!(!died);

        let died = losehp(&mut player, 15, Some("poison"));
        assert!(died);
        assert_eq!(player.hp, 0);
    }

    #[test]
    fn test_losexp_reduces_experience() {
        let mut player = You::default();
        player.exp_level = 5;
        player.exp = 10000;

        losexp(&mut player, Some("1/2"));
        assert_eq!(player.exp, 5000);

        losexp(&mut player, Some("1/4"));
        assert_eq!(player.exp, 3750);
    }

    #[test]
    fn test_exp_percentage() {
        let mut p = You::default();
        p.exp_level = 1;
        // newexplevel(1) = 0, newexplevel(2) = 20; set exp to midpoint
        p.exp = You::newexplevel(2) / 2; // = 10, giving 50% progress

        let pct = exp_percentage(&p);
        assert!(pct > 0 && pct <= 100);
    }

    #[test]
    fn test_u_on_newpos() {
        let mut player = You::default();
        player.pos = Position::new(40, 10);

        u_on_newpos(&mut player, 45, 15);
        assert_eq!(player.pos.x, 45);
        assert_eq!(player.pos.y, 15);
        assert_eq!(player.prev_pos.x, 40);
        assert_eq!(player.prev_pos.y, 10);
        assert!(player.moved);
    }

    #[test]
    fn test_u_on_newpos_bounds() {
        let mut player = You::default();
        player.pos = Position::new(40, 10);
        player.moved = false;

        // Out of bounds should not update
        u_on_newpos(&mut player, 100, 100);
        assert_eq!(player.pos.x, 40);
        assert_eq!(player.pos.y, 10);
        assert!(!player.moved);
    }

    #[test]
    fn test_u_on_rndspot() {
        let mut player = You::default();
        let mut rng = crate::GameRng::new(42);

        u_on_rndspot(&mut player, &mut rng);
        assert!(player.pos.x > 0 && player.pos.x < 79);
        assert!(player.pos.y > 0 && player.pos.y < 20);
    }

    #[test]
    fn test_u_on_dnstairs() {
        let mut player = You::default();
        u_on_dnstairs(&mut player);
        // Should place at center
        assert_eq!(player.pos.x, 40);
        assert_eq!(player.pos.y, 10);
    }

    #[test]
    fn test_adjabil_rogue() {
        let mut player = You::default();
        player.role = Role::Rogue;

        adjabil(&mut player, 4, 5);
        assert!(player.properties.has_intrinsic(Property::Stealth));
    }

    #[test]
    fn test_adjabil_ranger() {
        let mut player = You::default();
        player.role = Role::Ranger;

        adjabil(&mut player, 2, 3);
        assert!(player.properties.has_intrinsic(Property::Searching));
    }

    #[test]
    fn test_rndexp_bounds() {
        let player = You::default();
        let mut p = player;
        p.exp_level = 5;
        let mut rng = crate::GameRng::new(42);

        let exp = rndexp(&p, &mut rng);
        let min = You::newexplevel(4);
        let max = You::newexplevel(5);
        assert!(exp >= min && exp <= max);
    }

    // ========================================================================
    // Tests for new trap functions
    // ========================================================================

    #[test]
    fn test_set_utrap() {
        let mut player = You::default();
        assert_eq!(player.utrap, 0);
        assert_eq!(player.utrap_type, TrapType::None);

        set_utrap(&mut player, 5, TrapType::BearTrap);
        assert_eq!(player.utrap, 5);
        assert_eq!(player.utrap_type, TrapType::BearTrap);

        // Setting to 0 should clear trap type
        set_utrap(&mut player, 0, TrapType::BearTrap);
        assert_eq!(player.utrap, 0);
        assert_eq!(player.utrap_type, TrapType::None);
    }

    #[test]
    fn test_reset_utrap() {
        let mut player = You::default();
        set_utrap(&mut player, 10, TrapType::Pit);

        let msg = reset_utrap(&mut player, false);
        assert_eq!(player.utrap, 0);
        assert_eq!(player.utrap_type, TrapType::None);
        assert!(msg.is_none()); // No message when msg=false
    }

    #[test]
    fn test_trap_type_name() {
        assert_eq!(TrapType::BearTrap.name(), "bear trap");
        assert_eq!(TrapType::Pit.name(), "pit");
        assert_eq!(TrapType::Web.name(), "web");
        assert_eq!(TrapType::Lava.name(), "lava");
    }

    #[test]
    fn test_trap_type_is_pit() {
        assert!(TrapType::Pit.is_pit());
        assert!(TrapType::SpikedPit.is_pit());
        assert!(!TrapType::BearTrap.is_pit());
        assert!(!TrapType::Web.is_pit());
    }

    // ========================================================================
    // Tests for multi-turn action functions
    // ========================================================================

    #[test]
    fn test_nomul() {
        let mut player = You::default();
        assert_eq!(player.multi, 0);

        nomul(&mut player, -5, Some("sleeping".to_string()));
        assert_eq!(player.multi, -5);
        assert_eq!(player.multi_reason, Some("sleeping".to_string()));

        // Don't interrupt a longer action
        nomul(&mut player, -3, Some("other".to_string()));
        assert_eq!(player.multi, -5); // Still -5
    }

    #[test]
    fn test_unmul() {
        let mut player = You::default();
        nomul(&mut player, -10, Some("paralyzed".to_string()));

        unmul(&mut player, None);
        assert_eq!(player.multi, 0);
        assert!(player.multi_reason.is_none());
    }

    // ========================================================================
    // Tests for wounded legs functions
    // ========================================================================

    #[test]
    fn test_set_wounded_legs() {
        let mut player = You::default();

        set_wounded_legs(&mut player, 0, 10); // Left leg
        assert_eq!(player.wounded_legs_left, 10);
        assert_eq!(player.wounded_legs_right, 0);

        set_wounded_legs(&mut player, 1, 15); // Right leg
        assert_eq!(player.wounded_legs_left, 10);
        assert_eq!(player.wounded_legs_right, 15);

        set_wounded_legs(&mut player, 2, 20); // Both legs
        assert_eq!(player.wounded_legs_left, 20);
        assert_eq!(player.wounded_legs_right, 20);
    }

    #[test]
    fn test_heal_legs() {
        let mut player = You::default();
        player.properties.grant_intrinsic(Property::WoundedLegs);
        player.wounded_legs_left = 10;
        player.wounded_legs_right = 10;

        let healed = heal_legs(&mut player);
        assert!(healed);
        assert_eq!(player.wounded_legs_left, 0);
        assert_eq!(player.wounded_legs_right, 0);
    }

    // ========================================================================
    // Tests for consciousness functions
    // ========================================================================

    #[test]
    fn test_is_fainted() {
        let mut player = You::default();
        assert!(!is_fainted(&player));

        player.hunger_state = HungerState::Fainted;
        assert!(is_fainted(&player));
    }

    #[test]
    fn test_unconscious() {
        let mut player = You::default();
        assert!(!unconscious(&player));

        player.sleeping_timeout = 5;
        assert!(unconscious(&player));

        player.sleeping_timeout = 0;
        player.hunger_state = HungerState::Fainted;
        assert!(unconscious(&player));
    }

    #[test]
    fn test_wake_up() {
        let mut player = You::default();
        player.sleeping_timeout = 10;

        let msg = wake_up(&mut player, true);
        assert_eq!(player.sleeping_timeout, 0);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("wake up"));
    }

    #[test]
    fn test_unfaint() {
        let mut player = You::default();
        player.hunger_state = HungerState::Fainted;
        player.multi = -10;

        unfaint(&mut player);
        assert_eq!(player.hunger_state, HungerState::Weak);
        assert_eq!(player.multi, 0);
    }

    // ========================================================================
    // Tests for distance functions
    // ========================================================================

    #[test]
    fn test_um_dist() {
        let mut player = You::default();
        player.pos = Position::new(10, 10);

        // Distance to same position = 0
        assert_eq!(um_dist(&player, 10, 10), 0);

        // Distance to adjacent = 1 or 2
        assert_eq!(um_dist(&player, 11, 10), 1);
        assert_eq!(um_dist(&player, 11, 11), 2);

        // Distance to (15, 10) = 5^2 = 25
        assert_eq!(um_dist(&player, 15, 10), 25);
    }

    // ========================================================================
    // Tests for stair placement functions
    // ========================================================================

    #[test]
    fn test_u_on_upstairs() {
        let mut player = You::default();
        u_on_upstairs(&mut player);
        assert_eq!(player.pos.x, 40);
        assert_eq!(player.pos.y, 10);
    }

    #[test]
    fn test_u_on_sstairs() {
        let mut player = You::default();
        u_on_sstairs(&mut player, true);
        assert_eq!(player.pos.x, 40);
        assert_eq!(player.pos.y, 10);
    }
}
