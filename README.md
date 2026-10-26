# adif-manip

A Rust library for manipulating amateur radio logs in the [ADIF](https://adif.org.uk/) format.
Both file formats defined by the specification are supported:

- **ADI** (`.adi`, `.adif`): the tag-based text format
- **ADX** (`.adx`): the XML format

Both are read into the same `AdifDocument` type, so the rest of your code does not need to
care which format a log came from. Writing produces ADI.

## Features

- Header fields, QSO records and the free-form text before the first header field (the preamble)
- Field names are case-insensitive: `record.field("call")` and `record.field("CALL")` are the same
- Data type indicators (`<FREQ:5:N>`, `TYPE="N"`) are kept as `DataType` values
- Application-defined fields are available as `APP_{PROGRAMID}_{FIELDNAME}` in both formats
- User-defined field definitions (`USERDEFn`) with their enumeration or range constraints
- Selectable interpretation of ADI data lengths for files containing non-ASCII text
- A leading UTF-8 byte order mark is skipped
- Byte range of each record in the source text (`record.span()`), for passing records through unchanged
- ADI headers and records written with lengths counted in any `LengthMode`, with field names checked

## Usage

```toml
[dependencies]
adif-manip = "0.2.0"
```

### Reading an ADI file

```rust
use adif_manip::{LengthMode, read_adi};

let text = "Exported log\n<ADIF_VER:5>3.1.6 <EOH>\n\
            <CALL:6>JL1HIS <BAND:3>40M <FREQ:5:N>7.041 <EOR>\n";
let document = read_adi(text, LengthMode::Bytes)?;

assert_eq!(document.preamble(), "Exported log\n");
assert_eq!(document.header("ADIF_VER"), Some("3.1.6"));

for record in document.records() {
    println!("{:?} on {:?}", record.field("CALL"), record.field("BAND"));
}
```

### Reading an ADX file

```rust
use adif_manip::read_adx;

let xml = std::fs::read_to_string("log.adx")?;
let document = read_adx(&xml)?;
println!("{} records", document.records().len());
```

### Data types

```rust
use adif_manip::document::DataType;

let record = &document.records()[0];
assert_eq!(record.field_type("FREQ"), Some(DataType::Number));
assert_eq!(record.field_type("CALL"), None);
```

`record.fields()` and `document.headers()` give every field as a `Field`, which holds both
the value and the data type.

### User-defined fields

```rust
use adif_manip::document::ValueConstraint;

for definition in document.user_defined_fields() {
    match definition.constraint() {
        Some(ValueConstraint::Enumeration(values)) => {
            println!("{} is one of {values:?}", definition.name())
        }
        Some(ValueConstraint::Range { lower, upper }) => {
            println!("{} is between {lower} and {upper}", definition.name())
        }
        None => println!("{} is unconstrained", definition.name()),
    }
}
```

Definitions are returned in `USERDEFn` order. The values of user-defined fields are
available on records under their uppercased names, like any other field.

### Writing an ADI file

```rust
use adif_manip::{LengthMode, write};

let mut text = write::header("Exported log\n", [("ADIF_VER", "3.1.6")], LengthMode::Bytes)?;
text += &write::record([("CALL", "JL1HIS"), ("BAND", "40M")], LengthMode::Bytes)?;

assert_eq!(
    text,
    "Exported log\n<ADIF_VER:5>3.1.6 <EOH>\n<CALL:6>JL1HIS <BAND:3>40M <EOR>\n"
);
```

`write::header` and `write::record` take fields as `(name, value)` pairs and write them in
the order given. Each field is followed by a space, and `<EOH>` and `<EOR>` by a newline.
Field names are written as given and must be non-empty ASCII without `,`, `:`, `<`, `>`,
`{`, `}`, control characters or leading and trailing spaces, and must not be `EOH` or
`EOR`. The preamble must be non-empty and contain no `<`, so that readers do not take it
for data.

## ADI data lengths

The ADIF specification counts data lengths in characters, and ADI files are meant to
contain ASCII only. In practice, loggers write non-ASCII text and count its length in
different ways. `LengthMode` selects how `read_adi` interprets lengths, and how
`write::header` and `write::record` count them:

| Mode                            | Length is counted in                |
| ------------------------------- | ----------------------------------- |
| `LengthMode::Bytes`             | UTF-8 bytes                         |
| `LengthMode::Codepoints`        | Unicode scalar values               |
| `LengthMode::Graphemes`         | Extended grapheme clusters          |
| `LengthMode::Utf16Units`        | UTF-16 code units                   |
| `LengthMode::EncodedBytes(enc)` | Bytes in the `encoding_rs` encoding |

For ASCII-only files, all modes give the same result. `Utf16Units` is what programs that
measure strings in UTF-16 (Qt, Delphi) write. `EncodedBytes` is for files in a legacy
encoding such as Shift_JIS, which the caller decodes to a `&str` first; the crate
re-exports `encoding_rs`, so `LengthMode::EncodedBytes(adif_manip::encoding_rs::SHIFT_JIS)`
names one. Each character is counted on its own, so stateful encodings such as
ISO-2022-JP are not supported, and a character the encoding cannot represent counts as
the numeric character reference `encoding_rs` would write for it.

### Detecting lengths

When a file's lengths may be counted in more than one way, possibly differently from field
to field, `read_adi_detecting` takes the candidate modes and decides for each value:

```rust
use adif_manip::{LengthMode, read_adi_detecting};

let text = "<NAME:6>太郎 <NAME_INTL:2>太郎 <EOR>";
let document = read_adi_detecting(text, &[LengthMode::Bytes, LengthMode::Codepoints])?;
let fields = document.records()[0].fields();

assert_eq!(fields["NAME"].value(), "太郎");
assert_eq!(fields["NAME"].length_mode(), Some(LengthMode::Bytes));
assert_eq!(fields["NAME_INTL"].length_mode(), Some(LengthMode::Codepoints));
```

A value is read in the mode that read the previous value if the next tag follows it, and
otherwise in the first candidate after which the next tag follows. When none does, the
value is read as `read_adi` would read it in the mode of the previous value. The order of
the candidates matters only for the first value and for values that fit several modes.
`Field::length_mode` tells which mode a non-ASCII ADI value was read in, also for
`read_adi`; it is `None` for ASCII values, where all modes agree, and for ADX.

## Errors

`read_adi`, `read_adi_detecting` and `read_adx` return `adif_manip::error::AdifError`. It wraps `AdiError`,
`AdxError` or an XML syntax error. ADI error positions are byte offsets from the start of
the input.

`write::header` and `write::record` return `adif_manip::error::WriteError` for an invalid
field name or preamble.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE](LICENSE))
- MIT license ([LICENSE.MIT](LICENSE.MIT))

at your option.
