use crate::sql::{Query, ColumnName, QualifiedColumnIdentifier};
use crate::pine_syntax::{PineNode, OperationNode, Operation, TableNameNode, ColumnNameNode, Position};
use std::result::Result as StdResult;

#[derive(Debug)]
struct BuildError {
    message: String,
    position: Position,
}

type Result<'a> = StdResult<Query<'a>, BuildError>;

trait QueryBuilder {
    fn build<'a>(&self, pine: &'a PineNode) -> Result<'a>;
}

struct PineTranslator;

#[derive(Default)]
struct SingleUseQueryBuilder<'a> {
    query: Query<'a>,
}

impl QueryBuilder for PineTranslator {
    fn build<'a>(&self, pine: &'a PineNode) -> Result<'a> {
        let builder = SingleUseQueryBuilder::new();

        builder.build(pine)
    }
}

impl<'a> SingleUseQueryBuilder<'a> {
    fn new() -> SingleUseQueryBuilder<'a> {
        Default::default()
    }

    fn build(mut self, pine: &'a PineNode) -> Result<'a> {
        for operation_node in &pine.inner.operations {
            self.apply_operation(operation_node)?;
        }

        Ok(self.query)
    }

    fn apply_operation(&mut self, operation_node: &'a OperationNode) -> StdResult<(), BuildError> {
        match operation_node.inner {
            Operation::From(ref table) => self.apply_from(table),
            Operation::Select(ref selections) => self.apply_selections(selections)?,
            _ => unimplemented!()
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &'a TableNameNode) {
        self.reset_selection(&table.inner);
    }

    fn apply_selections(&mut self, selections: &'a Vec<ColumnNameNode>) -> StdResult<(), BuildError> {
        if selections.len() == 0 {
            return Ok(());
        }

        let position = selections[0].position;
        let table = self.require_table(position)?;
        let mut selections: Vec<_> = selections.iter()
            .map(|name_node| name_node.inner.as_str())
            .map(|column| QualifiedColumnIdentifier { table, column })
            .collect();

        self.query.selections.append(&mut selections);

        Ok(())
    }

    fn reset_selection(&mut self, table: &'a str) {
        self.query.from = Some(table);

        // existing selections are cleared to, maybe add a select+: operation that keeps previous selections
        self.query.selections.clear();
    }

    fn require_table(&self, pine_position: Position) -> StdResult<&'a str, BuildError> {
        match self.query.from {
            Some(table) => Ok(table),
            None => Err(BuildError {
                message: "Must specify a from: clause before using select:.".to_string(),
                position: pine_position
            })
        }
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
        let query = query_builder.build(&pine).unwrap();

        assert_eq!("users", query.from.unwrap());
    }

    #[test]
    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = PineTranslator{};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[1], ("users", "name"));
    }

    fn from(table: &'static str) -> PineNode {
        let mut pine = make_blank_pine();
        append_operation(&mut pine, Operation::From(make_node(table.to_string())));

        pine
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