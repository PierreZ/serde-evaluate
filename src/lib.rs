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
//! use serde_evaluate::{extractor::FieldExtractor, value::FieldScalarValue, error::EvaluateError};
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
pub mod error;
pub mod extractor;
pub mod serializer;
pub mod value;
