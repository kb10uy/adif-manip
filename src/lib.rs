pub mod document;
pub mod error;
mod format;
mod length;
pub mod write;

use roxmltree::Document;

use crate::{
    document::{AdifDocument, IntoAdifDocument},
    error::AdifError,
    format::{adi::AdiDocument, adx::AdxDocument},
};

pub use encoding_rs;
pub use length::LengthMode;

pub fn read_adi(adi_text: &str, length_mode: LengthMode) -> Result<AdifDocument, AdifError> {
    read_adi_detecting(adi_text, &[length_mode])
}

/// Reads ADI whose lengths may be counted in any of `length_modes`, deciding for each value.
///
/// # Panics
///
/// Panics if `length_modes` is empty.
pub fn read_adi_detecting(
    adi_text: &str,
    length_modes: &[LengthMode],
) -> Result<AdifDocument, AdifError> {
    let adi = AdiDocument::parse(adi_text, length_modes)?;
    Ok(adi.into_adif_document())
}

pub fn read_adx(adx_text: &str) -> Result<AdifDocument, AdifError> {
    let xml = Document::parse(adx_text)?;
    let adx = AdxDocument::parse(&xml)?;
    Ok(adx.into_adif_document())
}

#[cfg(test)]
mod tests {
    use encoding_rs::SHIFT_JIS;

    use crate::{LengthMode, document::DataType, read_adi, read_adi_detecting, read_adx};

    #[test]
    fn keeps_adi_data_types() {
        let adi = read_adi(
            "h<USERDEF1:3:N>EPC<EOH><CALL:6>JL1HIS<APP_X_Y:1:b>Y<EOR>",
            LengthMode::Bytes,
        )
        .expect("must parse");
        assert_eq!(adi.header_type("USERDEF1"), Some(DataType::Number));
        assert_eq!(adi.records()[0].field_type("call"), None);
        assert_eq!(
            adi.records()[0].field_type("APP_X_Y"),
            Some(DataType::Boolean)
        );
        assert_eq!(adi.records()[0].field("APP_X_Y"), Some("Y"));
    }

    #[test]
    fn keeps_adx_data_types() {
        let adx = read_adx(include_str!("../fixtures/example.adx")).expect("must parse");
        assert_eq!(adx.header_type("USERDEF2"), Some(DataType::Enumeration));
        assert_eq!(
            adx.records()[0].field_type("APP_MONOLOG_COMPRESSION"),
            Some(DataType::String)
        );
        assert_eq!(adx.records()[0].field_type("CALL"), None);
    }

    #[test]
    fn exposes_record_spans() {
        let text = "\u{feff}h<EOH>\n<CALL:3>ABC<EOR>\n junk <CALL:3>DEF<eor>\n";
        let adi = read_adi(text, LengthMode::Bytes).expect("must parse");
        let spans: Vec<_> = adi.records().iter().map(|r| r.span()).collect();
        assert_eq!(spans, [Some(10..26), Some(33..49)]);
        assert_eq!(&text[10..26], "<CALL:3>ABC<EOR>");
        assert_eq!(&text[33..49], "<CALL:3>DEF<eor>");

        let text =
            "<ADX><HEADER></HEADER><RECORDS>\n<RECORD><CALL>ABC</CALL></RECORD>\n</RECORDS></ADX>";
        let adx = read_adx(text).expect("must parse");
        let span = adx.records()[0].span().expect("must have span");
        assert_eq!(&text[span], "<RECORD><CALL>ABC</CALL></RECORD>");
    }

    #[test]
    fn exposes_user_defined_fields_in_id_order() {
        let adi = read_adi("h<USERDEF2:4>SIZE<USERDEF1:3:N>EPC<EOH>", LengthMode::Bytes)
            .expect("must parse");
        let names: Vec<_> = adi.user_defined_fields().iter().map(|u| u.name()).collect();
        assert_eq!(names, ["EPC", "SIZE"]);

        let adx = read_adx(include_str!("../fixtures/example.adx")).expect("must parse");
        let names: Vec<_> = adx.user_defined_fields().iter().map(|u| u.name()).collect();
        assert_eq!(names, ["EPC", "SWEATERSIZE", "SHOESIZE"]);
    }

    #[test]
    fn detects_lengths_per_field() {
        let text = "<NAME:6>太郎 <NAME_INTL:2>太郎 <CALL:6>JA1XYZ <EOR>\n<NAME:2>花子 <EOR>\n";
        let adi = read_adi_detecting(text, &[LengthMode::Bytes, LengthMode::Codepoints])
            .expect("must parse");
        let [first, second] = adi.records() else {
            panic!("must have two records");
        };
        assert_eq!(first.field("NAME"), Some("太郎"));
        assert_eq!(first.field("NAME_INTL"), Some("太郎"));
        assert_eq!(first.field("CALL"), Some("JA1XYZ"));
        assert_eq!(second.field("NAME"), Some("花子"));

        let mode = |record: &crate::document::Record, name| record.fields()[name].length_mode();
        assert_eq!(mode(first, "NAME"), Some(LengthMode::Bytes));
        assert_eq!(mode(first, "NAME_INTL"), Some(LengthMode::Codepoints));
        assert_eq!(mode(first, "CALL"), None);
        assert_eq!(mode(second, "NAME"), Some(LengthMode::Codepoints));
    }

    #[test]
    fn tells_shift_jis_bytes_from_utf16_units() {
        let text = "<NAME:4>太郎 <EOR>\n<NAME:2>花子 <EOR>\n";
        let modes = [LengthMode::EncodedBytes(SHIFT_JIS), LengthMode::Utf16Units];
        let adi = read_adi_detecting(text, &modes).expect("must parse");
        let names: Vec<_> = adi.records().iter().map(|r| r.field("NAME")).collect();
        assert_eq!(names, [Some("太郎"), Some("花子")]);
    }

    #[test]
    fn reports_the_mode_of_non_ascii_values_read_in_one_mode() {
        let adi =
            read_adi("<NAME:6>太郎 <CALL:6>JA1XYZ <EOR>", LengthMode::Bytes).expect("must parse");
        let fields = adi.records()[0].fields();
        assert_eq!(fields["NAME"].length_mode(), Some(LengthMode::Bytes));
        assert_eq!(fields["CALL"].length_mode(), None);
    }
}
