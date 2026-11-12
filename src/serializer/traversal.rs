//! Walks a value along the extraction path.
//!
//! Only the fields and map entries named by the path are serialized. When the
//! final segment is reached, the value is handed to the capture [`Mode`].

use crate::error::EvaluateError;
use serde::ser::{self, Impossible};
use serde::{Serialize, Serializer};

use super::capture::Mode;
use super::key::key_to_string;
use super::skip::Skip;

/// Result of a traversal that did not fail.
#[derive(Debug)]
pub(crate) enum Extraction<V> {
    /// The target was reached and captured.
    Found(V),
    /// The target was not reached. `failed_at_index` is the index of the first
    /// path segment that could not be matched.
    Missing { failed_at_index: usize },
}

/// Extracts the value at `path` from `record` using the capture mode `M`.
pub(crate) fn extract<M: Mode, T: ?Sized + Serialize>(
    record: &T,
    path: &[String],
) -> Result<Extraction<M::Output>, EvaluateError> {
    let mut traversal = Traversal::<M> {
        path,
        depth: 0,
        matched: 0,
        key_matched: false,
        found: None,
    };
    record.serialize(&mut traversal)?;
    Ok(match traversal.found {
        Some(value) => Extraction::Found(value),
        None => Extraction::Missing {
            failed_at_index: traversal.matched,
        },
    })
}

struct Traversal<'p, M: Mode> {
    path: &'p [String],
    /// Index of the path segment searched for in the container being serialized.
    depth: usize,
    /// Number of leading path segments matched so far (deepest point reached).
    matched: usize,
    /// Whether the last map key matched the current segment.
    key_matched: bool,
    found: Option<M::Output>,
}

impl<M: Mode> Traversal<'_, M> {
    fn matches(&self, key: &str) -> bool {
        self.found.is_none() && self.path.get(self.depth).is_some_and(|s| s == key)
    }

    /// Records that the segment at `depth` matched and returns the next depth.
    fn advance(&mut self) -> usize {
        let next = self.depth + 1;
        self.matched = self.matched.max(next);
        next
    }

    /// Continues into `value`, the child named by the current segment.
    fn descend<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), EvaluateError> {
        let next = self.advance();
        if next == self.path.len() {
            self.found = Some(M::capture(value)?);
            return Ok(());
        }
        self.depth = next;
        let result = value.serialize(&mut *self);
        self.depth -= 1;
        result
    }

    /// Handles a struct field that matched the current segment but was skipped
    /// by `#[serde(skip_serializing_if = ...)]`.
    fn skipped(&mut self) {
        if self.advance() == self.path.len() {
            self.found = Some(M::absent());
        }
    }
}

impl<'a, 'p, M: Mode> Serializer for &'a mut Traversal<'p, M> {
    type Ok = ();
    type Error = EvaluateError;

    type SerializeSeq = Skip;
    type SerializeTuple = Skip;
    type SerializeTupleStruct = Skip;
    type SerializeTupleVariant = Impossible<(), EvaluateError>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<(), EvaluateError>;

    // Scalars cannot contain the next segment: the path is not found.
    impl_ignore_methods! {
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
        serialize_str(&str),
        serialize_bytes(&[u8]),
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    /// `Some` is transparent: the path continues into the inner value.
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
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

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(Skip)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(Skip)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(Skip)
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
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
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

impl<M: Mode> ser::SerializeMap for &mut Traversal<'_, M> {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        // A key that cannot be turned into a string is simply not a match.
        self.key_matched = self.found.is_none()
            && self.depth < self.path.len()
            && key_to_string(key).is_some_and(|k| self.matches(&k));
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        if std::mem::take(&mut self.key_matched) {
            self.descend(value)
        } else {
            // Values that are not on the path are never serialized.
            Ok(())
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<M: Mode> ser::SerializeStruct for &mut Traversal<'_, M> {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.matches(key) {
            self.descend(value)
        } else {
            Ok(())
        }
    }

    fn skip_field(&mut self, key: &'static str) -> Result<(), Self::Error> {
        if self.matches(key) {
            self.skipped();
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
