//! Provides a convenient way to print test errors.
//!
//! The strategy here is to create the appropriate data structures for each section of the desired
//! output, and then just implement `Display` for them.
//!
//! The output style is based on the output style of rustc/cargo and other tools like this. I use
//! the colored crate for nicer output.

use crate::engine::tests::error_print::zip::GreedyZip;
use colored::{Color, Colorize};
use std::fmt::{Display, Formatter};
use std::ops::Range;

mod zip;

/// Renders to something like:
///
/// ```txt
/// test tests/pine-tests.sql:: humans | s: id name ... FAILURE
/// error: unexpected outcome
///    --> src/tests/pine-tests.sql:9:9
///     |
///  9  |  -- Test: humans | s: id name
///     |           ^^^^^^^^^^^^^^^^^^^ processing this query
///  10 |/ SELECT id, name
///  11 || FROM humans
///  12 || LIMIT 10
///     |\ ^^^^^^^^^^^^^^^^^
///
///                 Expected                 |                  Found
/// -----------------------------------------------------------------------------------
/// SELECT id, name                          <
/// FROM humans                              <
/// LIMIT 10                                 <
/// ```
pub struct TestErrorReport<'a> {
    pub header: TestHeaderLine<'a>,
    pub message: ErrorMessage<'a>,
    pub file_extract: FileExtract<'a>,
    pub diff: Option<TestOutcomeDiff<'a>>,
}

/// Renders to something like:
///
/// ```txt
/// test tests/pine-tests.sql:: humans | s: id name ... FAILURE
/// ```
pub struct TestHeaderLine<'a> {
    pub module: &'a str,
    pub input: &'a str,
    pub outcome: TestOutcome,
}

/// Renders to something like:
///
/// ```txt
/// error: unexpected outcome
/// ```
pub struct ErrorMessage<'a> {
    pub message: &'a str,
}

/// Renders to something like:
///
/// ```txt
///    --> src/tests/pine-tests.sql:9:9
///     |
///  9  |  -- Test: humans | s: id name
///     |           ^^^^^^^^^^^^^^^^^^^ processing this query
///  10 |/ SELECT id, name
///  11 || FROM humans
///  12 || LIMIT 10
///     |\ ^^^^^^^^^^^^^^^^^
/// ```
pub struct FileExtract<'a> {
    pub location: TestLocation<'a>,
    pub test_input: TestInput<'a>,
    pub message: FileExtractMessage<'a>,
}

/// Renders to something like:
///
/// ```txt
///    --> src/tests/pine-tests.sql:9:9
/// ```
pub struct TestLocation<'a> {
    pub gutter_width: usize,
    pub file_path: &'a str,
    pub line: usize,
    pub column: usize,
}

/// Renders to something like:
///
/// ```txt
///     |
///  9  |  -- Test: humans | s: id name
///     |           ^^^^^^^^^^^^^^^^^^^ processing this query
/// ```
pub struct TestInput<'a> {
    pub gutter_width: usize,
    pub content: &'a str,
    pub line: usize,
    pub highlight: Range<usize>,
}

/// Renders to something like:
///
/// ```txt
///  10 |/ SELECT id, name
///  11 || FROM humans
///  12 || LIMIT 10
///     |\ ^^^^^^^^^^^^^^^^^
/// ```
/// Or other messages right below the test input, for example syntax errors.
pub enum FileExtractMessage<'a> {
    ExpectedOutcome(ExpectedOutcome<'a>),
    RenderingError(&'a crate::error::Error),
}

/// Renders to something like:
///
/// ```txt
///  10 |/ SELECT id, name
///  11 || FROM humans
///  12 || LIMIT 10
///     |\ ^^^^^^^^^^^^^^^^^
/// ```
pub struct ExpectedOutcome<'a> {
    pub gutter_width: usize,
    pub content: &'a str,
    pub start_line: usize,
}

/// Renders to something like:
///
/// ```txt
///                 Expected                 |                  Found
/// -----------------------------------------------------------------------------------
/// SELECT id, name                          <
/// FROM humans                              <
/// LIMIT 10                                 <
/// ```
pub struct TestOutcomeDiff<'a> {
    pub expected: &'a str,
    pub found: &'a str,
}

pub enum TestOutcome {
    // Success,
    Failure,
}

impl<'a> Display for TestErrorReport<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Since all of our data structs here implement Display printing the result means just
        // printing the structs in the right order.
        writeln!(f)?;
        writeln!(f, "{}", self.header)?;
        writeln!(f, "{}", self.message)?;
        writeln!(f, "{}", self.file_extract)?;
        writeln!(f)?;

        if let Some(diff) = &self.diff {
            write!(f, "{}", diff)?;
        }

        Ok(())
    }
}

