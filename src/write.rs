use crate::{LengthMode, error::WriteError};

pub fn header<N, V, F>(
    preamble: &str,
    fields: F,
    length_mode: LengthMode,
) -> Result<String, WriteError>
where
    N: AsRef<str>,
    V: AsRef<str>,
    F: IntoIterator<Item = (N, V)>,
{
    if preamble.is_empty() || preamble.contains('<') {
        return Err(WriteError::Preamble);
    }
    let mut out = preamble.to_string();
    write_fields(&mut out, fields, length_mode)?;
    out.push_str(
        "<EOH>
",
    );
    Ok(out)
}

pub fn record<N, V, F>(fields: F, length_mode: LengthMode) -> Result<String, WriteError>
where
    N: AsRef<str>,
    V: AsRef<str>,
    F: IntoIterator<Item = (N, V)>,
{
    let mut out = String::new();
    write_fields(&mut out, fields, length_mode)?;
    out.push_str(
        "<EOR>
",
    );
    Ok(out)
}

fn write_fields<N, V, F>(
    out: &mut String,
    fields: F,
    length_mode: LengthMode,
) -> Result<(), WriteError>
where
    N: AsRef<str>,
    V: AsRef<str>,
    F: IntoIterator<Item = (N, V)>,
{
    for (name, value) in fields {
        let (name, value) = (name.as_ref(), value.as_ref());
        if !is_field_name(name) {
            return Err(WriteError::FieldName(name.to_string()));
        }
        out.push_str(&format!("<{name}:{}>{value} ", length_mode.count(value)));
    }
    Ok(())
}

fn is_field_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(' ')
        && !name.ends_with(' ')
        && !name.eq_ignore_ascii_case("EOH")
        && !name.eq_ignore_ascii_case("EOR")
        && name
            .bytes()
            .all(|b| (b.is_ascii_graphic() || b == b' ') && !b",:<>{}".contains(&b))
}

#[cfg(test)]
mod tests {
    use encoding_rs::SHIFT_JIS;

    use crate::{LengthMode, error::WriteError, read_adi};

    use super::{header, record};

    #[test]
    fn writes_header_and_record() {
        let header = header(
            "Exported
",
            [("ADIF_VER", "3.1.5")],
            LengthMode::Bytes,
        )
        .expect("must write");
        assert_eq!(
            header,
            "Exported
<ADIF_VER:5>3.1.5 <EOH>
"
        );

        let record =
            record([("CALL", "JL1HIS"), ("NAME", "")], LengthMode::Bytes).expect("must write");
        assert_eq!(
            record,
            "<CALL:6>JL1HIS <NAME:0> <EOR>
"
        );
    }

    #[test]
    fn counts_lengths_by_mode() {
        let fields = [("NAME", "太郎👨‍👩‍👧‍👦")];
        assert_eq!(
            record(fields, LengthMode::Bytes).expect("must write"),
            "<NAME:31>太郎👨‍👩‍👧‍👦 <EOR>
"
        );
        assert_eq!(
            record(fields, LengthMode::Codepoints).expect("must write"),
            "<NAME:9>太郎👨‍👩‍👧‍👦 <EOR>
"
        );
        assert_eq!(
            record(fields, LengthMode::Graphemes).expect("must write"),
            "<NAME:3>太郎👨‍👩‍👧‍👦 <EOR>
"
        );
    }

    #[test]
    fn rejects_invalid_field_names() {
        for name in [
            "", " CALL", "CALL ", "CA:LL", "CA,LL", "CA<LL", "CA>LL", "CA{LL", "CA}LL", "CALL
",
            "NAMÉ", "eoh", "EOR",
        ] {
            assert_eq!(
                record([(name, "x")], LengthMode::Bytes),
                Err(WriteError::FieldName(name.to_string())),
                "{name:?}"
            );
        }
        assert!(record([("APP_X_MY FIELD", "x")], LengthMode::Bytes).is_ok());
    }

    #[test]
    fn rejects_preambles_read_as_data() {
        for preamble in [
            "",
            "<",
            "Log by <JL1HIS>
",
        ] {
            assert_eq!(
                header(preamble, [("ADIF_VER", "3.1.5")], LengthMode::Bytes),
                Err(WriteError::Preamble),
                "{preamble:?}"
            );
        }
    }

    #[test]
    fn reads_back_what_it_writes() {
        let fields = [
            ("CALL", "JL1HIS"),
            ("NAME", "太郎👨‍👩‍👧‍👦"),
            ("COMMENT", "<EOR> in a note"),
            ("APP_X_EMPTY", ""),
        ];
        for mode in [
            LengthMode::Bytes,
            LengthMode::Codepoints,
            LengthMode::Graphemes,
            LengthMode::Utf16Units,
            LengthMode::EncodedBytes(SHIFT_JIS),
        ] {
            let text = header(
                "Exported
",
                [("ADIF_VER", "3.1.5")],
                mode,
            )
            .expect("must write")
                + &record(fields, mode).expect("must write");
            let document = read_adi(&text, mode).expect("must read");
            assert_eq!(document.header("ADIF_VER"), Some("3.1.5"));
            let [read] = document.records() else {
                panic!("must read one record");
            };
            assert_eq!(read.fields().len(), fields.len());
            for (name, value) in fields {
                assert_eq!(read.field(name), Some(value), "{mode:?} {name}");
            }
        }
    }
}
