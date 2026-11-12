use crate::error::EvaluateError;
use crate::serializer::{extract, Extraction, ListMode, Mode, ScalarMode};
use crate::value::FieldScalarValue;
use serde::Serialize;

#[cfg(feature = "tracing")]
use tracing::instrument;

// =============================================================================
// Path Validation Helper
// =============================================================================

/// Validates and converts path segments to a Vec<String>.
///
/// Returns an error if the path is empty or any segment is empty.
fn validate_path<S: AsRef<str>>(segments: &[S]) -> Result<Vec<String>, EvaluateError> {
    if segments.is_empty() {
        return Err(EvaluateError::InvalidPath(
            "Path cannot be empty".to_string(),
        ));
    }

    let segments: Vec<String> = segments.iter().map(|s| s.as_ref().to_string()).collect();

    if segments.iter().any(|s| s.is_empty()) {
        return Err(EvaluateError::InvalidPath(
            "Path segments cannot be empty".to_string(),
        ));
    }

    Ok(segments)
}

/// Rejects an empty top-level field name, for the infallible `new` constructors.
fn check_field_name(field_name: &str) -> Result<(), EvaluateError> {
    if field_name.is_empty() {
        return Err(EvaluateError::InvalidPath(
            "Field name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn field_not_found(field_name: &str) -> EvaluateError {
    EvaluateError::FieldNotFound {
        field_name: field_name.to_string(),
    }
}

fn nested_field_not_found(path: &[String], failed_at_index: usize) -> EvaluateError {
    EvaluateError::NestedFieldNotFound {
        path: path.to_vec(),
        failed_at_index: Some(failed_at_index),
    }
}

/// Shared implementation of every extractor: runs the traversal and maps a
/// missing target to the extractor-specific error.
fn run<M, T, F>(record: &T, path: &[String], not_found: F) -> Result<M::Output, EvaluateError>
where
    M: Mode,
    T: ?Sized + Serialize,
    F: FnOnce(usize) -> EvaluateError,
{
    match extract::<M, T>(record, path)? {
        Extraction::Found(value) => Ok(value),
        Extraction::Missing { failed_at_index } => Err(not_found(failed_at_index)),
    }
}

// =============================================================================
// Scalar Extractors
// =============================================================================

/// Facilitates the extraction of a scalar value from a specified field within a `Serialize`able struct.
///
/// This struct holds the configuration for the extraction, namely the target field name.
/// The primary way to use this is via the associated function [`FieldExtractor::evaluate`].
#[derive(Debug, Clone)]
pub struct FieldExtractor {
    /// Single-segment path holding the field name.
    path: Vec<String>,
}

impl FieldExtractor {
    /// Creates a new `FieldExtractor` configured to target the specified field name.
    ///
    /// Accepts any type that can be converted into a `String`, such as `&str`.
    /// The name is taken literally: `"a.b"` targets a field named `a.b`, not a nested
    /// path. Use [`NestedFieldExtractor`] for nested fields. An empty name makes
    /// [`FieldExtractor::evaluate`] return [`EvaluateError::InvalidPath`].
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all))]
    pub fn new<S: Into<String>>(field_name: S) -> Self {
        FieldExtractor {
            path: vec![field_name.into()],
        }
    }

    /// Extracts the scalar value of the configured `field_name` from the given `record`.
    ///
    /// This method drives the custom serialization process to capture the field's value.
    ///
    /// # Arguments
    ///
    /// * `record`: A reference to a struct that implements `serde::Serialize`.
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError` if:
    /// * The field name is empty ([`EvaluateError::InvalidPath`]).
    /// * The `field_name` is not found in the `record` ([`EvaluateError::FieldNotFound`]).
    /// * The `field_name`'s value is not a supported scalar type ([`EvaluateError::UnsupportedType`]).
    /// * Any other Serde serialization error occurs.
    #[cfg_attr(feature = "tracing", instrument(level = "debug", skip(record), err))]
    pub fn evaluate<T: Serialize>(&self, record: &T) -> Result<FieldScalarValue, EvaluateError> {
        let field_name = &self.path[0];
        check_field_name(field_name)?;
        run::<ScalarMode, _, _>(record, &self.path, |_| field_not_found(field_name))
    }
}

/// Extracts a potentially nested scalar field value using a pre-defined path.
///
/// This struct allows specifying a path as a sequence of field names.
/// It uses the `FieldValueExtractorSerializer` internally to traverse the structure.
#[derive(Debug, Clone)]
pub struct NestedFieldExtractor {
    /// The sequence of field names representing the path to the target value.
    path_segments: Vec<String>,
}

impl NestedFieldExtractor {
    /// Creates a new `NestedFieldExtractor` from a slice of path segments.
    ///
    /// Each element in the input slice represents a step in the path.
    ///
    /// # Arguments
    ///
    /// * `path_segments`: A slice where each element can be converted into a `&str`
    ///   (e.g., `&str`, `String`).
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError::InvalidPath` if the input slice is empty or if any
    /// segment converts to an empty string.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all, err))]
    pub fn new_from_path<S: AsRef<str>>(path_segments: &[S]) -> Result<Self, EvaluateError> {
        Ok(NestedFieldExtractor {
            path_segments: validate_path(path_segments)?,
        })
    }

    /// Evaluates the extractor against the given serializable value using the configured path.
    ///
    /// This triggers the serialization process, traversing the nested structure according
    /// to `path_segments` and intercepting the target field's value.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to a value that implements `serde::Serialize`.
    ///
    /// # Returns
    ///
    /// * `Ok(FieldScalarValue)` if the field at the specified path is found and is a supported scalar type.
    /// * `Err(EvaluateError::NestedFieldNotFound)` if a segment cannot be matched, including when
    ///   an intermediate value is a scalar, a sequence or `None`. `failed_at_index` is the index
    ///   of the first segment that could not be matched.
    /// * `Err(EvaluateError::UnsupportedType)` if the final value is not a supported scalar.
    /// * `Err(EvaluateError::UnsupportedVariant)` if the path goes through or ends on an
    ///   externally tagged enum variant carrying data.
    #[cfg_attr(feature = "tracing", instrument(level = "debug", skip(value), err))]
    pub fn evaluate<T: Serialize>(&self, value: &T) -> Result<FieldScalarValue, EvaluateError> {
        run::<ScalarMode, _, _>(value, &self.path_segments, |index| {
            nested_field_not_found(&self.path_segments, index)
        })
    }
}

