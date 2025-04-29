pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

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
    Unit, // Represents the unit type `()`
    None, // Represents Option::None explicitly
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
    capturing: bool, // True if the next value should be captured
}

impl<'a> FieldValueExtractorSerializer<'a> {
    fn new(field_name: &'a str) -> Self {
        FieldValueExtractorSerializer {
            target_field_name: field_name,
            result: None,
            capturing: false,
        }
    }
}

// Implement the Serializer trait
// Note: Many methods can be stubbed out or return Ok(()) as we only care about capturing one value.
impl<'a> Serializer for &'a mut FieldValueExtractorSerializer<'_> {
    type Ok = ();
    type Error = EvaluateError; // Use our custom error type

    // We only care about struct serialization for top-level extraction
    type SerializeSeq = serde::ser::Impossible<(), Self::Error>;
    type SerializeTuple = serde::ser::Impossible<(), Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<(), Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<(), Self::Error>;
    type SerializeMap = serde::ser::Impossible<(), Self::Error>;
    type SerializeStruct = Self; // Use Self for struct serialization
    type SerializeStructVariant = serde::ser::Impossible<(), Self::Error>; // Variants not supported yet

    // --- Capture Methods ---
    // These methods check the `capturing` flag and store the value if set.

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::Bool(v));
            self.capturing = false;
        }
        Ok(())
    }

    // Integers - Capture as i32 or u64 based on what fits
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::I8(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::I16(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::I32(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::I64(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::I128(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::U8(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::U16(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::U32(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::U64(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::U128(v));
            self.capturing = false;
        }
        Ok(())
    }

    // Floats
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::F32(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::F64(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::Char(v));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::String(v.to_string()));
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::Bytes(v.to_vec()));
            self.capturing = false;
        }
        Ok(())
    }

    // Option types
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::None);
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        // If capturing an Option, just serialize the inner value.
        // The `capturing` flag remains true, so the inner value's serialize_* method handles the capture.
        value.serialize(self)
    }

    // Unit types
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        if self.capturing {
            self.result = Some(FieldScalarValue::Unit);
            self.capturing = false;
        }
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(())
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
            Ok(()) // Not capturing, do nothing.
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
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        // Only process if we haven't already found and stored the result.
        if self.result.is_none() {
            // Check if this is the field we are looking for
            if key == self.target_field_name {
                self.capturing = true; // Set flag: capture the *next* value serialized
                // Serialize the value. This will call one of the `serialize_*` methods above,
                // which should capture the value and reset `capturing` to false.
                value.serialize(&mut **self)?; // Pass the mutable reference correctly

                // If capturing is still true after the call, it means the value's type
                // wasn't one we explicitly handle (e.g., it was a sequence or map).
                // Reset the flag and potentially signal an error or specific state.
                if self.capturing {
                    self.capturing = false;
                    // For now, we implicitly treat this as 'field found but not a scalar',
                    // which will lead to evaluate returning the "not found or unsupported type" error.
                    // Alternatively, could set a specific error state here.
                }
            } else {
                // Not the target field, do nothing with the value.
                // Must return Ok(()) to allow Serde to proceed to the next field.
            }
        }
        // If result is Some, we've already found it, so just Ok.
        Ok(())
    }

    // Called after all fields are processed.
    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// 2. FieldExtractor Struct
#[derive(Debug)]
pub struct FieldExtractor {
    field_name: String,
}

impl FieldExtractor {
    // Add public constructor
    pub fn new(field_name: String) -> Self {
        FieldExtractor { field_name }
    }

    // 3. Evaluation Method (Implementation using the custom serializer)
    pub fn evaluate<T: Serialize>(&self, record: &T) -> Result<FieldScalarValue, EvaluateError> {
        let mut serializer = FieldValueExtractorSerializer::new(&self.field_name);
        match record.serialize(&mut serializer) {
            Ok(_) => {
                // Serialization succeeded, check if the serializer captured a result
                serializer.result.ok_or_else(|| {
                    // If no result was captured, the field wasn't found or wasn't a supported scalar type
                    EvaluateError::FieldNotFound {
                        field_name: self.field_name.clone(),
                    }
                })
            }
            Err(e) => {
                // Serialization itself failed (e.g., trying to extract a non-scalar like a sequence)
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
    struct MyRecord {
        id: u64,
        name: String,
        value: Option<i32>,
        tags: Vec<String>,
        nested: Option<NestedData>,
        temperature: f32,
        initial: char,
        #[serde(with = "serde_bytes")]
        data_bytes: Vec<u8>,
        marker: (),
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    struct NestedData {
        timestamp: u64,
        description: String,
    }

    // Helper to create a standard record for tests
    fn create_test_record() -> MyRecord {
        MyRecord {
            id: 1,
            name: "Example".to_string(),
            value: Some(42),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            nested: Some(NestedData {
                timestamp: 1234567890,
                description: "Nested example data".to_string(),
            }),
            temperature: 98.6,
            initial: 'X',
            data_bytes: vec![0x01, 0x02, 0x03],
            marker: (),
        }
    }

    #[test]
    fn initial_setup_compiles() {
        let record = create_test_record();
        assert_eq!(record.id, 1);
    }

    // --- FieldExtractor Tests ---

    #[test]
    fn test_extract_string_field() {
        let record = create_test_record();
        let extractor = FieldExtractor::new("name".to_string());
        let result = extractor.evaluate(&record);
        assert_eq!(result, Ok(FieldScalarValue::String("Example".to_string())));
    }

    #[test]
    fn test_extract_int_field_some() {
        let record = create_test_record();
        let extractor = FieldExtractor::new("value".to_string());
        let result = extractor.evaluate(&record);
        // Note: 'value' is Option<i32>. The serializer handles Some(v) -> v
        assert_eq!(result, Ok(FieldScalarValue::I32(42)));
    }

    #[test]
    fn test_extract_uint_field() {
        let record = create_test_record();
        let extractor = FieldExtractor::new("id".to_string());
        let result = extractor.evaluate(&record);
        // Note: id is u64, captured as Uint
        assert_eq!(result, Ok(FieldScalarValue::U64(1)));
    }

    #[test]
    fn test_extract_option_field_none() {
        let mut record = create_test_record();
        record.value = None; // Set the optional field to None
        let extractor = FieldExtractor::new("value".to_string());
        let result = extractor.evaluate(&record);
        assert_eq!(result, Ok(FieldScalarValue::None));
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
        let extractor = FieldExtractor::new("tags".to_string());
        let result = extractor.evaluate(&record);
        assert!(
            matches!(result, Err(EvaluateError::UnsupportedType { type_name }) if type_name == "sequence")
        );
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
        assert_eq!(result, Ok(FieldScalarValue::Bytes(vec![0x01, 0x02, 0x03])));
    }

    #[test]
    fn test_extract_unit_field() {
        let record = create_test_record();
        let extractor = FieldExtractor::new("marker".to_string());
        let result = extractor.evaluate(&record);
        assert_eq!(result, Ok(FieldScalarValue::Unit));
    }

    // TODO: Add tests for nested field access (e.g., "nested.timestamp") - requires path parsing in FieldExtractor
    // TODO: Add tests for more complex type mismatches if needed.
}
