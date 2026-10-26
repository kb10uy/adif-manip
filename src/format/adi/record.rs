use std::collections::HashMap;

use crate::{
    LengthMode,
    document::{DataType, FieldName},
    format::adi::{data::read_field_value, error::AdiError, tag::Tag},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record<'a> {
    pub fields: HashMap<FieldName<'a>, (&'a str, Option<DataType>)>,
}

impl<'a> Record<'a> {
    pub fn parse(text: &'a str, length_mode: LengthMode) -> Result<(Record<'a>, usize), AdiError> {
        let mut fields = HashMap::new();
        let mut consumed = 0;
        loop {
            match Tag::parse(&text[consumed..]) {
                Ok((
                    Tag::Specifier {
                        name,
                        value_length,
                        data_type,
                    },
                    c,
                )) => {
                    consumed += c;
                    let value = read_field_value(text, consumed, length_mode, value_length)?;
                    fields.insert(FieldName::new(name), (value, data_type));
                    consumed += value.len();
                }
                Ok((Tag::EndOfRecord, c)) => {
                    consumed += c;
                    break;
                }

                Ok(_) => return Err(AdiError::NoEor(consumed)),
                Err(e) => return Err(AdiError::Tag(consumed, e)),
            }
        }

        Ok((Record { fields }, consumed))
    }
}

#[cfg(test)]
mod tests {
    use crate::{LengthMode, document::DataType};

    use super::Record;

    #[test]
    fn parses_record() {
        let expected = Record {
            fields: vec![
                ("CALL".into(), ("JL1HIS", None)),
                ("FREQ".into(), ("7.041", Some(DataType::Number))),
            ]
            .into_iter()
            .collect(),
        };
        assert_eq!(
            Record::parse("<CALL:6>JL1HIS<FREQ:5:n>7.041<eor>", LengthMode::Bytes),
            Ok((expected, 34))
        );
    }
}
