# adif-manip

A Rust library for manipulating amateur radio logs in the [ADIF](https://adif.org.uk/) format.
Both file formats defined by the specification are supported:

- **ADI** (`.adi`, `.adif`): the tag-based text format
- **ADX** (`.adx`): the XML format

Both are read into the same `AdifDocument` type, so the rest of your code does not need to
care which format a log came from.

## Features

- Header fields, QSO records and the free-form text before the first header field (the preamble)
- Field names are case-insensitive: `record.field("call")` and `record.field("CALL")` are the same
- Data type indicators (`<FREQ:5:N>`, `TYPE="N"`) are kept as `DataType` values
- Application-defined fields are available as `APP_{PROGRAMID}_{FIELDNAME}` in both formats
- User-defined field definitions (`USERDEFn`) with their enumeration or range constraints
- Selectable interpretation of ADI data lengths for files containing non-ASCII text
- A leading UTF-8 byte order mark is skipped
- Byte range of each record in the source text (`record.span()`), for passing records through unchanged

## Usage

```toml
[dependencies]
adif-manip = "0.1.0"
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

## ADI data lengths

The ADIF specification counts data lengths in characters, and ADI files are meant to
contain ASCII only. In practice, loggers write non-ASCII text and count its length in
different ways. `LengthMode` selects how `read_adi` interprets lengths:

| Mode                     | Length is counted in       |
| ------------------------ | -------------------------- |
| `LengthMode::Bytes`      | UTF-8 bytes                |
| `LengthMode::Codepoints` | Unicode scalar values      |
| `LengthMode::Graphemes`  | Extended grapheme clusters |

For ASCII-only files, all three modes give the same result.

## Errors

`read_adi` and `read_adx` return `adif_manip::error::AdifError`. It wraps `AdiError`,
`AdxError` or an XML syntax error. ADI error positions are byte offsets from the start of
the input.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE](LICENSE))
- MIT license ([LICENSE.MIT](LICENSE.MIT))

at your option.
