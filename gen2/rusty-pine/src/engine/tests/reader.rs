//! Reads an .sql file that contains our integration tests.
//!
//! The way we approach testing, besides the classic rust unit tests, is to have special .sql
//! files containing integration tests.
//!
//! These files contain multiple tests that look like this:
//! ```sql
//! --       v_________________v--- this is the input
//! -- Test: humans | s: id name
//! SELECT id, name
//! FROM humans
//! LIMIT 10
//! ```
//! All tests start with "-- Test:" followed by the input pine on the same line. The next lines
//! until a blank line are the expected output.
//!
//! A single .sql file can contain multiple tests.
//!
//! It's also possible to specify the database structure in these .sql files. Simply put your
//! "CREATE TABLE X" queries before the first test.
//! Because we don't support 100% of MySQL features, and because I didn't want to make parsing these
//! too complex, there are limits on what they need to look like. For example, each crate table
//! query must end in a ";".
//!
//! ```sql
//! create table `people` (
//!     `id`           int auto_increment,
//!     `name`         varchar(256) null,
//!     `dateOfBirth`  date         not null,
//!     `placeOfBirth` varchar(256) not null,
//!     primary key (`id`),
//! );
//! ```
use crate::analyze::Server;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::ops::Range;
use std::path::PathBuf;

mod setup_parser;

type TestLineIterator = Peekable<Enumerate<Lines<BufReader<File>>>>;

pub struct SqlTestFileReader {
    pub file_path: PathBuf,
    /// The server spec can be set as create table statements at the beginning of the test file.
    ///
    /// The same server is used for all tests in the same .sql file.
    pub mock_server: Server,
    // Peekable<> because we sometimes scan for the next line.
    // Enumerate<> because we want line numbers.
    // Lines<> because we scan line by line.
    /// The test reader will walk all the lines in the test files one by one. At first (on
    /// construction) the mock_server will be built. Then each test will be emitted using
    /// the Iterator implementation until the end of the file.
    lines: TestLineIterator,
}

impl Iterator for SqlTestFileReader {
    type Item = Result<Test, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        for (line_nr, line_res) in self.lines.by_ref() {
            match line_res {
                Ok(line) if line.starts_with("-- Test: ") => {
                    return Some(self.create_test(line_nr, line));
                }
                Ok(_) => {
                    // Ignore this line: any line not in a -- Test: block is ignored.
                }
                Err(err) => {
                    return Some(Err(err.into()));
                }
            }
        }

        None
    }
}

impl SqlTestFileReader {
    pub fn new(file_path: PathBuf) -> Result<Self, crate::Error> {
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);

        let mut lines = reader.lines().enumerate().peekable();
        let mock_server = setup_parser::read_mock_server(&file_path, &mut lines)?;

        Ok(SqlTestFileReader {
            lines,
            file_path,
            mock_server,
        })
    }

    /// Once we find a "-- Test: " section, we consider the content on the *same* line as
    /// the text input pine, and the following lines (until a blank line) as the expected
    /// output.
    fn create_test(&mut self, line_nr: usize, input_line: String) -> Result<Test, crate::Error> {
        let input_range = "-- Test: ".len()..input_line.len();
        let mut content = vec![input_line];

        for (_, line_res) in self.lines.by_ref() {
            let line = line_res?;

            if line.trim().is_empty() {
                // Empty line => end of test
                break;
            }

            content.push(line);
        }

        let content = content.join("\n");

        Ok(Test {
            file: self.file(),
            line_nr: line_nr + 1, // they don't start at 0
            output_range: (input_range.end + 1)..content.len(),
            content,
            input: input_range,
        })
    }

    fn file(&self) -> String {
        self.file_path
            .to_str()
            .expect("We know it exists")
            .to_owned()
    }
}

/// One of our integration tests.
///
/// Keeping all the info related to where we found the test helps us a lot when printing test
/// failures.
pub struct Test {
    pub file: String,
    pub line_nr: usize,
    content: String,
    input: Range<usize>,
    output_range: Range<usize>,
}

impl Test {
    pub fn input(&self) -> &str {
        &self.content[self.input.clone()]
    }

    pub fn input_line(&self) -> &str {
        &self.content[0..self.input.end]
    }

    pub fn expected(&self) -> &str {
        &self.content[self.output_range.clone()]
    }

    pub fn input_range(&self) -> Range<usize> {
        self.input.clone()
    }
}
