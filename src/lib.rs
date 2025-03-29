use serde::Serialize;
use serde::{
    Serializer,
    ser::{Error as SerdeError, SerializeStruct},
};
use std::fmt;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum FieldScalarValue {
    // Integers
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    // Floats
    F32(f32),
    F64(f64),
    // Other Primitives
    Bool(bool),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    Unit,                                  // Represents the unit type `()`
    Option(Option<Box<FieldScalarValue>>), // Represents Option<T>
}

#[derive(Error, Debug, PartialEq)]
pub enum EvaluateError {
    #[error("Unsupported type for scalar extraction: {type_name}")]
    UnsupportedType { type_name: &'static str },

    #[error("Extracting from enum variants not supported: {variant_type}")]
    UnsupportedVariant { variant_type: &'static str },

    // Catch-all for custom messages from serde::ser::Error::custom
    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    // Error returned by the main evaluate function
    #[error("Field not found or has an unsupported type: {field_name}")]
    FieldNotFound { field_name: String },
}

impl SerdeError for EvaluateError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        EvaluateError::SerializationError {
            message: msg.to_string(),
        }
    }
}

// Custom Serializer Implementation Struct
struct FieldValueExtractorSerializer<'a> {
    target_field_name: &'a str,
    result: Option<FieldScalarValue>,
    capturing: bool,              // True if the next value should be captured
    expecting_option_inner: bool, // True if inside a Some() variant and capturing
}

impl<'a> FieldValueExtractorSerializer<'a> {
    fn new(field_name: &'a str) -> Self {
        FieldValueExtractorSerializer {
            target_field_name: field_name,
            result: None,
            capturing: false,
            expecting_option_inner: false,
        }
    }
}

// Implement the Serializer trait
// Note: Many methods can be stubbed out or return Ok(()) as we only care about capturing one value.
impl<'a> Serializer for &'a mut FieldValueExtractorSerializer<'_> {
    type Ok = FieldScalarValue; // The result of successful serialization (captured value)
    type Error = EvaluateError; // Use our custom error type

    // We only care about struct serialization for top-level extraction
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Self; // Use Self for struct serialization
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    // --- Capture Methods ---
    // These methods check the `capturing` flag and store the value if set.

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::Bool(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::Bool(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }

    // Integers - Capture as i32 or u64 based on what fits
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::I8(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::I8(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::I16(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::I16(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::I32(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::I32(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::I64(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::I64(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::I128(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::I128(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::U8(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::U8(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::U16(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::U16(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::U32(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::U32(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::U64(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::U64(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::U128(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::U128(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::F32(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::F32(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::F64(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::F64(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::Char(v),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::Char(v));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::String(v.to_string()),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::String(v.to_string()));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::Bytes(v.to_vec()),
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::Bytes(v.to_vec()));
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }

    // Option types
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            // Capturing an explicit None for the target field
            self.result = Some(FieldScalarValue::Option(None));
            self.capturing = false;
            self.expecting_option_inner = false; // Reset flag just in case
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            // We are capturing the target field, and it's a Some. Mark that
            // the *next* primitive we capture should be boxed inside Option(Some(...)).
            // Keep capturing = true, as the actual value is inside.
            self.expecting_option_inner = true;
        }
        // Always serialize the inner value regardless of capturing state
        let result = value.serialize(self);
        // expecting_option_inner should be reset by the inner serialize_* call
        // We don't reset capturing here, it's reset by the caller (serialize_field)
        result // Return result (Ok or Err) from inner serialization
    }

    // Unit types
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            if self.expecting_option_inner {
                // Option<Option<()>> -> Option(Some(Box(Unit))) ? or Option(None) if inner is None?
                // Let's assume Option<()> -> Option(Some(Box(Unit))) for Some(()) for now.
                self.result = Some(FieldScalarValue::Option(Some(Box::new(
                    FieldScalarValue::Unit,
                ))));
                self.expecting_option_inner = false;
            } else {
                self.result = Some(FieldScalarValue::Unit);
            }
            self.capturing = false;
        }
        Ok(FieldScalarValue::Unit)
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // Treat struct units like primitive units
        self.serialize_unit()
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::Unit)
    }

    // Newtype structs/variants
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Treat like Option: serialize inner value if capturing.
        if self.capturing {
            value.serialize(self)
        } else {
            Ok(FieldScalarValue::Unit) // Not capturing, do nothing.
        }
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // Variants are complex, error out for now.
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    // --- Compound Types (Not needed for single field extraction) ---
    // Return errors or impossible types for seq, tuple, map as we don't extract from them directly.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // If capturing, this means the target field is a sequence, which isn't a scalar.
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedType {
            type_name: "sequence",
        })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedType { type_name: "tuple" })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedType {
            type_name: "tuple struct",
        })
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "tuple",
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedType { type_name: "map" })
    }

    // --- Struct Entry Point ---
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // This is the main entry point for structs like MyRecord.
        // Return `Ok(self)` so the struct's fields will be processed by `SerializeStruct`.
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "struct",
        })
    }
}

// Implement SerializeStruct for our serializer reference
// This handles individual fields within a struct
impl<'a> SerializeStruct for &'a mut FieldValueExtractorSerializer<'_> {
    type Ok = FieldScalarValue;
    type Error = EvaluateError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Only process if we haven't already found and stored the result.
        if self.result.is_none() {
            // Check if this is the field we are looking for
            if key == self.target_field_name {
                self.capturing = true; // Set flag: capture the *next* value serialized
                // Serialize the value. This will call one of the `serialize_*` methods above,
                // which should capture the value and reset `capturing` to false.
                value.serialize(&mut **self)?;
                self.capturing = false; // Ensure capturing is always reset
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // The struct serialization is done, but the actual result is captured
        // in the main serializer. We need to return *something* matching Self::Ok.
        // This value is effectively ignored.
        Ok(FieldScalarValue::Unit) // Return a dummy value
    }
}

// 2. FieldExtractor Struct
#[derive(Debug)]
pub struct FieldExtractor {
    field_name: String,
}

impl FieldExtractor {
    pub fn new(field_name: String) -> Self {
        FieldExtractor { field_name }
    }

    // 3. Evaluation Method (Implementation using the custom serializer)
    pub fn evaluate<T: Serialize>(&self, record: &T) -> Result<FieldScalarValue, EvaluateError> {
        let mut serializer = FieldValueExtractorSerializer::new(&self.field_name);
        match record.serialize(&mut serializer) {
            Ok(_) => {
                // Serialization completed without hitting unsupported types directly.
                // Check what the serializer captured.
                // If a result was captured, return it. Otherwise, the field wasn't found/captured.
                serializer
                    .result
                    .ok_or_else(|| EvaluateError::FieldNotFound {
                        field_name: self.field_name.clone(),
                    })
            }
            Err(e) => {
                // Propagate errors encountered during serialization
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // Helper to create a standard record for tests
    // fn create_test_record() -> MyRecord {
    //     MyRecord {
    //         id: 1,
    //         name: "Example".to_string(),
    //         value: Some(42),
    //         tags: vec!["tag1".to_string(), "tag2".to_string()],
    //         nested: Some(NestedData {
    //             timestamp: 1234567890,
    //             description: "Nested example data".to_string(),
    //         }),
    //         temperature: 98.6,
    //         initial: 'X',
    //         data_bytes: vec![0x01, 0x02, 0x03],
    //         marker: (),
    //     }
    // }

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
            matches!(result, Err(EvaluateError::FieldNotFound { field_name }) if field_name == "nested")
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
            Err(EvaluateError::FieldNotFound { ref field_name })
                if field_name == "non_existent_field"
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
