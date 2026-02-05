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

/// Generates scalar serialize methods for `ScalarCaptureSerializer`.
///
/// Each method sets `self.value` to the corresponding `FieldScalarValue` variant.
///
/// # Usage
/// ```ignore
/// impl_scalar_capture_methods! {
///     serialize_bool(bool) => Bool,
///     serialize_i32(i32) => I32,
/// }
/// ```
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

/// Generates scalar serialize methods for `FieldValueExtractorSerializer`.
///
/// Each method calls `capture_value()` with the corresponding `FieldScalarValue` variant.
///
/// # Usage
/// ```ignore
/// impl_extractor_capture_methods! {
///     serialize_bool(bool) => Bool,
///     serialize_i32(i32) => I32,
/// }
/// ```
macro_rules! impl_extractor_capture_methods {
    ($($method:ident($ty:ty) => $variant:ident),* $(,)?) => {
        $(
            fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                self.capture_value(FieldScalarValue::$variant(v))
            }
        )*
    };
}

/// Generates rejection methods for `StringKeySerializer`.
///
/// Each method returns an `UnsupportedType` error since map keys must be strings.
///
/// # Usage
/// ```ignore
/// impl_key_reject_methods! {
///     serialize_bool(bool),
///     serialize_i32(i32),
///     serialize_unit(),
/// }
/// ```
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
