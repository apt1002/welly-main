use std::{io};
use io::{BufRead, Write};
use ansi_term::Colour::{Red};

use super::{Location, Loc};

// ----------------------------------------------------------------------------

/// The return type of [`Repl::lines()`].
///
/// `self.1` is a suffix of `self.0`.
struct Lines<'a>(&'a str, std::str::Chars<'a>);

impl<'a> Lines<'a> {
    /// Returns the byte position we've reached in `self.0`.
    fn pos(&self) -> usize { self.0.len() - self.1.as_str().len() }
}

impl<'a> Iterator for Lines<'a> {
    type Item = Loc<&'a str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1.as_str().len() == 0 { return None; }
        let start = self.pos();
        while let Some(c) = self.1.next() {
            if c == '\n' { break; }
        }
        let end = self.pos();
        Some(Loc(&self.0[start..end], Location {start, end}))
    }
}

// ----------------------------------------------------------------------------

/// Represents the state of a Read-Eval-Print-Loop.
///
/// # Example
///
/// ```
/// use std::io::{self, Write};
/// use welly_main::{self as wm, Location, Loc, Repl};
///
/// /// We accept commands ending with `;` and not containing "error".
/// fn check(command: Loc<&str>) -> wm::Result<String> {
///     if let Some(index) = command.0.find("error") {
///         let loc = Location {
///             start: command.1.start + index,
///             end: command.1.start + index + 5,
///         };
///         Err(Loc("Found an error", loc))?;
///     }
///     if !command.0.ends_with(';') {
///         Err(wm::Error::InsufficientInput)?;
///     }
///     Ok(command.0.into())
/// }
///
/// fn main() -> io::Result<()> {
///     let mut stdin = io::stdin().lock();
///     let mut stdout = io::stdout();
///     let mut repl = Repl::default();
///     while !repl.is_complete {
///         writeln!(stdout, "\nWelly!")?;
///         if let Some(command) = repl.command(&mut stdin, &mut stdout, check)? {
///             writeln!(stdout, "You typed a valid command:\n{}", command);
///         }
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Default)]
pub struct Repl {
    /// The command-line history.
    history: String,

    /// [`Self::command()`] sets this to `true` at the end of the input.
    pub is_complete: bool,
}

impl Repl {
    /// Returns an iterator over the lines of the history, with [`Location`]s.
    ///
    /// Unlike [`std::str::Lines`], line terminators *are* included.
    pub fn lines(&self) -> impl Iterator<Item=Loc<&str>> { Lines(&*self.history, self.history.chars()) }

    /// Print an error message with optional source code context.
    ///
    /// - output - where to write the message.
    /// - msg - the error message.
    /// - loc - the source code location within the command-line history.
    pub fn report(&self, output: &mut impl Write, msg: &str, loc: Option<Location>) -> io::Result<()> {
        writeln!(output, "\n{}: {}", Red.paint("Error"), msg)?;
        let Some(loc) = loc else { return Ok(()); };
        // TODO: Use an index to keep the following fast as the history grows.
        let lines: Vec<Loc<&str>> = self.lines().filter(|Loc(_, line_loc)| line_loc.overlaps(loc)).collect();
        let loc = loc - lines.first().expect("Location does not overlap with history").1.start;
        let context: String = lines.into_iter().map(|line| line.0).collect();
        writeln!(output, "{}{}{}",
            &context[..loc.start],
            Red.paint(&context[loc.start .. loc.end]),
            &context[loc.end..],
        )?;
        Ok(())
    }

    /// Read a command from `input`, check it, and report errors to `output`.
    /// Set `is_complete` if `input` ends.
    ///
    /// Returns:
    /// - `Ok(Some(ret))` if `check` returns `Ok(ret)`.
    /// - `Ok(None)` if `check` returns an error that can't be fixed by more
    ///   input.
    /// - `Err` if `input` or `output` do so.
    pub fn command<T>(
        &mut self,
        input: &mut impl BufRead,
        output: &mut impl Write,
        check: impl Fn(Loc<&str>) -> crate::Result<T>,
    ) -> io::Result<Option<T>> {
        let start = self.history.len();
        loop {
            output.flush()?;
            if input.read_line(&mut self.history)? == 0 { self.is_complete = true; }
            let loc = Location {start, end: self.history.len()};
            let command = Loc(&self.history[start..], loc);
            match check(command) {
                Ok(ret) => { return Ok(Some(ret)); },
                Err(crate::Error::InsufficientInput) if !self.is_complete => { continue; }
                Err(e) => {
                    let mut ret = Ok(());
                    e.report(|msg, loc| {
                        if ret.is_ok() { ret = self.report(output, msg, loc); }
                    });
                    self.history.truncate(start);
                    return ret.and(Ok(None));
                },
            }
        }
    }
}
