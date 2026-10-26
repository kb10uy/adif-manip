use std::collections::HashMap;

use crate::{
    LengthMode,
    document::{DataType, FieldName, UserDefinedField, ValueConstraint},
    format::adi::{data::read_field_value, error::AdiError, tag::Tag},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header<'a> {
    pub preamble: &'a str,
    pub fields: HashMap<FieldName<'a>, (&'a str, Option<DataType>)>,
    pub user_defined_fields: Vec<UserDefinedField>,
}

impl<'a> Header<'a> {
    pub fn parse(
        text: &'a str,
        length_mode: LengthMode,
    ) -> Result<(Option<Header<'a>>, usize), AdiError> {
        // > If the first character in an ADI file is <, it contains no Header.
        // https://adif.org.uk/316/ADIF_316.htm#ADI_File_Format
        if text.starts_with('<') {
            return Ok((None, 0));
        }
        let Some(header_start) = Tag::find(text) else {
            return Err(AdiError::NoData);
        };
        let preamble = &text[..header_start];

        let mut fields = HashMap::new();
        let mut consumed = header_start;
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
                Ok((Tag::EndOfHeader, c)) => {
                    consumed += c;
                    break;
                }

                Ok(_) => return Err(AdiError::NoEoh),
                Err(e) => return Err(AdiError::Tag(consumed, e)),
            }
        }

        let mut user_defined_fields = fields
            .iter()
            .filter_map(|(name, &(value, data_type))| {
                parse_user_defined_field(name, value, data_type).transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        user_defined_fields.sort_by_key(UserDefinedField::id);

        Ok((
            Some(Header {
                preamble,
                fields,
                user_defined_fields,
            }),
            consumed,
        ))
    }
}

fn parse_user_defined_field(
    name: &FieldName,
    value: &str,
    data_type: Option<DataType>,
) -> Result<Option<UserDefinedField>, AdiError> {
    let Some(id) = name
        .as_str()
        .strip_prefix("USERDEF")
        .and_then(|n| n.parse().ok())
    else {
        return Ok(None);
    };

    let (field_name, constraint) = match value.split_once(',') {
        Some((field_name, braced)) => {
            let constraint = ValueConstraint::parse(braced)
                .ok_or_else(|| AdiError::InvalidUserDefinedField(name.as_str().to_string()))?;
            (field_name, Some(constraint))
        }
        None => (value, None),
    };
    Ok(Some(UserDefinedField::new(
        id,
        field_name.to_ascii_uppercase(),
        data_type,
        constraint,
    )))
}

#[cfg(test)]
mod tests {
    use crate::{
        LengthMode,
        document::{DataType, UserDefinedField, ValueConstraint},
        format::adi::error::AdiError,
    };

    use super::Header;

    #[test]
    fn parses_header() {
        let adi_text = include_str!("../../../fixtures/basic-header.adi");

        let expected = Header {
            preamble: "Fixture ADI File\n",
            fields: vec![
                ("ADIF_VER".into(), ("3.1.6", None)),
                ("CREATED_TIMESTAMP".into(), ("20260120 000000", None)),
                ("PROGRAMID".into(), ("jelgen", None)),
                ("PROGRAMVERSION".into(), ("0.1.0", None)),
            ]
            .into_iter()
            .collect(),
            user_defined_fields: vec![],
        };
        assert_eq!(
            Header::parse(adi_text, LengthMode::Bytes),
            Ok((Some(expected), 122))
        );
    }

    #[test]
    fn keeps_last_of_case_insensitive_duplicates() {
        let (header, _) = Header::parse("x<adif_ver:1>1<ADIF_VER:1>2<EOH>", LengthMode::Bytes)
            .expect("must parse");
        let header = header.expect("must have header");
        assert_eq!(header.fields.len(), 1);
        assert_eq!(header.fields.get(&"ADIF_VER".into()), Some(&("2", None)));
    }

    #[test]
    fn parses_user_defined_fields() {
        let text =
            "x<USERDEF3:15>ShoeSize,{5:20}<USERDEF1:3:N>EPC<userdef2:19:E>SweaterSize,{S,M,L}<EOH>";
        let (header, _) = Header::parse(text, LengthMode::Bytes).expect("must parse");
        assert_eq!(
            header.expect("must have header").user_defined_fields,
            vec![
                UserDefinedField::new(1, "EPC", Some(DataType::Number), None),
                UserDefinedField::new(
                    2,
                    "SWEATERSIZE",
                    Some(DataType::Enumeration),
                    Some(ValueConstraint::Enumeration(vec![
                        "S".to_string(),
                        "M".to_string(),
                        "L".to_string(),
                    ])),
                ),
                UserDefinedField::new(
                    3,
                    "SHOESIZE",
                    None,
                    Some(ValueConstraint::Range {
                        lower: "5".to_string(),
                        upper: "20".to_string(),
                    }),
                ),
            ]
        );
    }

    #[test]
    fn rejects_malformed_user_defined_field() {
        assert_eq!(
            Header::parse("x<USERDEF1:9>Size,S|M|L<EOH>", LengthMode::Bytes),
            Err(AdiError::InvalidUserDefinedField("USERDEF1".to_string()))
        );
    }

    #[test]
    fn keeps_non_tag_brackets_in_preamble() {
        let (header, _) =
            Header::parse("Log by <JL1HIS>\n<EOH>", LengthMode::Bytes).expect("must parse");
        assert_eq!(
            header.expect("must have header").preamble,
            "Log by <JL1HIS>\n"
        );
    }
}
