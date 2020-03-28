mod neighbour_builder;
mod single_use_builder;

use super::{BuildResult, QueryBuilder};
use crate::pine_syntax::ast::{Node, Operation, Pine};
use crate::query::naive_builder::neighbour_builder::build_neighbours_query;
use single_use_builder::SingleUseQueryBuilder;

/// Has no concept of context, more complex queries will fail to build
#[derive(Debug)]
pub struct NaiveBuilder;

impl QueryBuilder for &NaiveBuilder {
    fn build(self, pine: &Node<Pine>) -> BuildResult {
        match pine.last_operation() {
            Some(Node {
                inner: Operation::ShowNeighbours(_),
                ..
            }) => build_neighbours_query(pine),
            _ => {
                let builder = SingleUseQueryBuilder::new(pine);

                builder.build()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::QualifiedColumnIdentifier;
    use super::{NaiveBuilder, QueryBuilder};
    use crate::common::{BinaryFilterType, UnaryFilterType};
    use crate::pine_syntax::ast::*;
    use crate::pine_syntax::ast::{
        ColumnIdentifier as AstColumnIdentifier, Node, Operation as AstOperation,
        Order as AstOrder, Pine,
    };
    use crate::query::{Filter as SqlFilter, Operand as SqlOperand, Order as SqlOrder};

    #[test]
    fn build_from_query() {
        let pine = from("users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!("users", query.from);
    }

    #[test]
    fn build_with_limit() {
        let pine = with_limit("100");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(100, query.limit);
    }

    #[test]
    fn double_limits_overrides_previous_limit() {
        let mut pine = with_limit("100");
        append_operation(&mut pine, AstOperation::Limit(node(Value::Numeric("200"))));

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(200, query.limit);
    }

    #[test]
    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[1], ("users", "name"));
    }

    #[test]
    fn build_filter_query() {
        let rhs = Operand::Column(node(AstColumnIdentifier::Implicit(node("id"))));
        let lhs = Operand::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(
            query.filters[0],
            SqlFilter::Binary(("users", "id").into(), "3".into(), BinaryFilterType::Equals)
        );
    }

    #[test]
    fn build_is_null_filter() {
        let mut pine = from("users");
        let column = Operand::Column(node(AstColumnIdentifier::Implicit(node("id"))));
        let column = node(column);
        let filter = node(Filter::Unary(column, UnaryFilterType::IsNull));
        append_operation(&mut pine, AstOperation::Filter(vec![filter]));

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(
            query.filters[0],
            SqlFilter::Unary(("users", "id").into(), UnaryFilterType::IsNull)
        );
    }

    #[test]
    fn build_filter_query_with_explicit_column() {
        let rhs = Operand::Column(node(AstColumnIdentifier::Explicit(
            node("users"),
            node("id"),
        )));
        let lhs = Operand::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(
            query.filters[0],
            SqlFilter::Binary(("users", "id").into(), "3".into(), BinaryFilterType::Equals)
        );
    }

    #[test]
    fn build_join_query() {
        let pine = join("users", "friends");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.from, "friends");
        assert_eq!(query.joins[0], "users");
    }

    #[test]
    fn build_order() {
        let order_1 = Operand::Column(node(AstColumnIdentifier::Explicit(
            node("users"),
            node("id"),
        )));
        let order_2 = Operand::Value(node(Value::Numeric("3")));
        let order = vec![
            node(AstOrder::Ascending(node(order_1))),
            node(AstOrder::Descending(node(order_2))),
        ];
        let mut pine = select(&["id", "name"], "users");
        append_operation(&mut pine, AstOperation::Order(order));

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap().query();

        assert_eq!(query.order[0], SqlOrder::Ascending(("users", "id").into()));
        assert_eq!(query.order[1], SqlOrder::Descending(("3").into()));
    }

    fn make_equals(
        rhs: Operand<'static>,
        lhs: Operand<'static>,
        table: &'static str,
    ) -> Node<Pine<'static>> {
        let mut pine = from(table);

        let rhs = node(rhs);
        let lhs = node(lhs);
        let filter = node(Filter::Binary(rhs, lhs, BinaryFilterType::Equals));

        append_operation(&mut pine, AstOperation::Filter(vec![filter]));

        pine
    }

    fn join(from_table: &'static str, join: &'static str) -> Node<Pine<'static>> {
        let mut pine = from(from_table);

        let join = node(join);
        append_operation(&mut pine, AstOperation::Join(join));

        pine
    }

    fn from(table: &'static str) -> Node<Pine> {
        let mut pine = make_blank_pine();
        append_operation(&mut pine, AstOperation::From(node(table)));

        pine
    }

    fn with_limit(limit: &'static str) -> Node<Pine> {
        let mut pine = from("dummy");
        append_operation(&mut pine, AstOperation::Limit(node(Value::Numeric(limit))));

        pine
    }

    fn select(columns: &[&'static str], table: &'static str) -> Node<Pine<'static>> {
        let mut pine = from(table);
        append_operation(
            &mut pine,
            AstOperation::Select(
                columns
                    .iter()
                    .map(|c| {
                        node(Operand::Column(node(AstColumnIdentifier::Implicit(node(
                            *c,
                        )))))
                    })
                    .collect(),
            ),
        );

        pine
    }

    fn make_blank_pine() -> Node<Pine<'static>> {
        node(Pine {
            operations: vec![],
            pine_string: "",
        })
    }

    fn append_operation(pine: &mut Node<Pine<'static>>, op: AstOperation<'static>) {
        pine.inner.operations.push(node(op));
    }

    fn node<T>(inner: T) -> Node<T> {
        Node {
            inner,
            position: Default::default(),
        }
    }

    impl PartialEq<(&str, &str)> for QualifiedColumnIdentifier {
        fn eq(&self, other: &(&str, &str)) -> bool {
            self.table == other.0 && self.column == other.1
        }
    }

    use crate::query::structure::Operand as QuerySelection;
    impl PartialEq<(&str, &str)> for QuerySelection {
        fn eq(&self, other: &(&str, &str)) -> bool {
            match self {
                QuerySelection::Column(column) => column == other,
                _ => panic!("cannot compare (str,str) to function call"),
            }
        }
    }

    impl From<(&str, &str)> for SqlOperand {
        fn from(other: (&str, &str)) -> SqlOperand {
            SqlOperand::Column(QualifiedColumnIdentifier {
                table: other.0.to_string(),
                column: other.1.to_string(),
            })
        }
    }

    impl From<&str> for SqlOperand {
        fn from(other: &str) -> SqlOperand {
            SqlOperand::Value(other.to_string())
        }
    }
}
