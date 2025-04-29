//! This library provides a mechanism to extract the value of a single field
//! from any struct that implements `serde::Serialize` **without** needing to
//! deserialize the entire struct. It's particularly useful when you only need
//! one specific piece of data from a potentially large or complex structure,
//! potentially residing within nested structs or maps.
//!
//! The extracted value is returned as a `FieldScalarValue` enum, which covers
//! common scalar types (integers, floats, bool, string, char, bytes, unit, and options of these).
//!
//! ## How it Works
//!
//! It uses a custom Serde `Serializer` (`FieldValueExtractorSerializer`) that intercepts
//! the serialization process. When the target field path (which can include struct fields
//! and map keys separated by dots) is encountered, its scalar value is captured.
//! Serialization of other fields or parts of the structure is skipped efficiently.
//!
//! ## Usage
//!
//! ```rust
//! use serde::Serialize;
//! use std::collections::HashMap;
//! use serde_evaluate::{extractor::{FieldExtractor, NestedFieldExtractor}, value::FieldScalarValue, error::EvaluateError};
//!
//! #[derive(Serialize)]
//! struct MyData {
//!     id: u32,
//!     name: String,
//!     active: bool,
//!     score: Option<f64>,
//!     #[serde(with = "serde_bytes")]
//!     raw_data: Vec<u8>,
//!     nested: InnerData,
//!     data_map: HashMap<String, InnerData>,
//! }
//!
//! #[derive(Serialize)]
//! struct InnerData {
//!     value: i32,
//!     description: Option<String>,
//! }
//!
//! fn main() -> Result<(), EvaluateError> {
//!     let mut map = HashMap::new();
//!     map.insert("entry1".to_string(), InnerData { value: -10, description: None });
//!     map.insert("entry2".to_string(), InnerData { value: 20, description: Some("Second".to_string()) });
//!
//!     let data = MyData {
//!         id: 101,
//!         name: "Example".to_string(),
//!         active: true,
//!         score: Some(95.5),
//!         raw_data: vec![1, 2, 3, 4],
//!         nested: InnerData { value: 5, description: Some("Nested Desc".to_string()) },
//!         data_map: map,
//!     };
//!
//!     // --- Basic Field Extraction ---
//!
//!     // Extract the 'name' field (top-level)
//!     let name_value = FieldExtractor::new("name").evaluate(&data)?;
//!     assert_eq!(name_value, FieldScalarValue::String("Example".to_string()));
//!
//!     // Extract the 'active' field (top-level)
//!     let active_value = FieldExtractor::new("active").evaluate(&data)?;
//!     assert_eq!(active_value, FieldScalarValue::Bool(true));
//!
//!     // Extract the 'score' field (Option<f64>, top-level)
//!     let score_value = FieldExtractor::new("score").evaluate(&data)?;
//!     assert_eq!(score_value, FieldScalarValue::Option(Some(Box::new(FieldScalarValue::F64(95.5)))));
//!
//!     // Extract the 'raw_data' field (Vec<u8> handled via serde_bytes, top-level)
//!     let bytes_value = FieldExtractor::new("raw_data").evaluate(&data)?;
//!     assert_eq!(bytes_value, FieldScalarValue::Bytes(vec![1, 2, 3, 4]));
//!
//!     // --- Nested Field Extraction ---
//!
//!     // Extract nested field 'nested.value'
//!     let nested_val_extractor = NestedFieldExtractor::new_from_path(&["nested", "value"])?;
//!     let nested_val = nested_val_extractor.evaluate(&data)?;
//!     assert_eq!(nested_val, FieldScalarValue::I32(5));
//!
//!     // Extract nested Option field 'nested.description'
//!     let nested_desc_extractor = NestedFieldExtractor::new_from_path(&["nested", "description"])?;
//!     let nested_desc = nested_desc_extractor.evaluate(&data)?;
//!     assert_eq!(nested_desc, FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String("Nested Desc".to_string())))));
//!
//!     // Extract field within map value 'data_map.entry1.value'
//!     let map_val1_extractor = NestedFieldExtractor::new_from_path(&["data_map", "entry1", "value"])?;
//!     let map_val1 = map_val1_extractor.evaluate(&data)?;
//!     assert_eq!(map_val1, FieldScalarValue::I32(-10));
//!
//!      // Extract Option field within map value 'data_map.entry2.description'
//!     let map_val2_desc_extractor = NestedFieldExtractor::new_from_path(&["data_map", "entry2", "description"])?;
//!     let map_val2_desc = map_val2_desc_extractor.evaluate(&data)?;
//!     assert_eq!(map_val2_desc, FieldScalarValue::Option(Some(Box::new(FieldScalarValue::String("Second".to_string())))));
//!
//!     // Extract Option field within map value that is None 'data_map.entry1.description'
//!     let map_val1_desc_extractor = NestedFieldExtractor::new_from_path(&["data_map", "entry1", "description"])?;
//!     let map_val1_desc = map_val1_desc_extractor.evaluate(&data)?;
//!     assert_eq!(map_val1_desc, FieldScalarValue::Option(None));
//!
//!     // --- Error Cases ---
//!
//!     // Trying to extract a non-existent top-level field returns FieldNotFound
//!     let missing_result = FieldExtractor::new("address").evaluate(&data);
//!     assert!(matches!(missing_result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "address"));
//!
//!     // Trying to extract a non-existent nested field returns FieldNotFound (with path up to failure)
//!     let missing_nested_extractor = NestedFieldExtractor::new_from_path(&["nested", "bad_field"])?;
//!     let missing_nested_result = missing_nested_extractor.evaluate(&data);
//!     assert!(matches!(missing_nested_result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "nested.bad_field"));
//!
//!     // Trying to extract from a non-existent map key returns FieldNotFound (with path up to failure)
//!     let missing_map_key_extractor = NestedFieldExtractor::new_from_path(&["data_map", "missing_key", "value"])?;
//!     let missing_map_key_result = missing_map_key_extractor.evaluate(&data);
//!     assert!(matches!(missing_map_key_result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "data_map"));
//!
//!     // Trying to extract a non-existent field within a valid map entry returns FieldNotFound (with path up to map)
//!     let missing_map_inner_extractor = NestedFieldExtractor::new_from_path(&["data_map", "entry1", "bad_field"])?;
//!     let missing_map_inner_result = missing_map_inner_extractor.evaluate(&data);
//!     assert!(matches!(missing_map_inner_result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "data_map"));
//!
//!     // Trying to extract a non-scalar field (struct) itself returns UnsupportedType
//!     let nested_struct_extractor = NestedFieldExtractor::new_from_path(&["nested"])?;
//!     let nested_struct_result = nested_struct_extractor.evaluate(&data);
//!     assert!(matches!(nested_struct_result, Err(EvaluateError::UnsupportedType { .. })));
//!
//!     // Trying to extract a non-scalar field (map) itself returns UnsupportedType
//!     let map_extractor = NestedFieldExtractor::new_from_path(&["data_map"])?;
//!     let map_result = map_extractor.evaluate(&data);
//!     assert!(matches!(map_result, Err(EvaluateError::UnsupportedType { .. })));
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Supported Types
//!
//! The final target of the extraction path must be one of the following types
//! (or an `Option` thereof), which can be extracted as `FieldScalarValue` variants:
//! - `bool`
//! - `i8`, `i16`, `i32`, `i64`, `i128`
//! - `u8`, `u16`, `u32`, `u64`, `u128`
//! - `f32`, `f64`
//! - `char`
//! - `String`, `&str`
//! - `Vec<u8>` (requires `#[serde(with = "serde_bytes")]` on the field)
//! - Unit (`()`)
//!
//! Attempting to extract a field path that ultimately points to other types like nested structs,
//! sequences (except `Vec<u8` with `serde_bytes`), maps, or enums with data will result in an
//! `EvaluateError::UnsupportedType`. Similarly, if any intermediate part of the path
//! (e.g., `middle` in `top.middle.leaf`) is not a struct or a map, extraction will fail.
//!
//! **Note:** While `Option<Scalar>` fields can be extracted directly (yielding `FieldScalarValue::Option(Some(scalar))`
//! or `FieldScalarValue::Option(None)`), traversing *through* an `Option` to access fields within the `Some` variant
//! (e.g., `opt_struct.inner_field`) is currently **not supported**. The extraction path must target the `Option` itself.
//!
//!
// Declare modules
pub mod error;
pub mod extractor;
pub mod serializer;
pub mod value;

// Re-export public API
/// Errors that can occur during field extraction.
pub use error::EvaluateError;
/// Public interface for extracting top-level scalar field values.
pub use extractor::FieldExtractor;
/// Public interface for extracting nested scalar field values.
pub use extractor::NestedFieldExtractor;
/// Enum representing the possible scalar values that can be extracted.
pub use value::FieldScalarValue;
