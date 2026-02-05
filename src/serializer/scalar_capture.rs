//! ScalarCaptureSerializer for capturing individual scalar values.
//!
//! Used by ListCapture to serialize individual list elements.

use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::{Serialize, Serializer};

/// A minimal serializer that captures exactly one scalar value.
/// Used to capture individual elements when extracting from a list.
pub(crate) struct ScalarCaptureSerializer {
    value: Option<FieldScalarValue>,
}

impl ScalarCaptureSerializer {
    pub(crate) fn new() -> Self {
        ScalarCaptureSerializer { value: None }
    }

    pub(crate) fn into_result(self) -> Option<FieldScalarValue> {
        self.value
    }
}

impl Serializer for &mut ScalarCaptureSerializer {
    type Ok = ();
    type Error = EvaluateError;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    // Use macro for simple scalar captures
    impl_scalar_capture_methods! {
        serialize_bool(bool) => Bool,
        serialize_i8(i8) => I8,
        serialize_i16(i16) => I16,
        serialize_i32(i32) => I32,
        serialize_i64(i64) => I64,
        serialize_i128(i128) => I128,
        serialize_u8(u8) => U8,
        serialize_u16(u16) => U16,
        serialize_u32(u32) => U32,
        serialize_u64(u64) => U64,
        serialize_u128(u128) => U128,
        serialize_f32(f32) => F32,
        serialize_f64(f64) => F64,
        serialize_char(char) => Char,
    }

    // These need special handling for conversion
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::String(v.to_string()));
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::Bytes(v.to_vec()));
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::Option(None));
        Ok(())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        // For Option<T> in list elements, serialize the inner value first
        value.serialize(&mut *self)?;
        // Wrap the captured value in Option(Some(...))
        if let Some(inner) = self.value.take() {
            self.value = Some(FieldScalarValue::Option(Some(Box::new(inner))));
        }
        Ok(())
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::Unit);
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::Unit);
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.value = Some(FieldScalarValue::Unit);
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
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
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "nested sequence",
        })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(EvaluateError::UnsupportedType { type_name: "tuple" })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
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
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "tuple",
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(EvaluateError::UnsupportedType { type_name: "map" })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: "struct",
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "struct",
        })
    }
}
