use super::ast::{
    ColumnNameNode, Condition as AstCondition, FilterNode, Operation, OperationNode, PineNode,
    Position, TableNameNode,
};
use super::{PineParseError, Result};
use crate::sql::{
    Condition as SqlCondition, Filter as SqlFilter, QualifiedColumnIdentifier, Query,
};
use std::result::Result as StdResult;

type InternalError = StdResult<(), PineParseError>;

pub trait QueryBuilder {
    fn build<'a>(&self, pine: &'a PineNode) -> Result;
}

pub struct PineTranslator;

#[derive(Default)]
struct SingleUseQueryBuilder {
    query: Query,
    current_table: Option<String>,
}

impl QueryBuilder for PineTranslator {
    fn build<'a>(&self, pine: &'a PineNode) -> Result {
        let builder = SingleUseQueryBuilder::new();

        builder.build(pine)
    }
}

impl<'a> SingleUseQueryBuilder {
    fn new() -> SingleUseQueryBuilder {
        Default::default()
    }

    fn build(mut self, pine: &'a PineNode) -> Result {
        for operation_node in pine {
            self.apply_operation(operation_node)?;
        }

        self.finalize(pine)?;

        Ok(self.query)
    }

    fn apply_operation(&mut self, operation_node: &'a OperationNode) -> InternalError {
        match operation_node.inner {
            Operation::From(ref table) => self.apply_from(table),
            Operation::Select(ref selections) => self.apply_selections(selections)?,
            Operation::Filter(ref filters) => self.apply_filters(filters)?,
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &'a TableNameNode) {
        self.reset_selection(&table.inner);
    }

    fn apply_selections(&mut self, selections: &'a [ColumnNameNode]) -> InternalError {
        if selections.is_empty() {
            return Ok(());
        }

        let table = self.require_table(selections[0].position)?;
        let mut selections: Vec<_> = selections
            .iter()
            .map(|name_node| name_node.inner.to_string())
            .map(|column| QualifiedColumnIdentifier {
                table: table.clone(),
                column,
            })
            .collect();

        self.query.selections.append(&mut selections);

        Ok(())
    }

    fn apply_filters(&mut self, filters: &'a [FilterNode]) -> StdResult<(), PineParseError> {
        if filters.is_empty() {
            return Ok(());
        }

        let table = self.require_table(filters[0].position)?;
        let mut filters: Vec<_> = filters
            .iter()
            .map(|filter_node| {
                let column = filter_node.inner.column.inner.to_string();
                let column = QualifiedColumnIdentifier {
                    table: table.clone(),
                    column,
                };
                let condition: SqlCondition = (&filter_node.inner.condition.inner).into();

                SqlFilter { column, condition }
            })
            .collect();

        self.query.filters.append(&mut filters);

        Ok(())
    }

    fn reset_selection(&mut self, table: &'a str) {
        self.current_table = Some(table.to_string());

        // existing selections are cleared to, maybe add a select+: operation that keeps previous selections
        self.query.selections.clear();
    }

    fn finalize(&mut self, pine: &'a PineNode) -> StdResult<(), PineParseError> {
        match self.current_table.clone() {
            Some(table) => {
                self.query.from = table;
                Ok(())
            }
            None => Err(PineParseError {
                message: "Missing a 'from:' statement".to_string(),
                position: pine.position,
            }),
        }
    }

    fn require_table(&self, pine_position: Position) -> StdResult<String, PineParseError> {
        match &self.current_table {
            Some(table) => Ok(table.clone()),
            None => Err(PineParseError {
                message: "Place a 'from:' statement in front fo this".to_string(),
                position: pine_position,
            }),
        }
    }
}

impl<'a> From<&'a AstCondition<'a>> for SqlCondition {
    fn from(other: &'a AstCondition) -> Self {
        match other {
            AstCondition::Equals(ref value) => SqlCondition::Equals(value.inner.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PineTranslator, QueryBuilder};
    use crate::pine_syntax::ast::*;
    use crate::sql::{Condition as SqlCondition, QualifiedColumnIdentifier};

    #[test]
    fn build_from_query() {
        let pine = from("users");

        let query_builder = PineTranslator {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!("users", query.from);
    }

    #[test]
    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = PineTranslator {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[1], ("users", "name"));
    }

    #[test]
    fn build_filter_query() {
        let pine = filter("id", Condition::Equals(make_node("3")), "users");

        let query_builder = PineTranslator {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(query.filters[0].column, ("users", "id"));
        assert_eq!(
            query.filters[0].condition,
            SqlCondition::Equals("3".to_string())
        );
    }

    fn filter(
        column: &'static str,
        condition: Condition<'static>,
        table: &'static str,
    ) -> PineNode<'static> {
        let mut pine = from(table);

        let condition = make_node(condition);
        let column = make_node(column);
        let filter = make_node(Filter { column, condition });

        append_operation(&mut pine, Operation::Filter(vec![filter]));

        pine
    }

    fn from(table: &'static str) -> PineNode {
        let mut pine = make_blank_pine();
        append_operation(&mut pine, Operation::From(make_node(table)));

        pine
    }

    fn select(columns: &[&'static str], table: &'static str) -> PineNode<'static> {
        let mut pine = from(table);
        append_operation(
            &mut pine,
            Operation::Select(columns.iter().map(|c| make_node(*c)).collect()),
        );

        pine
    }

    fn make_blank_pine() -> PineNode<'static> {
        make_node(Pine { operations: vec![] })
    }

    fn append_operation(pine: &mut PineNode<'static>, op: Operation<'static>) {
        pine.inner.operations.push(make_node(op));
    }

    fn make_node<T>(inner: T) -> Node<T> {
        Node {
            inner,
            position: Default::default(),
        }
    }

    type QualifiedColumnShorthand = (&'static str, &'static str);
    impl<'a> PartialEq<QualifiedColumnShorthand> for QualifiedColumnIdentifier {
        fn eq(&self, other: &QualifiedColumnShorthand) -> bool {
            self.table == other.0 && self.column == other.1
        }
    }
}
