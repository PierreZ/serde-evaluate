use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::ser::{self /*, SerializeStruct */}; // Removed unused import
use serde::{Serialize, Serializer};

// Define a dummy struct to handle skipping compound types
pub(crate) struct Skip;

impl ser::SerializeTuple for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeMap for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeSeq for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// Custom Serializer Implementation Struct
// This struct drives the serialization process, looking for the target field path.
pub(crate) struct FieldValueExtractorSerializer {
    path: Vec<String>,                   // The full path to the target field
    current_path_index: usize,           // Current index/depth in the path traversal
    capturing: bool, // True if the next value should be captured (at the final path segment)
    expecting_option_inner: bool, // True if inside a Some() variant and capturing
    result: Option<FieldScalarValue>, // Stores the final extracted value
    current_map_key_match: Option<bool>, // Tracks if the current map key matched the path segment
}

// Implement helper methods for the serializer
impl FieldValueExtractorSerializer {
    // Updated simple constructor: uses new_nested
    pub(crate) fn new(field_name: &str) -> Self {
        // Create a single-element slice for the path
        Self::new_nested(vec![field_name.to_string()])
    }

    // Implement the nested constructor
    pub(crate) fn new_nested(path_segments: Vec<String>) -> Self {
        FieldValueExtractorSerializer {
            path: path_segments,
            current_path_index: 0, // Start at the beginning of the path
            capturing: false,      // Not capturing initially
            expecting_option_inner: false,
            result: None,
            current_map_key_match: None, // Initialize map key match state
        }
    }

    // Called by individual serialize_* methods.
    // If capturing is true, stores the value (wrapping in Option if needed)
    // and resets flags.
    // This needs update to check current_path_index vs path.len()
    fn capture_value(&mut self, value: FieldScalarValue) {
        if self.capturing {
            // Only capture if we have reached the end of the specified path.
            if self.current_path_index == self.path.len() {
                self.result = if self.expecting_option_inner {
                    Some(FieldScalarValue::Option(Some(Box::new(value))))
                } else {
                    Some(value)
                };
                // Reset flags after successful capture at the correct path depth.
                self.capturing = false;
                self.expecting_option_inner = false;
            } else {
                // If capturing is true but index hasn't reached the end, it's an internal logic error
                // in how capturing was set (likely in serialize_field or serialize_some).
                // Reset flags defensively, but don't store the value.
                self.capturing = false;
                self.expecting_option_inner = false;
            }
        }
    }

    pub(crate) fn into_result(self) -> Option<FieldScalarValue> {
        self.result
    }
}

