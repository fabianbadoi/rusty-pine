//! Integration tests.
//!
//! This module runs examples of (input, output) pairs from the src/tests folder as tests.
//! The idea is that when adding a new feature, you can either start by expanding the grammar file
//! and then work your way up, or start by adding a new integration test, and work your way down.
//!
//! This module also makes sure to display test failures in a human friendly way, but you will have
//! to run the tests with `cargo test -- --nocapture` to see the output.
use crate::engine::tests::error_print::{
    ErrorMessage, ExpectedOutcome, FileExtract, FileExtractMessage, TestErrorReport,
    TestHeaderLine, TestInput, TestLocation, TestOutcome, TestOutcomeDiff,
};
use crate::engine::tests::reader::{SqlTestFileReader, Test};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use thiserror::Error;

mod error_print;
mod reader;

/// The only actual rust #\[test\] we have is this, it will scan the entire src/tests folder
/// and run everything from there.
#[test]
fn run_integration_tests() {
    let results = run_all_tests_in_test_folder();

    // TODO need better because we don't display anything after the first failure, even if
    // we have other failures.
    for result in results {
        assert!(result.is_success(), "{}", result);
    }
}

fn run_all_tests_in_test_folder() -> Vec<SuiteResult> {
    // I've chosen to put my integration tests in this folder. It's not a rust module, it's just a
    // folder.
    let test_files_dir = Path::new("src/tests");
    let test_files = fs::read_dir(test_files_dir).expect("Failed to read test files directory");

    let mut results = Vec::<SuiteResult>::new();

    // I could have also used .map(|file| ...) instead
    for file in test_files.flatten() {
        let file_path = file.path();
        // Rust strings are UTF-8, but the files on an OS can be in other encodings. Because of
        // this .to_str() can actually fail! I don't really care, so you'll see me unwrapping OS
        // strings like that all over the place.
        let file_path_as_str = file_path.to_str().unwrap().to_owned();
        let test_reader = SqlTestFileReader::new(file_path);

        match test_reader {
            Ok(reader) => {
                results.push(run_tests(reader));
            }
            // This can happen if we can't open the file at all.
            Err(error) => results.push(SuiteResult {
                file: file_path_as_str,
                result: Err(error),
            }),
        };
    }

    results
}

fn run_tests(tests: SqlTestFileReader) -> SuiteResult {
    let mut test_results = Vec::new();
    let file = tests.file_path.to_str().unwrap().to_owned();

    for test in tests {
        match test {
            Ok(test) => {
                let outcome = run_single_test(&test);
                test_results.push(TestResult { test, outcome });
            }
            // Even if we successfully read bytes from the file, they might not be valid UTF-8
            // bytes, so failures are still possible even here.
            Err(error) => {
                return SuiteResult {
                    file,
                    result: Err(error),
                }
            }
        };
    }

    SuiteResult {
        file,
        result: Ok(test_results),
    }
}

fn run_single_test(test: &Test) -> Outcome {
    let found_output = super::render(test.input());

    if let Err(error) = found_output {
        return Outcome::Error(error.into());
    }

    let found_output = found_output.unwrap();
    if test.expected() == found_output {
        Outcome::Success
    } else {
        Outcome::Error(TestError::DifferentOutput(found_output))
    }
}

struct SuiteResult {
    file: String,
    result: Result<Vec<TestResult>, std::io::Error>,
}

struct TestResult {
    test: Test,
    outcome: Outcome,
}

enum Outcome {
    Success,
    Error(TestError),
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        match self {
            Outcome::Success => true,
            Outcome::Error(_) => false,
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
enum TestError {
    #[error("Expected something else :\n{0}")]
    DifferentOutput(String),
    RenderError(#[from] crate::error::Error),
}

impl TestResult {
    /// Returns a printable error report.
    ///
    /// ```
    /// # let report: TestErrorReport = todo()!;
    /// println!("{report}"); // like I said, convenient
    /// ```
    pub fn to_error_report(&self) -> Option<TestErrorReport> {
        let gutter_width = (self.test.line_nr + self.test.expected().lines().count())
            .to_string()
            .len()
            + 2;

        match &self.outcome {
            Outcome::Success => None,
            Outcome::Error(err) => match err {
                TestError::DifferentOutput(found) => Some(TestErrorReport {
                    header: self.test_header_line(TestOutcome::Failure),
                    file_extract: self.test_highlighting_file_extract(),
                    message: ErrorMessage {
                        message: "unexpected outcome",
                    },
                    diff: Some(TestOutcomeDiff {
                        expected: self.test.expected(),
                        found: found.as_str(),
                    }),
                }),
                TestError::RenderError(error) => Some(TestErrorReport {
                    header: self.test_header_line(TestOutcome::Failure),
                    file_extract: self.file_extract_with_error(error),
                    message: ErrorMessage {
                        message: "Processing error",
                    },
                    diff: None,
                }),
            },
        }
    }

    fn test_header_line(&self, test_outcome: TestOutcome) -> TestHeaderLine {
        TestHeaderLine {
            module: self.test.file.split_once('/').expect("We know it's fine").1,
            input: self.test.input(),
            outcome: test_outcome,
        }
    }

    fn test_highlighting_file_extract(&self) -> FileExtract {
        let gutter_width = (self.test.line_nr + self.test.expected().lines().count())
            .to_string()
            .len()
            + 2;

        FileExtract {
            location: TestLocation {
                file_path: self.test.file.as_str(),
                line: self.test.line_nr,
                column: self.test.input_range().start,
                gutter_width,
            },
            test_input: TestInput {
                line: self.test.line_nr,
                content: self.test.input_line(),
                highlight: self.test.input_range(),
                gutter_width,
            },
            message: FileExtractMessage::ExpectedOutcome(ExpectedOutcome {
                start_line: self.test.line_nr + self.test.input().lines().count(),
                content: self.test.expected(),
                gutter_width,
            }),
        }
    }

    fn file_extract_with_error<'a>(&'a self, error: &'a crate::error::Error) -> FileExtract<'a> {
        let gutter_width = (self.test.line_nr + self.test.expected().lines().count())
            .to_string()
            .len()
            + 2;

        FileExtract {
            location: TestLocation {
                file_path: self.test.file.as_str(),
                line: self.test.line_nr,
                column: self.test.input_range().start,
                gutter_width,
            },
            test_input: TestInput {
                line: self.test.line_nr,
                content: self.test.input_line(),
                highlight: self.test.input_range(),
                gutter_width,
            },
            message: FileExtractMessage::RenderingError(error),
        }
    }
}

impl SuiteResult {
    pub fn is_success(&self) -> bool {
        match &self.result {
            Ok(tests) => tests.iter().all(|t| t.outcome.is_success()),
            Err(_) => false,
        }
    }
}

impl Display for SuiteResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.result {
            Ok(tests) => {
                let failures = tests.iter().filter_map(|t| t.to_error_report());
                for failure in failures {
                    writeln!(f, "{failure}")?;
                }
                Ok(())
            }
            Err(err) => writeln!(f, "{err}"),
        }
    }
}
