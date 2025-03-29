use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::ser::{self, SerializeStruct};
use serde::{Serialize, Serializer};

// Define a dummy struct to handle skipping compound types
pub(crate) struct Skip;

impl ser::SerializeTuple for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
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

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
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

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
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

    fn serialize_element<T: ?Sized>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
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
    path: Vec<String>,                // The full path to the target field
    current_path_index: usize,        // Current index/depth in the path traversal
    capturing: bool, // True if the next value should be captured (at the final path segment)
    expecting_option_inner: bool, // True if inside a Some() variant and capturing
    result: Option<FieldScalarValue>, // Stores the final extracted value
}

// Implement helper methods for the serializer
impl FieldValueExtractorSerializer {
    // Updated simple constructor: uses new_nested
    pub(crate) fn new(field_name: &str) -> Self {
        // Create a single-element slice for the path
        Self::new_nested(vec![field_name.to_string()])
        /* Old implementation:
        FieldValueExtractorSerializer {
            target_field_name: field_name,
            capturing: false,
            expecting_option_inner: false,
            result: None,
        }
        */
    }

    // Implement the nested constructor
    pub(crate) fn new_nested(path_segments: Vec<String>) -> Self {
        // This needs to be updated to use the new fields and correct type
        // unimplemented!("Implement new_nested() with path and current_path_index")
        FieldValueExtractorSerializer {
            path: path_segments,
            current_path_index: 0, // Start at the beginning of the path
            capturing: false,      // Not capturing initially
            expecting_option_inner: false,
            result: None,
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
                eprintln!(
                    "Warning: capture_value called while capturing=true but current_path_index ({}) != path.len() ({}). Path: {:?}",
                    self.current_path_index,
                    self.path.len(),
                    self.path
                );
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
    type SerializeMap = Skip;
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

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        // For newtype structs (e.g., struct Seconds(u64)), serialize the inner value.
        // The capturing flag state is passed through.
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
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
        if self.capturing {
            self.capturing = false;
            Err(EvaluateError::UnsupportedType { type_name: "map" })
        } else {
            // Skip the map content if not capturing.
            Ok(Skip)
        }
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // If capturing, the target field cannot be a struct itself
        if self.capturing {
            // This should ideally not happen if logic is correct, as capturing should only be true
            // when serialize_field encounters the *last* segment and serializes its value.
            self.capturing = false;
            eprintln!(
                "Warning: serialize_struct called while capturing=true. Path: {:?}, Index: {}",
                self.path, self.current_path_index
            );
            Err(EvaluateError::UnsupportedType {
                type_name: "struct (at capture)",
            })
        } else if self.path.is_empty() || self.current_path_index >= self.path.len() {
            // Path is exhausted or invalid, cannot proceed into struct meaningfully for extraction
            // If path is empty, FieldNotFound(""). If index >= len, means traversal went wrong somewhere.
            Err(EvaluateError::FieldNotFound {
                field_name: self.path.join("."),
            }) // Corrected construction
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

// Implement SerializeStruct to handle fields within a struct.
impl SerializeStruct for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
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
