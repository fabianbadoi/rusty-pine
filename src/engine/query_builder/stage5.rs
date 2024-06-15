use crate::analyze::{Column, Server};
use crate::engine::query_builder::sql_introspection::Introspective;
use crate::engine::query_builder::{
    BinaryCondition, Computation, Condition, DatabaseName, ExplicitJoin, FunctionCall,
    LiteralValue, Query, Selectable, SelectedColumn, Table, UnaryCondition,
};
use crate::engine::syntax::{
    OptionalInput, Stage4BinaryCondition, Stage4ColumnInput, Stage4ComputationInput,
    Stage4Condition, Stage4FunctionCall, Stage4Join, Stage4LiteralValue, Stage4Query,
    Stage4Selectable, Stage4UnaryCondition, TableInput,
};
use crate::engine::{
    JoinConditions, LimitHolder, LiteralValueHolder, OrderHolder, QueryBuildError,
    SelectableHolder, Sourced,
};
use std::fmt::Debug;

pub struct Stage5Builder<'a> {
    input: Stage4Query<'a>,
    from: Sourced<TableInput<'a>>,
    server: &'a Server,
}

impl<'a> Stage5Builder<'a> {
    pub fn new(input: Stage4Query<'a>, server: &'a Server) -> Self {
        let from = input.from;

        Stage5Builder {
            input,
            from,
            server,
        }
    }

    pub fn try_build(self) -> Result<Query, QueryBuildError> {
        let select = self.process_selects();
        let select = self.process_unselects(select)?;
        let filters = self.process_filters();
        let joins = self.process_joins()?;
        let orders = self.process_orders();
        let group_by = self.process_group_by();

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
            filters,
            orders,
            group_by,
            limit: self.input.limit.map(|limit| limit.into()),
        })
    }

    fn process_selects(&self) -> Vec<Sourced<Selectable>> {
        self.input
            .selected_columns
            .iter()
            .map(Clone::clone)
            .map(|selectable| selectable.map(|selectable| self.process_selectable(selectable)))
            .collect()
    }

    /// Processes "unselects".
    ///
    /// Take the following example:
    /// ```text
    /// people | u: name
    /// ```
    ///
    /// The first sub-pine would lead to a "SELECT *". In this case we need to make sure the
    /// unselection works.
    /// In this case, I've opted to expand the wildcard "*" into all the columns, and then remove the
    /// columns that are unselected.
    ///
    /// We will know what columns the table has base on the information we have from analyzing the
    /// server beforehand.
    fn process_unselects(
        &self,
        selects: Vec<Sourced<Selectable>>,
    ) -> Result<Vec<Sourced<Selectable>>, QueryBuildError> {
        self.input
            .unselected_columns
            .iter()
            // Naively, we'd just use a filter/map or for loop. However, we need to expand the list
            // of selects whenever we come across a wildcard select ("*") that matches.
            // In the case of filter/map, this is not possible. In the case of a loop, it would mean
            // adding to the source iter while iterating, which is not possible.
            // Fold is actually the natural mechanism to achieve what we want.
            .try_fold(selects, |selects, unselect| {
                // This "*" to "column1, column2, ..." if the unselect matches.
                let expanded_selects: Vec<_> = self.expand_unselect(&unselect.it, selects)?;

                // This removes unselected columns
                let filtered_selects = expanded_selects
                    .into_iter()
                    .filter(|select| unselect.it != select.it)
                    .collect::<Vec<_>>();

                Ok(filtered_selects)
            })
    }

    fn process_filters(&self) -> Vec<Sourced<Condition>> {
        self.process_conditions(&self.input.filters)
    }

    fn process_orders(&self) -> Vec<Sourced<OrderHolder<Selectable>>> {
        self.input
            .orders
            .iter()
            .map(|order| {
                order.map_ref(|order| OrderHolder {
                    selectable: order
                        .selectable
                        .map_ref(|comp| self.process_selectable(comp.clone())),
                    direction: order.direction,
                })
            })
            .collect()
    }

    fn process_group_by(&self) -> Vec<Sourced<Selectable>> {
        self.input
            .group_by
            .iter()
            .map(Clone::clone)
            .map(|selectable| selectable.map(|selectable| self.process_selectable(selectable)))
            .collect()
    }

    fn process_joins(&self) -> Result<Vec<Sourced<ExplicitJoin>>, QueryBuildError> {
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
                .join_conditions(join.source_table.it, join.target_table.it)?,
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

    fn process_condition(&self, condition: Stage4Condition) -> Condition {
        match condition {
            Stage4Condition::Unary(unary) => {
                Condition::Unary(unary.map(|unary| self.process_unary_condition(unary)))
            }
            Stage4Condition::Binary(binary) => {
                Condition::Binary(binary.map(|condition| self.process_binary_condition(condition)))
            }
        }
    }

    fn process_binary_condition(&self, condition: Stage4BinaryCondition) -> BinaryCondition {
        let left = condition.left.map(|left| self.process_computation(left));
        let comparison = condition.comparison;
        let right = condition.right.map(|right| self.process_computation(right));

        BinaryCondition {
            left,
            comparison,
            right,
        }
    }

    fn process_unary_condition(&self, condition: Stage4UnaryCondition) -> UnaryCondition {
        match condition {
            Stage4UnaryCondition::IsNull(computation) => UnaryCondition::IsNull(
                computation.map(|computation| self.process_computation(computation)),
            ),
            Stage4UnaryCondition::IsNotNull(computation) => UnaryCondition::IsNotNull(
                computation.map(|computation| self.process_computation(computation)),
            ),
        }
    }

    fn expand_unselect(
        &self,
        unselect: &Stage4ColumnInput,
        selects: Vec<Sourced<Selectable>>,
    ) -> Result<Vec<Sourced<Selectable>>, QueryBuildError> {
        selects
            .into_iter()
            .try_fold(Vec::new(), |mut new_selects, selectable| {
                if let Some(select) = selectable.it.as_selected_column() {
                    if unselect_matches_wildcard(unselect, select) {
                        let mut wildcard_expansion = self.select_table_columns(unselect.table)?;

                        new_selects.append(&mut wildcard_expansion);
                    }
                } else {
                    new_selects.push(selectable);
                }

                Ok(new_selects)
            })
    }

    fn select_table_columns(
        &self,
        table: Sourced<TableInput>,
    ) -> Result<Vec<Sourced<Selectable>>, QueryBuildError> {
        let selected_columns = self
            .server
            .columns(table.it)?
            .iter()
            .map(|column| self.as_selectable(column, table))
            .collect();

        Ok(selected_columns)
    }

    fn as_selectable(&self, column: &Column, table: Sourced<TableInput>) -> Sourced<Selectable> {
        let table = if self.is_single_table_query() {
            None
        } else {
            Some(table.into())
        };
        let column = Sourced::from_introspection(column.name.clone().into());

        let computation = Sourced::from_introspection(SelectedColumn { table, column });

        Sourced::from_introspection(Selectable::Computation(Sourced::from_introspection(
            Computation::SelectedColumn(computation),
        )))
    }

    fn is_single_table_query(&self) -> bool {
        self.input.joins.is_empty()
    }
}

