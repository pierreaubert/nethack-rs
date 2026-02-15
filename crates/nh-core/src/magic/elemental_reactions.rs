//! Elemental reaction system - Elements interact with each other
//!
//! When different elemental spells affect the same area, they can react to create
//! new effects: amplification, cancellation, transformation, explosions, or chain reactions.

use serde::{Deserialize, Serialize};

/// Basic element types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementType {
    Fire,
    Cold,
    Lightning,
    Acid,
    Poison,
    Force,
    Necrotic,
    Radiant,
    Water,
}

impl ElementType {
    /// Get all element types
    pub fn all() -> &'static [ElementType] {
        &[
            ElementType::Fire,
            ElementType::Cold,
            ElementType::Lightning,
            ElementType::Acid,
            ElementType::Poison,
            ElementType::Force,
            ElementType::Necrotic,
            ElementType::Radiant,
            ElementType::Water,
        ]
    }

    /// Get element name
    pub const fn name(&self) -> &'static str {
        match self {
            ElementType::Fire => "fire",
            ElementType::Cold => "cold",
            ElementType::Lightning => "lightning",
            ElementType::Acid => "acid",
            ElementType::Poison => "poison",
            ElementType::Force => "force",
            ElementType::Necrotic => "necrotic",
            ElementType::Radiant => "radiant",
            ElementType::Water => "water",
        }
    }
}

/// Types of elemental reactions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReactionType {
    /// Elements amplify each other's effects
    Amplification,
    /// Elements cancel each other out
    Cancellation,
    /// One element transforms into another
    Transformation,
    /// Elements create a small explosion
    Explosion,
    /// Elements create a chain reaction
    ChainReaction,
}

impl ReactionType {
    /// Get reaction name
    pub const fn name(&self) -> &'static str {
        match self {
            ReactionType::Amplification => "amplification",
            ReactionType::Cancellation => "cancellation",
            ReactionType::Transformation => "transformation",
            ReactionType::Explosion => "explosion",
            ReactionType::ChainReaction => "chain reaction",
        }
    }
}

/// A specific elemental reaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementalReaction {
    pub element1: ElementType,
    pub element2: ElementType,
    pub reaction: ReactionType,
    pub damage_modifier: f32,
    pub area_modifier: f32,
}

impl ElementalReaction {
    /// Get all known reactions
    pub fn all_reactions() -> &'static [ElementalReaction] {
        &[
            // Amplifications
            ElementalReaction {
                element1: ElementType::Fire,
                element2: ElementType::Fire,
                reaction: ReactionType::Amplification,
                damage_modifier: 1.5,
                area_modifier: 1.25,
            },
            ElementalReaction {
                element1: ElementType::Lightning,
                element2: ElementType::Fire,
                reaction: ReactionType::Amplification,
                damage_modifier: 1.3,
                area_modifier: 1.2,
            },
            // Cancellations
            ElementalReaction {
                element1: ElementType::Fire,
                element2: ElementType::Cold,
                reaction: ReactionType::Cancellation,
                damage_modifier: 0.0,
                area_modifier: 0.5,
            },
            ElementalReaction {
                element1: ElementType::Fire,
                element2: ElementType::Acid,
                reaction: ReactionType::Cancellation,
                damage_modifier: 0.5,
                area_modifier: 0.7,
            },
            // Transformations
            ElementalReaction {
                element1: ElementType::Lightning,
                element2: ElementType::Water,
                reaction: ReactionType::Transformation,
                damage_modifier: 2.0,
                area_modifier: 1.0,
            },
            // Explosions
            ElementalReaction {
                element1: ElementType::Acid,
                element2: ElementType::Force,
                reaction: ReactionType::Explosion,
                damage_modifier: 2.5,
                area_modifier: 2.0,
            },
            ElementalReaction {
                element1: ElementType::Lightning,
                element2: ElementType::Lightning,
                reaction: ReactionType::Explosion,
                damage_modifier: 1.8,
                area_modifier: 1.5,
            },
        ]
    }

    /// Check if two elements react
    pub fn check_reaction(elem1: ElementType, elem2: ElementType) -> Option<ElementalReaction> {
        for reaction in Self::all_reactions() {
            if (reaction.element1 == elem1 && reaction.element2 == elem2)
                || (reaction.element1 == elem2 && reaction.element2 == elem1)
            {
                return Some(*reaction);
            }
        }
        None
    }
}

/// Recent elemental activity by position
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementalReactionTracker {
    /// Recent elements by position: (x, y) -> (element, turn_number)
    pub recent_elements: hashbrown::HashMap<(i8, i8), Vec<(ElementType, u32)>>,
    /// Current turn number for aging effects
    pub current_turn: u32,
}

