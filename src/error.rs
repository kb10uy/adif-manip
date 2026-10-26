use roxmltree::Error as XmlError;
use thiserror::Error as ThisError;

pub use crate::format::{
    adi::error::{AdiError, TagError},
    adx::error::AdxError,
};

#[derive(Debug, ThisError)]
pub enum AdifError {
    #[error("ADI error {0}")]
    Adi(#[from] AdiError),

    #[error("ADX error {0}")]
    Adx(#[from] AdxError),

    #[error("XML error {0}")]
    Xml(#[from] XmlError),
}

#[derive(Debug, ThisError, PartialEq, Eq)]
pub enum WriteError {
    #[error("invalid field name: {0:?}")]
    FieldName(String),

    #[error("preamble must be non-empty and contain no '<'")]
    Preamble,
}
