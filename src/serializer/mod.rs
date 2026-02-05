//! Serializer module containing the custom Serde serializers for field extraction.
//!
//! This module is split into focused submodules:
//! - `skip`: Skip struct for efficiently skipping non-target content
//! - `scalar_capture`: Serializer for capturing individual scalar values from list elements
//! - `key`: Serializer for extracting string keys from maps
//! - `list`: List capture logic for extracting Vec<T> fields
//! - `extractor`: Main FieldValueExtractorSerializer

// =============================================================================
// Macros for reducing repetitive serialize method implementations
// =============================================================================

/// Macro for ScalarCaptureSerializer: sets self.value directly.
macro_rules! impl_scalar_capture_methods {
    ($($method:ident($ty:ty) => $variant:ident),* $(,)?) => {
        $(
            fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                self.value = Some(FieldScalarValue::$variant(v));
                Ok(())
            }
        )*
    };
}

/// Macro for FieldValueExtractorSerializer: calls capture_value().
macro_rules! impl_extractor_capture_methods {
    ($($method:ident($ty:ty) => $variant:ident),* $(,)?) => {
        $(
            fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                self.capture_value(FieldScalarValue::$variant(v))
            }
        )*
    };
}

/// Macro for StringKeySerializer: returns "Map key must be a string" error.
macro_rules! impl_key_reject_methods {
    ($($method:ident($($ty:ty),*)),* $(,)?) => {
        $(
            fn $method(self, $(_: $ty),*) -> Result<Self::Ok, Self::Error> {
                Err(EvaluateError::UnsupportedType {
                    type_name: "Map key must be a string",
                })
            }
        )*
    };
}

// Note: Macros defined above are automatically available to submodules

mod extractor;
mod key;
mod list;
mod scalar_capture;
mod skip;

pub(crate) use extractor::FieldValueExtractorSerializer;

use crate::value::FieldScalarValue;

/// Extraction mode for the serializer.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum ExtractionMode {
    /// Extract a single scalar value (default behavior).
    #[default]
    Scalar,
    /// Extract a list of scalar values from a sequence.
    List,
}

/// Helper function to wrap a value in N levels of Option(Some(...))
pub(crate) fn wrap_in_options(value: FieldScalarValue, level: u8) -> FieldScalarValue {
    let mut current = value;
    for _ in 0..level {
        current = FieldScalarValue::Option(Some(Box::new(current)));
    }
    current
}
