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

pub struct SqlTestFileReader {
    pub file_path: PathBuf,
    /// The server spec can be set as create table statements at the beginning of the test file.
    pub mock_server: Server,
    // Peekable<> because we scan for the next line.
    // Enumerate<> because we want line numbers.
    // Lines<> because we scan line by line.
    /// The test reader will walk all the lines in the test files one by one. At first (on
    /// construction) the mock_server will be built. Then each test will be emitted using
    /// the Iterator implementation until the end of the file.
    lines: Peekable<Enumerate<Lines<BufReader<File>>>>,
}

impl Iterator for SqlTestFileReader {
    type Item = Result<Test, crate::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Advances through the file looking for -- Test: sections
        while let Some((_, peek)) = self.lines.peek() {
            match peek {
                Ok(next_line) if next_line.starts_with("-- Test: ") => {
                    return Some(self.create_test());
                }
                Ok(_) => {
                    // Ignore this line: any line not in a -- Test: block is ignored.
                    // Advance to the next line.
                    self.lines.next();
                }
                Err(_) => {
                    // Some problems reading the file. For example UTF-8 encoding issues or the file
                    // getting deleted.
                    return Some(Err(self.unwrap_next_line_err()));
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
    fn create_test(&mut self) -> Result<Test, crate::Error> {
        let (line_nr, read_result) = self.lines.next().expect("Already tested in Self::next()");

        let input_line = read_result.expect("Already tested in Self::next() but a bit later");
        let input_range = "-- Test: ".len()..input_line.len();
        let mut content = vec![input_line];

        while let Some((_, next_line)) = self.lines.peek() {
            if next_line.is_err() {
                // some problem reading the file
                return Err(self.lines.next().unwrap().1.err().unwrap().into());
            }

            let next_line = next_line.as_ref().unwrap();
            if next_line.trim() == "" {
                // Empty line => end of test
                break;
            }

            content.push(
                self.lines
                    .next()
                    .expect("Checked right above")
                    .1 // we don't care about the line number
                    .expect("Checked right above but lower"),
            );
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

    /// Gets the next error. Panics if the next line is not an error.
    ///
    /// Given that we peek to see the next line, and that next line can be an IO error, we end up
    /// writing this snake of a one-liner in multiple places.
    fn unwrap_next_line_err(&mut self) -> crate::Error {
        self.lines.next().unwrap().1.err().unwrap().into()
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
