//! Main FieldValueExtractorSerializer for extracting field values.
//!
//! This is the core serializer that intercepts Serde serialization to extract
//! targeted field values without full deserialization.

use crate::error::EvaluateError;
use crate::value::FieldScalarValue;
use serde::ser;
use serde::{Serialize, Serializer};

use super::key::StringKeySerializer;
use super::list::{ListCapture, SeqSerializer};
use super::skip::Skip;
use super::{wrap_in_options, ExtractionMode};

// =============================================================================
// State Separation: Config, State, and Result
// =============================================================================

/// Immutable extraction configuration.
struct ExtractorConfig {
    /// Target path segments to traverse.
    path: Vec<String>,
    /// Whether extracting a scalar or list.
    extraction_mode: ExtractionMode,
}

/// Mutable traversal state during serialization.
#[derive(Default)]
struct TraversalState {
    /// Current depth in the path traversal.
    current_path_index: usize,
    /// Tracks if the current map key matched the path segment.
    current_map_key_match: Option<bool>,
    /// True if the next serialize_* call is for the final target value.
    ready_to_capture: bool,
    /// Tracks nesting level when capturing Option<Option<...>>.
    option_nesting_level: u8,
}

/// Extraction results.
#[derive(Default)]
struct ExtractionResult {
    /// The final extracted scalar value.
    value: Option<FieldScalarValue>,
    /// Collected list elements (when in List mode).
    list_values: Vec<FieldScalarValue>,
}

// =============================================================================
// Main Serializer
// =============================================================================

/// Custom Serializer Implementation for extracting field values.
pub(crate) struct FieldValueExtractorSerializer {
    /// Immutable configuration.
    config: ExtractorConfig,
    /// Mutable traversal state.
    state: TraversalState,
    /// Extraction results.
    result: ExtractionResult,
}

impl FieldValueExtractorSerializer {
    /// Core constructor with explicit path and extraction mode.
    fn with_mode(path: Vec<String>, mode: ExtractionMode) -> Self {
        FieldValueExtractorSerializer {
            config: ExtractorConfig {
                path,
                extraction_mode: mode,
            },
            state: TraversalState::default(),
            result: ExtractionResult::default(),
        }
    }

    /// Creates a new serializer for extracting a single top-level scalar field.
    pub(crate) fn new(field_name: &str) -> Self {
        Self::with_mode(vec![field_name.to_string()], ExtractionMode::Scalar)
    }

    /// Creates a new serializer for extracting a nested scalar field by path.
    pub(crate) fn new_nested(path_segments: Vec<String>) -> Self {
        Self::with_mode(path_segments, ExtractionMode::Scalar)
    }

    /// Creates a serializer configured to extract a list from a top-level field.
    pub(crate) fn new_list(field_name: &str) -> Self {
        Self::with_mode(vec![field_name.to_string()], ExtractionMode::List)
    }

    /// Creates a serializer configured to extract a list from a nested path.
    pub(crate) fn new_nested_list(path_segments: Vec<String>) -> Self {
        Self::with_mode(path_segments, ExtractionMode::List)
    }

    /// Called by individual scalar serialize_* methods.
    /// Captures the value if ready_to_capture flag is set,
    /// potentially wrapping based on option_nesting_level.
    fn capture_value(&mut self, value: FieldScalarValue) -> Result<(), EvaluateError> {
        if self.state.ready_to_capture {
            // Wrap the value according to the current nesting level.
            self.result.value = Some(wrap_in_options(value, self.state.option_nesting_level));
        }
        Ok(())
    }

    pub(crate) fn into_result(self) -> Option<FieldScalarValue> {
        self.result.value
    }

    /// Returns the collected list values after list extraction.
    /// Returns Some(vec) if list extraction was successful (even if empty).
    /// Returns None if the target field was not found.
    pub(crate) fn into_list_result(self) -> Option<Vec<FieldScalarValue>> {
        // If value is set, we found the field
        if self.result.value.is_some() {
            Some(self.result.list_values)
        } else {
            None
        }
    }

    /// Helper method for ListCapture to push a captured value.
    pub(crate) fn push_list_value(&mut self, value: FieldScalarValue) {
        self.result.list_values.push(value);
    }

    /// Helper method for ListCapture to mark the list as found.
    pub(crate) fn set_list_found(&mut self) {
        self.result.value = Some(FieldScalarValue::Unit); // Sentinel to indicate success
        self.state.ready_to_capture = false;
    }

    // Helper accessors for cleaner code in trait implementations
    fn path(&self) -> &[String] {
        &self.config.path
    }

    fn extraction_mode(&self) -> ExtractionMode {
        self.config.extraction_mode
    }
}

// =============================================================================
// Serializer Trait Implementation
// =============================================================================

