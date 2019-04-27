mod ast;
mod pest;
mod pest_tree_translation;

use ::pest::Parser;
pub use self::ast::*;

use std::convert::From;
use ::pest::error::Error as PestError;
use self::pest::Rule;

#[derive(Debug)]
pub struct PineParseError(());
impl From<PestError<Rule>> for PineParseError {
    fn from(pest_error: PestError<Rule>) -> Self {
        panic!("{}", pest_error);
        // TODO this needs to be better
        PineParseError(())
    }
}

pub trait PineParserTrait {
    fn parse(input: &str) -> Result<Pine, PineParseError>;
}

struct PineParser;

impl PineParserTrait for PineParser {
    fn parse(input: &str) -> Result<Pine, PineParseError> {
        let ast = pest::PinePestParser::parse(pest::Rule::pine, input)?.next()
            .expect("Pest should have failed to parse this input");

        let pine = pest_tree_translation::translate(ast);

        Ok(pine)
    }
}

#[cfg(test)]
mod tests {
    use super::{PineParser, PineParserTrait, Operation};

    #[test]
    fn parsing_simple_form_statement() {
        let pine = PineParser::parse("from: users | select: id, name | where: id = 3 x = 4").unwrap();

        assert_eq!("from", pine.item[0].item.get_name());
        assert_eq!("select", pine.item[1].item.get_name());
        assert_eq!("filter", pine.item[2].item.get_name());

        if let Operation::From(ref table_name) = pine.item[0].item {
            assert_eq!("users", table_name.item);
        }
    }
}