// =============================================================================
// Composite Extractor
// =============================================================================

/// Extracts multiple independent scalar fields from a single `Serialize`able record,
/// returning them as an ordered `Vec<FieldScalarValue>`.
///
/// This is useful for building composite index keys where multiple field values
/// are combined into a single ordered key.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use serde_evaluate::{CompositeFieldExtractor, FieldScalarValue, EvaluateError};
///
/// #[derive(Serialize)]
/// struct Record {
///     last_name: String,
///     first_name: String,
///     age: u32,
/// }
///
/// fn main() -> Result<(), EvaluateError> {
///     let record = Record {
///         last_name: "Smith".to_string(),
///         first_name: "John".to_string(),
///         age: 30,
///     };
///
///     let extractor = CompositeFieldExtractor::new(&["last_name", "first_name", "age"])?;
///     let values = extractor.evaluate(&record)?;
///
///     assert_eq!(values, vec![
///         FieldScalarValue::String("Smith".to_string()),
///         FieldScalarValue::String("John".to_string()),
///         FieldScalarValue::U32(30),
///     ]);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CompositeFieldExtractor {
    extractors: Vec<ScalarExtractor>,
}

/// One field of a [`CompositeFieldExtractor`]. Top-level names report
/// [`EvaluateError::FieldNotFound`], like [`FieldExtractor`]; paths report
/// [`EvaluateError::NestedFieldNotFound`], like [`NestedFieldExtractor`].
#[derive(Debug, Clone)]
enum ScalarExtractor {
    Field(FieldExtractor),
    Nested(NestedFieldExtractor),
}

impl ScalarExtractor {
    fn evaluate<T: Serialize>(&self, record: &T) -> Result<FieldScalarValue, EvaluateError> {
        match self {
            ScalarExtractor::Field(e) => e.evaluate(record),
            ScalarExtractor::Nested(e) => e.evaluate(record),
        }
    }
}

