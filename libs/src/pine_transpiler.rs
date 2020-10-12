use crate::analyzer::offline as analyzer_offline;
use crate::error::PineError;
use crate::pine_syntax::{PestPineParser, PineParser};
use crate::query::{NaiveBuilder, QueryBuilder, Renderable};
use crate::sql::{Renderer, SmartRenderer};
use crate::Config;

type TranspileResult<O> = Result<O, PineError>;

pub trait Transpiler<I, O> {
    fn transpile(self, input: I) -> TranspileResult<O>;
}

pub type MySqlTranspiler = GenericTranspiler<PestPineParser, NaiveBuilder, SmartRenderer>;

#[derive(Debug)]
pub struct GenericTranspiler<Parser, Builder, Renderer> {
    parser: Parser,
    builder: Builder,
    renderer: Renderer,
}

impl<'a, 'b, I, O, P, B, R> Transpiler<I, O> for &'a GenericTranspiler<P, B, R>
where
    // TODO all of these should be 'regular' traits
    &'a P: PineParser,
    &'a B: QueryBuilder,
    &'a R: Renderer<Renderable, O>,
    I: Into<&'b str>,
{
    fn transpile(self, input: I) -> TranspileResult<O> {
        let pine = self.parser.parse(input.into())?;
        let query = self.builder.build(&pine)?;

        self.renderer.render(&query)
    }
}

pub fn offline(config: &Config, db_name: &str) -> Result<MySqlTranspiler, PineError> {
    let analyezer = analyzer_offline(config).unwrap();
    let database = analyezer.analyze(db_name)?;

    Ok(GenericTranspiler {
        parser: PestPineParser {},
        builder: NaiveBuilder {},
        renderer: SmartRenderer::for_tables(database.tables),
    })
}

#[cfg(test)]
pub mod demo {
    use super::*;
    use crate::sql::structure::Table;

    pub fn transpiler_for(tables: Vec<Table>) -> MySqlTranspiler {
        GenericTranspiler {
            parser: PestPineParser {},
            builder: NaiveBuilder {},
            renderer: SmartRenderer::for_tables(tables),
        }
    }
}
