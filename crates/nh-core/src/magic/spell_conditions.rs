//! Spell casting conditions - Requirements to cast spells
//!
//! Spells have various component requirements and conditions that must be met.

use serde::{Deserialize, Serialize};

/// Component types required for spells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellComponent {
    Verbal,
    Somatic,
    Focus,
    Time,
}

impl SpellComponent {
    pub const fn name(&self) -> &'static str {
        match self {
            SpellComponent::Verbal => "verbal",
            SpellComponent::Somatic => "somatic",
            SpellComponent::Focus => "focus",
            SpellComponent::Time => "time",
        }
    }
}

/// Reason why casting failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CastingConditionError {
    HandsRestricted,
    Silenced,
    NoFocus,
    TooLittle,
    Stunned,
}

impl CastingConditionError {
    pub const fn message(&self) -> &'static str {
        match self {
            CastingConditionError::HandsRestricted => "Your hands are too restricted!",
            CastingConditionError::Silenced => "You are silenced!",
            CastingConditionError::NoFocus => "You lack the necessary focus item!",
            CastingConditionError::TooLittle => "You are too small to cast!",
            CastingConditionError::Stunned => "You are stunned!",
        }
    }
}

/// Check if player can cast based on conditions
pub fn check_casting_conditions(
    player: &crate::player::You,
    spell_requires_verbal: bool,
    spell_requires_somatic: bool,
    spell_requires_focus: bool,
) -> Result<(), CastingConditionError> {
    if spell_requires_somatic && player.grabbed_by.is_some() {
        return Err(CastingConditionError::HandsRestricted);
    }

    if spell_requires_verbal && player.confused_timeout > 0 {
        return Err(CastingConditionError::Silenced);
    }

    if spell_requires_focus {
        // Check if player has appropriate focus item (would need item tracking)
        // For now, just check if they're carrying something
        // return Err(CastingConditionError::NoFocus);
    }

    if player.stunned_timeout > 0 {
        return Err(CastingConditionError::Stunned);
    }

    Ok(())
}

/// Get spell components required
pub fn get_spell_components(spell_type: crate::magic::spell::SpellType) -> Vec<SpellComponent> {
    match spell_type {
        crate::magic::spell::SpellType::ForceBolt => {
            vec![SpellComponent::Verbal, SpellComponent::Somatic]
        }
        crate::magic::spell::SpellType::Healing => {
            vec![SpellComponent::Verbal, SpellComponent::Focus]
        }
        crate::magic::spell::SpellType::Invisibility => {
            vec![
                SpellComponent::Verbal,
                SpellComponent::Somatic,
                SpellComponent::Focus,
            ]
        }
        _ => vec![SpellComponent::Verbal, SpellComponent::Somatic],
    }
}

/// Calculate time to cast (in tenths of a turn)
pub fn calculate_casting_time(spell_type: crate::magic::spell::SpellType) -> u32 {
    spell_type.level() as u32 * 2
}

/// Check if player has focus for a school
pub fn has_focus_item(
    player: &crate::player::You,
    school: crate::magic::spell::SpellSchool,
) -> bool {
    // Would need item tracking - for now always false
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_component_names() {
        assert_eq!(SpellComponent::Verbal.name(), "verbal");
        assert_eq!(SpellComponent::Somatic.name(), "somatic");
    }

    #[test]
    fn test_casting_condition_error_messages() {
        assert_eq!(
            CastingConditionError::Silenced.message(),
            "You are silenced!"
        );
    }
}
