use super::ArrowFieldExt;
use crate::{error::Error, Record};
use datafusion::arrow::{
    array::Array,
    datatypes::{DataType, Field, Schema, UnionMode},
};
use std::sync::Arc;
use toml::{Table, Value};

/// Extension trait for [`Schema`](datafusion::arrow::datatypes::Schema).
pub(super) trait ArrowSchemaExt {
    /// Attempts to create a `Schema` from an Avro record.
    fn try_from_avro_record(record: &Record) -> Result<Schema, Error>;

    /// Attempts to create a `Schema` from the TOML table configuration.
    fn try_from_toml_table(table: &Table) -> Result<Schema, Error>;

    /// Collects columns in the Avro records.
    fn collect_columns_from_avro_records(
        &self,
        records: &[Record],
    ) -> Vec<Arc<dyn Array + 'static>>;
}

impl ArrowSchemaExt for Schema {
    fn try_from_avro_record(record: &Record) -> Result<Schema, Error> {
        let mut fields = Vec::with_capacity(record.len());
        for (field, value) in record {
            let field = Field::try_from_avro_record_entry(field, value)?;
            fields.push(field);
        }
        Ok(Schema::new(fields))
    }

    fn try_from_toml_table(table: &Table) -> Result<Schema, Error> {
        let mut fields = Vec::with_capacity(table.len());
        for (key, value) in table {
            let name = key.to_owned();
            let data_type = match value {
                Value::String(value_type) => parse_arrow_data_type(value_type)?,
                Value::Array(array) => {
                    let array_length = array.len();
                    if array_length == 1 && let Some(Value::String(value_type)) = array.first() {
                        let item_data_type = parse_arrow_data_type(&value_type)?;
                        let field = Field::new("item", item_data_type, true);
                        DataType::List(Box::new(field))
                    } else if array_length >= 2 {
                        let mut fields = Vec::with_capacity(array_length);
                        let mut positions = Vec::with_capacity(array_length);
                        for (index, value) in array.iter().enumerate() {
                            if let Value::String(value_type) = value {
                                let data_type = parse_arrow_data_type(value_type)?;
                                let field = Field::new(index.to_string(), data_type, true);
                                fields.push(field);
                                positions.push(i8::try_from(index)?);
                            }
                        }
                        DataType::Union(fields, positions, UnionMode::Dense)
                    } else {
                        return Err(Error::new(format!("schema for `{key}` should be nonempty")));
                    }
                }
                Value::Table(table) => {
                    let schema = Self::try_from_toml_table(table)?;
                    DataType::Struct(schema.fields)
                }
                _ => return Err(Error::new(format!("schema for `{key}` is invalid"))),
            };
            fields.push(Field::new(name, data_type, true));
        }
        Ok(Schema::new(fields))
    }

    fn collect_columns_from_avro_records(
        &self,
        records: &[Record],
    ) -> Vec<Arc<dyn Array + 'static>> {
        let fields = self.fields();
        let mut columns = Vec::with_capacity(fields.len());
        for field in fields {
            let column = field.collect_values_from_avro_records(records);
            columns.push(column);
        }
        columns
    }
}

/// Parses the arrow data type.
fn parse_arrow_data_type(value_type: &str) -> Result<DataType, Error> {
    let data_type = match value_type {
        "null" => DataType::Null,
        "boolean" => DataType::Boolean,
        "int" => DataType::Int32,
        "long" => DataType::Int64,
        "float" => DataType::Float32,
        "double" => DataType::Float64,
        "bytes" => DataType::Binary,
        "string" => DataType::Utf8,
        _ => {
            let message = format!("parsing `{value_type}` as Arrow data type is unsupported");
            return Err(Error::new(message));
        }
    };
    Ok(data_type)
}
