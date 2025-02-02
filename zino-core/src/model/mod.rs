//! Application specific models.
use crate::{request::Validation, Map, Record};
use apache_avro::types::Value as AvroValue;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonValue;

mod column;
mod mutation;
mod query;
mod row;

pub use column::{Column, EncodeColumn};
pub use mutation::Mutation;
pub use query::Query;
pub use row::DecodeRow;

/// General data model.
pub trait Model: Default + Serialize + DeserializeOwned {
    /// Creates a new instance.
    fn new() -> Self;

    /// Updates the model using the json object and returns the validation result.
    #[must_use]
    fn read_map(&mut self, data: &Map) -> Validation;

    /// Attempts to construct a model from a json object.
    #[inline]
    fn try_from_map(data: Map) -> Result<Self, serde_json::Error> {
        serde_json::from_value(JsonValue::from(data))
    }

    /// Attempts to construct a model from an Avro record.
    #[inline]
    fn try_from_avro_record(data: Record) -> Result<Self, apache_avro::Error> {
        apache_avro::from_value(&AvroValue::Record(data))
    }

    /// Consumes the model and returns as a json object.
    ///
    /// # Panics
    ///
    /// It will panic if the model cann't be converted to a json object.
    #[must_use]
    fn into_map(self) -> Map {
        match serde_json::to_value(self) {
            Ok(JsonValue::Object(map)) => map,
            _ => panic!("the model cann't be converted to a json object"),
        }
    }

    /// Consumes the model and returns as an Avro record.
    ///
    /// # Panics
    ///
    /// It will panic if the model cann't be converted to an Avro record.
    #[must_use]
    fn into_avro_record(self) -> Record {
        match apache_avro::to_value(self) {
            Ok(AvroValue::Record(record)) => record,
            _ => panic!("the model cann't be converted to an Avro record"),
        }
    }
}
