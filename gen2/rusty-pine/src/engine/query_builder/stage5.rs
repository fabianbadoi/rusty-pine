use crate::analyze::Server;
use crate::engine::query_builder::sql_introspection::Introspective;
use crate::engine::query_builder::{
    Computation, Condition, DatabaseName, ExplicitJoin, FunctionCall, LiteralValue, Query,
    Selectable, SelectedColumn, Table,
};
use crate::engine::syntax::{
    OptionalInput, Stage4ColumnInput, Stage4ComputationInput, Stage4Condition, Stage4FunctionCall,
    Stage4Join, Stage4LiteralValue, Stage4Rep, Stage4Selectable, TableInput,
};
use crate::engine::{JoinConditions, LiteralValueHolder, QueryBuildError, Sourced};

pub struct Stage5Builder<'a> {
    input: Stage4Rep<'a>,
    from: Sourced<TableInput<'a>>,
    server: &'a Server,
}

// TODO this isn't really done yet.
// We're missing some stages before this where we do joins and automatically adding sum/count
// for groups.
// This will have to be renamed in the future
impl<'a> Stage5Builder<'a> {
    pub fn new(input: Stage4Rep<'a>, server: &'a Server) -> Self {
        let from = input.from;

        Stage5Builder {
            input,
            from,
            server,
        }
    }

    pub fn try_build(mut self) -> Result<Query, QueryBuildError> {
        let select = self.process_selects();
        let joins = self.process_joins()?;

        // This makes sure we select FROM the table from the last pine.
        let from = match self.input.joins.last() {
            None => self.from.into(),
            Some(last_join) => last_join.it.target_table.into(),
        };

        Ok(Query {
            input: self.input.input.to_owned(),
            from,
            joins,
            select,
            limit: self.input.limit.clone(),
        })
    }

    fn process_selects(&mut self) -> Vec<Sourced<Selectable>> {
        self.input
            .selected_columns
            .iter()
            .map(Clone::clone)
            .map(|computation| computation.map(|selectable| self.process_selectable(selectable)))
            .collect()
    }

    fn process_joins(&mut self) -> Result<Vec<Sourced<ExplicitJoin>>, QueryBuildError> {
        let joins: Result<Vec<_>, QueryBuildError> = self
            .input
            .joins
            .iter()
            .map(|j| j.map_ref(|j| self.process_join(j)))
            .map(Sourced::unwrap_result)
            .collect();

        joins
    }

    fn process_join(&self, join: &Stage4Join) -> Result<ExplicitJoin, QueryBuildError> {
        let conditions = match &join.conditions {
            JoinConditions::Auto => self
                .server
                .join_conditions(self.from.it, join.target_table.it)?,
            JoinConditions::Explicit(conditions) => self.process_conditions(conditions),
        };

        Ok(ExplicitJoin {
            join_type: join.join_type,
            // We join to the SOURCE table because we always swap the tables of join.
            // `people | preference` should result in:
            // SELECT FROM preference JOIN people
            // It should not result in:
            // SELECT FROM people JOIN preferences
            //
            // This is just a design decision I made.
            target_table: join.source_table.into(),
            conditions,
        })
    }

    fn process_selectable(&self, selectable: Stage4Selectable) -> Selectable {
        match selectable {
            Stage4Selectable::Condition(condition) => {
                Selectable::Condition(condition.map(|condition| self.process_condition(condition)))
            }
            Stage4Selectable::Computation(computation) => Selectable::Computation(
                computation.map(|computation| self.process_computation(computation)),
            ),
        }
    }

    fn process_computation(&self, computation: Stage4ComputationInput) -> Computation {
        let simplify_columns_and_tables: bool = self.is_single_table_query();

        if simplify_columns_and_tables {
            Computation::without_table_name(computation)
        } else {
            match computation {
                Stage4ComputationInput::Column(column) => {
                    Computation::SelectedColumn(column.into())
                }
                Stage4ComputationInput::FunctionCall(fn_call) => {
                    Computation::FunctionCall(fn_call.into())
                }
                Stage4ComputationInput::Value(value) => Computation::Value(value.into()),
            }
        }
    }
    fn process_condition(&self, computation: Stage4Condition) -> Condition {
        let left = computation.left.map(|left| self.process_computation(left));
        let comparison = computation.comparison;
        let right = computation
            .right
            .map(|right| self.process_computation(right));

        Condition {
            left,
            comparison,
            right,
        }
    }

    fn process_conditions(
        &self,
        conditions: &[Sourced<Stage4Condition>],
    ) -> Vec<Sourced<Condition>> {
        conditions
            .iter()
            .map(Clone::clone)
            .map(|c| c.map(|c| self.process_condition(c)))
            .collect()
    }

    fn is_single_table_query(&self) -> bool {
        self.input.joins.is_empty()
    }
}

impl From<TableInput<'_>> for Table {
    fn from(value: TableInput<'_>) -> Self {
        Table {
            db: match value.database {
                OptionalInput::Implicit => None,
                OptionalInput::Specified(value) => Some(value.into()),
            },
            name: value.table.into(),
        }
    }
}

impl From<Stage4ComputationInput<'_>> for Computation {
    fn from(value: Stage4ComputationInput) -> Self {
        match value {
            Stage4ComputationInput::Column(column) => Computation::SelectedColumn(column.into()),
            Stage4ComputationInput::FunctionCall(fn_call) => {
                Computation::FunctionCall(fn_call.into())
            }
            Stage4ComputationInput::Value(value) => Computation::Value(value.into()),
        }
    }
}

impl From<Stage4ColumnInput<'_>> for SelectedColumn {
    fn from(value: Stage4ColumnInput<'_>) -> Self {
        SelectedColumn {
            table: Some(value.table.into()),
            column: value.column.into(),
        }
    }
}

impl From<Stage4FunctionCall<'_>> for FunctionCall {
    fn from(value: Stage4FunctionCall) -> Self {
        FunctionCall {
            fn_name: value.fn_name.into(),
            params: value.params.into_iter().map(|param| param.into()).collect(),
        }
    }
}

impl<T> From<T> for DatabaseName
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        DatabaseName(value.as_ref().to_string())
    }
}

impl From<Stage4LiteralValue<'_>> for LiteralValue {
    fn from(value: Stage4LiteralValue<'_>) -> Self {
        match value {
            Stage4LiteralValue::Number(number) => LiteralValueHolder::Number(number.into()),
            Stage4LiteralValue::String(string) => LiteralValueHolder::String(string.into()),
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
