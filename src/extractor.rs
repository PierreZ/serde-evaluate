use crate::error::EvaluateError;
use crate::serializer::FieldValueExtractorSerializer;
use crate::value::FieldScalarValue;
use serde::Serialize;

/// Extracts a single scalar field value from a serializable struct.
#[derive(Debug, Clone)]
pub struct FieldExtractor {
    field_name: String,
}

impl FieldExtractor {
    /// Creates a new extractor for the given field name.
    pub fn new(field_name: String) -> Self {
        FieldExtractor { field_name }
    }

    /// Evaluates the extractor against a serializable record.
    ///
    /// Returns the scalar value of the field if found and supported, otherwise an error.
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