// Implement the Serializer trait
// Delegates most primitive types to capture_value.
// Handles Option and compound types specifically.
impl Serializer for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    type SerializeSeq = Skip;
    type SerializeTuple = Skip;
    type SerializeTupleStruct = Skip;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Self; // Use Self for map serialization
    type SerializeStruct = Self; // Use Self for struct serialization
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Bool(v));
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I8(v));
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I16(v));
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I32(v));
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I64(v));
        Ok(())
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I128(v));
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U8(v));
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U16(v));
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U32(v));
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U64(v));
        Ok(())
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U128(v));
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::F32(v));
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::F64(v));
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Char(v));
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::String(v.to_string()));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Bytes(v.to_vec()));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // Called for Option::None
        if self.capturing {
            // Capturing an explicit None for the target field
            self.result = Some(FieldScalarValue::Option(None));
            self.capturing = false;
            self.expecting_option_inner = false; // Reset flag just in case
        }
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Called for Option::Some(value)
        if self.capturing {
            // Target field is an Option::Some. Set flag to wrap the *next* value captured.
            self.expecting_option_inner = true;
            // Don't reset self.capturing; the inner value serialization needs it.
        }
        // Serialize the inner value. This will call a serialize_* method which
        // *might* capture the value (if self.capturing is still true) and will
        // reset expecting_option_inner via capture_value.
        let result = value.serialize(&mut *self);
        // Reset flag *after* inner serialization, in case inner wasn't captured.
        self.expecting_option_inner = false;
        result
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Unit);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // This case might occur for `Option<MyUnitEnum::Variant>`
        // Treat like Option<()>, capture a Unit if needed.
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // For newtype structs (e.g., struct Seconds(u64)), serialize the inner value.
        // The capturing flag state is passed through.
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Capturing from inside enum variants is not supported.
        if self.capturing {
            self.capturing = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    // --- Methods for Unsupported Compound Types ---
    // These return an error if self.capturing is true, because the target field
    // itself cannot be a sequence, map, etc., it must be a scalar or Option<Scalar>.

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if self.capturing {
            self.capturing = false;
            Err(EvaluateError::UnsupportedType {
                type_name: "sequence",
            })
        } else {
            // If not capturing, we might be inside a struct field that *is* a sequence,
            // but it's not our target. Allow serialization to proceed to skip it.
            Ok(Skip)
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        if self.capturing {
            self.capturing = false;
            Err(EvaluateError::UnsupportedType { type_name: "tuple" })
        } else {
            // Skip the tuple content if not capturing.
            Ok(Skip)
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if self.capturing {
            self.capturing = false;
            Err(EvaluateError::UnsupportedType {
                type_name: "tuple struct",
            })
        } else {
            // Skip the tuple struct content if not capturing.
            Ok(Skip)
        }
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
        // Check if we are at the end of the path and expecting a scalar
        if self.path.is_empty() {
            // Should not happen with NestedFieldExtractor
            // This case indicates an internal inconsistency or misuse.
            return Err(EvaluateError::FieldNotFound {
                field_name: "<internal error: empty path>".to_string(),
            });
        }
        if self.current_path_index >= self.path.len() {
            // Path exhausted, but we got a map. This is only valid if we are NOT capturing the final value.
            // If we were capturing, it means the path pointed to a map, not a scalar.
            if self.capturing {
                // Path ended here, but it's a map, not a scalar.
                self.capturing = false; // Stop capturing
                return Err(EvaluateError::UnsupportedType {
                    type_name: "map", // Path pointed to a map
                });
            } else {
                // Path exhausted and not capturing, treat self as the SerializeMap state directly.
                // This skips the map's content, which is correct.
                return Ok(self);
            }
        }

        // If we are here, current_path_index < path.len().
        // We are expecting to traverse *into* this map.
        // Ensure capturing is true because we need to process keys/values.
        self.capturing = true;

        // Okay to proceed, return self to handle map key/value pairs
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // Check if path is exhausted OR if we are capturing the final value
        if self.current_path_index >= self.path.len() {
            if self.capturing {
                // Path exhausted, but we got a struct, and we are capturing.
                // This means the final target path element pointed to a struct, not a scalar.
                // Use capture_unsupported helper
                self.capturing = false;
                Err(EvaluateError::UnsupportedType {
                    type_name: "struct",
                })
            } else {
                // Path is exhausted but not capturing. This is fine, we're just skipping.
                Ok(self)
            }
        } else {
            // Allow serialization to proceed into the struct's fields
            Ok(self)
        }
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

// Implement SerializeMap to handle key-value pairs within a map.
impl ser::SerializeMap for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Check if we should even attempt to match the key
        if !self.capturing || self.current_path_index >= self.path.len() {
            self.current_map_key_match = Some(false);
            return Ok(());
        }

        // Use StringKeySerializer to capture the key as a string
        let mut key_serializer = StringKeySerializer { key: None };
        key.serialize(&mut key_serializer)?;
        let captured_key = key_serializer.key;
        if let Some(key_str) = captured_key {
            let expected_key = &self.path[self.current_path_index];
            if key_str == *expected_key {
                // Dereference expected_key
                // Key matches the current path segment!
                // Set flag, but DO NOT increment index here.
                self.current_map_key_match = Some(true);
            } else {
                // Key does not match the current path segment
                self.current_map_key_match = Some(false);
            }
        } else {
            // Key was not a string, cannot match path segment
            self.current_map_key_match = None; // Indicate key wasn't string/error
        }
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key_match_status = self.current_map_key_match.take(); // Consume the match status

        match key_match_status {
            Some(true) => {
                // Key matched! Serialize the value, advancing the path index for the nested call.
                self.current_path_index += 1;
                let res = value.serialize(&mut **self); // Recurse/capture
                                                        // Backtrack the index *after* the value serialization is complete.
                self.current_path_index -= 1;
                res?; // Propagate any errors from the value serialization
            }
            Some(false) | None => {
                // Key did not match, wasn't a string, or error. Skip the value.
                let mut dummy_serializer = FieldValueExtractorSerializer {
                    path: vec![],
                    current_path_index: 1, // Ensures path is considered "exhausted"
                    result: None,
                    capturing: false,
                    expecting_option_inner: false, // Added default
                    current_map_key_match: None,   // Added default
                };
                value.serialize(&mut dummy_serializer)?;
            }
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // No error checking needed here. The extractor will determine if the
        // full path was successfully resolved by checking the final result.
        Ok(())
    }
}

// Implement SerializeStruct to handle fields within a struct.
impl ser::SerializeStruct for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // If we've already found the result, skip remaining fields.
        if self.result.is_some() {
            return Ok(());
        }

        // Check if path is valid and key matches the *current* segment
        if self.current_path_index < self.path.len() {
            let expected_key = &self.path[self.current_path_index];
            if key == expected_key {
                // Match found for the current path segment.
                let is_last_segment = self.current_path_index == self.path.len() - 1;

                if is_last_segment {
                    // This is the final segment. Set capturing and serialize the value.
                    self.capturing = true;
                    // Increment index *before* serializing value, so capture_value check works correctly.
                    self.current_path_index += 1;
                    let res = value.serialize(&mut **self);
                    // Reset capturing flag defensively after serialization attempt.
                    self.capturing = false;
                    // Decrement index back *after* serialization of this field is done.
                    // Allows subsequent fields at the *parent* level to be processed correctly if needed (though result is already set usually).
                    self.current_path_index -= 1;
                    res?; // Propagate error if serialization/capture failed.
                } else {
                    // Not the last segment. Need to go deeper.
                    // Increment index before recursing into the value.
                    self.current_path_index += 1;
                    // Serialize the nested value. This might call serialize_struct again.
                    let res = value.serialize(&mut **self);
                    // Decrement index *after* recursing. Backtrack for the next field at the current level.
                    self.current_path_index -= 1;
                    res?; // Propagate error if nested serialization failed.
                }
            } else {
                // Key doesn't match the current path segment. Skip this field.
                // Serialize the value anyway to allow Serde to skip its content.
                // Ensure capturing is false during the skip.
                let was_capturing = self.capturing;
                self.capturing = false;
                let _ = value.serialize(&mut **self); // Ignore result/errors for non-target fields
                self.capturing = was_capturing; // Restore capturing state if it was somehow true?
            }
        } else {
            // current_path_index >= path.len(). This shouldn't be reached if serialize_struct check works.
            // Means we are serializing fields after the target path was fully traversed or failed.
            // We can simply skip serializing further fields in this struct.
            // However, Serde expects all fields to be serialized, so we still call serialize.
            let _ = value.serialize(&mut **self);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // End of struct. If we traversed the whole path but didn't capture a value,
        // it means the final field was not found or was an unsupported type.
        // The `into_result` method combined with initial state handles FieldNotFound.
        // Errors during traversal (e.g., UnsupportedType) are returned earlier.
        Ok(())
    }
}

// Helper serializer to extract a string key
struct StringKeySerializer {
    key: Option<String>,
}

impl Serializer for &mut StringKeySerializer {
    type Ok = ();
    type Error = EvaluateError;
    // Implement only the methods needed to capture a string
    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.key = Some(v.to_string());
        Ok(())
    }

    // Return error for any other type attempted as key
    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_i128(self, _v: i128) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_u128(self, _v: u128) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(EvaluateError::UnsupportedType {
            type_name: "Option",
        })
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "Map key must be a string",
        })
    }
}
