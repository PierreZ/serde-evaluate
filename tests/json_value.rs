// Tests for extracting fields from structs containing serde_json::Value fields.

use serde::Serialize;
use serde_evaluate::error::EvaluateError;
use serde_evaluate::extractor::{
    FieldExtractor, ListFieldExtractor, NestedFieldExtractor, NestedListFieldExtractor,
};
use serde_evaluate::value::FieldScalarValue;
use serde_evaluate::CompositeFieldExtractor;

#[derive(Serialize)]
struct RecordWithJsonValue {
    id: i32,
    name: String,
    metadata: serde_json::Value,
}

// =============================================================================
// FieldExtractor: Basic scalar Value extraction
// =============================================================================

#[test]
fn test_extract_non_value_field_from_struct_with_value() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"key": "value"}),
    };

    let extractor = FieldExtractor::new("name");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::String("test".to_string())));
}

#[test]
fn test_extract_json_value_string() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::Value::String("hello".to_string()),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::String("hello".to_string())));
}

#[test]
fn test_extract_json_value_bool() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::Value::Bool(true),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Bool(true)));
}

#[test]
fn test_extract_json_value_number_u64() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!(42u64),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::U64(42)));
}

#[test]
fn test_extract_json_value_number_i64() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!(-10i64),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::I64(-10)));
}

#[test]
fn test_extract_json_value_number_f64() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!(99.5),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::F64(99.5)));
}

#[test]
fn test_extract_json_value_null() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::Value::Null,
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Unit));
}

#[test]
fn test_extract_json_value_array_unsupported() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!([1, 2, 3]),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert!(
        matches!(
            result,
            Err(EvaluateError::UnsupportedType {
                type_name: "sequence"
            })
        ),
        "Expected UnsupportedType for JSON array, got {:?}",
        result
    );
}

#[test]
fn test_extract_json_value_object_unsupported() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"key": "value"}),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert!(
        matches!(
            result,
            Err(EvaluateError::UnsupportedType { type_name: "map" })
        ),
        "Expected UnsupportedType for JSON object, got {:?}",
        result
    );
}

// =============================================================================
// NestedFieldExtractor: Traverse INTO a Value::Object
// =============================================================================

#[test]
fn test_nested_extract_into_json_object() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"region": "us-east-1", "priority": 5}),
    };

    let extractor = NestedFieldExtractor::new_from_path(&["metadata", "region"]).unwrap();
    let result = extractor.evaluate(&record);
    assert_eq!(
        result,
        Ok(FieldScalarValue::String("us-east-1".to_string()))
    );
}

#[test]
fn test_nested_extract_deeply_into_json_object() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"config": {"region": "us-east-1"}}),
    };

    let extractor = NestedFieldExtractor::new_from_path(&["metadata", "config", "region"]).unwrap();
    let result = extractor.evaluate(&record);
    assert_eq!(
        result,
        Ok(FieldScalarValue::String("us-east-1".to_string()))
    );
}

#[test]
fn test_nested_extract_json_object_key_not_found() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"region": "us-east-1"}),
    };

    let extractor = NestedFieldExtractor::new_from_path(&["metadata", "missing"]).unwrap();
    let result = extractor.evaluate(&record);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_object_value_is_object() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"config": {"nested": true}}),
    };

    // Trying to extract "config" which is itself an object → UnsupportedType
    let extractor = NestedFieldExtractor::new_from_path(&["metadata", "config"]).unwrap();
    let result = extractor.evaluate(&record);
    assert!(
        matches!(
            result,
            Err(EvaluateError::UnsupportedType { type_name: "map" })
        ),
        "Expected UnsupportedType for nested JSON object, got {:?}",
        result
    );
}

// =============================================================================
// Option<serde_json::Value>
// =============================================================================

#[derive(Serialize)]
struct RecordWithOptionalJsonValue {
    id: i32,
    metadata: Option<serde_json::Value>,
}

#[test]
fn test_extract_option_json_value_some_string() {
    let record = RecordWithOptionalJsonValue {
        id: 1,
        metadata: Some(serde_json::Value::String("hello".to_string())),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::String("hello".to_string())
        ))))
    );
}

