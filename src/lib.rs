//! A top-level for the Welly programming language. This crate provides error
//! reporting, including source code locations. It also provides a REPL.

pub use ansi_term;

mod loc;
pub use loc::{Location, Loc, Locate};

mod error;
pub use error::{Report, Error, Result};

mod repl;
pub use repl::{Repl};
