use std::fmt::{self, Debug, Display};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct DisplayError(Error);

impl Debug for DisplayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Into<Error>> From<T> for DisplayError {
    fn from(display: T) -> Self {
        DisplayError(display.into())
    }
}
