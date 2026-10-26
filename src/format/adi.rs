mod data;
pub mod error;
mod header;
mod record;
mod tag;

use std::ops::Range;

use crate::{
    LengthMode,
    document::{AdifDocument, Field, IntoAdifDocument},
    format::adi::{error::AdiError, header::Header, record::Record, tag::Tag},
};

#[derive(Debug, Clone)]
pub struct AdiDocument<'a> {
    header: Option<Header<'a>>,
    records: Vec<(Record<'a>, Range<usize>)>,
}

impl<'a> AdiDocument<'a> {
    pub fn parse(text: &'a str, length_mode: LengthMode) -> Result<AdiDocument<'a>, AdiError> {
        let mut consumed = if text.starts_with('\u{feff}') {
            '\u{feff}'.len_utf8()
        } else {
            0
        };

        let (header, header_consumed) =
            Header::parse(&text[consumed..], length_mode).map_err(|e| e.offset_by(consumed))?;
        consumed += header_consumed;

        let mut records = vec![];
        while let Some(tag_start) = Tag::find(&text[consumed..]) {
            let start = consumed + tag_start;
            let (record, record_consumed) =
                Record::parse(&text[consumed..], length_mode).map_err(|e| e.offset_by(consumed))?;
            consumed += record_consumed;
            records.push((record, start..consumed));
        }

        Ok(AdiDocument { header, records })
    }
}

impl<'a> IntoAdifDocument for AdiDocument<'a> {
    fn into_adif_document(self) -> AdifDocument {
        let (preamble, headers, user_defined_fields) = match self.header {
            Some(h) => (
                h.preamble,
                Some(
                    h.fields
                        .into_iter()
                        .map(|(k, (v, t))| (k.as_str().to_string(), Field::new(v, t))),
                ),
                h.user_defined_fields,
            ),
            None => ("", None, vec![]),
        };
        let (records, spans): (Vec<_>, Vec<_>) = self.records.into_iter().unzip();
        let records = records.into_iter().map(|r| {
            r.fields
                .into_iter()
                .map(|(k, (v, t))| (k.as_str().to_string(), Field::new(v, t)))
        });
        AdifDocument::new(
            preamble.to_string(),
            headers.into_iter().flatten(),
            user_defined_fields,
            records,
        )
        .with_record_spans(spans)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AdiDocument, LengthMode,
        error::{AdiError, TagError},
    };

    #[test]
    fn reports_error_position_from_document_start() {
        let text = "h\n<EOH>\n<CALL:3>ABC<EOR>\n<CALL:3>ABC";
        assert_eq!(
            AdiDocument::parse(text, LengthMode::Bytes).map(|_| ()),
            Err(AdiError::Tag(36, TagError::NotValidTag))
        );
    }

    #[test]
    fn skips_byte_order_mark() {
        let adi =
            AdiDocument::parse("\u{feff}<CALL:3>ABC<EOR>", LengthMode::Bytes).expect("must parse");
        assert!(adi.header.is_none());
        assert_eq!(adi.records.len(), 1);
        assert_eq!(
            adi.records[0].0.fields.get(&"CALL".into()),
            Some(&("ABC", None))
        );
    }

    #[test]
    fn includes_byte_order_mark_in_error_position() {
        assert_eq!(
            AdiDocument::parse("\u{feff}<CALL:3>ABC<EOH>", LengthMode::Bytes).map(|_| ()),
            Err(AdiError::NoEor(14))
        );
    }
}
