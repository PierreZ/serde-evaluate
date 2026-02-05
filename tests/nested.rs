use serde::Serialize;
use serde_evaluate::{
    error::EvaluateError, extractor::NestedFieldExtractor, value::FieldScalarValue,
};
use std::collections::BTreeMap;

// Define nested structures for testing

#[derive(Serialize)]
struct Level3 {
    target_field: i32,
    other_field: String,
}

#[derive(Serialize)]
struct Level2 {
    level3: Level3,
    another_val: bool,
}

#[derive(Serialize)]
struct Level1 {
    level2: Level2,
    id: u64,
}

#[derive(Serialize)]
struct Inner {
    value: i32,
    another_value: Option<String>,
    nested_struct: InnerMost,
}

#[derive(Serialize)]
struct InnerMost {
    deep_value: bool,
}

#[derive(Serialize)]
struct Middle {
    inner: Inner,
    primitive: u64,
}

#[derive(Serialize)]
struct Outer {
    middle: Middle,
    name: String,
}

#[derive(Serialize)]
struct MapValueStruct {
    value: i32,
    description: Option<String>,
}

#[derive(Serialize)]
struct MapTestData {
    id: String,
    data_map: BTreeMap<String, MapValueStruct>,
    simple_map: BTreeMap<String, u32>,
}

#[test]
fn test_extract_deeply_nested_field() {
    let data = Level1 {
        id: 101,
        level2: Level2 {
            another_val: true,
            level3: Level3 {
                target_field: 999,
                other_field: "hello".to_string(),
            },
        },
    };

    // Define the path to the target field
    let path = vec!["level2", "level3", "target_field"];

    // Create the extractor (this part should succeed)
    let extractor = serde_evaluate::NestedFieldExtractor::new_from_path(&path)
        .expect("Failed to create extractor");

    // Evaluate the extractor (this part will fail until evaluate is implemented)
    let result = extractor.evaluate(&data);
    assert_eq!(result, Ok(FieldScalarValue::I32(999)));
}

#[test]
fn test_extract_deeply_nested_field_comprehensive() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 123,
                another_value: Some("test".to_string()),
                nested_struct: InnerMost { deep_value: true },
            },
            primitive: 999,
        },
        name: "outer_name".to_string(),
    };

    // --- Valid Cases ---
    let extractor = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
        "middle", "inner", "value",
    ])
    .unwrap();
    let result = extractor.evaluate(&data);
    assert_eq!(result.unwrap(), FieldScalarValue::I32(123));

    let extractor_option = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
        "middle",
        "inner",
        "another_value",
    ])
    .unwrap();
    let result_option = extractor_option.evaluate(&data);
    assert_eq!(
        result_option.unwrap(),
        FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String("test".to_string()))))
    );

    let extractor_deepest = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
        "middle",
        "inner",
        "nested_struct",
        "deep_value",
    ])
    .unwrap();
    let result_deepest = extractor_deepest.evaluate(&data);
    assert_eq!(result_deepest.unwrap(), FieldScalarValue::Bool(true));

    // --- Error Cases ---

    // Non-existent intermediate field
    let extractor_bad_intermediate =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
            "middle",
            "non_existent",
            "value",
        ])
        .unwrap();
    let result_bad_intermediate = extractor_bad_intermediate.evaluate(&data);
    assert!(
        matches!(result_bad_intermediate, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["middle".to_string(), "non_existent".to_string(), "value".to_string()]),
        "Unexpected result for bad intermediate path: {:?}",
        result_bad_intermediate
    );

    // Non-existent final field
    let extractor_bad_final = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
        "middle",
        "inner",
        "non_existent",
    ])
    .unwrap();
    let result_bad_final = extractor_bad_final.evaluate(&data);
    assert!(
        matches!(result_bad_final, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["middle".to_string(), "inner".to_string(), "non_existent".to_string()]),
        "Unexpected result for bad final path: {:?}",
        result_bad_final
    );

    // Attempt to traverse through a non-struct primitive
    let extractor_traverse_primitive =
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
            "middle",
            "primitive",
            "should_fail",
        ])
        .unwrap();
    let result_traverse_primitive = extractor_traverse_primitive.evaluate(&data);
    assert!(
        matches!(result_traverse_primitive, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["middle".to_string(), "primitive".to_string(), "should_fail".to_string()]),
        "Expected NestedFieldNotFound, got {:?}",
        result_traverse_primitive
    );

    // Attempt to extract a non-scalar (struct) as the final value
    let extractor_non_scalar = serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&[
        "middle",
        "inner",
        "nested_struct",
    ])
    .unwrap();
    let result_non_scalar = extractor_non_scalar.evaluate(&data);
    assert!(matches!(
        result_non_scalar,
        Err(EvaluateError::UnsupportedType { type_name: _ })
    ));

    // Test path parsing errors handled by new_from_path
    assert!(
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&Vec::<&str>::new())
            .is_err()
    ); // Empty Vec
    assert!(
        serde_evaluate::extractor::NestedFieldExtractor::new_from_path(&["field1", "", "field3"])
            .is_err()
    ); // Empty segment
}

