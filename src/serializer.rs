use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::ser::{self};
use serde::{Serialize, Serializer};

// Helper function to wrap a value in N levels of Option(Some(...))
fn wrap_in_options(value: FieldScalarValue, level: u8) -> FieldScalarValue {
    let mut current = value;
    for _ in 0..level {
        current = FieldScalarValue::Option(Some(Box::new(current)));
    }
    current
}

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

// -----------------------------------------------------------------------------

// Custom Serializer Implementation Struct
pub(crate) struct FieldValueExtractorSerializer {
    path: Vec<String>,                   // Target path segments
    current_path_index: usize,           // Current depth in the path traversal
    value: Option<FieldScalarValue>,     // Stores the final extracted value
    current_map_key_match: Option<bool>, // Tracks if the current map key matched the path segment
    ready_to_capture: bool, // True if the *next* serialize_* call is for the final target value
    option_nesting_level: u8, // Tracks nesting level when capturing Option<Option<...>>
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
            current_path_index: 0,
            value: None,
            current_map_key_match: None,
            ready_to_capture: false, // Initialize flags
            option_nesting_level: 0,
        }
    }

    // Called by individual scalar serialize_* methods.
    // Captures the value if ready_to_capture flag is set,
    // potentially wrapping based on option_nesting_level.
    fn capture_value(&mut self, value: FieldScalarValue) -> Result<(), EvaluateError> {
        if self.ready_to_capture {
            // Wrap the value according to the current nesting level.
            self.value = Some(wrap_in_options(value, self.option_nesting_level));
            // Reset nesting level? No, let serialize_some handle decrementing.
            // Do not reset ready_to_capture here; the caller (serialize_field/value) manages it.
        }
        // If not ready_to_capture, do nothing.
        Ok(())
    }

    pub(crate) fn into_result(self) -> Option<FieldScalarValue> {
        self.value
    }
}

