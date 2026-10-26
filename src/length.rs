use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthMode {
    Bytes,
    Codepoints,
    Graphemes,
}

impl LengthMode {
    /// Returns the length of `value` counted in this mode.
    pub fn count(self, value: &str) -> usize {
        match self {
            LengthMode::Bytes => value.len(),
            LengthMode::Codepoints => value.chars().count(),
            LengthMode::Graphemes => value.graphemes(true).count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LengthMode;

    #[test]
    fn counts_by_mode() {
        let value = "ABÄあ👨‍👩‍👧‍👦";
        assert_eq!(LengthMode::Bytes.count(value), 32);
        assert_eq!(LengthMode::Codepoints.count(value), 11);
        assert_eq!(LengthMode::Graphemes.count(value), 5);
    }
}
