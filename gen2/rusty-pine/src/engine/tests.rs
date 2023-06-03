use crate::engine::tests::error_print::{
    ErrorMessage, ExpectedOutcome, FileExtract, TestErrorReport, TestHeaderLine, TestInput,
    TestLocation, TestOutcome, TestOutcomeDiff,
};
use crate::engine::tests::reader::{SqlTestFileReader, Test};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use thiserror::Error;

mod error_print;
mod reader;

#[test]
fn run_integration_tests() {
    let results = run_all_tests_in_test_folder();

    // TODO need better
    for result in results {
        assert!(result.is_success(), "{}", result);
    }
}

fn run_all_tests_in_test_folder() -> Vec<SuiteResult> {
    let test_files_dir = Path::new("src/tests");
    let test_files = fs::read_dir(test_files_dir).expect("Failed to read test files directory");
    let mut results = Vec::<SuiteResult>::new();

    for file in test_files.flatten() {
        let file_path = file.path();
        let file_path_as_str = file_path.to_str().unwrap().to_owned();
        let test_reader = SqlTestFileReader::new(file_path);

        match test_reader {
            Ok(reader) => {
                results.push(run_tests(reader));
            }
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

impl TestResult {
    pub fn to_error_report(&self) -> Option<TestErrorReport> {
        let gutter_width = (self.test.line_nr + self.test.expected().lines().count())
            .to_string()
            .len()
            + 2;

        match &self.outcome {
            Outcome::Success => None,
            Outcome::Error(err) => match err {
                TestError::DifferentOutput(found) => Some(TestErrorReport {
                    header: TestHeaderLine {
                        module: self.test.file.split_once('/').expect("We know it's fine").1,
                        input: self.test.input(),
                        outcome: TestOutcome::Failure,
                    },
                    message: ErrorMessage {
                        message: "unexpected outcome",
                    },
                    file_extract: FileExtract {
                        location: TestLocation {
                            file_path: self.test.file.as_str(),
                            line: self.test.line_nr,
                            column: self.test.input_range().start,
                            gutter_width: gutter_width,
                        },
                        test_input: TestInput {
                            line: self.test.line_nr,
                            content: self.test.input_line(),
                            highlight: self.test.input_range(),
                            left_pad: gutter_width,
                        },
                        expected_outcome: ExpectedOutcome {
                            start_line: self.test.line_nr + self.test.input().lines().count(),
                            content: self.test.expected(),
                            left_pad: gutter_width,
                        },
                    },
                    diff: TestOutcomeDiff {
                        expected: self.test.expected(),
                        found: found.as_str(),
                    },
                }),
                TestError::RenderError(_) => {
                    todo!()
                }
            },
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
                let failures = tests.iter().map(|t| t.to_error_report()).flatten();
                for failure in failures {
                    writeln!(f, "{}", failure)?;
                }
                Ok(())
            }
            Err(err) => writeln!(f, "{err}"),
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

fn run_single_test(test: &Test) -> Outcome {
    let found_output = super::render(test.input());

    if found_output.is_err() {
        return Outcome::Error(found_output.unwrap_err().into());
    }

    let found_output = found_output.unwrap();
    if test.expected() == found_output {
        Outcome::Success
    } else {
        Outcome::Error(TestError::DifferentOutput(found_output))
    }
}