#[test]
fn test_extract_option_json_value_none() {
    let record = RecordWithOptionalJsonValue {
        id: 1,
        metadata: None,
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Option(None)));
}

#[test]
fn test_extract_option_json_value_some_null() {
    // Some(Value::Null) is distinct from None — the Option is Some, but the Value is Null
    let record = RecordWithOptionalJsonValue {
        id: 1,
        metadata: Some(serde_json::Value::Null),
    };

    let extractor = FieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::Unit
        ))))
    );
}

// =============================================================================
// CompositeFieldExtractor: Mix of typed + Value fields
// =============================================================================

#[test]
fn test_composite_with_json_value_fields() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::Value::String("hello".to_string()),
    };

    let extractor = CompositeFieldExtractor::new(&["id", "metadata"]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::I32(1),
            FieldScalarValue::String("hello".to_string()),
        ]
    );
}

#[test]
fn test_composite_with_nested_json_path() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"region": "us-east-1"}),
    };

    let extractor =
        CompositeFieldExtractor::new_from_paths(&[&["id"], &["metadata", "region"]]).unwrap();
    let values = extractor.evaluate(&record).unwrap();
    assert_eq!(
        values,
        vec![
            FieldScalarValue::I32(1),
            FieldScalarValue::String("us-east-1".to_string()),
        ]
    );
}

// =============================================================================
// ListFieldExtractor: Value::Array as a list
// =============================================================================

#[test]
fn test_list_extract_json_value_array_of_strings() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!(["a", "b", "c"]),
    };

    let extractor = ListFieldExtractor::new("metadata");
    let result = extractor.evaluate(&record).unwrap();
    assert_eq!(
        result,
        vec![
            FieldScalarValue::String("a".to_string()),
            FieldScalarValue::String("b".to_string()),
            FieldScalarValue::String("c".to_string()),
        ]
    );
}

#[test]
fn test_list_extract_json_value_array_of_numbers() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!([1, 2, 3]),
    };

    let extractor = ListFieldExtractor::new("metadata");
    let result = extractor.evaluate(&record).unwrap();
    assert_eq!(
        result,
        vec![
            FieldScalarValue::U64(1),
            FieldScalarValue::U64(2),
            FieldScalarValue::U64(3),
        ]
    );
}

#[test]
fn test_list_extract_json_value_empty_array() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!([]),
    };

    let extractor = ListFieldExtractor::new("metadata");
    let result = extractor.evaluate(&record).unwrap();
    assert_eq!(result, vec![]);
}

#[test]
fn test_list_extract_json_value_array_of_objects_fails() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!([{"a": 1}]),
    };

    let extractor = ListFieldExtractor::new("metadata");
    let result = extractor.evaluate(&record);
    assert!(
        matches!(result, Err(EvaluateError::UnsupportedType { .. })),
        "Expected UnsupportedType for array of objects, got {:?}",
        result
    );
}

// =============================================================================
// NestedListFieldExtractor: List inside a JSON object
// =============================================================================

#[test]
fn test_nested_list_extract_from_json_object() {
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"tags": ["a", "b"]}),
    };

    let extractor = NestedListFieldExtractor::new_from_path(&["metadata", "tags"]).unwrap();
    let result = extractor.evaluate(&record).unwrap();
    assert_eq!(
        result,
        vec![
            FieldScalarValue::String("a".to_string()),
            FieldScalarValue::String("b".to_string()),
        ]
    );
}

// =============================================================================
// Field ordering: complex Value before target field
// =============================================================================

#[derive(Serialize)]
struct ValueBeforeTarget {
    metadata: serde_json::Value,
    name: String,
}

#[test]
fn test_extract_field_after_complex_json_value() {
    let record = ValueBeforeTarget {
        metadata: serde_json::json!({
            "deeply": {"nested": {"object": {"with": "many", "levels": [1, 2, 3]}}},
            "another_key": [{"complex": true}, {"data": "here"}]
        }),
        name: "target".to_string(),
    };

    let extractor = FieldExtractor::new("name");
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::String("target".to_string())));
}

