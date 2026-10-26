use unicode_segmentation::UnicodeSegmentation;

use crate::{LengthMode, format::adi::error::AdiError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldValue<'a> {
    Found(&'a str),
    InvalidBoundary,
    NotEnough(usize),
}

/// Splits `text` at specified `length` according to `mode`.
pub fn get_field_value(text: &str, mode: LengthMode, length: usize) -> FieldValue<'_> {
    if length == 0 {
        return FieldValue::Found("");
    }

    match mode {
        LengthMode::Graphemes => split(
            text,
            text.grapheme_indices(true).map(|(i, g)| (i + g.len(), 1)),
            length,
        ),
        _ => split(
            text,
            text.char_indices()
                .map(|(i, c)| (i + c.len_utf8(), mode.char_len(c))),
            length,
        ),
    }
}

fn split<I: Iterator<Item = (usize, usize)>>(
    text: &str,
    units: I,
    length: usize,
) -> FieldValue<'_> {
    let mut counted = 0;
    for (end, unit_length) in units {
        counted += unit_length;
        if counted == length {
            return FieldValue::Found(&text[..end]);
        } else if counted > length {
            return FieldValue::InvalidBoundary;
        }
    }
    FieldValue::NotEnough(counted)
}

pub fn read_field_value(
    text: &str,
    position: usize,
    mode: LengthMode,
    length: usize,
) -> Result<&str, AdiError> {
    match get_field_value(&text[position..], mode, length) {
        FieldValue::Found(v) => Ok(v),
        FieldValue::InvalidBoundary => Err(AdiError::CharacterBoundary(position)),
        FieldValue::NotEnough(available) => Err(AdiError::ValueTooShort {
            expected: length,
            available,
        }),
    }
}

#[cfg(test)]
mod tests {
    use encoding_rs::SHIFT_JIS;

    use super::{FieldValue, LengthMode, get_field_value};

    #[test]
    fn splits_by_bytes() {
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Bytes, 3),
            FieldValue::Found("ABC")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Bytes, 7),
            FieldValue::Found("ABCÄË")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Bytes, 15),
            FieldValue::Found("ABCÄËÖあい")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Bytes, 43),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Bytes, 68),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦")
        );
    }

    #[test]
    fn splits_by_codepoints() {
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Codepoints, 3),
            FieldValue::Found("ABC")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Codepoints, 5),
            FieldValue::Found("ABCÄË")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Codepoints, 8),
            FieldValue::Found("ABCÄËÖあい")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Codepoints, 16),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Codepoints, 23),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦")
        );
    }

    #[test]
    fn splits_by_graphemes() {
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Graphemes, 3),
            FieldValue::Found("ABC")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Graphemes, 5),
            FieldValue::Found("ABCÄË")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Graphemes, 8),
            FieldValue::Found("ABCÄËÖあい")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Graphemes, 10),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦")
        );
        assert_eq!(
            get_field_value("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦", LengthMode::Graphemes, 11),
            FieldValue::Found("ABCÄËÖあいう👨‍👩‍👧‍👦👨‍👩‍👧‍👦")
        );
    }

    #[test]
    fn splits_by_utf16_units() {
        assert_eq!(
            get_field_value("あ👨‍👩‍👧‍👦", LengthMode::Utf16Units, 3),
            FieldValue::Found("あ👨")
        );
        assert_eq!(
            get_field_value("あ👨‍👩‍👧‍👦", LengthMode::Utf16Units, 2),
            FieldValue::InvalidBoundary
        );
    }

    #[test]
    fn splits_by_encoded_bytes() {
        let mode = LengthMode::EncodedBytes(SHIFT_JIS);
        assert_eq!(
            get_field_value("JA太郎ｶ<EOR>", mode, 7),
            FieldValue::Found("JA太郎ｶ")
        );
        assert_eq!(
            get_field_value("JA太郎ｶ<EOR>", mode, 3),
            FieldValue::InvalidBoundary
        );
    }

    #[test]
    fn counts_available_length_by_mode() {
        assert_eq!(
            get_field_value("Äあ👨‍👩‍👧‍👦", LengthMode::Bytes, 99),
            FieldValue::NotEnough(30)
        );
        assert_eq!(
            get_field_value("Äあ👨‍👩‍👧‍👦", LengthMode::Codepoints, 99),
            FieldValue::NotEnough(9)
        );
        assert_eq!(
            get_field_value("Äあ👨‍👩‍👧‍👦", LengthMode::Graphemes, 99),
            FieldValue::NotEnough(3)
        );
    }
}
