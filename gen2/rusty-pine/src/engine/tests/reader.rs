use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::iter::{Enumerate, Peekable};
use std::ops::Range;
use std::path::PathBuf;

pub struct SqlTestFileReader {
    pub file_path: PathBuf,
    // Peekable<> because we scan for the next line
    // Enumerate<> because we want line numbers
    // Lines<> because we scan line by line
    pub lines: Peekable<Enumerate<Lines<BufReader<File>>>>,
}

impl Iterator for SqlTestFileReader {
    type Item = Result<Test, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((_, next_line)) = self.lines.peek() {
            if next_line.is_err() {
                // some problem reading the file
                return Some(Err(self.lines.next().unwrap().1.err().unwrap()));
            }

            let next_line = next_line.as_ref().unwrap();
            if !next_line.starts_with("-- Test: ") {
                // Any line not in a test "block" is ignored
                self.lines.next();
                continue;
            }

            return Some(self.create_test());
        }

        None
    }
}

impl SqlTestFileReader {
    pub fn new(file_path: PathBuf) -> Result<Self, std::io::Error> {
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);

        let lines = reader.lines().enumerate().peekable();

        Ok(SqlTestFileReader { lines, file_path })
    }

    fn create_test(&mut self) -> Result<Test, std::io::Error> {
        let (line_nr, read_result) = self.lines.next().expect("Already tested in Self::next()");

        let input_line = read_result.expect("Already tested in Self::next() but a bit later");
        let input_range = "-- Test: ".len()..input_line.len();
        let mut content = vec![input_line];

        while let Some((_, next_line)) = self.lines.peek() {
            if next_line.is_err() {
                // some problem reading the file
                return Err(self.lines.next().unwrap().1.err().unwrap());
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
}

/// One of our integration tests.
///
/// Keeping all of the info related to where we found the test helps us a lot when printing test
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
