use crate::pine_syntax::{Parser, QueryParser};
use crate::sql::{Renderer, StringRenderer};
use crate::pine_syntax::PineError;
use crate::sql::Query;
use crate::ParseError;

type TranslateResult<O> = Result<O, ParseError>;

pub trait Translator<I, O> {
    fn translate(self, input: I) -> TranslateResult<O>;
}

pub type PineTranslator = ComposedTranslator<Parser, StringRenderer>;

pub struct ComposedTranslator<P, R> {
    query_parser: P,
    query_renderer: R,
}

impl<'a, I, O, P, R> Translator<I, O> for &'a ComposedTranslator<P, R>
where
    &'a P: QueryParser<I>,
    &'a R: Renderer<O, Query>
{
    fn translate(self, input: I) -> TranslateResult<O> {
        let query = self.query_parser.parse(input)?;
        let output = self.query_renderer.render(&query);
        
        Ok(output)
    }
}

impl PineTranslator {
    pub fn new() -> PineTranslator {
        PineTranslator {
            query_parser: Parser::new(),
            query_renderer: StringRenderer {}
        }
    }
}
