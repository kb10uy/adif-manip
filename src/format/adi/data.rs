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

pub struct LengthReader<'m> {
    modes: &'m [LengthMode],
    last: usize,
}

impl<'m> LengthReader<'m> {
    pub fn new(modes: &'m [LengthMode]) -> LengthReader<'m> {
        assert!(!modes.is_empty(), "at least one length mode is needed");
        LengthReader { modes, last: 0 }
    }

    /// Reads the value of `length` at `position`, preferring the mode that read the previous value
    /// and then a mode whose value is followed by the next tag. The mode is returned for non-ASCII values.
    pub fn read<'t>(
        &mut self,
        text: &'t str,
        position: usize,
        length: usize,
    ) -> Result<(&'t str, Option<LengthMode>), AdiError> {
        let rest = &text[position..];
        let order =
            std::iter::once(self.last).chain((0..self.modes.len()).filter(|&i| i != self.last));
        let mut fallback = None;
        for i in order {
            if let FieldValue::Found(value) = get_field_value(rest, self.modes[i], length) {
                let next = rest[value.len()..].trim_start();
                if next.is_empty() || next.starts_with('<') {
                    return Ok(self.found(i, value));
                }
                fallback.get_or_insert((i, value));
            }
        }
        match fallback {
            Some((i, value)) => Ok(self.found(i, value)),
            None => match get_field_value(rest, self.modes[self.last], length) {
                FieldValue::NotEnough(available) => Err(AdiError::ValueTooShort {
                    expected: length,
                    available,
                }),
                _ => Err(AdiError::CharacterBoundary(position)),
            },
        }
    }

    fn found<'t>(&mut self, i: usize, value: &'t str) -> (&'t str, Option<LengthMode>) {
        self.last = i;
        (value, (!value.is_ascii()).then_some(self.modes[i]))
    }
}

#[cfg(test)]
mod tests {
    use encoding_rs::SHIFT_JIS;

    use crate::format::adi::error::AdiError;

    use super::{FieldValue, LengthMode, LengthReader, get_field_value};

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

    #[test]
    fn reads_values_in_the_mode_that_reaches_the_next_tag() {
        let mut reader = LengthReader::new(&[LengthMode::Codepoints, LengthMode::Bytes]);
        let text = "太郎 <NAME:2>花子 <EOR>";
        assert_eq!(
            reader.read(text, 0, 6),
            Ok(("太郎", Some(LengthMode::Bytes)))
        );
        assert_eq!(
            reader.read(text, 15, 2),
            Ok(("花子", Some(LengthMode::Codepoints)))
        );
    }

    #[test]
    fn prefers_the_mode_of_the_previous_value() {
        let mut reader = LengthReader::new(&[LengthMode::Codepoints, LengthMode::Bytes]);
        assert_eq!(
            reader.read("太郎 <EOR>", 0, 6),
            Ok(("太郎", Some(LengthMode::Bytes)))
        );
        assert_eq!(
            reader.read("Ä<<EOR>", 0, 2),
            Ok(("Ä", Some(LengthMode::Bytes)))
        );
    }

    #[test]
    fn falls_back_to_a_value_not_followed_by_a_tag() {
        let mut reader = LengthReader::new(&[LengthMode::Bytes, LengthMode::Codepoints]);
        assert_eq!(reader.read("ABCD", 0, 2), Ok(("AB", None)));
        assert_eq!(
            reader.read("太郎", 0, 2),
            Ok(("太郎", Some(LengthMode::Codepoints)))
        );
    }

    #[test]
    fn reports_the_error_of_the_preferred_mode() {
        let mut reader = LengthReader::new(&[LengthMode::Bytes]);
        assert_eq!(
            reader.read("x太", 1, 2),
            Err(AdiError::CharacterBoundary(1))
        );
        assert_eq!(
            reader.read("x太", 1, 9),
            Err(AdiError::ValueTooShort {
                expected: 9,
                available: 3
            })
        );
    }
}
