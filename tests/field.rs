// Tests for the FieldExtractor (originally in lib.rs)

// Update imports to use the crate name since this is an integration test
use serde::{Deserialize, Serialize};
use serde_evaluate::error::EvaluateError;
use serde_evaluate::extractor::FieldExtractor;
use serde_evaluate::value::FieldScalarValue;
use std::collections::BTreeMap;

// Define test-specific structs here (copied from lib.rs tests)
#[derive(Serialize, Deserialize, Debug, Clone)]
struct NestedStruct {
    inner_field: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestRecord {
    id: i32,
    name: String,
    active: bool,
    count: Option<i32>,
    missing_count: Option<i32>,
    temperature: f32,
    initial: char,
    #[serde(with = "serde_bytes")] // Add this attribute
    data_bytes: Vec<u8>,
    unit_val: (),
    // New fields for Option tests
    opt_bool_some: Option<bool>,
    opt_bool_none: Option<bool>,
    opt_char_some: Option<char>,
    opt_char_none: Option<char>,
    opt_string_some: Option<String>,
    opt_string_none: Option<String>,
    #[serde(with = "serde_bytes")] // Add this attribute
    opt_bytes_some: Option<Vec<u8>>,
    #[serde(with = "serde_bytes")] // Add this attribute
    opt_bytes_none: Option<Vec<u8>>,
    opt_unit_some: Option<()>, // Note: () is the unit type
    opt_unit_none: Option<()>, // Note: () is the unit type
    opt_vec: Option<Vec<i32>>, // Option containing non-scalar
}

// New struct for map test
#[derive(Serialize)]
struct MapTestStruct {
    id: i32,
    my_map: BTreeMap<String, i32>,
}

fn create_test_record() -> TestRecord {
    TestRecord {
        id: 101,
        name: "Test Record".to_string(),
        active: true,
        count: Some(42),
        missing_count: None,
        temperature: 98.6,
        initial: 'X',
        data_bytes: vec![1, 2, 3, 4],
        unit_val: (),
        // Initialize new fields
        opt_bool_some: Some(true),
        opt_bool_none: None,
        opt_char_some: Some('Z'),
        opt_char_none: None,
        opt_string_some: Some("Hello Option".to_string()),
        opt_string_none: None,
        opt_bytes_some: Some(vec![10, 20, 30]),
        opt_bytes_none: None,
        opt_unit_some: Some(()), // Some variant of Option<()>
        opt_unit_none: None,     // None variant of Option<()>
        opt_vec: Some(vec![11, 22, 33]),
    }
}

#[test]
fn initial_setup_compiles() {
    let record = create_test_record();
    assert_eq!(record.id, 101);
}

// --- FieldExtractor Tests ---

#[test]
fn test_extract_string_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("name".to_string());
    let result = extractor.evaluate(&record);
    assert_eq!(
        result,
        Ok(FieldScalarValue::String("Test Record".to_string()))
    );
}

#[test]
fn test_extract_uint_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("id".to_string());
    let result = extractor.evaluate(&record);
    // Note: id is i32, captured as Int
    assert_eq!(result, Ok(FieldScalarValue::I32(101)));
}

#[test]
fn test_extract_missing_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("non_existent_field".to_string());
    let result = extractor.evaluate(&record);
    assert!(
        matches!(result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "non_existent_field")
    );
}

#[test]
fn test_extract_f32_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("temperature".to_string());
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::F32(98.6)));
}

#[test]
fn test_extract_char_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("initial".to_string());
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Char('X')));
}

#[test]
fn test_extract_bytes_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("data_bytes".to_string());
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Bytes(vec![1, 2, 3, 4])));
}

#[test]
fn test_extract_unit_field() {
    let record = create_test_record();
    let extractor = FieldExtractor::new("unit_val".to_string());
    let result = extractor.evaluate(&record);
    assert_eq!(result, Ok(FieldScalarValue::Unit));
}

// New test demonstrating Option handling
#[test]
fn test_extract_various_option_fields() {
    // Test case 1: Extracting an existing Option<i32> field with Some value
    let record = create_test_record();
    let extractor_some = FieldExtractor::new("count".to_string());
    let result_some = extractor_some.evaluate(&record);
    assert_eq!(
        result_some,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::I32(42)
        ))))
    );

    // Test case 2: Extracting an existing Option<i32> field with None value
    let extractor_none = FieldExtractor::new("missing_count".to_string());
    let result_none = extractor_none.evaluate(&record);
    assert_eq!(result_none, Ok(FieldScalarValue::Option(None)));

    // Test case 3: Extracting a non-existent field (should still be FieldNotFound error)
    let extractor_non_existent = FieldExtractor::new("non_existent_field".to_string());
    let result_non_existent = extractor_non_existent.evaluate(&record);
    assert!(matches!(
        result_non_existent,
        Err(EvaluateError::FieldNotFound { field_name }) if field_name == "non_existent_field"
    ));

    // Test case 4: Attempting to extract a field that exists but is not Option
    // (e.g., extracting 'name' which is String, not Option<String>)
    // The current implementation handles this correctly, extracting the scalar value directly.
    let extractor_string = FieldExtractor::new("name".to_string());
    let result_string = extractor_string.evaluate(&record);
    assert_eq!(
        result_string,
        Ok(FieldScalarValue::String("Test Record".to_string()))
    );
}

