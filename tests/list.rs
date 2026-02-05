// Tests for ListFieldExtractor and NestedListFieldExtractor

use serde::Serialize;
use serde_evaluate::error::EvaluateError;
use serde_evaluate::extractor::{ListFieldExtractor, NestedListFieldExtractor};
use serde_evaluate::value::FieldScalarValue;

// =============================================================================
// Test Structs
// =============================================================================

#[derive(Serialize)]
struct SimpleRecord {
    id: u32,
    tags: Vec<String>,
    scores: Vec<i32>,
    empty_list: Vec<String>,
    optional_tags: Option<Vec<String>>,
    missing_tags: Option<Vec<String>>,
}

#[derive(Serialize)]
struct NestedRecord {
    id: u32,
    metadata: Metadata,
}

#[derive(Serialize)]
struct Metadata {
    labels: Vec<String>,
    counts: Vec<i64>,
}

#[derive(Serialize)]
struct RecordWithStructList {
    items: Vec<Item>,
}

#[derive(Serialize)]
struct Item {
    name: String,
    value: i32,
}

#[derive(Serialize)]
struct RecordWithNestedList {
    nested_list: Vec<Vec<String>>,
}

fn create_simple_record() -> SimpleRecord {
    SimpleRecord {
        id: 1,
        tags: vec!["rust".to_string(), "serde".to_string(), "evaluate".to_string()],
        scores: vec![100, 200, 300],
        empty_list: vec![],
        optional_tags: Some(vec!["opt1".to_string(), "opt2".to_string()]),
        missing_tags: None,
    }
}

fn create_nested_record() -> NestedRecord {
    NestedRecord {
        id: 42,
        metadata: Metadata {
            labels: vec!["label1".to_string(), "label2".to_string()],
            counts: vec![10, 20, 30],
        },
    }
}

// =============================================================================
// ListFieldExtractor Tests
// =============================================================================

#[test]
fn test_extract_string_list() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("tags");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::String("rust".to_string()),
            FieldScalarValue::String("serde".to_string()),
            FieldScalarValue::String("evaluate".to_string()),
        ]
    );
}

#[test]
fn test_extract_int_list() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("scores");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::I32(100),
            FieldScalarValue::I32(200),
            FieldScalarValue::I32(300),
        ]
    );
}

#[test]
fn test_extract_empty_list() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("empty_list");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(result, vec![]);
}

#[test]
fn test_extract_option_vec_some() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("optional_tags");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::String("opt1".to_string()),
            FieldScalarValue::String("opt2".to_string()),
        ]
    );
}

#[test]
fn test_extract_option_vec_none() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("missing_tags");
    let result = extractor.evaluate(&record).unwrap();

    // Option<Vec<T>> = None should return empty list
    assert_eq!(result, vec![]);
}

#[test]
fn test_list_field_not_found() {
    let record = create_simple_record();
    let extractor = ListFieldExtractor::new("nonexistent");
    let result = extractor.evaluate(&record);

    assert!(matches!(
        result,
        Err(EvaluateError::FieldNotFound { field_name }) if field_name == "nonexistent"
    ));
}

#[test]
fn test_extract_list_of_structs_fails() {
    let record = RecordWithStructList {
        items: vec![
            Item {
                name: "item1".to_string(),
                value: 1,
            },
            Item {
                name: "item2".to_string(),
                value: 2,
            },
        ],
    };

    let extractor = ListFieldExtractor::new("items");
    let result = extractor.evaluate(&record);

    assert!(matches!(
        result,
        Err(EvaluateError::UnsupportedType { type_name }) if type_name == "struct"
    ));
}

#[test]
fn test_extract_nested_list_fails() {
    let record = RecordWithNestedList {
        nested_list: vec![vec!["a".to_string()], vec!["b".to_string()]],
    };

    let extractor = ListFieldExtractor::new("nested_list");
    let result = extractor.evaluate(&record);

    assert!(matches!(
        result,
        Err(EvaluateError::UnsupportedType { type_name }) if type_name == "nested sequence"
    ));
}

// =============================================================================
// NestedListFieldExtractor Tests
// =============================================================================

#[test]
fn test_extract_nested_string_list() {
    let record = create_nested_record();
    let extractor = NestedListFieldExtractor::new_from_path(&["metadata", "labels"]).unwrap();
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::String("label1".to_string()),
            FieldScalarValue::String("label2".to_string()),
        ]
    );
}

#[test]
fn test_extract_nested_int_list() {
    let record = create_nested_record();
    let extractor = NestedListFieldExtractor::new_from_path(&["metadata", "counts"]).unwrap();
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::I64(10),
            FieldScalarValue::I64(20),
            FieldScalarValue::I64(30),
        ]
    );
}

#[test]
fn test_nested_list_path_not_found() {
    let record = create_nested_record();
    let extractor = NestedListFieldExtractor::new_from_path(&["metadata", "nonexistent"]).unwrap();
    let result = extractor.evaluate(&record);

    assert!(matches!(
        result,
        Err(EvaluateError::NestedFieldNotFound { ref path })
            if path == &vec!["metadata".to_string(), "nonexistent".to_string()]
    ));
}

#[test]
fn test_nested_list_invalid_path_empty() {
    let result = NestedListFieldExtractor::new_from_path::<&str>(&[]);

    assert!(matches!(result, Err(EvaluateError::InvalidPath(_))));
}

#[test]
fn test_nested_list_invalid_path_empty_segment() {
    let result = NestedListFieldExtractor::new_from_path(&["metadata", ""]);

    assert!(matches!(result, Err(EvaluateError::InvalidPath(_))));
}

// =============================================================================
// Additional Edge Case Tests
// =============================================================================

#[test]
fn test_extract_list_with_bool_elements() {
    #[derive(Serialize)]
    struct BoolListRecord {
        flags: Vec<bool>,
    }

    let record = BoolListRecord {
        flags: vec![true, false, true],
    };

    let extractor = ListFieldExtractor::new("flags");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::Bool(true),
            FieldScalarValue::Bool(false),
            FieldScalarValue::Bool(true),
        ]
    );
}

#[test]
fn test_extract_list_with_float_elements() {
    #[derive(Serialize)]
    struct FloatListRecord {
        values: Vec<f64>,
    }

    let record = FloatListRecord {
        values: vec![1.5, 2.5, 3.5],
    };

    let extractor = ListFieldExtractor::new("values");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::F64(1.5),
            FieldScalarValue::F64(2.5),
            FieldScalarValue::F64(3.5),
        ]
    );
}

#[test]
fn test_extract_list_with_option_elements() {
    #[derive(Serialize)]
    struct OptionListRecord {
        opt_values: Vec<Option<i32>>,
    }

    let record = OptionListRecord {
        opt_values: vec![Some(1), None, Some(3)],
    };

    let extractor = ListFieldExtractor::new("opt_values");
    let result = extractor.evaluate(&record).unwrap();

    assert_eq!(
        result,
        vec![
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::I32(1)))),
            FieldScalarValue::Option(None),
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::I32(3)))),
        ]
    );
}
