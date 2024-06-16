use crate::error::ErrorKind;
use colored::Colorize;
use pest::error::InputLocation;
use std::fmt::{Display, Formatter};
use thiserror::Error;

type PestError = pest::error::Error<crate::engine::Rule>;

#[derive(Debug, Error)]
pub struct WrappedPestError(PestError);

impl Display for WrappedPestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = &self.0;

        write!(
            f,
            "{line}\n\
             {underline} {message}\n\
             ",
            line = error.line(),
            underline = self.underline().red().bold(),
            message = error.variant.message().bold().red(),
        )?;

        Ok(())
    }
}

impl WrappedPestError {
    fn underline(&self) -> String {
        match self.0.location {
            InputLocation::Pos(position) => {
                // Most people would just use line[..position] here, but this can secretly panic!
                let line_until_pos = self
                    .0
                    .line()
                    .get(..position)
                    .expect("Pest should not give positions out of bounds");

                format!("{blank_indent}^", blank_indent = blank(line_until_pos))
            }
            InputLocation::Span((start, end)) => {
                let line_until_start = self
                    .0
                    .line()
                    .get(..start)
                    .expect("Pest should not give start positions out of bounds");

                format!(
                    "{blank_indent}{underline}",
                    blank_indent = blank(line_until_start),
                    // This can panic! if pest fucks it up and puts start after end.
                    // This will also look like shit if the input has any tabs ¯\_(ツ)_/¯.
                    underline = "^".repeat(end - start),
                )
            }
        }
    }
}

fn blank(input: &str) -> String {
    input
        .chars()
        // If we just replace any char with a space, tabs will be much shorter.
        // So we have to preserve tabs.
        .map(|c| if c == '\t' { '\t' } else { ' ' })
        .collect()
}

// TODO try without this?
impl From<PestError> for ErrorKind {
    fn from(value: PestError) -> Self {
        WrappedPestError(value).into()
    }
}
