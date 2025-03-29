use serde::{Deserialize, Serialize};

/// Represents the scalar value extracted from a field.
///
/// This enum covers the range of primitive types and simple collections (like `Vec<u8>`)
/// that the [`FieldExtractor`](crate::FieldExtractor) can successfully extract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldScalarValue {
    /// Unit value (`()`).
    Unit,
    /// Boolean value (`bool`).
    Bool(bool),
    /// Signed 8-bit integer (`i8`).
    I8(i8),
    /// Signed 16-bit integer (`i16`).
    I16(i16),
    /// Signed 32-bit integer (`i32`).
    I32(i32),
    /// Signed 64-bit integer (`i64`).
    I64(i64),
    /// Signed 128-bit integer (`i128`).
    I128(i128),
    /// Unsigned 8-bit integer (`u8`).
    U8(u8),
    /// Unsigned 16-bit integer (`u16`).
    U16(u16),
    /// Unsigned 32-bit integer (`u32`).
    U32(u32),
    /// Unsigned 64-bit integer (`u64`).
    U64(u64),
    /// Unsigned 128-bit integer (`u128`).
    U128(u128),
    /// 32-bit floating point number (`f32`).
    F32(f32),
    /// 64-bit floating point number (`f64`).
    F64(f64),
    /// Character (`char`).
    Char(char),
    /// String (`String`).
    String(String),
    /// Byte array (`Vec<u8>`), typically extracted using `#[serde(with = "serde_bytes")]`.
    Bytes(Vec<u8>),
    /// Optional scalar value (`Option<T>`). Contains `None` or `Some(Box<FieldScalarValue>)`.
    Option(Option<Box<FieldScalarValue>>),
}
