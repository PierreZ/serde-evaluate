//! StringKeySerializer for extracting string keys from maps.
//!
//! Used to compare map keys against path segments during traversal.

use crate::error::EvaluateError;
use serde::{Serialize, Serializer};

/// Helper serializer to extract a string key from map serialization.
pub(crate) struct StringKeySerializer {
    pub(crate) key: Option<String>,
}

impl Serializer for &mut StringKeySerializer {
    type Ok = ();
    type Error = EvaluateError;

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

    // All non-string types return an error as map keys must be strings
    impl_key_reject_methods! {
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
        serialize_f32(f32),
        serialize_f64(f64),
        serialize_char(char),
        serialize_bytes(&[u8]),
        serialize_unit(),
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
