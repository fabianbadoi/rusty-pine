use super::pest_tree_translation::{IntermediateFormParser, PineParser};
use super::pine_to_query::{PineTranslator, QueryBuilder};
use super::Result;

pub trait QueryParser<I> {
    fn parse(self, input: I) -> Result;
}

pub type Parser = ComposedParser<PineParser, PineTranslator>;

pub struct ComposedParser<P, B> {
    pest_parser: P,
    query_builder: B,
}

impl<'a, 'b, I, P, T> QueryParser<I> for &'a ComposedParser<P, T>
where
    &'a P: IntermediateFormParser,
    I: Into<&'b str>,
    &'a T: QueryBuilder,
{
    fn parse(self, input: I) -> Result {
        let pine = self.pest_parser.parse(input.into())?;
        let query = self.query_builder.build(&pine);

        query
    }
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            pest_parser: PineParser {},
            query_builder: PineTranslator {},
        }
    }
}
