use super::pest_tree_translation::{IntermediateFormParser, PineParser};
use super::pine_to_query::{PineTranslator, QueryBuilder};
use super::Result;

pub trait QueryParser<T> {
    fn parse(self, input: T) -> Result;
}

pub type Parser = PestPineParser<PineParser, PineTranslator>;

pub struct PestPineParser<A, B> {
    pest_parser: A,
    query_builder: B,
}

impl<'a, T> QueryParser<T> for &PestPineParser<PineParser, PineTranslator>
where
    T: Into<&'a str>,
{
    fn parse(self, input: T) -> Result {
        let pine = self.pest_parser.parse(input.into())?;

        self.query_builder.build(&pine)
    }
}
