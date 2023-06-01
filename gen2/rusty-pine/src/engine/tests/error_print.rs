use colored::Colorize;
use std::fmt::{Display, Formatter};
use std::ops::Range;

pub struct TestErrorReport<'a> {
    pub header: TestHeaderLine<'a>,
    pub message: ErrorMessage<'a>,
    pub file_extract: FileExtract<'a>,
    pub diff: TestOutcomeDiff<'a>,
}

/// test rusty_pine::engine::tests:: humans | s: id name ... FAILED
pub struct TestHeaderLine<'a> {
    pub module: &'a str,
    pub input: &'a str,
    pub outcome: TestOutcome,
}

/// error: unexpected output
pub struct ErrorMessage<'a> {
    pub message: &'a str,
}

///    --> src/engine/tests/pine-tests.sql:145
///     |
/// 145 | -- Test: humans | s: id name
///     |          ------------------- output for this pine
/// 123 | /                 format!("{} {}", " ".repeat(line_number.len()), "|")
/// 124 | |                     .bold()
/// 125 | |                     .blue(),
/// 126 | |                 format!(
/// 127 | |                     "{} {}",
/// 128 | |                     "^".repeat(expected_output.len()),
/// 129 | |                     "did not match this"
/// 130 | |                 )
/// 131 | |                 .bold()
/// 132 | \                 .red(),
///     |
pub struct FileExtract<'a> {
    pub location: TestLocation<'a>,
    pub test_input: TestInput<'a>,
    pub expected_outcome: ExpectedOutcome<'a>,
}

///    --> src/engine/tests/pine-tests.sql:145
pub struct TestLocation<'a> {
    pub left_pad: usize,
    pub file_path: &'a str,
    pub line: usize,
    pub column: usize,
}

///     |
/// 145 | -- Test: humans | s: id name
///     |          ------------------- output for this pine
pub struct TestInput<'a> {
    pub left_pad: usize,
    pub content: &'a str,
    pub line: usize,
    pub highlight: Range<usize>,
}

/// 123 | /                 format!("{} {}", " ".repeat(line_number.len()), "|")
/// 124 | |                     .bold()
/// 125 | |                     .blue(),
/// 126 | |                 format!(
/// 127 | |                     "{} {}",
/// 128 | |                     "^".repeat(expected_output.len()),
/// 129 | |                     "did not match this"
/// 130 | |                 )
/// 131 | |                 .bold()
/// 132 | \                 .red(),
///     |
pub struct ExpectedOutcome<'a> {
    pub left_pad: usize,
    pub content: &'a str,
    pub start_line: usize,
}

/// Left:  1
/// Right: 2
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
        writeln!(f, "+===============================")?;
        writeln!(f, "{}", self.header)?;
        writeln!(f, "{}", self.message)?;
        writeln!(f, "{}", self.file_extract)?;
        write!(f, "{}", self.diff)?;

        writeln!(f)?;
        writeln!(f, "+===============================")
    }
}

impl<'a> Display for TestHeaderLine<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "test {}:: {} ... {}",
            self.module.strip_prefix("rusty_pine::").unwrap(),
            self.input,
            self.outcome
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
        write!(f, "{}", self.expected_outcome)
    }
}

impl<'a> Display for TestLocation<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = " ".repeat(self.left_pad - 1);
        let arrow = "-->".blue().bold();
        let file = self.file_path;
        let line = self.line;
        let column = self.column;

        write!(f, "{gutter}{arrow} {file}:{line}:{column}")
    }
}

impl<'a> Display for TestInput<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = self.left_pad;

        let vertical_space = gutter.with("", "");

        let input_line = gutter.with(self.line, format!("  {}", self.content));

        let underline = "^".repeat(self.highlight.len());
        let highlight = gutter.with(
            "",
            format!(
                "  {:^margin_left$}{underline} output for this query",
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

impl<'a> Display for ExpectedOutcome<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let gutter = self.left_pad;
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
        let left = self.expected.replace('\n', "\\n").red();
        let right = "hi ".replace('\n', "\\n").red();

        writeln!(f, "Left:  {left}")?;
        writeln!(f, "Right: {right}")
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