// =============================================================================
// NestedFieldExtractor: intermediate path is scalar (not object/map)
// =============================================================================

#[test]
fn test_nested_extract_json_intermediate_scalar_should_error() {
    // Path ["details", "rating"] but "details" is a string, not an object.
    // Should error with NestedFieldNotFound, not return the string.
    let json = serde_json::json!({"details": "not_an_object", "score": 42});
    let extractor = NestedFieldExtractor::new_from_path(&["details", "rating"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when intermediate path is scalar, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_intermediate_number_should_error() {
    // Path ["score", "sub"] but "score" is a number.
    let json = serde_json::json!({"score": 42, "name": "test"});
    let extractor = NestedFieldExtractor::new_from_path(&["score", "sub"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when intermediate path is number, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_intermediate_bool_should_error() {
    // Path ["active", "sub"] but "active" is a bool.
    let json = serde_json::json!({"active": true});
    let extractor = NestedFieldExtractor::new_from_path(&["active", "sub"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when intermediate path is bool, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_intermediate_null_should_error() {
    // Path ["data", "sub"] but "data" is null.
    let json = serde_json::json!({"data": null});
    let extractor = NestedFieldExtractor::new_from_path(&["data", "sub"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when intermediate path is null, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_intermediate_array_should_error() {
    // Path ["details", "sub"] but "details" is an array, not an object.
    let json = serde_json::json!({"details": [1, 2, 3]});
    let extractor = NestedFieldExtractor::new_from_path(&["details", "sub"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when intermediate path is array, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_deeper_intermediate_scalar_should_error() {
    // Path ["a", "b", "c"] but "b" is a scalar, not an object.
    let json = serde_json::json!({"a": {"b": "scalar_not_object"}});
    let extractor = NestedFieldExtractor::new_from_path(&["a", "b", "c"]).unwrap();
    let result = extractor.evaluate(&json);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound when deeper intermediate path is scalar, got {:?}",
        result
    );
}

#[test]
fn test_nested_extract_json_same_key_at_multiple_levels() {
    // Path ["a", "a"] on {"a": {"a": 42}} — same key name at both levels.
    let json = serde_json::json!({"a": {"a": 42}});
    let extractor = NestedFieldExtractor::new_from_path(&["a", "a"]).unwrap();
    let result = extractor.evaluate(&json);
    assert_eq!(result, Ok(FieldScalarValue::U64(42)));
}

#[test]
fn test_nested_extract_struct_json_value_intermediate_scalar_should_error() {
    // Struct field "metadata" is a serde_json::Value containing {"config": "flat_string"}.
    // Path ["metadata", "config", "region"] should error because "config" is a string.
    let record = RecordWithJsonValue {
        id: 1,
        name: "test".to_string(),
        metadata: serde_json::json!({"config": "flat_string"}),
    };
    let extractor = NestedFieldExtractor::new_from_path(&["metadata", "config", "region"]).unwrap();
    let result = extractor.evaluate(&record);
    assert!(
        matches!(result, Err(EvaluateError::NestedFieldNotFound { .. })),
        "Expected NestedFieldNotFound for struct→JSON intermediate scalar, got {:?}",
        result
    );
}

// =============================================================================
// Deep JSON nesting (4+ levels)
// =============================================================================

#[test]
fn test_nested_extract_json_five_levels_deep() {
    let json = serde_json::json!({"a": {"b": {"c": {"d": {"e": 42}}}}});
    let extractor = NestedFieldExtractor::new_from_path(&["a", "b", "c", "d", "e"]).unwrap();
    let result = extractor.evaluate(&json);
    assert_eq!(result, Ok(FieldScalarValue::U64(42)));
}

// =============================================================================
// Bare JSON object (no struct wrapper)
// =============================================================================

#[test]
fn test_extract_field_from_bare_json_object() {
    let json = serde_json::json!({"name": "test", "count": 42});
    let extractor = FieldExtractor::new("name");
    let result = extractor.evaluate(&json);
    assert_eq!(result, Ok(FieldScalarValue::String("test".to_string())));
}
