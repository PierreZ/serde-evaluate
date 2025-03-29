//! This library provides a mechanism to extract the value of a single field
//! from any struct that implements `serde::Serialize` **without** needing to
//! deserialize the entire struct. It's particularly useful when you only need
//! one specific piece of data from a potentially large or complex structure.
//!
//! The extracted value is returned as a `FieldScalarValue` enum, which covers
//! common scalar types (integers, floats, bool, string, char, bytes, unit, and options of these).
//!
//! ## How it Works
//!
//! It uses a custom Serde `Serializer` (`FieldValueExtractorSerializer`) that intercepts
//! the serialization process. When the target field is encountered, its value is captured.
//! Serialization of other fields is skipped efficiently.
//!
//! ## Usage
//!
//! ```rust
//! use serde::Serialize;
//! use serde_evaluate::{FieldExtractor, FieldScalarValue, EvaluateError};
//!
//! #[derive(Serialize)]
//! struct MyData {
//!     id: u32,
//!     name: String,
//!     active: bool,
//!     score: Option<f64>,
//!     #[serde(with = "serde_bytes")]
//!     raw_data: Vec<u8>,
//!     nested: InnerData, // Unsupported type for direct extraction
//! }
//!
//! #[derive(Serialize)]
//! struct InnerData {
//!     value: i32,
//! }
//!
//! fn main() -> Result<(), EvaluateError> {
//!     let data = MyData {
//!         id: 101,
//!         name: "Example".to_string(),
//!         active: true,
//!         score: Some(95.5),
//!         raw_data: vec![1, 2, 3, 4],
//!         nested: InnerData { value: -5 },
//!     };
//!
//!     // Extract the 'name' field
//!     let name_value = FieldExtractor::new("name").evaluate(&data)?;
//!     assert_eq!(name_value, FieldScalarValue::String("Example".to_string()));
//!
//!     // Extract the 'active' field
//!     let active_value = FieldExtractor::new("active").evaluate(&data)?;
//!     assert_eq!(active_value, FieldScalarValue::Bool(true));
//!
//!     // Extract the 'score' field (Option<f64>)
//!     let score_value = FieldExtractor::new("score").evaluate(&data)?;
//!     assert_eq!(score_value, FieldScalarValue::Option(Some(Box::new(FieldScalarValue::F64(95.5)))));
//!
//!     // Extract the 'raw_data' field (Vec<u8> handled via serde_bytes)
//!     let bytes_value = FieldExtractor::new("raw_data").evaluate(&data)?;
//!     assert_eq!(bytes_value, FieldScalarValue::Bytes(vec![1, 2, 3, 4]));
//!
//!     // Trying to extract a non-existent field returns FieldNotFound
//!     let missing_result = FieldExtractor::new("address").evaluate(&data);
//!     assert!(matches!(missing_result, Err(EvaluateError::FieldNotFound { .. })));
//!
//!     // Trying to extract a non-scalar field (struct) returns UnsupportedType
//!     let nested_result = FieldExtractor::new("nested").evaluate(&data);
//!     assert!(matches!(nested_result, Err(EvaluateError::UnsupportedType { type_name }) if type_name == "struct"));
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Types
//!
//! The following types (and `Option`s thereof) can be extracted as `FieldScalarValue` variants:
//! - `bool`
//! - `i8`, `i16`, `i32`, `i64`, `i128`
//! - `u8`, `u16`, `u32`, `u64`, `u128`
//! - `f32`, `f64`
//! - `char`
//! - `String`, `&str`
//! - `Vec<u8>` (requires `#[serde(with = "serde_bytes")]`)
//! - Unit (`()`)
//!
//! Other types like nested structs, sequences (except `Vec<u8` with `serde_bytes`), maps, enums with data
//! will result in an `EvaluateError::UnsupportedType`.

// Declare modules
mod error;
mod extractor;
mod serializer; // Keep internal for now
mod value;

// Re-export public API
/// Errors that can occur during field extraction.
pub use error::EvaluateError;
/// Public interface for extracting scalar field values.
pub use extractor::FieldExtractor;
/// Enum representing the possible scalar values that can be extracted.
pub use value::FieldScalarValue;

#[cfg(test)]
mod tests {
    // Update imports to use the new module structure
    use super::error::EvaluateError;
    use super::extractor::FieldExtractor;
    use super::value::FieldScalarValue;
    use serde::{Deserialize, Serialize};

