use serde::{Deserialize, Serialize};

/// Represents the possible scalar values extracted from a field.
/// Includes an Option variant to handle Option<Scalar> types explicitly.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum FieldScalarValue {
    // Basic scalar types
    Unit,
    Bool(bool),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    F32(f32),
    F64(f64),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
    // Represents Option<T> where T is one of the above scalar types.
    // Boxed to keep the size of FieldScalarValue predictable.
    Option(Option<Box<FieldScalarValue>>),
}
