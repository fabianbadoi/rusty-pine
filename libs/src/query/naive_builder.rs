use super::{BuildResult, QueryBuilder};
use crate::error::Position;
use crate::error::SyntaxError;
use crate::pine_syntax::ast::{
    ColumnNameNode, Condition as AstCondition, FilterNode, Operation, OperationNode, PineNode,
    TableNameNode, ValueNode,
};
use crate::query::{
    Condition as SqlCondition, Filter as SqlFilter, QualifiedColumnIdentifier, Query,
};
use log::{info, debug};

/// Has no concept of context, more complex queries will fail to build
#[derive(Debug)]
pub struct NaiveBuilder;

struct SingleUseQueryBuilder<'a> {
    pine: &'a PineNode<'a>,
    query: Query,
    current_table: Option<String>,
    from_table: Option<String>,
}

impl QueryBuilder for &NaiveBuilder {
    fn build(self, pine: &PineNode) -> BuildResult {
        let builder = SingleUseQueryBuilder::new(pine);

        builder.build()
    }
}

impl<'a> SingleUseQueryBuilder<'a> {
    fn new(pine: &'a PineNode) -> SingleUseQueryBuilder<'a> {
        SingleUseQueryBuilder {
            pine: pine,
            current_table: None,
            from_table: None,
            query: Default::default(),
        }
    }

    fn build(mut self) -> BuildResult {
        info!("Building query object from initial representation");

        for operation_node in self.pine {
            debug!("Applying {:?}", operation_node);
            self.apply_operation(operation_node)?;
        }

        self.finalize()?;

        info!("Done");

        Ok(self.query)
    }

    fn apply_operation(&mut self, operation_node: &OperationNode) -> InternalResult {
        match operation_node.inner {
            Operation::From(ref table) => self.apply_from(table),
            Operation::Join(ref table) => self.apply_join(table),
            Operation::Select(ref selections) => self.apply_selections(selections)?,
            Operation::Filter(ref filters) => self.apply_filters(filters)?,
            Operation::Limit(ref limit) => self.apply_limit(limit)?,
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &TableNameNode) {
        debug!("Found from: {:?}", table);

        self.current_table = Some(table.inner.to_string());

        if self.from_table.is_none() {
            self.from_table = Some(table.inner.to_string());
        }
    }

    fn apply_join(&mut self, table: &TableNameNode) {
        debug!("Found join: {:?}", table);

        self.current_table = Some(table.inner.to_string());
        self.query.joins.push(table.inner.to_string());
    }

    fn apply_selections(&mut self, selections: &[ColumnNameNode]) -> InternalResult {
        debug!("Found select: {:?}", selections);

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

    fn apply_filters(&mut self, filters: &[FilterNode]) -> Result<(), SyntaxError> {
        debug!("Found where: {:?}", filters);

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

    fn apply_limit(&mut self, value: &ValueNode) -> Result<(), SyntaxError> {
        use std::str::FromStr;
        debug!("Found limit: {:?}", value);

        match usize::from_str(value.inner) {
            Ok(limit) => {
                self.query.limit = limit;
                Ok(())
            }
            // Pest will make sure the values are actually numeric, but they may be
            // unrepresentable by usize.
            Err(parse_error) => Err(SyntaxError::Positioned {
                message: format!("{}", parse_error),
                position: value.position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }

    fn finalize(&mut self) -> Result<(), SyntaxError> {
        match self.from_table.clone() {
            Some(table) => {
                self.query.from = table;
                Ok(())
            }
            None => Err(SyntaxError::Positioned {
                message: "Missing a 'from:' statement".to_string(),
                position: self.pine.position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }

    fn require_table(&self, pine_position: Position) -> Result<String, SyntaxError> {
        match &self.current_table {
            Some(table) => Ok(table.clone()),
            None => Err(SyntaxError::Positioned {
                message: "Place a 'from:' statement in front fo this".to_string(),
                position: pine_position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }
}

impl<'a> From<&AstCondition<'a>> for SqlCondition {
    fn from(other: &AstCondition<'a>) -> Self {
        match other {
            AstCondition::Equals(ref value) => SqlCondition::Equals(value.inner.to_string()),
        }
    }
}

type InternalResult = Result<(), SyntaxError>;

#[cfg(test)]
mod tests {
    use super::super::{Condition as SqlCondition, QualifiedColumnIdentifier};
    use super::{NaiveBuilder, QueryBuilder};
    use crate::pine_syntax::ast::*;

    #[test]
    fn build_from_query() {
        let pine = from("users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!("users", query.from);
    }

    #[test]
    fn build_with_limit() {
        let pine = with_limit("100");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(100, query.limit);
    }

    #[test]
    fn double_limits_overrides_previous_limit() {
        let mut pine = with_limit("100");
        append_operation(&mut pine, Operation::Limit(make_node("200")));

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(200, query.limit);
    }

    #[test]
    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[1], ("users", "name"));
    }

    #[test]
    fn build_filter_query() {
        let pine = filter("id", Condition::Equals(make_node("3")), "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(query.filters[0].column, ("users", "id"));
        assert_eq!(
            query.filters[0].condition,
            SqlCondition::Equals("3".to_string())
        );
    }

    #[test]
    fn build_join_query() {
        let pine = join("users", "friends");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.from, "users");
        assert_eq!(query.joins[0], "friends");
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

    fn join(from_table: &'static str, join: &'static str) -> PineNode<'static> {
        let mut pine = from(from_table);

        let join = make_node(join);
        append_operation(&mut pine, Operation::Join(join));

        pine
    }

    fn from(table: &'static str) -> PineNode {
        let mut pine = make_blank_pine();
        append_operation(&mut pine, Operation::From(make_node(table)));

        pine
    }

    fn with_limit(limit: &'static str) -> PineNode {
        let mut pine = from("dummy");
        append_operation(&mut pine, Operation::Limit(make_node(limit)));

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
        make_node(Pine {
            operations: vec![],
            pine_string: "",
        })
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
    impl PartialEq<QualifiedColumnShorthand> for QualifiedColumnIdentifier {
        fn eq(&self, other: &QualifiedColumnShorthand) -> bool {
            self.table == other.0 && self.column == other.1
        }
    }
}
