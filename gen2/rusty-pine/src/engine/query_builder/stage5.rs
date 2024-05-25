use crate::analyze::Server;
use crate::engine::query_builder::{
    Computation, ExplicitJoin, FunctionCall, Limit, Query, SelectedColumn, Source, Sourced, Table,
    ToSource,
};
use crate::engine::syntax::{
    OptionalInput, Stage3ExplicitJoin, Stage4ColumnInput, Stage4ComputationInput,
    Stage4FunctionCall, Stage4LimitInput, Stage4Rep, TableInput,
};
use crate::engine::QueryBuildError;

pub struct Stage5Builder<'a> {
    input: Stage4Rep<'a>,
    from: Sourced<Table>,
    server: &'a Server,
}

// TODO this isn't really done yet.
// We're missing some stages before this where we do joins and automatically adding sum/count
// for groups.
// This will have to be renamed in the future
impl<'a> Stage5Builder<'a> {
    pub fn new(input: Stage4Rep<'a>, server: &'a Server) -> Self {
        let from = input.from.into();

        Stage5Builder {
            input,
            from,
            server,
        }
    }

    pub fn try_build(mut self) -> Result<Query, QueryBuildError> {
        let select = self.process_selects();
        let joins = self.process_joins();

        Ok(Query {
            input: self.input.input.to_owned(),
            from: self.from,
            joins,
            select,
            limit: (&self.input.limit).into(),
        })
    }

    fn process_selects(&mut self) -> Vec<Sourced<Computation>> {
        let simplify_columns_and_tables: bool = self.is_single_table_query();

        self.input
            .selected_columns
            .iter()
            .map(|computation| {
                if simplify_columns_and_tables {
                    Computation::without_table_name(computation)
                } else {
                    computation.into()
                }
            })
            .collect()
    }

    fn process_joins(&mut self) -> Vec<Sourced<ExplicitJoin>> {
        self.input
            .joins
            .iter()
            .map(|j| {
                // if let Stage4Join::Explicit(j) = j {
                self.from = j.target_table.into();

                // We always set the "from" table to the last join, and switch the join direction
                // so it still looks good.
                j.clone().switch().into()
                // } else {
                //     todo!()
                // }
            })
            .collect()
    }

    fn is_single_table_query(&self) -> bool {
        self.input.joins.is_empty()
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

impl From<&Stage4ComputationInput<'_>> for Sourced<Computation> {
    fn from(value: &Stage4ComputationInput) -> Self {
        match value {
            Stage4ComputationInput::Column(column) => column.into(),
            Stage4ComputationInput::FunctionCall(fn_call) => fn_call.into(),
        }
    }
}

impl From<&Stage4ColumnInput<'_>> for Sourced<Computation> {
    fn from(value: &Stage4ColumnInput<'_>) -> Self {
        Sourced {
            it: Computation::SelectedColumn(SelectedColumn {
                table: Some(value.table.into()), // TODO hide table if not needed
                column: value.column.to_sourced(),
            }),
            source: Source::Input(value.position),
        }
    }
}

impl From<&Stage4FunctionCall<'_>> for Sourced<Computation> {
    fn from(value: &Stage4FunctionCall<'_>) -> Self {
        Sourced {
            it: Computation::FunctionCall(FunctionCall {
                fn_name: value.fn_name.to_sourced(),
                params: value
                    .params
                    .iter()
                    .map(<&Stage4ComputationInput>::into)
                    .collect(),
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

impl From<Stage3ExplicitJoin<'_>> for Sourced<ExplicitJoin> {
    fn from(value: Stage3ExplicitJoin<'_>) -> Self {
        Sourced {
            it: ExplicitJoin {
                join_type: Sourced {
                    it: value.join_type,
                    source: Source::Implicit,
                },
                target_table: value.target_table.into(),
                source_arg: (&value.source_arg).into(),
                target_arg: (&value.target_arg).into(),
            },
            source: Source::Input(value.position),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::analyze::{Server, ServerParams};
    use crate::engine::query_builder::stage5::Stage5Builder;
    use crate::engine::syntax::parse_to_stage4;

    #[test]
    fn test_try_from_simple() {
        let server = Server {
            params: ServerParams {
                hostname: "".to_string(),
                port: 0,
                user: "".to_string(),
                default_database: "".into(),
            },
            databases: Default::default(),
        };
        let builder = Stage5Builder::new(parse_to_stage4("table | s: id").unwrap(), &server);

        let result = builder.try_build();

        assert!(result.is_ok());

        let query = result.unwrap();

        assert_eq!(query.from.it.name.it.0, "table");
    }
}
