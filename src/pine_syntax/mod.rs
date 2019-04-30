mod ast;
mod pest;
mod pest_tree_translation;
mod pine_to_query;

use self::ast::*;
use ::pest::Parser;

use self::pest::Rule;
use ::pest::error::Error as PestError;
use std::convert::From;

#[derive(Debug)]
pub struct PineParseError(());
impl From<PestError<Rule>> for PineParseError {
    fn from(pest_error: PestError<Rule>) -> Self {
        panic!("{}", pest_error);
        // TODO this needs to be better
        PineParseError(())
    }
}

trait IntermediateFormParser {
    fn parse(self, input: &str) -> Result<PineNode, PineParseError>;
}

struct PineParser;

impl IntermediateFormParser for &PineParser {
    fn parse(self, input: &str) -> Result<PineNode, PineParseError> {
        let ast = pest::PinePestParser::parse(pest::Rule::pine, input)?
            .next()
            .expect("Pest should have failed to parse this input");

        let pine = pest_tree_translation::translate(ast);

        Ok(pine)
    }
}

#[cfg(test)]
mod tests {
    use super::{IntermediateFormParser, Operation, PineParser};

    #[test]
    fn parsing_simple_form_statement() {
        let parser = PineParser {};
        let pine_node = parser
            .parse("from: users | select: id, name | where: id = 3 x = 4")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[2].inner.get_name());

        if let Operation::From(ref table_name) = pine_node.inner.operations[0].inner {
            assert_eq!("users", table_name.inner);
        }
    }
}
