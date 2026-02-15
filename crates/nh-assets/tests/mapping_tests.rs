use nh_assets::mapping::*;
use serde_json;

#[test]
fn test_serialize_icon_definition() {
    let def = ItemIconDefinition {
        tui_char: '!',
        tui_color: "yellow".to_string(),
        bevy_sprite: "sprites/potion_yellow.png".to_string(),
    };
    
    let json = serde_json::to_string(&def).unwrap();
    assert!(json.contains(r#""tui_char":"!""#));
    assert!(json.contains(r#""tui_color":"yellow""#));
    assert!(json.contains(r#""bevy_sprite":"sprites/potion_yellow.png""#));
}

#[test]
fn test_deserialize_mapping() {
    let json = r#"{
        "mappings": [
            {
                "identifier": {
                    "object_type": 100
                },
                "icon": {
                    "tui_char": "(",
                    "tui_color": "gray",
                    "bevy_sprite": "sprites/sword.png"
                }
            }
        ]
    }"#;
    
    let mapping: AssetMapping = serde_json::from_str(json).unwrap();
    assert_eq!(mapping.mappings.len(), 1);
    let m = &mapping.mappings[0];
    assert_eq!(m.identifier.object_type, Some(100));
    assert_eq!(m.icon.tui_char, '(');
}
