use nh_assets::registry::*;
use nh_assets::mapping::*;
use nh_core::object::{Object, ObjectClass, Material, ObjectId};

#[test]
fn test_registry_lookup_by_type() {
    let mapping = AssetMapping {
        mappings: vec![
            AssetMappingEntry {
                identifier: ItemIdentifier {
                    object_type: Some(100),
                    ..Default::default()
                },
                icon: ItemIconDefinition {
                    tui_char: '(',
                    tui_color: "gray".to_string(),
                    bevy_sprite: "sword.png".to_string(),
                },
            }
        ],
    };
    
    let registry = AssetRegistry::new(mapping);
    let mut obj = Object::new(ObjectId(1), 100, ObjectClass::Weapon);
    
    let icon = registry.get_icon(&obj).expect("Icon should be found");
    assert_eq!(icon.tui_char, '(');
}

#[test]
fn test_registry_fallback_to_class() {
    let mapping = AssetMapping {
        mappings: vec![
            AssetMappingEntry {
                identifier: ItemIdentifier {
                    class: Some(ObjectClass::Weapon),
                    ..Default::default()
                },
                icon: ItemIconDefinition {
                    tui_char: ')',
                    tui_color: "white".to_string(),
                    bevy_sprite: "weapon.png".to_string(),
                },
            }
        ],
    };
    
    let registry = AssetRegistry::new(mapping);
    let mut obj = Object::new(ObjectId(1), 999, ObjectClass::Weapon);
    
    let icon = registry.get_icon(&obj).expect("Icon should be found by class fallback");
    assert_eq!(icon.tui_char, ')');
}

#[test]
fn test_registry_specificity_priority() {
    let mapping = AssetMapping {
        mappings: vec![
            AssetMappingEntry {
                identifier: ItemIdentifier {
                    class: Some(ObjectClass::Weapon),
                    ..Default::default()
                },
                icon: ItemIconDefinition {
                    tui_char: ')',
                    tui_color: "white".to_string(),
                    bevy_sprite: "weapon.png".to_string(),
                },
            },
            AssetMappingEntry {
                identifier: ItemIdentifier {
                    object_type: Some(100),
                    ..Default::default()
                },
                icon: ItemIconDefinition {
                    tui_char: '(',
                    tui_color: "gray".to_string(),
                    bevy_sprite: "sword.png".to_string(),
                },
            }
        ],
    };
    
    let registry = AssetRegistry::new(mapping);
    let mut obj = Object::new(ObjectId(1), 100, ObjectClass::Weapon);
    
    let icon = registry.get_icon(&obj).expect("Icon should be found");
    // Should prefer object_type (specificity 10) over class (specificity 1)
    assert_eq!(icon.tui_char, '(');
}
