use serde::ser::Error as SerdeError;
use thiserror::Error;

/// Helper to format the NestedFieldNotFound error message.
fn format_nested_field_error(path: &[String], failed_at_index: Option<usize>) -> String {
    let path_str = path.join(".");
    match failed_at_index {
        Some(idx) if idx < path.len() => {
            format!(
                "Nested field path '{}' not found (failed at segment {}: '{}')",
                path_str, idx, path[idx]
            )
        }
        _ => format!("Nested field path '{}' not found", path_str),
    }
}

/// Errors that can occur during evaluation.
#[derive(Error, Debug, PartialEq)]
pub enum EvaluateError {
    /// The target field specified for extraction was not found in the serialized data.
    #[error("Field '{field_name}' not found in the struct")]
    FieldNotFound {
        /// The name of the field that was not found.
        field_name: String,
    },

    /// The target nested field specified by the path was not found.
    #[error("{}", format_nested_field_error(path, *failed_at_index))]
    NestedFieldNotFound {
        /// The full path segments that were not found.
        path: Vec<String>,
        /// The index of the path segment where traversal failed (if known).
        failed_at_index: Option<usize>,
    },

    /// An intermediate part of the path pointed to a non-struct type.
    #[error("Cannot traverse non-struct type at path segment {index}: '{segment}'")]
    NotAStruct {
        /// The path segment that was not a struct.
        segment: String,
        /// The index in the path where this occurred.
        index: usize,
    },

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
