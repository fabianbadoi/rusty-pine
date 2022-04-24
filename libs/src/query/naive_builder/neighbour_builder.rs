use crate::pine_syntax::ast::{MetaOperation, Node, Operation, Pine};
use crate::query::{Renderable, RenderableMetaOperation, TableName};

pub fn build_meta_query(pine: &Node<Pine>) -> Renderable {
    if let Some(Node {
        inner: Operation::Meta(meta_op),
        ..
    }) = pine.last_operation()
    {
        let meta_op = match meta_op {
            MetaOperation::ShowNeighbours(_) => build_neighbours_query(pine),
            MetaOperation::ShowColumns(_) => build_columns_query(pine),
        };

        Renderable::Meta(meta_op)
    } else {
        panic!("found non-meta operation in build_meta_query")
    }
}

fn build_columns_query(pine: &Node<Pine>) -> RenderableMetaOperation {
    RenderableMetaOperation::ShowColumns(last_table(&pine.inner))
}

fn build_neighbours_query(pine: &Node<Pine>) -> RenderableMetaOperation {
    RenderableMetaOperation::ShowNeighbours(last_table(&pine.inner))
}

fn last_table(pine: &Pine) -> TableName {
    for operation in pine.operations.iter().rev() {
        if let Operation::Join(table_node) | Operation::From(table_node) = &operation.inner {
            return table_node.inner.to_string();
        }
    }

    panic!("All pines must have at least one table")
}

#[cfg(test)]
mod test {
    use crate::pine_syntax::ast::{MetaOperation, Node, Operation, Pine};
    use crate::query::naive_builder::neighbour_builder::build_neighbours_query;
    use crate::query::RenderableMetaOperation;

    #[test]
    fn test_builds_neighbour_query() {
        let pine = Node {
            inner: Pine {
                operations: vec![
                    Node {
                        inner: Operation::From(Node {
                            inner: "table_name",
                            ..Default::default()
                        }),
                        position: Default::default(),
                    },
                    Node {
                        inner: Operation::Meta(MetaOperation::ShowNeighbours(Default::default())),
                        position: Default::default(),
                    },
                ],
                pine_string: "",
            },
            position: Default::default(),
        };

        let result = build_neighbours_query(&pine);

        assert_eq!(
            result,
            RenderableMetaOperation::ShowNeighbours("table_name".to_string())
        );
    }
}