impl<'a> Display for TestHeaderLine<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "test {}:: {} ... {}",
            self.module, self.input, self.outcome
        )
    }
}

impl<'a> Display for ErrorMessage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", "error".red().bold(), self.message.bold())
    }
}

impl<'a> Display for FileExtract<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.location)?;
        writeln!(f, "{}", self.test_input)?;
        write!(f, "{}", self.message)
    }
}

impl<'a> Display for TestLocation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = " ".repeat(self.gutter_width - 1);
        // An example of using the colored crate: .blue() and .bold() are not part of &str, but
        // are added as a trait impl on &str.
        let arrow = "-->".blue().bold();
        let file = self.file_path;
        let line = self.line;
        let column = self.column;

        write!(f, "{gutter}{arrow} {file}:{line}:{column}")
    }
}

impl<'a> Display for TestInput<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = self.gutter_width;

        // File extracts also show the line numbers in the gutter: ` <nr> |  <line content>`
        let vertical_space = gutter.with("", "");

        let input_line = gutter.with(self.line, format!("  {}", self.content));

        let underline = "^".repeat(self.highlight.len());
        let highlight = gutter.with(
            "",
            format!(
                "  {:^margin_left$}{underline} processing this query",
                " ",
                margin_left = self.highlight.start
            )
            .blue()
            .bold(),
        );

        writeln!(f, "{vertical_space}")?;
        writeln!(f, "{input_line}")?;
        write!(f, "{highlight}")
    }
}

impl<'a> Display for FileExtractMessage<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FileExtractMessage::ExpectedOutcome(expected) => write!(f, "{}", expected),
            FileExtractMessage::RenderingError(error) => {
                write!(f, "{}", error)
            }
        }
    }
}

impl<'a> Display for ExpectedOutcome<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = self.gutter_width;
        let mut max_line_width = 0;

        for (i, output_line) in self.content.lines().enumerate() {
            max_line_width = max_line_width.max(output_line.len());

            let bar_char = if i == 0 { "/" } else { "|" };
            let print_line = gutter.with(
                self.start_line + i,
                format!("{} {}", bar_char.red().bold(), output_line),
            );

            writeln!(f, "{print_line}")?;
        }

        let underline = gutter.with(
            "",
            format!(
                "{} {}",
                "\\".red().bold(),
                "^".repeat(max_line_width + 2).red().bold()
            ),
        );
        write!(f, "{underline}")
    }
}

impl<'a> Display for TestOutcomeDiff<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let left_lines = self.expected.lines();
        let right_lines = self.found.lines();
        let diff = GreedyZip::new(left_lines, right_lines);

        let header = format!("{:^60} | {:^60}", "Expected", "Found")
            .blue()
            .bold();
        let border = "-".repeat(60 + 60 + 3).blue().bold();

        writeln!(f, "{header}")?;
        writeln!(f, "{border}")?;

        for diff_line in diff {
            use zip::ZipItem::*;

            let (left, mid, right, color) = match diff_line {
                Both(left, right) => {
                    if left == right {
                        (left, ' ', right, Color::Green)
                    } else {
                        (left, '|', right, Color::Red)
                    }
                }
                LeftOnly(left) => (left, '<', "", Color::Red),
                RightOnly(right) => ("", '>', right, Color::Red),
            };

            let left = &left[..(60.min(left.len()))];
            let right = &right[..(60.min(right.len()))];

            let line = format!("{:<60} {} {:<60}", left, mid, right).color(color);

            writeln!(f, "{line}")?;
        }

        Ok(())
    }
}

static BAR: Bar = Bar();

impl Display for TestOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestOutcome::Failure => write!(f, "{}", "FAILURE".red()),
        }
    }
}

struct WithGutter<G, T> {
    gutter_width: usize,
    gutter_content: G,
    line_content: T,
}

impl<G, T> Display for WithGutter<G, T>
where
    G: Display,
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let width = self.gutter_width;
        let gutter = self.gutter_content.to_string().bold().blue();
        let line = &self.line_content;

        write!(f, "{:^width$}{BAR}{line}", gutter)
    }
}

trait AddGutter {
    fn with<G, T>(self, gutter: G, line: T) -> WithGutter<G, T>;
}

impl<W> AddGutter for W
where
    W: Into<usize>,
{
    fn with<G, T>(self, gutter: G, line: T) -> WithGutter<G, T> {
        WithGutter {
            gutter_content: gutter,
            line_content: line,
            gutter_width: self.into(),
        }
    }
}

struct Bar();
impl Display for Bar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "|".blue().bold())
    }
}
