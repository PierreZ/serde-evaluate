use crate::error::EvaluateError;
use crate::serializer::FieldValueExtractorSerializer;
use crate::value::FieldScalarValue;
use serde::Serialize;

/// Facilitates the extraction of a scalar value from a specified field within a `Serialize`able struct.
///
/// This struct holds the configuration for the extraction, namely the target field name.
/// The primary way to use this is via the associated function [`FieldExtractor::evaluate`].
#[derive(Debug, Clone)]
pub struct FieldExtractor {
    field_name: String,
}

impl FieldExtractor {
    /// Creates a new `FieldExtractor` configured to target the specified field name.
    ///
    /// Accepts any type that can be converted into a `String`, such as `&str`.
    pub fn new<S: Into<String>>(field_name: S) -> Self {
        FieldExtractor {
            field_name: field_name.into(),
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
    /// * The `field_name` is not found in the `record` ([`EvaluateError::FieldNotFound`]).
    /// * The `field_name`'s value is not a supported scalar type ([`EvaluateError::UnsupportedType`]).
    /// * Any other Serde serialization error occurs.
    pub fn evaluate<T: Serialize>(&self, record: &T) -> Result<FieldScalarValue, EvaluateError> {
        let mut serializer = FieldValueExtractorSerializer::new(&self.field_name);
        // Attempt to serialize the record using our custom serializer.
        record.serialize(&mut serializer)?;

        // After serialization, check if the serializer captured a result.
        serializer
            .into_result()
            .ok_or_else(|| EvaluateError::FieldNotFound {
                field_name: self.field_name.clone(),
            })
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
    pub fn new_from_path<S: AsRef<str>>(path_segments: &[S]) -> Result<Self, EvaluateError> {
        if path_segments.is_empty() {
            return Err(EvaluateError::InvalidPath(
                "Path cannot be empty".to_string(),
            ));
        }

        let segments: Vec<String> = path_segments
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        if segments.iter().any(|s| s.is_empty()) {
            return Err(EvaluateError::InvalidPath(
                "Path segments cannot be empty".to_string(),
            ));
        }

        Ok(NestedFieldExtractor {
            path_segments: segments,
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
    /// * `Err(EvaluateError)` if the path is invalid, an intermediate field is not a struct,
    ///   the final field is not found or has an unsupported type, or a serialization error occurs.
    pub fn evaluate<T: Serialize>(&self, value: &T) -> Result<FieldScalarValue, EvaluateError> {
        // Clone the path segments because new_nested takes ownership, but evaluate only has &self.
        let mut serializer = FieldValueExtractorSerializer::new_nested(self.path_segments.clone());

        // Attempt to serialize the record using our custom serializer.
        value.serialize(&mut serializer)?;

        // After serialization, check if the serializer captured a result.
        serializer
            .into_result()
            .ok_or_else(|| EvaluateError::FieldNotFound {
                field_name: self.path_segments.join("."),
            })
    }
}
