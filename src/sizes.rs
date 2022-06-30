use fixed::types::I12F52;
use fixed::ParseFixedError;
use fixed_macro::fixed;
use std::str::FromStr;
use thiserror::Error;

/// Support dimensions up to 2^11 (2048mm)
type Num = I12F52;

const MM_PER_IN: Num = fixed!(25.4: I12F52);

/// A dimension in fractional mm, as used in KiCad file formats
#[derive(Debug)]
pub struct Length(Num);

#[derive(Error, Debug)]
pub enum ParseLengthError {
    #[error("invalid suffix, expected `in` or `mm`")]
    InvalidSuffix,
    #[error("failed to parse as number: {0}")]
    InvalidNumber(#[from] ParseFixedError),
}

impl FromStr for Length {
    type Err = ParseLengthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_suffix("mm") {
            let num: Num = s.parse()?;
            Ok(Self(num))
        } else if let Some(s) = s.strip_suffix("in") {
            let num: Num = s.parse()?;
            Ok(Self(num * MM_PER_IN))
        } else {
            Err(ParseLengthError::InvalidSuffix)
        }
    }
}