// Helper function to test Option extraction for both Some and None cases
fn assert_option_extraction(
    record: &TestRecord,
    some_field_name: &str,
    expected_some_value: FieldScalarValue,
    none_field_name: &str,
) {
    // Test Some variant
    let extractor_some = FieldExtractor::new(some_field_name.to_string());
    assert_eq!(
        extractor_some.evaluate(record),
        Ok(FieldScalarValue::Option(Some(Box::new(
            expected_some_value
        ))))
    );

    // Test None variant
    let extractor_none = FieldExtractor::new(none_field_name.to_string());
    assert_eq!(
        extractor_none.evaluate(record),
        Ok(FieldScalarValue::Option(None))
    );
}

#[test]
fn test_extract_option_bool() {
    let record = create_test_record();
    assert_option_extraction(
        &record,
        "opt_bool_some",
        FieldScalarValue::Bool(true),
        "opt_bool_none",
    );
}

#[test]
fn test_extract_option_char() {
    let record = create_test_record();
    assert_option_extraction(
        &record,
        "opt_char_some",
        FieldScalarValue::Char('Z'),
        "opt_char_none",
    );
}

#[test]
fn test_extract_option_string() {
    let record = create_test_record();
    assert_option_extraction(
        &record,
        "opt_string_some",
        FieldScalarValue::String("Hello Option".to_string()),
        "opt_string_none",
    );
}

#[test]
fn test_extract_option_bytes() {
    let record = create_test_record();
    assert_option_extraction(
        &record,
        "opt_bytes_some",
        FieldScalarValue::Bytes(vec![10, 20, 30]),
        "opt_bytes_none",
    );
}

#[test]
fn test_extract_option_unit() {
    let record = create_test_record();
    assert_option_extraction(
        &record,
        "opt_unit_some",
        FieldScalarValue::Unit,
        "opt_unit_none",
    );
}

// --- Tests for non-scalar Option contents (should fail) ---

#[test]
fn test_extract_option_non_scalar_vec() {
    // Attempting to extract an Option<Vec<i32>> should fail because Vec is not scalar
    let record = create_test_record();
    let extractor = FieldExtractor::new("opt_vec".to_string());
    let result = extractor.evaluate(&record);
    // Similar to the struct case, expect UnsupportedType for the sequence.
    assert!(
        matches!(result, Err(EvaluateError::UnsupportedType { type_name }) if type_name == "sequence"),
        "Expected UnsupportedType for Option<Vec>, got {:?}",
        result
    );
}

// New test for map field extraction attempt
#[test]
fn test_extract_map_field_unsupported() {
    let mut map_data = BTreeMap::new();
    map_data.insert("key1".to_string(), 1);
    map_data.insert("key2".to_string(), 2);

    let record = MapTestStruct {
        id: 1,
        my_map: map_data,
    };

    let extractor = FieldExtractor::new("my_map");
    let result = extractor.evaluate(&record);

    assert!(
        matches!(result, Err(EvaluateError::UnsupportedType { .. })),
        "Expected UnsupportedType error for map field, got {:?}",
        result
    );
}

#[test]
fn test_nested_option_u32_some_some() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_u32: Option<Option<u32>>,
    }

    let test_struct = TestStruct {
        nested_option_u32: Some(Some(123u32)),
    };

    let result = FieldExtractor::new("nested_option_u32").evaluate(&test_struct);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::U32(123))))
        ))))
    );
}

#[test]
fn test_nested_option_u32_some_none() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_u32: Option<Option<u32>>,
    }

    let test_struct = TestStruct {
        nested_option_u32: Some(None),
    };

    let result = FieldExtractor::new("nested_option_u32").evaluate(&test_struct);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::Option(None)
        ))))
    );
}

#[test]
fn test_nested_option_u32_none() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_u32: Option<Option<u32>>,
    }

    let test_struct = TestStruct {
        nested_option_u32: None,
    };

    let result = FieldExtractor::new("nested_option_u32").evaluate(&test_struct);
    assert_eq!(result, Ok(FieldScalarValue::Option(None)));
}

#[test]
fn test_nested_option_string_some_some() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_string: Option<Option<String>>,
    }

    let test_struct = TestStruct {
        nested_option_string: Some(Some("hello".to_string())),
    };

    let result = FieldExtractor::new("nested_option_string").evaluate(&test_struct);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String(
                "hello".to_string()
            ))))
        ))))
    );
}

#[test]
fn test_nested_option_string_some_none() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_string: Option<Option<String>>,
    }

    let test_struct = TestStruct {
        nested_option_string: Some(None),
    };

    let result = FieldExtractor::new("nested_option_string").evaluate(&test_struct);
    assert_eq!(
        result,
        Ok(FieldScalarValue::Option(Some(Box::new(
            FieldScalarValue::Option(None)
        ))))
    );
}

#[test]
fn test_nested_option_string_none() {
    #[derive(Serialize)]
    struct TestStruct {
        nested_option_string: Option<Option<String>>,
    }

    let test_struct = TestStruct {
        nested_option_string: None,
    };

    let result = FieldExtractor::new("nested_option_string").evaluate(&test_struct);
    assert_eq!(result, Ok(FieldScalarValue::Option(None)));
}
