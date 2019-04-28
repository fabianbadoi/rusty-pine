use crate::sql::{Query, QualifiedColumnIdentifier, Filter, Condition};
use crate::pine_syntax::{PineNode, OperationNode, Operation, TableNameNode};

trait QueryBuilder {
    fn build<'a>(&self, pine: &'a PineNode) -> Query<'a>;
}

struct PineTranslator {

}

impl QueryBuilder for PineTranslator {
    fn build<'a>(&self, pine: &'a PineNode) -> Query<'a> {
        let mut query: Query = Default::default();

        for operation_node in &pine.inner.operations {
            self.apply_operation(&mut query, operation_node);
        }

        query
    }
}

impl PineTranslator {
    fn apply_operation<'a>(&self, query: &mut Query<'a>, operation_node: &'a OperationNode) {
        match operation_node.inner {
            Operation::From(ref table) => self.apply_from(query, table),
            _ => unimplemented!()
        };
    }

    fn apply_from<'a>(&self, query: &mut Query<'a>, table: &'a TableNameNode) {
        query.from = Some(&table.inner)
    }
}

#[cfg(test)]
mod tests {
    use crate::pine_syntax::*;
    use super::{QueryBuilder, PineTranslator};
    use crate::sql::QualifiedColumnIdentifier;
    
    #[test]
    fn build_from_query() {
        let pine = from("users");

        let query_builder = PineTranslator{};
        let query = query_builder.build(&pine);

        assert_eq!("users", query.from.unwrap());
    }

    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = PineTranslator{};
        let query = query_builder.build(&pine);

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[0], ("users", "name"));
    }

    fn from(table: &'static str) -> PineNode {
        make_pine_with(Operation::From(make_node(table.to_string())))
    }

    fn select(columns: &[&'static str], table: &'static str) -> PineNode {
        let mut pine = from(table);
        append_operation(
            &mut pine,
            Operation::Select(
                columns.iter()
                    .map(|c| make_node(c.to_string()))
                    .collect()
            )
        );

        pine
    }

    fn make_pine_with(op: Operation) -> PineNode {
        let mut pine = make_blank_pine();
        append_operation(&mut pine, op);

        pine
    }

    fn make_blank_pine() -> PineNode {
        make_node(Pine { operations: vec![] })
    }

    fn append_operation(pine: &mut PineNode, op: Operation) {
        pine.inner.operations.push(make_node(op));
    }

    fn make_node<T>(inner: T) -> Node<T> {
        Node {
            inner,
            position: Default::default()
        }
    }

    type QualifiedColumnShorthand = (&'static str, &'static str);
    impl<'a> PartialEq<QualifiedColumnShorthand> for QualifiedColumnIdentifier<'a> {
        fn eq(&self, other: &QualifiedColumnShorthand) -> bool {
            self.table == other.0 && self.column == other.1
        }
    }
}