fn unselect_matches_wildcard(unselect: &Stage4ColumnInput, select: &SelectedColumn) -> bool {
    if select.column.it.0 != "*" {
        // hardcoded wildcard char, oh yeaaah!
        return false;
    }

    match &select.table {
        None => true, // implicit match
        Some(table) => table.it == unselect.table.it,
    }
}

impl PartialEq<Selectable> for Stage4ColumnInput<'_> {
    fn eq(&self, other: &Selectable) -> bool {
        match other.as_selected_column() {
            None => false,
            Some(column) => self == column,
        }
    }
}

impl PartialEq<SelectedColumn> for Stage4ColumnInput<'_> {
    fn eq(&self, other: &SelectedColumn) -> bool {
        let table_matches = match &other.table {
            None => true,
            Some(table) => table.it == self.table.it,
        };

        table_matches && self.column.it.name == other.column.it.0
    }
}

impl PartialEq<TableInput<'_>> for Table {
    fn eq(&self, other: &TableInput<'_>) -> bool {
        if self.name.it.0 != other.table.it.name {
            return false;
        }

        match (&self.db, other.database) {
            (None, OptionalInput::Implicit) => true,
            (Some(self_db), OptionalInput::Specified(other_db)) => self_db.it.0 == other_db.it.name,
            _ => false,
        }
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

impl<S> LimitHolder<LiteralValueHolder<S>>
where
    S: Clone + Debug,
{
    fn into<D>(self) -> LimitHolder<LiteralValueHolder<D>>
    where
        D: From<S> + Clone + Debug,
    {
        match self {
            LimitHolder::Implicit() => LimitHolder::Implicit(),
            LimitHolder::RowCount(count) => LimitHolder::RowCount(count.map(|count| count.into())),
            LimitHolder::Range { start, count } => LimitHolder::Range {
                start: start.map(|start| start.into()),
                count: count.map(|count| count.into()),
            },
        }
    }
}

impl<T> SelectableHolder<T, Computation>
where
    T: Clone,
{
    pub fn as_selected_column(&self) -> Option<&SelectedColumn> {
        let computation = if let SelectableHolder::Computation(comp) = self {
            &comp.it
        } else {
            return None;
        };

        match computation {
            Computation::SelectedColumn(column) => Some(&column.it),
            _ => None,
        }
    }
}

impl PartialEq<Option<Sourced<Table>>> for Sourced<TableInput<'_>> {
    fn eq(&self, other: &Option<Sourced<Table>>) -> bool {
        match other {
            None => true, // because it's implicit TODO this is garbage
            Some(table) => table.it == self.it,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::analyze::{Server, ServerParams};
    use crate::engine::query_builder::stage5::Stage5Builder;
    use crate::engine::syntax::{parse_to_stage4, Stage4Rep};

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

        let stage4 = parse_to_stage4("table | s: id").unwrap();

        let builder = match stage4 {
            Stage4Rep::Query(query) => Stage5Builder::new(query, &server),
            Stage4Rep::ShowNeighbors(_) => panic!("this must be a query"),
        };

        let result = builder.try_build();

        assert!(result.is_ok());

        let query = result.unwrap();

        assert_eq!(query.from.it.name.it.0, "table");
    }
}
