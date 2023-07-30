use crate::engine::query_builder::{
    ColumnName, Limit, Query, Select, SelectedColumn, Source, Sourced, Table, ToSource,
};
use crate::engine::syntax::{
    OptionalInput, Stage4ColumnInput, Stage4LimitInput, Stage4Rep, TableInput,
};

#[derive(Debug)]
pub enum Stage5Error {}
pub struct Stage5Builder {}

// TODO this isn't really done yet.
// We're missing some stages before this where we do joins and automatically adding sum/count
// for groups.
// This will have to be renamed in the future
impl Stage5Builder {
    pub fn try_build(&self, input: Stage4Rep) -> Result<Query, Stage5Error> {
        let from = input.from.into();
        let simplify_columns_and_tables: bool = self.is_single_table_query(&input);

        Ok(Query {
            from,
            input: input.input.to_owned(),
            select: input
                .selected_columns
                .iter()
                .map(|column| {
                    if simplify_columns_and_tables {
                        Select::from_singly_selected(column)
                    } else {
                        column.into()
                    }
                })
                .collect(),
            limit: (&input.limit).into(),
        })
    }
    fn is_single_table_query(&self, p0: &Stage4Rep) -> bool {
        // can only be true now, will change once we can add joins
        true
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

impl From<&Stage4ColumnInput<'_>> for Sourced<Select> {
    fn from(value: &Stage4ColumnInput<'_>) -> Self {
        Sourced {
            it: Select::SelectedColumn(SelectedColumn {
                table: Some(value.table.into()), // TODO hide table if not needed
                column: value.column.to_sourced(),
            }),
            source: Source::Input(value.position),
        }
    }
}

impl From<&Stage4LimitInput> for Sourced<Limit> {
    fn from(value: &Stage4LimitInput) -> Self {
        match value {
            Stage4LimitInput::Implicit() => Sourced {
                it: Limit::Implicit(),
                source: Source::Implicit,
            },
            Stage4LimitInput::RowCountLimit(rows, position) => Sourced {
                it: Limit::RowCountLimit(*rows),
                source: position.into(),
            },
            Stage4LimitInput::RangeLimit(range, position) => Sourced {
                it: Limit::RangeLimit(range.clone()),
                source: position.into(),
            },
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
