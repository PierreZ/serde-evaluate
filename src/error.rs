use serde::ser::Error as SerdeError;
use thiserror::Error;

/// Errors that can occur during evaluation.
#[derive(Error, Debug, PartialEq)]
pub enum EvaluateError {
    /// The target field specified for extraction was not found in the serialized data.
    #[error("Field '{field_name}' not found in the struct")]
    FieldNotFound {
        /// The name of the field that was not found.
        field_name: String,
    },

    /// An intermediate part of the path pointed to a non-struct type.
    #[error("Cannot traverse non-struct type at path segment: {0}")]
    NotAStruct(String),

    /// The value of the target field has a type that is not supported for scalar extraction.
    /// (e.g., nested structs, sequences other than `Vec<u8>`, maps).
    #[error("Unsupported type for scalar extraction: {type_name}")]
    UnsupportedType {
        /// A string representation of the unsupported type name.
        type_name: &'static str,
    },

    /// The value of the target field has a variant type that is not supported for scalar extraction.
    #[error("Unsupported variant type for scalar extraction: {variant_type}")]
    UnsupportedVariant {
        /// A string representation of the unsupported variant type.
        variant_type: &'static str,
    },

    /// The provided path string or segments were invalid (e.g., empty or contained empty segments).
    #[error("Invalid field path provided: {0}")]
    InvalidPath(String),

    /// An underlying Serde serialization error occurred.
    #[error("Serialization error: {message}")]
    SerializationError {
        /// A custom error message from serde::ser::Error::custom.
        message: String,
    },
}

impl SerdeError for EvaluateError {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        EvaluateError::SerializationError {
            message: msg.to_string(),
        }
    }
}
