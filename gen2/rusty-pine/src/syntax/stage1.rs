//! The stage 1 representation is just the raw output from Pest
use pest::iterators::Pairs;
use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Stage1Error {
    #[error("Invalid syntax, failed to parse:\n{0}")]
    InvalidSyntax(#[from] pest::error::Error<Rule>),
}
#[derive(Parser)]
#[grammar = "syntax/pine.pest"]
struct Stage1Parser;

pub fn parse_stage1(input: &str) -> Result<Stage1Rep<'_>, crate::Error> {
    let pest = Stage1Parser::parse(Rule::root, input)?;

    Ok(Stage1Rep { input, pest })
}

#[derive(Debug)]
pub struct Stage1Rep<'a> {
    pub input: &'a str,
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
            r#"Invalid syntax, failed to parse:
 --> 1:6
  |
1 | test 012-test
  |      ^---
  |
  = expected"#,
            &format!("{}", error)[0..94]
        );
    }
}
