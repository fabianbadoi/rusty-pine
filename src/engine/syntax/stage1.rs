//! The stage 1 representation is just the raw output from Pest
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

/// Parsing error
///
/// Our syntax is constructed so once it's parsed, it will *always* be valid.
/// Stage 1 can fail, but in terms of parsing to a Pine, nothing else can.
/// This means this stage is the only one that can have an error.
#[derive(Error, Debug)]
pub enum Stage1Error {
    // Pest has quite nice error output, which rely on. See the test::test_error test for an example.
    /// Invalid syntax error
    #[error("Invalid syntax, failed to parse:\n{0}")]
    InvalidSyntax(#[from] pest::error::Error<Rule>),
}

/// Pest parser
///
/// Pest will autogenerate all of the code needed, and will also give an enum called "Rule" that
/// will have all the rule names from the pine.pest file.
#[derive(Parser)]
#[grammar = "engine/syntax/pine.pest"]
struct Stage1Parser;

pub fn parse_stage1(input: &str) -> Result<Stage1Rep<'_>, crate::error::Error> {
    let pest = Stage1Parser::parse(
        // we've constructed our grammar to always start with a Rule:root node.
        Rule::root,
        input,
    )?; // "?" automatically transforms Pest errors into Stage1Errors into crate::error:Errors

    Ok(Stage1Rep { pest })
}

/// Pest pair holder
///
/// Up until a later stage, all of the data can be directly found in the input string.
/// This means we can have all of our *stuff* reference substrings of the input directly. This means
/// we will not eat extra memory copying parts of the input string around.
///
/// All of the <'a> parameters get annoying VERY QUICKLY.
#[derive(Debug)]
pub struct Stage1Rep<'a> {
    pub pest: Pairs<'a, Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_base() {
        let result = Stage1Parser::parse(Rule::root, "table");
        assert!(result.is_ok());

        let root = result.unwrap().peek();
        assert!(root.is_some());
    }

    #[test]
    fn test_error() {
        let error = parse_stage1("test 012-test").unwrap_err();

        assert_eq!(
            &format!("{}", error),
            "\u{1b}[1mInvalid syntax, failed to parse\u{1b}[0m\ntest 012-test\n\
            \u{1b}[1;31m        ^\u{1b}[0m \u{1b}[1;31mexpected EOI, show_neighbors_pine, condition, or comparison_symbol\u{1b}[0m\n\
            "
        );
    }
}
