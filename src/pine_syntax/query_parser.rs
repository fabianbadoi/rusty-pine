use super::pest_tree_translation::{IntermediateFormParser, PineParser};
use super::pine_to_query::{PineTranslator, QueryBuilder};
use super::Result;

pub trait QueryParser {
    fn parse(self, input: &str) -> Result;
}

pub type Parser = PestPineParser<PineParser, PineTranslator>;

pub struct PestPineParser<A, B> {
    pest_parser: A,
    query_builder: B,
}

impl QueryParser for &PestPineParser<PineParser, PineTranslator> {
    fn parse(self, input: &str) -> Result {
        let pine = self.pest_parser.parse(input)?;

        self.query_builder.build(&pine)
    }
}
