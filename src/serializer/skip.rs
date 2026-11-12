//! Skip serializer for compound values that are not on the extraction path.

use crate::error::EvaluateError;
use serde::ser;
use serde::Serialize;

/// Swallows the elements of sequences and tuples that are not on the path.
pub(crate) struct Skip;

impl ser::SerializeSeq for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTuple for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Skip {
    type Ok = ();
    type Error = EvaluateError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _value: &T) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
