use crate::{LengthMode, document::DataType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    value: String,
    data_type: Option<DataType>,
    length_mode: Option<LengthMode>,
}

impl Field {
    pub fn new<V: Into<String>>(value: V, data_type: Option<DataType>) -> Field {
        Field {
            value: value.into(),
            data_type,
            length_mode: None,
        }
    }

    pub(crate) fn with_length_mode(mut self, length_mode: Option<LengthMode>) -> Field {
        self.length_mode = length_mode;
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn data_type(&self) -> Option<DataType> {
        self.data_type
    }

    /// Returns the length mode an ADI value was read in, if the value is not ASCII.
    pub fn length_mode(&self) -> Option<LengthMode> {
        self.length_mode
    }
}
