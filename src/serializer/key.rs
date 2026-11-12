//! Converts map keys to strings so they can be compared with path segments.
//!
//! Follows serde_json's rules for map keys: strings, chars, integers, bools,
//! unit enum variants and newtype wrappers around any of those. Any other key
//! cannot be addressed by a path and is treated as a non-match.

use crate::error::EvaluateError;
use serde::ser::Impossible;
use serde::{Serialize, Serializer};

/// Returns the string form of a map key, or `None` if the key cannot be stringified.
pub(super) fn key_to_string<T: ?Sized + Serialize>(key: &T) -> Option<String> {
    key.serialize(KeySerializer).ok()
}

struct KeySerializer;

macro_rules! impl_to_string_methods {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                Ok(v.to_string())
            }
        )*
    };
}

const NOT_A_KEY: EvaluateError = EvaluateError::UnsupportedType {
    type_name: "map key",
};

impl Serializer for KeySerializer {
    type Ok = String;
    type Error = EvaluateError;

    type SerializeSeq = Impossible<String, EvaluateError>;
    type SerializeTuple = Impossible<String, EvaluateError>;
    type SerializeTupleStruct = Impossible<String, EvaluateError>;
    type SerializeTupleVariant = Impossible<String, EvaluateError>;
    type SerializeMap = Impossible<String, EvaluateError>;
    type SerializeStruct = Impossible<String, EvaluateError>;
    type SerializeStructVariant = Impossible<String, EvaluateError>;

    impl_to_string_methods! {
        serialize_str(&str),
        serialize_char(char),
        serialize_bool(bool),
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    impl_reject_methods! { NOT_A_KEY;
        fn serialize_f32(v: f32) -> Self::Ok;
        fn serialize_f64(v: f64) -> Self::Ok;
        fn serialize_bytes(v: &[u8]) -> Self::Ok;
        fn serialize_none() -> Self::Ok;
        fn serialize_some<T>(value: &T) -> Self::Ok;
        fn serialize_unit() -> Self::Ok;
        fn serialize_unit_struct(name: &'static str) -> Self::Ok;
        fn serialize_newtype_variant<T>(name: &'static str, index: u32, variant: &'static str, value: &T) -> Self::Ok;
        fn serialize_seq(len: Option<usize>) -> Self::SerializeSeq;
        fn serialize_tuple(len: usize) -> Self::SerializeTuple;
        fn serialize_tuple_struct(name: &'static str, len: usize) -> Self::SerializeTupleStruct;
        fn serialize_tuple_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeTupleVariant;
        fn serialize_map(len: Option<usize>) -> Self::SerializeMap;
        fn serialize_struct(name: &'static str, len: usize) -> Self::SerializeStruct;
        fn serialize_struct_variant(name: &'static str, index: u32, variant: &'static str, len: usize) -> Self::SerializeStructVariant;
    }
}
