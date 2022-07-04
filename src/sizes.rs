use derive_more::{Add, Neg, Sub};
use fixed::types::I12F52;
use fixed::ParseFixedError;
use fixed_macro::fixed;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Mul;
use std::str::FromStr;
use thiserror::Error;

/// Support dimensions up to 2^11 (2048mm)
type Num = I12F52;

const MM_PER_IN: Num = fixed!(25.4: I12F52);

/// A dimension in fractional mm, as used in KiCad file formats
#[derive(Copy, Clone, PartialEq, Add, Sub, Neg)]
pub struct KicadDim(pub Num);

impl Debug for KicadDim {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)?;
        f.write_str("mm")
    }
}

impl Display for KicadDim {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Error, Debug)]
pub enum ParseKicadDimError {
    #[error("invalid suffix, expected `in` or `mm`")]
    InvalidSuffix,
    #[error("failed to parse as number: {0}")]
    InvalidNumber(#[from] ParseFixedError),
}

impl FromStr for KicadDim {
    type Err = ParseKicadDimError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_suffix("mm") {
            let num: Num = s.parse()?;
            Ok(Self(num))
        } else if let Some(s) = s.strip_suffix("in") {
            let num: Num = s.parse()?;
            Ok(Self(num * MM_PER_IN))
        } else {
            Err(ParseKicadDimError::InvalidSuffix)
        }
    }
}

impl Mul<PixelDim> for KicadDim {
    type Output = Self;

    fn mul(self, rhs: PixelDim) -> Self::Output {
        Self(self.0 * Num::from_num(rhs.0))
    }
}

/// A position in KiCad.
///
/// Right and down positive.
#[derive(Debug)]
pub struct KicadPos {
    pub x: KicadDim,
    pub y: KicadDim,
}

impl Mul<PixelPos> for KicadPos {
    type Output = Self;

    fn mul(self, rhs: PixelPos) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

/// A pixel index.
#[derive(Copy, Clone, PartialEq, PartialOrd, Add)]
pub struct PixelDim(pub u32);

impl Debug for PixelDim {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl PixelDim {
    pub fn abs_diff(self, other: Self) -> Self {
        Self(self.0.abs_diff(other.0))
    }
}

/// A pixel position.
///
/// Right and down positive.
#[derive(Debug, Copy, Clone, Add)]
pub struct PixelPos {
    pub x: PixelDim,
    pub y: PixelDim,
}

impl PixelPos {
    pub const X1: PixelPos = PixelPos {
        x: PixelDim(1),
        y: PixelDim(0),
    };
    pub const Y1: PixelPos = PixelPos {
        x: PixelDim(0),
        y: PixelDim(1),
    };
}
