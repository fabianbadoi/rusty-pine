use crate::engine::query_builder::{
    ColumnName, Query, Select, SelectedColumn, Source, Sourced, Table, ToSource,
};
use crate::engine::syntax::{OptionalInput, Stage4ColumnInput, Stage4Rep, TableInput};

#[derive(Debug)]
pub enum Stage5Error {}
pub struct Stage5Builder {}

impl Stage5Builder {
    pub fn try_build(&self, input: Stage4Rep) -> Result<Query, Stage5Error> {
        let from = input.from.into();

        Ok(Query {
            from,
            input: input.input.to_owned(),
            select: input
                .selected_columns
                .into_iter()
                .map(|s| s.into())
                .collect(),
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

impl From<Stage4ColumnInput<'_>> for Sourced<Select> {
    fn from(value: Stage4ColumnInput<'_>) -> Self {
        Sourced {
            it: Select::SelectedColumn(SelectedColumn {
                table: Some(value.table.into()), // TODO hide table if not needed
                column: value.column.to_sourced(),
            }),
            source: Source::Input(value.position),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::engine::query_builder::stage5::Stage5Builder;
    use crate::engine::syntax::parse_to_stage4;

    #[test]
    fn test_try_from_simple() {
        let builder = Stage5Builder {};
        let result = builder.try_build(parse_to_stage4("table | s: id").unwrap());

        assert!(result.is_ok());

        let query = result.unwrap();

        assert_eq!(query.from.it.name.it.0, "table");
    }
}