    // Define test-specific structs here
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
        nested: NestedStruct,
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

    fn create_test_record() -> TestRecord {
        TestRecord {
            id: 101,
            name: "Test Record".to_string(),
            active: true,
            count: Some(42),
            missing_count: None,
            nested: NestedStruct {
                inner_field: "Inner Value".to_string(),
            },
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
    fn test_extract_int_field_some() {
        let record = create_test_record();
        let extractor = FieldExtractor::new("count".to_string());
        let result = extractor.evaluate(&record);
        // Note: 'value' is Option<i32>. Should be captured as Option(Some(Box(I32(42))))
        assert_eq!(
            result,
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::I32(42)
            ))))
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
    fn test_extract_option_field_none() {
        let mut record = create_test_record();
        record.count = None; // Set the optional field to None
        let extractor = FieldExtractor::new("count".to_string());
        let result = extractor.evaluate(&record);
        assert_eq!(result, Ok(FieldScalarValue::Option(None)));
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
    fn test_extract_non_scalar_field_vec() {
        // Attempting to extract a Vec<String> should fail as it's not a scalar
        let record = create_test_record();
        let extractor = FieldExtractor::new("data_bytes".to_string());
        let result = extractor.evaluate(&record);
        assert_eq!(result, Ok(FieldScalarValue::Bytes(vec![1, 2, 3, 4])));
    }

    #[test]
    fn test_extract_non_scalar_field_struct() {
        // Attempting to extract a nested struct should fail
        let record = create_test_record();
        let extractor = FieldExtractor::new("nested".to_string());
        let result = extractor.evaluate(&record);
        assert!(
            matches!(result, Err(EvaluateError::UnsupportedType { type_name }) if type_name == "struct"),
            "Expected UnsupportedType error for struct field"
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

    // --- New Tests for various Option types ---

    #[test]
    fn test_extract_option_bool() {
        let record = create_test_record();
        // Some(true)
        let extractor_some = FieldExtractor::new("opt_bool_some".to_string());
        assert_eq!(
            extractor_some.evaluate(&record),
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::Bool(true)
            ))))
        );
        // None
        let extractor_none = FieldExtractor::new("opt_bool_none".to_string());
        assert_eq!(
            extractor_none.evaluate(&record),
            Ok(FieldScalarValue::Option(None))
        );
    }

    #[test]
    fn test_extract_option_char() {
        let record = create_test_record();
        // Some('Z')
        let extractor_some = FieldExtractor::new("opt_char_some".to_string());
        assert_eq!(
            extractor_some.evaluate(&record),
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::Char('Z')
            ))))
        );
        // None
        let extractor_none = FieldExtractor::new("opt_char_none".to_string());
        assert_eq!(
            extractor_none.evaluate(&record),
            Ok(FieldScalarValue::Option(None))
        );
    }

    #[test]
    fn test_extract_option_string() {
        let record = create_test_record();
        // Some("Hello Option")
        let extractor_some = FieldExtractor::new("opt_string_some".to_string());
        assert_eq!(
            extractor_some.evaluate(&record),
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::String("Hello Option".to_string())
            ))))
        );
        // None
        let extractor_none = FieldExtractor::new("opt_string_none".to_string());
        assert_eq!(
            extractor_none.evaluate(&record),
            Ok(FieldScalarValue::Option(None))
        );
    }

    #[test]
    fn test_extract_option_bytes() {
        let record = create_test_record();
        // Some(vec![10, 20, 30])
        let extractor_some = FieldExtractor::new("opt_bytes_some".to_string());
        assert_eq!(
            extractor_some.evaluate(&record),
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::Bytes(vec![10, 20, 30])
            ))))
        );
        // None
        let extractor_none = FieldExtractor::new("opt_bytes_none".to_string());
        assert_eq!(
            extractor_none.evaluate(&record),
            Ok(FieldScalarValue::Option(None))
        );
    }

    #[test]
    fn test_extract_option_unit() {
        let record = create_test_record();
        // Some(())
        let extractor_some = FieldExtractor::new("opt_unit_some".to_string());
        assert_eq!(
            extractor_some.evaluate(&record),
            Ok(FieldScalarValue::Option(Some(Box::new(
                FieldScalarValue::Unit
            ))))
        );
        // None
        let extractor_none = FieldExtractor::new("opt_unit_none".to_string());
        assert_eq!(
            extractor_none.evaluate(&record),
            Ok(FieldScalarValue::Option(None))
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
}
