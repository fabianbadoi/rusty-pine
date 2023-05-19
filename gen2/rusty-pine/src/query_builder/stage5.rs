use crate::query_builder::{DatabaseName, Query, Source, Sourced, Table, TableName, ToSource};
use crate::syntax::{OptionalInput, SqlIdentifierInput, Stage4Rep, TableInput};

#[derive(Debug)]
pub enum Stage5Error {}
pub struct Stage5Builder {}

impl Stage5Builder {
    pub fn try_build(&self, input: Stage4Rep) -> Result<Query, Stage5Error> {
        let from = input.from.into();

        Ok(Query {
            from,
            input: input.input.to_owned(),
        })
    }
}

impl From<TableInput<'_>> for Sourced<Table> {
    fn from(value: TableInput<'_>) -> Self {
        Sourced {
            it: Table {
                db: match value.database {
                    OptionalInput::Implicit => None,
                    OptionalInput::Specified(value) => Some(value.to_sourced()),
                },
                name: value.table.to_sourced(),
            },
            source: Source::Input(value.position),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::query_builder::stage5::Stage5Builder;
    use crate::syntax::parse_to_stage4;

    #[test]
    fn test_try_from_simple() {
        let builder = Stage5Builder {};
        let result = builder.try_build(parse_to_stage4("table | s: id").unwrap());

        assert!(result.is_ok());

        let query = result.unwrap();

        assert_eq!(query.from.it.name.it.0, "table");
    }
}
