use crate::error::ParseError;
use crate::pine_syntax::{PestPineParser, PineParser};
use crate::query::{NaiveBuilder, Query, QueryBuilder};
use crate::sql::{Renderer, StringRenderer};

type ParseResult<O> = Result<O, ParseError>;

pub trait Parser<I, O> {
    fn parse(self, input: I) -> ParseResult<O>;
}

pub type NaiveParser = GenericParser<PestPineParser, NaiveBuilder, StringRenderer>;

pub struct GenericParser<Parser, Builder, Renderer> {
    parser: Parser,
    builder: Builder,
    renderer: Renderer,
}

impl<'a, 'b, I, O, P, B, R> Parser<I, O> for &'a GenericParser<P, B, R>
where
    &'a P: PineParser,
    &'a B: QueryBuilder,
    &'a R: Renderer<Query, O>,
    I: Into<&'b str>,
{
    fn parse(self, input: I) -> ParseResult<O> {
        let pine = self.parser.parse(input.into())?;
        let query = self.builder.build(&pine)?;
        let output = self.renderer.render(&query);

        Ok(output)
    }
}

impl GenericParser<PestPineParser, NaiveBuilder, StringRenderer> {
    pub fn default() -> GenericParser<PestPineParser, NaiveBuilder, StringRenderer> {
        GenericParser {
            parser: PestPineParser {},
            builder: NaiveBuilder {},
            renderer: StringRenderer {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let parser = NaiveParser::default();
        let query = parser.parse("f: users | s: name | w: id = 3").unwrap();

        assert_eq!("SELECT name\nFROM users\nWHERE id = \"3\"", query);
    }
}
