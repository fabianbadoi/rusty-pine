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
    fn parse(input: &str) -> Result<PineNode, PineParseError>;
}

struct PineParser;

impl PineParserTrait for PineParser {
    fn parse(input: &str) -> Result<PineNode, PineParseError> {
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
        let pine_node = PineParser::parse("from: users | select: id, name | where: id = 3 x = 4").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[2].inner.get_name());

        if let Operation::From(ref table_name) = pine_node.inner.operations[0].inner {
            assert_eq!("users", table_name.inner);
        }
    }
}
