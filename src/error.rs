use serde::ser::Error as SerdeError;
use thiserror::Error;

/// Errors that can occur during evaluation.
#[derive(Error, Debug, PartialEq)]
pub enum EvaluateError {
    #[error("Field '{field_name}' not found in the struct")]
    FieldNotFound { field_name: String },

    #[error("Unsupported type for scalar extraction: {type_name}")]
    UnsupportedType { type_name: &'static str },

    #[error("Unsupported variant type for scalar extraction: {variant_type}")]
    UnsupportedVariant { variant_type: &'static str },

    // Catch-all for custom messages from serde::ser::Error::custom
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
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
