//! Magical item identification system
//!
//! Tracks player knowledge about magical items, identification types,
//! and manages item name discovery through use and research.

use serde::{Deserialize, Serialize};

/// Item identification state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentificationLevel {
    Unknown,     // Item name unknown
    Discovered,  // Name discovered through use
    Identified,  // Name confirmed through ID scroll
    FullyMapped, // All properties known
}

/// Knowledge tracking for a specific item type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemKnowledge {
    pub object_type: i16,
    pub identification_level: IdentificationLevel,
    pub times_used: i32,
    pub times_tested: i32,
    pub discovered_benefits: Vec<String>,
    pub discovered_hazards: Vec<String>,
}

impl ItemKnowledge {
    pub fn new(object_type: i16) -> Self {
        Self {
            object_type,
            identification_level: IdentificationLevel::Unknown,
            times_used: 0,
            times_tested: 0,
            discovered_benefits: Vec::new(),
            discovered_hazards: Vec::new(),
        }
    }

    /// Can identify item from use?
    pub fn can_auto_identify(&self) -> bool {
        self.times_used >= 3 || self.times_tested >= 2
    }
}

/// Player's item identification knowledge base
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdentificationKnowledge {
    pub known_items: std::collections::HashMap<i16, ItemKnowledge>,
}

impl IdentificationKnowledge {
    pub fn new() -> Self {
        Self {
            known_items: std::collections::HashMap::new(),
        }
    }

    /// Get knowledge about item type
    pub fn get_knowledge(&self, object_type: i16) -> Option<&ItemKnowledge> {
        self.known_items.get(&object_type)
    }

    /// Get or create knowledge
    pub fn get_or_create(&mut self, object_type: i16) -> &mut ItemKnowledge {
        self.known_items
            .entry(object_type)
            .or_insert_with(|| ItemKnowledge::new(object_type))
    }

    /// Mark item as used (discovery from use)
    pub fn mark_used(&mut self, object_type: i16) {
        let knowledge = self.get_or_create(object_type);
        knowledge.times_used += 1;

        // Auto-identify after enough use
        if knowledge.can_auto_identify()
            && knowledge.identification_level == IdentificationLevel::Unknown
        {
            knowledge.identification_level = IdentificationLevel::Discovered;
        }
    }

    /// Mark item as tested (tried but didn't use)
    pub fn mark_tested(&mut self, object_type: i16) {
        let knowledge = self.get_or_create(object_type);
        knowledge.times_tested += 1;
    }

    /// Identify item completely
    pub fn identify_item(&mut self, object_type: i16) {
        let knowledge = self.get_or_create(object_type);
        knowledge.identification_level = IdentificationLevel::Identified;
    }

    /// Record discovered benefit
    pub fn add_benefit(&mut self, object_type: i16, benefit: String) {
        let knowledge = self.get_or_create(object_type);
        if !knowledge.discovered_benefits.contains(&benefit) {
            knowledge.discovered_benefits.push(benefit);
        }
    }

    /// Record discovered hazard
    pub fn add_hazard(&mut self, object_type: i16, hazard: String) {
        let knowledge = self.get_or_create(object_type);
        if !knowledge.discovered_hazards.contains(&hazard) {
            knowledge.discovered_hazards.push(hazard);
        }
    }

    /// Check if item type is known
    pub fn is_known(&self, object_type: i16) -> bool {
        self.known_items
            .get(&object_type)
            .map(|k| k.identification_level != IdentificationLevel::Unknown)
            .unwrap_or(false)
    }

    /// Count identified items
    pub fn count_identified(&self) -> usize {
        self.known_items
            .values()
            .filter(|k| k.identification_level != IdentificationLevel::Unknown)
            .count()
    }
}

/// Identification result
#[derive(Debug, Clone)]
pub struct IdentificationResult {
    pub messages: Vec<String>,
    pub identified: bool,
    pub new_knowledge: bool,
}

