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
