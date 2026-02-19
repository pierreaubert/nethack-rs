use nh_assets::registry::AssetRegistry;
use nh_core::data::objects::OBJECTS;
use nh_core::object::{Object, ObjectId};
use strum::IntoEnumIterator;
use nh_core::object::ObjectClass;

#[test]
fn test_coverage_all_classes_mapped() {
    let assets_path = "../../assets/mapping.json";
    // We expect this to be run from crates/nh-assets
    let registry = AssetRegistry::load_from_file(assets_path).expect("Failed to load mapping.json");
    
    let mut missing_classes = Vec::new();
    
    for class in ObjectClass::iter() {
        if matches!(class, ObjectClass::Random | ObjectClass::IllObj) {
            continue;
        }
        
        let obj = Object::new(ObjectId(0), 0, class);
        if registry.get_icon(&obj).is_err() {
            missing_classes.push(class);
        }
    }
    
    assert!(missing_classes.is_empty(), "Missing asset mappings for classes: {:?}", missing_classes);
}

#[test]
fn test_coverage_all_objects_mapped() {
    let assets_path = "../../assets/mapping.json";
    let registry = AssetRegistry::load_from_file(assets_path).expect("Failed to load mapping.json");
    
    let mut unmapped_objects = Vec::new();
    
    for (i, def) in OBJECTS.iter().enumerate() {
        if matches!(def.class, ObjectClass::Random | ObjectClass::IllObj) {
            continue;
        }
        
        let mut obj = Object::new(ObjectId(0), i as i16, def.class);
        // NetHack core uses object_type as index into OBJECTS
        obj.object_type = i as i16;
        
        if registry.get_icon(&obj).is_err() {
            unmapped_objects.push(def.name);
        }
    }
    
    // We currently only have class-based mapping for most things, so this should pass
    // if all classes are covered.
    assert!(unmapped_objects.is_empty(), "Objects without any matching icon: {:?}", unmapped_objects);
}