impl IdentificationResult {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            identified: false,
            new_knowledge: false,
        }
    }

    pub fn with_message(mut self, msg: impl Into<String>) -> Self {
        self.messages.push(msg.into());
        self
    }
}

impl Default for IdentificationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Identify an item through scroll or spell
pub fn identify_from_scroll(
    knowledge: &mut IdentificationKnowledge,
    object_type: i16,
) -> IdentificationResult {
    let mut result = IdentificationResult::new();
    let item_knowledge = knowledge.get_or_create(object_type);

    match item_knowledge.identification_level {
        IdentificationLevel::Unknown => {
            item_knowledge.identification_level = IdentificationLevel::Identified;
            result.identified = true;
            result.new_knowledge = true;
            result
                .messages
                .push("The scroll reveals the item's true nature!".to_string());
        }
        IdentificationLevel::Discovered => {
            item_knowledge.identification_level = IdentificationLevel::Identified;
            result.identified = true;
            result
                .messages
                .push("You confirm what you suspected about this item.".to_string());
        }
        IdentificationLevel::Identified => {
            result
                .messages
                .push("You already know what this item is.".to_string());
        }
        IdentificationLevel::FullyMapped => {
            result
                .messages
                .push("You already know everything about this item.".to_string());
        }
    }

    result
}

/// Identify an item through experience
pub fn identify_from_use(
    knowledge: &mut IdentificationKnowledge,
    object_type: i16,
) -> IdentificationResult {
    let mut result = IdentificationResult::new();
    let item_knowledge = knowledge.get_or_create(object_type);

    item_knowledge.times_used += 1;

    if item_knowledge.identification_level == IdentificationLevel::Unknown
        && item_knowledge.times_used >= 3
    {
        item_knowledge.identification_level = IdentificationLevel::Discovered;
        result.identified = true;
        result.new_knowledge = true;
        result
            .messages
            .push("You've learned what this item does!".to_string());
    } else if item_knowledge.identification_level == IdentificationLevel::Unknown {
        result.messages.push(format!(
            "You're getting a sense of what this item does... ({} more uses to discover)",
            3 - item_knowledge.times_used
        ));
    }

    result
}

/// Get item description based on identification level
pub fn get_identified_description(
    knowledge: &IdentificationKnowledge,
    object_type: i16,
    base_name: &str,
) -> String {
    match knowledge.get_knowledge(object_type) {
        None => format!("unidentified {}", base_name),
        Some(item_knowledge) => match item_knowledge.identification_level {
            IdentificationLevel::Unknown => format!("unidentified {}", base_name),
            IdentificationLevel::Discovered => format!("{} (probably)", base_name),
            IdentificationLevel::Identified => base_name.to_string(),
            IdentificationLevel::FullyMapped => format!("{} (fully known)", base_name),
        },
    }
}

