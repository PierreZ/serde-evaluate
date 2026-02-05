//! List capture functionality for extracting Vec<T> fields.
//!
//! Contains SeqSerializer enum and ListCapture struct for FanOut-style list extraction.

use crate::error::EvaluateError;
use serde::ser;
use serde::Serialize;

use super::extractor::FieldValueExtractorSerializer;
use super::scalar_capture::ScalarCaptureSerializer;
use super::skip::Skip;

/// Enum to represent either Skip or ListCapture for SerializeSeq.
pub(crate) enum SeqSerializer<'a> {
    Skip(Skip),
    ListCapture(ListCapture<'a>),
}

impl ser::SerializeSeq for SeqSerializer<'_> {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SeqSerializer::Skip(s) => s.serialize_element(value),
            SeqSerializer::ListCapture(c) => c.serialize_element(value),
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        match self {
            SeqSerializer::Skip(s) => s.end(),
            SeqSerializer::ListCapture(c) => c.end(),
        }
    }
}

/// Captures each element of a sequence as a scalar value.
pub(crate) struct ListCapture<'a> {
    pub(crate) serializer: &'a mut FieldValueExtractorSerializer,
}

impl ser::SerializeSeq for ListCapture<'_> {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // Create a sub-serializer to capture this single element as a scalar
        let mut element_serializer = ScalarCaptureSerializer::new();
        value.serialize(&mut element_serializer)?;

        if let Some(scalar) = element_serializer.into_result() {
            self.serializer.push_list_value(scalar);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Mark that we found and processed the list by setting value to a sentinel
        // The actual values are in list_values
        self.serializer.set_list_found();
        Ok(())
    }
}