// Implement the main Serializer trait for our extractor
// Most methods either delegate to capture_value, handle path traversal, or return UnsupportedType/Skip.
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
        self.capture_value(FieldScalarValue::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I8(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I16(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I32(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I64(v))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::I128(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U8(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U16(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U32(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U64(v))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::U128(v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::F32(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::F64(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Char(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.ready_to_capture {
            // Construct the nested None value based on the *current* nesting level.
            let none_value =
                wrap_in_options(FieldScalarValue::Option(None), self.option_nesting_level);
            // Directly set the value, bypassing capture_value which would wrap again.
            self.value = Some(none_value);
            Ok(())
        } else {
            // Not capturing, None is just part of structure traversal.
            Ok(())
        }
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        if self.ready_to_capture {
            // Increment nesting level *before* serializing inner value.
            let original_level = self.option_nesting_level;
            self.option_nesting_level =
                original_level
                    .checked_add(1)
                    .ok_or(EvaluateError::UnsupportedType {
                        type_name: "Deeply Nested Option (>255 levels)",
                    })?;
            let result = value.serialize(&mut *self); // Serialize the inner value
                                                      // Restore nesting level *after* serializing inner value.
            self.option_nesting_level = original_level;
            result
        } else {
            // Not capturing, just pass through serialization
            value.serialize(&mut *self)
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Unit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Unit)
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
        if self.ready_to_capture {
            self.ready_to_capture = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    // --- Methods for Unsupported Compound Types ---
    // These return an error if self.capturing is true, because the target field
    // itself cannot be a sequence, map, etc., it must be a scalar or Option<Scalar>.

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if self.ready_to_capture {
            self.ready_to_capture = false;
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
        if self.ready_to_capture {
            self.ready_to_capture = false;
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
        if self.ready_to_capture {
            self.ready_to_capture = false;
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
        if self.ready_to_capture {
            self.ready_to_capture = false;
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
            if self.ready_to_capture {
                // Path ended here, but it's a map, not a scalar.
                self.ready_to_capture = false; // Stop capturing
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
        self.ready_to_capture = true;

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
            if self.ready_to_capture {
                // Path exhausted, but we got a struct, and we are capturing.
                // This means the final target path element pointed to a struct, not a scalar.
                // Use capture_unsupported helper
                self.ready_to_capture = false;
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
        if self.ready_to_capture {
            self.ready_to_capture = false;
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

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        // Reset key match state for each new key
        self.current_map_key_match = None;

        // Check if we are still traversing the path
        if self.value.is_none() && self.current_path_index < self.path.len() {
            // We need to compare this key against the current path segment.
            // Serialize the key to a string to compare.
            let mut key_serializer = StringKeySerializer { key: None };
            key.serialize(&mut key_serializer)?;

            if let Some(key_str) = key_serializer.key {
                if key_str == self.path[self.current_path_index] {
                    // Key matches the current path segment.
                    self.current_map_key_match = Some(true);
                } else {
                    // Key does not match.
                    self.current_map_key_match = Some(false);
                }
            } else {
                // Key wasn't a string, cannot match path segment.
                self.current_map_key_match = Some(false);
                // Potentially return an error? Or just skip?
                // Let's just skip the corresponding value.
            }
        } else {
            // Path exhausted or value already found, mark as no match.
            self.current_map_key_match = Some(false);
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        // Process based on whether the key matched
        match self.current_map_key_match.take() {
            // Consume the match state
            Some(true) => {
                // Key matched the current path segment.
                self.current_path_index += 1; // Move deeper
                let is_last_segment = self.current_path_index == self.path.len();

                let result = if is_last_segment {
                    // This is the target value.
                    self.ready_to_capture = true;
                    let res = value.serialize(&mut **self); // Serialize (may capture)
                    self.ready_to_capture = false; // Reset flag
                    res
                } else {
                    // Not the last segment, continue traversal.
                    value.serialize(&mut **self) // Serialize without setting capture flag
                };

                self.current_path_index -= 1; // Move back up after processing/traversing
                result
            }
            Some(false) | None => {
                // Key didn't match, or state was missing. Skip this value.
                // We still need to call serialize to allow Serde to process/skip the value's structure.
                let mut dummy_serializer = FieldValueExtractorSerializer::new_nested(vec![]); // Dummy state
                let _ = value.serialize(&mut dummy_serializer); // Ignore result/error
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // No specific action needed when ending map serialization in this context.
        Ok(())
    }
}

// Implement SerializeStruct to handle fields within a struct.
impl ser::SerializeStruct for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        // If we've already found the result, skip remaining fields.
        if self.value.is_some() {
            return Ok(());
        }

        // Check if we are still traversing the path
        if self.current_path_index < self.path.len() {
            // Compare the field key with the current path segment
            if key == self.path[self.current_path_index] {
                // Key matches.
                self.current_path_index += 1; // Move deeper
                let is_last_segment = self.current_path_index == self.path.len();

                let result = if is_last_segment {
                    // This is the target field's value.
                    self.ready_to_capture = true;
                    let res = value.serialize(&mut **self); // Serialize (may capture)
                    self.ready_to_capture = false; // Reset flag
                    res
                } else {
                    // Not the last segment, continue traversal into the value.
                    value.serialize(&mut **self) // Serialize without setting capture flag
                };

                self.current_path_index -= 1; // Move back up after processing/traversing
                return result; // Return result of serialization (could be error)
            }
        }

        // If key didn't match, or path index is beyond length, or value already found: skip.
        // We don't need to explicitly serialize to skip for structs like we might for maps/seqs,
        // just returning Ok allows SerializeStruct loop to continue to the next field.
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Check if path wasn't fully resolved after struct ended (if we were expecting more nesting)
        if self.value.is_none()
            && self.current_path_index > 0
            && self.current_path_index < self.path.len()
        {
            // We finished serializing a struct, but we were expecting more path segments.
            // This indicates the path was invalid for this structure.
            // We should have already bailed in serialize_struct or serialize_map if the path ended early.
            // This check might be redundant but acts as a safeguard.
            // Let's return FieldNotFound based on the *expected* next segment.
            Err(EvaluateError::NestedFieldNotFound {
                path: self.path.clone(),
            })
        } else {
            Ok(())
        }
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
