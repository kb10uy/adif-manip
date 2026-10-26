use encoding_rs::Encoding;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthMode {
    Bytes,
    Codepoints,
    Graphemes,
    Utf16Units,
    EncodedBytes(&'static Encoding),
}

impl LengthMode {
    /// Returns the length of `value` counted in this mode.
    pub fn count(self, value: &str) -> usize {
        match self {
            LengthMode::Graphemes => value.graphemes(true).count(),
            _ => value.chars().map(|c| self.char_len(c)).sum(),
        }
    }

    pub(crate) fn char_len(self, c: char) -> usize {
        match self {
            LengthMode::Bytes => c.len_utf8(),
            LengthMode::Codepoints | LengthMode::Graphemes => 1,
            LengthMode::Utf16Units => c.len_utf16(),
            LengthMode::EncodedBytes(encoding)
                if c.is_ascii() && encoding.is_ascii_compatible() =>
            {
                1
            }
            LengthMode::EncodedBytes(encoding) => {
                let mut buffer = [0; 4];
                encoding.encode(c.encode_utf8(&mut buffer)).0.len()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use encoding_rs::{SHIFT_JIS, WINDOWS_1252};

    use super::LengthMode;

    #[test]
    fn counts_by_mode() {
        let value = "ABÄあ👨‍👩‍👧‍👦";
        assert_eq!(LengthMode::Bytes.count(value), 32);
        assert_eq!(LengthMode::Codepoints.count(value), 11);
        assert_eq!(LengthMode::Graphemes.count(value), 5);
        assert_eq!(LengthMode::Utf16Units.count(value), 15);
    }

    #[test]
    fn counts_bytes_in_an_encoding() {
        assert_eq!(LengthMode::EncodedBytes(SHIFT_JIS).count("JA1太郎ｶﾅ"), 9);
        assert_eq!(LengthMode::EncodedBytes(WINDOWS_1252).count("Müller"), 6);
    }
}
