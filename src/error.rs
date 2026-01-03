use crate::loc::{Location, Loc};

/// Report a complicated error to the user.
pub trait Report {
    /// Report `self` by calling `log()` as often as needed.
    fn report(&self, log: &mut dyn FnMut(&str, Option<Location>));
}

/// The type of errors with source-code locations.
pub enum Error {
    /// The parser tried to read beyond the end of the input.
    ///
    /// If the input is complete, this is an error.
    /// Otherwise it indicates that we need more input from the user.
    InsufficientInput,

    /// An error with a constant message and just one [`Location`].
    Str(&'static str, Location),

    /// A more complicated error.
    Report(Box<dyn Report>),
}

impl Error {
    /// Report `self` to the user by calling `log` as often as needed.
    pub fn report(&self, mut log: impl FnMut(&str, Option<Location>)) {
        match self {
            Self::InsufficientInput => log("The program ends unexpectedly", None),
            Self::Str(message, loc) => log(message, Some(*loc)),
            Self::Report(e) => e.report(&mut log),
        }
    }
}

impl From<Loc<&'static str>> for Error {
    fn from(value: Loc<&'static str>) -> Self { Self::Str(value.0, value.1) }
}

impl<R: Report + 'static> From<R> for Error {
    fn from(value: R) -> Self { Self::Report(Box::new(value)) }
}

// ----------------------------------------------------------------------------

/// An abbreviation.
pub type Result<T> = std::result::Result<T, Error>;
