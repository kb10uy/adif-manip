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
        LengthMode::Bytes => {
            if text.len() < length {
                FieldValue::NotEnough(text.len())
            } else if text.is_char_boundary(length) {
                FieldValue::Found(&text[..length])
            } else {
                FieldValue::InvalidBoundary
            }
        }
        LengthMode::Codepoints => {
            // length > 0
            let mut text_end_chars = text.char_indices().skip(length - 1);

            if text_end_chars.next().is_none() {
                return FieldValue::NotEnough(text.chars().count());
            }
            match text_end_chars.next() {
                Some((e, _)) => FieldValue::Found(&text[..e]),
                None => FieldValue::Found(text),
            }
        }
        LengthMode::Graphemes => {
            // length > 0
            let mut text_end_graphemes = text.grapheme_indices(true).skip(length - 1);

            if text_end_graphemes.next().is_none() {
                return FieldValue::NotEnough(text.graphemes(true).count());
            }
            match text_end_graphemes.next() {
                Some((e, _)) => FieldValue::Found(&text[..e]),
                None => FieldValue::Found(text),
            }
        }
    }
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
