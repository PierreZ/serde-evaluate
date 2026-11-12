//! Custom Serde serializers used to extract field values.
//!
//! Extraction is split in two phases:
//! - `traversal`: walks structs and maps along the configured path. Values that are
//!   not on the path are never serialized.
//! - `capture`: once the final segment is reached, the target value is handed to a
//!   capture serializer, either for a single scalar or for a list of scalars.
//!
//! `key` turns map keys into strings so they can be compared with path segments,
//! and `skip` swallows sequences and tuples that are not on the path.

// =============================================================================
// Macros for reducing repetitive serialize method implementations
// =============================================================================

/// Generates `serialize_*` methods that ignore their scalar argument.
macro_rules! impl_ignore_methods {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method(self, _v: $ty) -> Result<Self::Ok, Self::Error> {
                Ok(Default::default())
            }
        )*
    };
}

/// Generates `serialize_*` methods that wrap their argument in a `FieldScalarValue` variant.
macro_rules! impl_scalar_methods {
    ($($method:ident($ty:ty) => $variant:ident),* $(,)?) => {
        $(
            fn $method(self, v: $ty) -> Result<Self::Ok, Self::Error> {
                Ok(FieldScalarValue::$variant(v))
            }
        )*
    };
}

/// Generates `serialize_*` methods that all fail with the same error.
///
/// Works for any method signature, including generic ones and the methods
/// returning compound serializers.
macro_rules! impl_reject_methods {
    ($err:expr; $(fn $method:ident $(<$g:ident>)? ($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(
            fn $method $(<$g: ?Sized + serde::Serialize>)? (self, $($arg: $ty),*) -> Result<$ret, Self::Error> {
                $(let _ = $arg;)*
                Err($err)
            }
        )*
    };
}

mod capture;
mod key;
mod skip;
mod traversal;

pub(crate) use capture::{ListMode, Mode, ScalarMode};
pub(crate) use traversal::{extract, Extraction};