#[test]
fn test_extract_nested_map_field() {
    // Prepare test data
    let mut data_map = BTreeMap::new();
    data_map.insert(
        "entry1".to_string(),
        MapValueStruct {
            value: -10,
            description: None,
        },
    );
    data_map.insert(
        "entry2".to_string(),
        MapValueStruct {
            value: 20,
            description: Some("Second".to_string()),
        },
    );
    let mut simple_map = BTreeMap::new();
    simple_map.insert("simple1".to_string(), 100);
    simple_map.insert("simple2".to_string(), 200);

    let data = MapTestData {
        id: "map_test_data".to_string(),
        data_map,
        simple_map,
    };

    // --- Success Cases ---

    // Extract scalar from struct in map
    let extractor1 = NestedFieldExtractor::new_from_path(&["data_map", "entry1", "value"]).unwrap();
    let result1 = extractor1.evaluate(&data).unwrap();
    assert_eq!(result1, FieldScalarValue::I32(-10));

    // Extract Option<String> (Some) from struct in map
    let extractor2 =
        NestedFieldExtractor::new_from_path(&["data_map", "entry2", "description"]).unwrap();
    let result2 = extractor2.evaluate(&data).unwrap();
    assert_eq!(
        result2,
        FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String(
            "Second".to_string()
        ))))
    );

    // Extract Option<String> (None) from struct in map
    let extractor3 =
        NestedFieldExtractor::new_from_path(&["data_map", "entry1", "description"]).unwrap();
    let result3 = extractor3.evaluate(&data).unwrap();
    assert_eq!(result3, FieldScalarValue::Option(None));

    // Extract scalar from simple map
    let extractor_simple = NestedFieldExtractor::new_from_path(&["simple_map", "simple2"]).unwrap();
    let result_simple = extractor_simple.evaluate(&data).unwrap();
    assert_eq!(result_simple, FieldScalarValue::U32(200));

    // --- Error Cases ---

    // Map key not found
    let extractor_missing_key =
        NestedFieldExtractor::new_from_path(&["data_map", "missing_key", "value"]).unwrap();
    let result_missing_key = extractor_missing_key.evaluate(&data);
    assert!(
        matches!(result_missing_key, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["data_map".to_string(), "missing_key".to_string(), "value".to_string()]),
        "Unexpected result for missing key: {:?}",
        result_missing_key
    );

    // Field not found within map value struct
    let extractor_missing_inner =
        NestedFieldExtractor::new_from_path(&["data_map", "entry1", "bad_field"]).unwrap();
    let result_missing_inner = extractor_missing_inner.evaluate(&data);
    assert!(
        matches!(result_missing_inner, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["data_map".to_string(), "entry1".to_string(), "bad_field".to_string()]),
        "Unexpected result for missing inner field: {:?}",
        result_missing_inner
    );

    // Try to extract map itself
    let extractor_map = NestedFieldExtractor::new_from_path(&["data_map"]).unwrap();
    let result_map = extractor_map.evaluate(&data);
    assert!(
        matches!(result_map, Err(EvaluateError::UnsupportedType { .. })),
        "Unexpected result for extracting map: {:?}",
        result_map
    );

    // Try to extract map value struct itself
    let extractor_map_value = NestedFieldExtractor::new_from_path(&["data_map", "entry1"]).unwrap();
    let result_map_value = extractor_map_value.evaluate(&data);
    assert!(
        matches!(result_map_value, Err(EvaluateError::UnsupportedType { .. })),
        "Unexpected result for extracting map value: {:?}",
        result_map_value
    );

    // Try to extract non-existent key from simple map
    let extractor_simple_missing =
        NestedFieldExtractor::new_from_path(&["simple_map", "missing"]).unwrap();
    let result_simple_missing = extractor_simple_missing.evaluate(&data);
    assert!(
        matches!(result_simple_missing, Err(EvaluateError::NestedFieldNotFound { ref path, .. }) if path == &vec!["simple_map".to_string(), "missing".to_string()]),
        "Unexpected result for missing simple key: {:?}",
        result_simple_missing
    );
}