impl CompositeFieldExtractor {
    /// Creates a new `CompositeFieldExtractor` for top-level field names.
    ///
    /// Each field name is treated as a single-segment path.
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError::InvalidPath` if the list is empty or any name is empty.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all, err))]
    pub fn new<S: AsRef<str>>(field_names: &[S]) -> Result<Self, EvaluateError> {
        if field_names.is_empty() {
            return Err(EvaluateError::InvalidPath(
                "Composite extractor requires at least one field".to_string(),
            ));
        }

        let extractors = field_names
            .iter()
            .map(|name| {
                check_field_name(name.as_ref())?;
                Ok(ScalarExtractor::Field(FieldExtractor::new(name.as_ref())))
            })
            .collect::<Result<Vec<_>, EvaluateError>>()?;

        Ok(CompositeFieldExtractor { extractors })
    }

    /// Creates a new `CompositeFieldExtractor` from a slice of field paths.
    ///
    /// Each inner slice represents the path segments to a field, supporting
    /// both top-level (single-segment) and nested (multi-segment) paths.
    ///
    /// # Example
    ///
    /// ```rust
    /// use serde::Serialize;
    /// use serde_evaluate::{CompositeFieldExtractor, FieldScalarValue, EvaluateError};
    ///
    /// #[derive(Serialize)]
    /// struct Record {
    ///     name: String,
    ///     address: Address,
    /// }
    ///
    /// #[derive(Serialize)]
    /// struct Address {
    ///     zip: String,
    /// }
    ///
    /// fn main() -> Result<(), EvaluateError> {
    ///     let record = Record {
    ///         name: "Alice".to_string(),
    ///         address: Address { zip: "90210".to_string() },
    ///     };
    ///
    ///     let extractor = CompositeFieldExtractor::new_from_paths(&[
    ///         &["name"],
    ///         &["address", "zip"],
    ///     ])?;
    ///     let values = extractor.evaluate(&record)?;
    ///
    ///     assert_eq!(values, vec![
    ///         FieldScalarValue::String("Alice".to_string()),
    ///         FieldScalarValue::String("90210".to_string()),
    ///     ]);
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError::InvalidPath` if the list is empty, any path is empty,
    /// or any path segment is empty.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all, err))]
    pub fn new_from_paths<S: AsRef<str>>(paths: &[&[S]]) -> Result<Self, EvaluateError> {
        if paths.is_empty() {
            return Err(EvaluateError::InvalidPath(
                "Composite extractor requires at least one field".to_string(),
            ));
        }

        let extractors = paths
            .iter()
            .map(|path| NestedFieldExtractor::new_from_path(path).map(ScalarExtractor::Nested))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CompositeFieldExtractor { extractors })
    }

    /// Extracts scalar values for all configured fields from the given record.
    ///
    /// Returns values in the same order as the fields were specified during construction.
    /// Fails fast on the first extraction error.
    ///
    /// # Arguments
    ///
    /// * `record`: A reference to a value that implements `serde::Serialize`.
    ///
    /// # Errors
    ///
    /// Returns the first `EvaluateError` encountered during extraction.
    #[cfg_attr(feature = "tracing", instrument(level = "debug", skip(record), err))]
    pub fn evaluate<T: Serialize>(
        &self,
        record: &T,
    ) -> Result<Vec<FieldScalarValue>, EvaluateError> {
        self.extractors
            .iter()
            .map(|extractor| extractor.evaluate(record))
            .collect()
    }
}

// =============================================================================
// List Extractors (FanOut-style)
// =============================================================================

/// Extracts a list of scalar values from a `Vec<T>` field where T is a scalar type.
///
/// This enables FanOut-style extraction where each element of a list is returned
/// separately, useful for indexing scenarios where each element needs to be
/// processed individually.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use serde_evaluate::{ListFieldExtractor, FieldScalarValue, EvaluateError};
///
/// #[derive(Serialize)]
/// struct Record {
///     id: u32,
///     tags: Vec<String>,
/// }
///
/// fn main() -> Result<(), EvaluateError> {
///     let record = Record {
///         id: 1,
///         tags: vec!["rust".to_string(), "serde".to_string()],
///     };
///
///     let extractor = ListFieldExtractor::new("tags");
///     let values = extractor.evaluate(&record)?;
///
///     assert_eq!(values, vec![
///         FieldScalarValue::String("rust".to_string()),
///         FieldScalarValue::String("serde".to_string()),
///     ]);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ListFieldExtractor {
    /// Single-segment path holding the field name.
    path: Vec<String>,
}

impl ListFieldExtractor {
    /// Creates a new `ListFieldExtractor` configured to target the specified field name.
    ///
    /// Accepts any type that can be converted into a `String`, such as `&str`.
    /// An empty name makes [`ListFieldExtractor::evaluate`] return [`EvaluateError::InvalidPath`].
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all))]
    pub fn new<S: Into<String>>(field_name: S) -> Self {
        ListFieldExtractor {
            path: vec![field_name.into()],
        }
    }

    /// Extracts all scalar elements from a sequence field (`Vec<T>`, `[T; N]`, sets...).
    ///
    /// # Arguments
    ///
    /// * `record`: A reference to a struct that implements `serde::Serialize`.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FieldScalarValue>)` containing each element as a scalar value.
    /// * `Ok(vec![])` for empty lists, `Option<Vec<T>>` with `None`, unit (JSON `null`),
    ///   and fields skipped with `#[serde(skip_serializing_if = ...)]`.
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError` if:
    /// * The field name is empty ([`EvaluateError::InvalidPath`]).
    /// * The `field_name` is not found in the `record` ([`EvaluateError::FieldNotFound`]).
    /// * The target is not a sequence, or its elements are not scalar types
    ///   ([`EvaluateError::UnsupportedType`]).
    #[cfg_attr(feature = "tracing", instrument(level = "debug", skip(record), err))]
    pub fn evaluate<T: Serialize>(
        &self,
        record: &T,
    ) -> Result<Vec<FieldScalarValue>, EvaluateError> {
        let field_name = &self.path[0];
        check_field_name(field_name)?;
        run::<ListMode, _, _>(record, &self.path, |_| field_not_found(field_name))
    }
}

/// Extracts a list of scalar values from a nested `Vec<T>` field using a path.
///
/// This enables FanOut-style extraction for lists within nested structures.
///
/// # Example
///
/// ```rust
/// use serde::Serialize;
/// use serde_evaluate::{NestedListFieldExtractor, FieldScalarValue, EvaluateError};
///
/// #[derive(Serialize)]
/// struct Record {
///     metadata: Metadata,
/// }
///
/// #[derive(Serialize)]
/// struct Metadata {
///     labels: Vec<String>,
/// }
///
/// fn main() -> Result<(), EvaluateError> {
///     let record = Record {
///         metadata: Metadata {
///             labels: vec!["label1".to_string(), "label2".to_string()],
///         },
///     };
///
///     let extractor = NestedListFieldExtractor::new_from_path(&["metadata", "labels"])?;
///     let values = extractor.evaluate(&record)?;
///
///     assert_eq!(values, vec![
///         FieldScalarValue::String("label1".to_string()),
///         FieldScalarValue::String("label2".to_string()),
///     ]);
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct NestedListFieldExtractor {
    path_segments: Vec<String>,
}

impl NestedListFieldExtractor {
    /// Creates a new `NestedListFieldExtractor` from a slice of path segments.
    ///
    /// Each element in the input slice represents a step in the path.
    ///
    /// # Arguments
    ///
    /// * `path_segments`: A slice where each element can be converted into a `&str`
    ///   (e.g., `&str`, `String`).
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError::InvalidPath` if the input slice is empty or if any
    /// segment converts to an empty string.
    #[cfg_attr(feature = "tracing", instrument(level = "trace", skip_all, err))]
    pub fn new_from_path<S: AsRef<str>>(path_segments: &[S]) -> Result<Self, EvaluateError> {
        Ok(NestedListFieldExtractor {
            path_segments: validate_path(path_segments)?,
        })
    }

    /// Extracts all scalar elements from a nested `Vec<T>` field.
    ///
    /// # Arguments
    ///
    /// * `value` - A reference to a value that implements `serde::Serialize`.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FieldScalarValue>)` containing each element as a scalar value.
    /// * `Ok(vec![])` for empty lists, `Option<Vec<T>>` with `None`, unit (JSON `null`),
    ///   and fields skipped with `#[serde(skip_serializing_if = ...)]`.
    ///
    /// # Errors
    ///
    /// Returns `EvaluateError` if:
    /// * The path is not found ([`EvaluateError::NestedFieldNotFound`]).
    /// * The target is not a sequence, or its elements are not scalar types
    ///   ([`EvaluateError::UnsupportedType`]).
    #[cfg_attr(feature = "tracing", instrument(level = "debug", skip(value), err))]
    pub fn evaluate<T: Serialize>(
        &self,
        value: &T,
    ) -> Result<Vec<FieldScalarValue>, EvaluateError> {
        run::<ListMode, _, _>(value, &self.path_segments, |index| {
            nested_field_not_found(&self.path_segments, index)
        })
    }
}
