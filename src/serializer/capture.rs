//! Captures the value found at the end of the extraction path.
//!
//! [`ScalarMode`] captures one [`FieldScalarValue`]. [`ListMode`] captures a
//! sequence whose elements are each captured as scalars.

use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::ser::{self, Impossible};
use serde::{Serialize, Serializer};

/// How the value at the end of the path is captured.
pub(crate) trait Mode {
    type Output;

    /// Captures `value`, the target of the path.
    fn capture<T: ?Sized + Serialize>(value: &T) -> Result<Self::Output, EvaluateError>;

    /// The value reported when the target field was skipped during serialization
    /// (`#[serde(skip_serializing_if = ...)]`).
    fn absent() -> Self::Output;
}

/// Captures a single scalar value.
pub(crate) struct ScalarMode;

impl Mode for ScalarMode {
    type Output = FieldScalarValue;

    fn capture<T: ?Sized + Serialize>(value: &T) -> Result<Self::Output, EvaluateError> {
        value.serialize(ScalarCapture { in_list: false })
    }

    fn absent() -> Self::Output {
        FieldScalarValue::Option(None)
    }
}

/// Captures a sequence of scalar values.
pub(crate) struct ListMode;

impl Mode for ListMode {
    type Output = Vec<FieldScalarValue>;

    fn capture<T: ?Sized + Serialize>(value: &T) -> Result<Self::Output, EvaluateError> {
        value.serialize(ListCapture)
    }

    fn absent() -> Self::Output {
        Vec::new()
    }
}

// =============================================================================
// Scalar capture
// =============================================================================

/// Serializer producing one [`FieldScalarValue`].
#[derive(Clone, Copy)]
struct ScalarCapture {
    /// True when capturing a list element, used to name nested sequences in errors.
    in_list: bool,
}

impl Serializer for ScalarCapture {
    type Ok = FieldScalarValue;
    type Error = EvaluateError;

    type SerializeSeq = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeTuple = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeTupleStruct = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeTupleVariant = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeMap = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeStruct = Impossible<FieldScalarValue, EvaluateError>;
    type SerializeStructVariant = Impossible<FieldScalarValue, EvaluateError>;

    impl_scalar_methods! {
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

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::Option(None))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        let inner = value.serialize(self)?;
        Ok(FieldScalarValue::Option(Some(Box::new(inner))))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::Unit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::Unit)
    }

    /// Unit variants are captured by name, as serde_json does.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(FieldScalarValue::String(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(EvaluateError::UnsupportedType {
            type_name: if self.in_list {
                "nested sequence"
            } else {
                "sequence"
            },
        })
    }

    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "newtype" };
        fn serialize_newtype_variant<T>(name: &'static str, index: u32, variant: &'static str, value: &T) -> Self::Ok;
    }
    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "tuple" };
        fn serialize_tuple_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeTupleVariant;
    }
    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "struct" };
        fn serialize_struct_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeStructVariant;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "tuple" };
        fn serialize_tuple(len: usize) -> Self::SerializeTuple;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "tuple struct" };
        fn serialize_tuple_struct(name: &'static str, len: usize) -> Self::SerializeTupleStruct;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "map" };
        fn serialize_map(len: Option<usize>) -> Self::SerializeMap;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "struct" };
        fn serialize_struct(name: &'static str, len: usize) -> Self::SerializeStruct;
    }
}

// =============================================================================
// List capture
// =============================================================================

/// Serializer producing the elements of a sequence as scalars.
struct ListCapture;

/// Error returned when the list target is not a sequence.
const NOT_A_LIST: EvaluateError = EvaluateError::UnsupportedType {
    type_name: "scalar (expected a sequence)",
};

impl Serializer for ListCapture {
    type Ok = Vec<FieldScalarValue>;
    type Error = EvaluateError;

    type SerializeSeq = ListCollector;
    type SerializeTuple = ListCollector;
    type SerializeTupleStruct = Impossible<Vec<FieldScalarValue>, EvaluateError>;
    type SerializeTupleVariant = Impossible<Vec<FieldScalarValue>, EvaluateError>;
    type SerializeMap = Impossible<Vec<FieldScalarValue>, EvaluateError>;
    type SerializeStruct = Impossible<Vec<FieldScalarValue>, EvaluateError>;
    type SerializeStructVariant = Impossible<Vec<FieldScalarValue>, EvaluateError>;

    /// `None` is an empty list.
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Vec::new())
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    /// Unit (for example JSON `null`) is an empty list, like `None`.
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Vec::new())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(ListCollector::with_capacity(len.unwrap_or(0)))
    }

    /// Fixed-size arrays (`[T; N]`) are serialized as tuples.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(ListCollector::with_capacity(len))
    }

    impl_reject_methods! { NOT_A_LIST;
        fn serialize_bool(v: bool) -> Self::Ok;
        fn serialize_i8(v: i8) -> Self::Ok;
        fn serialize_i16(v: i16) -> Self::Ok;
        fn serialize_i32(v: i32) -> Self::Ok;
        fn serialize_i64(v: i64) -> Self::Ok;
        fn serialize_i128(v: i128) -> Self::Ok;
        fn serialize_u8(v: u8) -> Self::Ok;
        fn serialize_u16(v: u16) -> Self::Ok;
        fn serialize_u32(v: u32) -> Self::Ok;
        fn serialize_u64(v: u64) -> Self::Ok;
        fn serialize_u128(v: u128) -> Self::Ok;
        fn serialize_f32(v: f32) -> Self::Ok;
        fn serialize_f64(v: f64) -> Self::Ok;
        fn serialize_char(v: char) -> Self::Ok;
        fn serialize_str(v: &str) -> Self::Ok;
        fn serialize_unit_struct(name: &'static str) -> Self::Ok;
        fn serialize_unit_variant(name: &'static str, index: u32, variant: &'static str) -> Self::Ok;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "bytes (expected a sequence)" };
        fn serialize_bytes(v: &[u8]) -> Self::Ok;
    }
    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "newtype" };
        fn serialize_newtype_variant<T>(name: &'static str, index: u32, variant: &'static str, value: &T) -> Self::Ok;
    }
    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "tuple" };
        fn serialize_tuple_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeTupleVariant;
    }
    impl_reject_methods! { EvaluateError::UnsupportedVariant { variant_type: "struct" };
        fn serialize_struct_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeStructVariant;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "tuple struct" };
        fn serialize_tuple_struct(name: &'static str, len: usize) -> Self::SerializeTupleStruct;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "map" };
        fn serialize_map(len: Option<usize>) -> Self::SerializeMap;
    }
    impl_reject_methods! { EvaluateError::UnsupportedType { type_name: "struct" };
        fn serialize_struct(name: &'static str, len: usize) -> Self::SerializeStruct;
    }
}

/// Collects sequence or tuple elements, capturing each one as a scalar.
struct ListCollector {
    items: Vec<FieldScalarValue>,
}

impl ListCollector {
    fn with_capacity(len: usize) -> Self {
        ListCollector {
            items: Vec::with_capacity(len),
        }
    }

    fn push<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), EvaluateError> {
        self.items
            .push(value.serialize(ScalarCapture { in_list: true })?);
        Ok(())
    }
}

impl ser::SerializeSeq for ListCollector {
    type Ok = Vec<FieldScalarValue>;
    type Error = EvaluateError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.push(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.items)
    }
}

impl ser::SerializeTuple for ListCollector {
    type Ok = Vec<FieldScalarValue>;
    type Error = EvaluateError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.push(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.items)
    }
}