/// Get identification progress for item
pub fn get_identification_progress(knowledge: &IdentificationKnowledge, object_type: i16) -> i32 {
    match knowledge.get_knowledge(object_type) {
        None => 0,
        Some(item_knowledge) => match item_knowledge.identification_level {
            IdentificationLevel::Unknown => 0,
            IdentificationLevel::Discovered => 50,
            IdentificationLevel::Identified => 75,
            IdentificationLevel::FullyMapped => 100,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_knowledge_creation() {
        let knowledge = ItemKnowledge::new(100);
        assert_eq!(knowledge.object_type, 100);
        assert_eq!(knowledge.identification_level, IdentificationLevel::Unknown);
    }

    #[test]
    fn test_can_auto_identify() {
        let mut knowledge = ItemKnowledge::new(100);
        assert!(!knowledge.can_auto_identify());

        knowledge.times_used = 3;
        assert!(knowledge.can_auto_identify());

        knowledge.times_used = 0;
        knowledge.times_tested = 2;
        assert!(knowledge.can_auto_identify());
    }

    #[test]
    fn test_identification_knowledge_get_or_create() {
        let mut kb = IdentificationKnowledge::new();
        assert!(kb.get_knowledge(100).is_none());

        let _ = kb.get_or_create(100);
        assert!(kb.get_knowledge(100).is_some());
    }

    #[test]
    fn test_mark_used_auto_identify() {
        let mut kb = IdentificationKnowledge::new();

        kb.mark_used(100);
        kb.mark_used(100);
        assert_eq!(
            kb.get_knowledge(100).unwrap().identification_level,
            IdentificationLevel::Unknown
        );

        kb.mark_used(100);
        assert_eq!(
            kb.get_knowledge(100).unwrap().identification_level,
            IdentificationLevel::Discovered
        );
    }

    #[test]
    fn test_identify_item() {
        let mut kb = IdentificationKnowledge::new();
        kb.identify_item(100);

        let knowledge = kb.get_knowledge(100).unwrap();
        assert_eq!(
            knowledge.identification_level,
            IdentificationLevel::Identified
        );
    }

    #[test]
    fn test_add_benefit() {
        let mut kb = IdentificationKnowledge::new();
        kb.add_benefit(100, "increases strength".to_string());

        let benefits = &kb.get_knowledge(100).unwrap().discovered_benefits;
        assert_eq!(benefits.len(), 1);
        assert!(benefits.contains(&"increases strength".to_string()));
    }

    #[test]
    fn test_add_benefit_duplicate() {
        let mut kb = IdentificationKnowledge::new();
        kb.add_benefit(100, "increases strength".to_string());
        kb.add_benefit(100, "increases strength".to_string());

        let benefits = &kb.get_knowledge(100).unwrap().discovered_benefits;
        assert_eq!(benefits.len(), 1); // Should not duplicate
    }

    #[test]
    fn test_is_known() {
        let mut kb = IdentificationKnowledge::new();
        assert!(!kb.is_known(100));

        kb.identify_item(100);
        assert!(kb.is_known(100));
    }

    #[test]
    fn test_count_identified() {
        let mut kb = IdentificationKnowledge::new();
        kb.identify_item(100);
        kb.identify_item(200);
        kb.identify_item(300);

        assert_eq!(kb.count_identified(), 3);
    }

    #[test]
    fn test_identify_from_scroll() {
        let mut kb = IdentificationKnowledge::new();
        let result = identify_from_scroll(&mut kb, 100);

        assert!(result.identified);
        assert!(result.new_knowledge);
        assert_eq!(
            kb.get_knowledge(100).unwrap().identification_level,
            IdentificationLevel::Identified
        );
    }

    #[test]
    fn test_identify_from_use() {
        let mut kb = IdentificationKnowledge::new();

        let result1 = identify_from_use(&mut kb, 100);
        assert!(!result1.identified);

        identify_from_use(&mut kb, 100);
        let result3 = identify_from_use(&mut kb, 100);

        assert!(result3.identified);
        assert_eq!(
            kb.get_knowledge(100).unwrap().identification_level,
            IdentificationLevel::Discovered
        );
    }

    #[test]
    fn test_get_identified_description_unknown() {
        let kb = IdentificationKnowledge::new();
        let desc = get_identified_description(&kb, 100, "sword");
        assert_eq!(desc, "unidentified sword");
    }

    #[test]
    fn test_get_identified_description_identified() {
        let mut kb = IdentificationKnowledge::new();
        kb.identify_item(100);

        let desc = get_identified_description(&kb, 100, "sword of sharpness");
        assert_eq!(desc, "sword of sharpness");
    }

    #[test]
    fn test_get_identification_progress() {
        let mut kb = IdentificationKnowledge::new();

        assert_eq!(get_identification_progress(&kb, 100), 0);

        kb.mark_used(100);
        kb.mark_used(100);
        kb.mark_used(100);
        assert_eq!(get_identification_progress(&kb, 100), 50);

        kb.identify_item(100);
        assert_eq!(get_identification_progress(&kb, 100), 75);
    }

    #[test]
    fn test_mark_tested() {
        let mut kb = IdentificationKnowledge::new();
        kb.mark_tested(100);

        assert_eq!(kb.get_knowledge(100).unwrap().times_tested, 1);
    }
}