impl<'a> Serializer for &'a mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    type SerializeSeq = SeqSerializer<'a>;
    type SerializeTuple = Skip;
    type SerializeTupleStruct = Skip;
    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    // Use macro for simple scalar captures
    impl_extractor_capture_methods! {
        serialize_bool(bool) => Bool,
        serialize_i8(i8) => I8,
        serialize_i16(i16) => I16,
        serialize_i32(i32) => I32,
        serialize_i64(i64) => I64,
        serialize_i128(i128) => I128,
        serialize_u8(u8) => U8,
        serialize_u16(u16) => U16,
        serialize_u32(u32) => U32,
        serialize_u64(u64) => U64,
        serialize_u128(u128) => U128,
        serialize_f32(f32) => F32,
        serialize_f64(f64) => F64,
        serialize_char(char) => Char,
    }

    // These need special handling for conversion
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::String(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Bytes(v.to_vec()))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        if self.state.ready_to_capture {
            match self.extraction_mode() {
                ExtractionMode::Scalar => {
                    // Construct the nested None value based on the *current* nesting level.
                    let none_value = wrap_in_options(
                        FieldScalarValue::Option(None),
                        self.state.option_nesting_level,
                    );
                    // Directly set the value, bypassing capture_value which would wrap again.
                    self.result.value = Some(none_value);
                }
                ExtractionMode::List => {
                    // Option<Vec<T>> = None results in empty list
                    // Set sentinel value to indicate we found the field (list_values remains empty)
                    self.result.value = Some(FieldScalarValue::Unit);
                    self.state.ready_to_capture = false;
                }
            }
            Ok(())
        } else {
            // Not capturing, None is just part of structure traversal.
            Ok(())
        }
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        if self.state.ready_to_capture {
            // Increment nesting level *before* serializing inner value.
            let original_level = self.state.option_nesting_level;
            self.state.option_nesting_level =
                original_level
                    .checked_add(1)
                    .ok_or(EvaluateError::UnsupportedType {
                        type_name: "Deeply Nested Option (>255 levels)",
                    })?;
            let result = value.serialize(&mut *self);
            // Restore nesting level *after* serializing inner value.
            self.state.option_nesting_level = original_level;
            result
        } else {
            // Not capturing, just pass through serialization
            value.serialize(&mut *self)
        }
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Unit)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.capture_value(FieldScalarValue::Unit)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.state.ready_to_capture {
            self.state.ready_to_capture = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "newtype",
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if self.state.ready_to_capture {
            match self.extraction_mode() {
                ExtractionMode::Scalar => {
                    self.state.ready_to_capture = false;
                    Err(EvaluateError::UnsupportedType {
                        type_name: "sequence",
                    })
                }
                ExtractionMode::List => {
                    // Return ListCapture to collect elements
                    Ok(SeqSerializer::ListCapture(ListCapture { serializer: self }))
                }
            }
        } else {
            Ok(SeqSerializer::Skip(Skip))
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        if self.state.ready_to_capture {
            self.state.ready_to_capture = false;
            Err(EvaluateError::UnsupportedType { type_name: "tuple" })
        } else {
            Ok(Skip)
        }
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        if self.state.ready_to_capture {
            self.state.ready_to_capture = false;
            Err(EvaluateError::UnsupportedType {
                type_name: "tuple struct",
            })
        } else {
            Ok(Skip)
        }
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        if self.state.ready_to_capture {
            self.state.ready_to_capture = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "tuple",
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if self.path().is_empty() {
            return Err(EvaluateError::FieldNotFound {
                field_name: "<internal error: empty path>".to_string(),
            });
        }
        if self.state.current_path_index >= self.path().len() {
            if self.state.ready_to_capture {
                self.state.ready_to_capture = false;
                return Err(EvaluateError::UnsupportedType { type_name: "map" });
            } else {
                return Ok(self);
            }
        }

        self.state.ready_to_capture = true;
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        if self.state.current_path_index >= self.path().len() {
            if self.state.ready_to_capture {
                self.state.ready_to_capture = false;
                Err(EvaluateError::UnsupportedType {
                    type_name: "struct",
                })
            } else {
                Ok(self)
            }
        } else {
            Ok(self)
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        if self.state.ready_to_capture {
            self.state.ready_to_capture = false;
        }
        Err(EvaluateError::UnsupportedVariant {
            variant_type: "struct",
        })
    }
}

// =============================================================================
// SerializeMap Implementation
// =============================================================================

impl ser::SerializeMap for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.state.current_map_key_match = None;

        if self.result.value.is_none() && self.state.current_path_index < self.config.path.len() {
            let mut key_serializer = StringKeySerializer { key: None };
            key.serialize(&mut key_serializer)?;

            if let Some(key_str) = key_serializer.key {
                if key_str == self.config.path[self.state.current_path_index] {
                    self.state.current_map_key_match = Some(true);
                } else {
                    self.state.current_map_key_match = Some(false);
                }
            } else {
                self.state.current_map_key_match = Some(false);
            }
        } else {
            self.state.current_map_key_match = Some(false);
        }
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        match self.state.current_map_key_match.take() {
            Some(true) => {
                self.state.current_path_index += 1;
                let is_last_segment = self.state.current_path_index == self.config.path.len();

                let result = if is_last_segment {
                    self.state.ready_to_capture = true;
                    let res = value.serialize(&mut **self);
                    self.state.ready_to_capture = false;
                    res
                } else {
                    value.serialize(&mut **self)
                };

                self.state.current_path_index -= 1;
                result
            }
            Some(false) | None => {
                let mut dummy_serializer = FieldValueExtractorSerializer::new_nested(vec![]);
                let _ = value.serialize(&mut dummy_serializer);
                Ok(())
            }
        }
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

// =============================================================================
// SerializeStruct Implementation
// =============================================================================

impl ser::SerializeStruct for &mut FieldValueExtractorSerializer {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        if self.result.value.is_some() {
            return Ok(());
        }

        if self.state.current_path_index < self.config.path.len()
            && key == self.config.path[self.state.current_path_index]
        {
            self.state.current_path_index += 1;
            let is_last_segment = self.state.current_path_index == self.config.path.len();

            let result = if is_last_segment {
                self.state.ready_to_capture = true;
                let res = value.serialize(&mut **self);
                self.state.ready_to_capture = false;
                res
            } else {
                value.serialize(&mut **self)
            };

            self.state.current_path_index -= 1;
            return result;
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.result.value.is_none()
            && self.state.current_path_index > 0
            && self.state.current_path_index < self.config.path.len()
        {
            Err(EvaluateError::NestedFieldNotFound {
                path: self.config.path.clone(),
                failed_at_index: Some(self.state.current_path_index),
            })
        } else {
            Ok(())
        }
    }
}
