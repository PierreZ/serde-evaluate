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
// This struct drives the serialization process, looking for the target field.
pub(crate) struct FieldValueExtractorSerializer<'a> {
    target_field_name: &'a str,
    capturing: bool,                  // True if the next value should be captured
    expecting_option_inner: bool,     // True if inside a Some() variant and capturing
    result: Option<FieldScalarValue>, // Stores the final extracted value
}

// Implement helper methods for the serializer
impl<'a> FieldValueExtractorSerializer<'a> {
    pub(crate) fn new(field_name: &'a str) -> Self {
        FieldValueExtractorSerializer {
            target_field_name: field_name,
            capturing: false,
            expecting_option_inner: false,
            result: None,
        }
    }

    // Called by individual serialize_* methods.
    // If capturing is true, stores the value (wrapping in Option if needed)
    // and resets flags.
    fn capture_value(&mut self, value: FieldScalarValue) {
        if self.capturing {
            self.result = if self.expecting_option_inner {
                Some(FieldScalarValue::Option(Some(Box::new(value))))
            } else {
                Some(value)
            };
            self.capturing = false;
            self.expecting_option_inner = false; // Always reset this after capture
        }
    }

    pub(crate) fn into_result(self) -> Option<FieldScalarValue> {
        self.result
    }
}

// Implement the Serializer trait
// Delegates most primitive types to capture_value.
// Handles Option and compound types specifically.
impl<'a> Serializer for &'a mut FieldValueExtractorSerializer<'_> {
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
        if self.capturing {
            // Found the target field, but it's a struct (unsupported)
            self.capturing = false; // Stop capturing
            Err(EvaluateError::UnsupportedType {
                type_name: "struct",
            })
        } else {
            // Not capturing, just skip the struct content
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
impl<'a> SerializeStruct for &'a mut FieldValueExtractorSerializer<'_> {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        // Check if this is the field we are looking for *before* serializing value
        if key == self.target_field_name {
            // If we already found it (e.g., duplicate field name?), ignore subsequent ones.
            if self.result.is_none() {
                self.capturing = true;
                // Serialize the value. This will call a serialize_* method.
                // That method will capture the value if it's a scalar/option
                // and reset self.capturing = false.
                // Or it will return UnsupportedType if the value is not extractable.
                value.serialize(&mut **self)?;
                // Defensive reset in case serialize_* didn't run capture_value (e.g., error)
                self.capturing = false;
            }
        } else {
            // Not the target field, but we still need to serialize it to advance the process.
            // The result is ignored, effectively skipping the field's content.
            // Pass the serializer through; it will ignore the value since capturing is false.
            let _ = value.serialize(&mut **self); // Ignore result, errors handled inside?
            // Alternative: value.serialize(&mut **self)?; // Propagate errors?
            // Let's stick with ignoring for now, assuming non-target fields shouldn't error out the whole process.
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // The end of serializing a struct's fields.
        // The actual result (if captured) is stored in self.result.
        // Return the standard Ok unit type.
        Ok(())
    }
}
