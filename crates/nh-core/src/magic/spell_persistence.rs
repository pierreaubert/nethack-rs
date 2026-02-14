//! Spell persistence system - Lingering zone effects from spells
//!
//! Spells create persistent effects that linger on the level and affect anything in range.

use serde::{Deserialize, Serialize};

/// Types of persistent spell effects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistentEffectType {
    DamageZone,
    HealingCircle,
    SlowField,
    PoisonCloud,
    HolyGround,
    CurseField,
}

impl PersistentEffectType {
    pub const fn name(&self) -> &'static str {
        match self {
            PersistentEffectType::DamageZone => "Damage Zone",
            PersistentEffectType::HealingCircle => "Healing Circle",
            PersistentEffectType::SlowField => "Slow Field",
            PersistentEffectType::PoisonCloud => "Poison Cloud",
            PersistentEffectType::HolyGround => "Holy Ground",
            PersistentEffectType::CurseField => "Curse Field",
        }
    }
}

/// Persistent spell effect on level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSpellEffect {
    pub center_x: i8,
    pub center_y: i8,
    pub effect_type: PersistentEffectType,
    pub radius: i32,
    pub duration: u32,
    pub damage_or_heal: i32,
    pub caster_level: u32,
}

impl PersistentSpellEffect {
    pub fn new(
        x: i8,
        y: i8,
        effect_type: PersistentEffectType,
        radius: i32,
        duration: u32,
        power: i32,
    ) -> Self {
        Self {
            center_x: x,
            center_y: y,
            effect_type,
            radius,
            duration,
            damage_or_heal: power,
            caster_level: 0,
        }
    }

    pub fn contains(&self, x: i8, y: i8) -> bool {
        let dx = (self.center_x - x).abs() as i32;
        let dy = (self.center_y - y).abs() as i32;
        dx * dx + dy * dy <= self.radius * self.radius
    }

    pub fn tick(&mut self) {
        if self.duration > 0 {
            self.duration -= 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.duration > 0
    }
}

/// Tracker for all persistent effects on a level
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistentEffectTracker {
    pub effects: Vec<PersistentSpellEffect>,
}

impl PersistentEffectTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_effect(&mut self, effect: PersistentSpellEffect) {
        if self.effects.len() < 50 {
            // Cap at 50 persistent effects
            self.effects.push(effect);
        }
    }

    pub fn tick_all(&mut self) {
        for effect in &mut self.effects {
            effect.tick();
        }
        self.effects.retain(|e| e.is_active());
    }

    pub fn get_effects_at(&self, x: i8, y: i8) -> Vec<&PersistentSpellEffect> {
        self.effects.iter().filter(|e| e.contains(x, y)).collect()
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }

    pub fn count(&self) -> usize {
        self.effects.len()
    }
}

pub fn create_persistent_effect(
    x: i8,
    y: i8,
    effect_type: PersistentEffectType,
    radius: i32,
    duration: u32,
    power: i32,
) -> PersistentSpellEffect {
    PersistentSpellEffect::new(x, y, effect_type, radius, duration, power)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persistent_spell_effect() {
        let effect = PersistentSpellEffect::new(5, 10, PersistentEffectType::DamageZone, 3, 5, 20);
        assert!(effect.contains(5, 10));
        assert!(effect.contains(6, 10));
        assert!(!effect.contains(10, 10));
    }

    #[test]
    fn test_persistent_effect_tracker() {
        let mut tracker = PersistentEffectTracker::new();
        let effect =
            PersistentSpellEffect::new(5, 10, PersistentEffectType::HealingCircle, 3, 3, 15);
        tracker.add_effect(effect);
        assert_eq!(tracker.count(), 1);

        tracker.tick_all();
        assert_eq!(tracker.count(), 1);

        for _ in 0..2 {
            tracker.tick_all();
        }
        assert_eq!(tracker.count(), 0);
    }
}