impl ElementalReactionTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an elemental effect at a position
    pub fn record_element(&mut self, x: i8, y: i8, element: ElementType) {
        let entry = self.recent_elements.entry((x, y)).or_insert_with(Vec::new);
        entry.push((element, self.current_turn));

        // Keep only last 10 effects at a position
        if entry.len() > 10 {
            entry.remove(0);
        }
    }

    /// Get recent elements at a position (last N turns)
    pub fn get_recent_elements(&self, x: i8, y: i8, last_turns: u32) -> Vec<ElementType> {
        self.recent_elements
            .get(&(x, y))
            .map(|effects| {
                effects
                    .iter()
                    .filter(|(_, turn)| self.current_turn.saturating_sub(*turn) <= last_turns)
                    .map(|(elem, _)| *elem)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check for reactions at a position
    pub fn check_position_reactions(&self, x: i8, y: i8) -> Vec<ElementalReaction> {
        let recent = self.get_recent_elements(x, y, 2); // Look at last 2 turns
        let mut reactions = Vec::new();

        for i in 0..recent.len() {
            for j in (i + 1)..recent.len() {
                if let Some(reaction) = ElementalReaction::check_reaction(recent[i], recent[j]) {
                    reactions.push(reaction);
                }
            }
        }

        reactions
    }

    /// Advance tracker to next turn
    pub fn next_turn(&mut self) {
        self.current_turn += 1;

        // Clean up old effects (older than 10 turns)
        for effects in self.recent_elements.values_mut() {
            effects.retain(|(_, turn)| self.current_turn.saturating_sub(*turn) <= 10);
        }
    }

    /// Clear all tracked elements
    pub fn clear(&mut self) {
        self.recent_elements.clear();
        self.current_turn = 0;
    }
}

/// Environmental hazard created by reactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalHazard {
    pub x: i8,
    pub y: i8,
    pub element: ElementType,
    pub damage_per_turn: i32,
    pub remaining_turns: u32,
}

impl EnvironmentalHazard {
    /// Create new environmental hazard
    pub fn new(x: i8, y: i8, element: ElementType, damage: i32, duration: u32) -> Self {
        Self {
            x,
            y,
            element,
            damage_per_turn: damage,
            remaining_turns: duration,
        }
    }

    /// Advance the hazard by one turn
    pub fn tick(&mut self) {
        if self.remaining_turns > 0 {
            self.remaining_turns -= 1;
        }
    }

    /// Check if hazard is still active
    pub fn is_active(&self) -> bool {
        self.remaining_turns > 0
    }
}

/// Create a hazard from elemental reaction
pub fn create_environmental_hazard(
    element: ElementType,
    damage: i32,
    x: i8,
    y: i8,
    duration: u32,
) -> EnvironmentalHazard {
    EnvironmentalHazard::new(x, y, element, damage, duration)
}

/// Apply reaction damage and effects
pub fn apply_reaction_damage(reaction: ElementalReaction, base_damage: i32) -> i32 {
    (base_damage as f32 * reaction.damage_modifier) as i32
}

/// Get reaction description
pub fn get_reaction_message(reaction: ElementalReaction) -> String {
    match reaction.reaction {
        ReactionType::Amplification => {
            format!(
                "The {} and {} magics amplify each other!",
                reaction.element1.name(),
                reaction.element2.name()
            )
        }
        ReactionType::Cancellation => {
            format!(
                "The {} and {} effects cancel out!",
                reaction.element1.name(),
                reaction.element2.name()
            )
        }
        ReactionType::Transformation => {
            format!(
                "The {} transforms into {}!",
                reaction.element1.name(),
                reaction.element2.name()
            )
        }
        ReactionType::Explosion => {
            format!(
                "The {} and {} create a violent explosion!",
                reaction.element1.name(),
                reaction.element2.name()
            )
        }
        ReactionType::ChainReaction => {
            format!(
                "A chain reaction of {} and {} magic!",
                reaction.element1.name(),
                reaction.element2.name()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_type_names() {
        assert_eq!(ElementType::Fire.name(), "fire");
        assert_eq!(ElementType::Lightning.name(), "lightning");
    }

    #[test]
    fn test_reaction_type_names() {
        assert_eq!(ReactionType::Amplification.name(), "amplification");
        assert_eq!(ReactionType::Cancellation.name(), "cancellation");
    }

    #[test]
    fn test_elemental_reaction_fire_cold() {
        let reaction = ElementalReaction::check_reaction(ElementType::Fire, ElementType::Cold);
        assert!(reaction.is_some());
        assert_eq!(reaction.unwrap().reaction, ReactionType::Cancellation);
    }

    #[test]
    fn test_elemental_reaction_tracker_creation() {
        let tracker = ElementalReactionTracker::new();
        assert_eq!(tracker.current_turn, 0);
        assert!(tracker.recent_elements.is_empty());
    }

    #[test]
    fn test_elemental_reaction_tracker_record() {
        let mut tracker = ElementalReactionTracker::new();
        tracker.record_element(5, 10, ElementType::Fire);
        let recent = tracker.get_recent_elements(5, 10, 5);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0], ElementType::Fire);
    }

    #[test]
    fn test_environmental_hazard_creation() {
        let hazard = EnvironmentalHazard::new(5, 10, ElementType::Fire, 10, 5);
        assert_eq!(hazard.x, 5);
        assert_eq!(hazard.y, 10);
        assert_eq!(hazard.damage_per_turn, 10);
        assert_eq!(hazard.remaining_turns, 5);
        assert!(hazard.is_active());
    }

    #[test]
    fn test_environmental_hazard_tick() {
        let mut hazard = EnvironmentalHazard::new(5, 10, ElementType::Fire, 10, 3);
        hazard.tick();
        assert_eq!(hazard.remaining_turns, 2);
        hazard.tick();
        hazard.tick();
        assert_eq!(hazard.remaining_turns, 0);
        assert!(!hazard.is_active());
    }

    #[test]
    fn test_apply_reaction_damage() {
        let reaction = ElementalReaction::check_reaction(ElementType::Lightning, ElementType::Fire)
            .expect("Should have amplification reaction");
        let damage = apply_reaction_damage(reaction, 100);
        assert!(damage > 100);
    }
}
