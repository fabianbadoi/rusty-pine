use crate::error::SyntaxError;
use crate::pine_syntax::ast::{Node, Operation, Pine};
use crate::query::{BuildResult, Renderable};

pub fn build_neighbours_query(pine: &Node<Pine>) -> BuildResult {
    let mut reverse = pine.inner.operations.iter().rev();
    let marker = reverse.next().unwrap();

    for operation in reverse {
        if let Operation::Join(table_node) | Operation::From(table_node) = &operation.inner {
            return Ok(Renderable::ShowNeighbours(table_node.inner.to_owned()));
        }
    }

    Err(SyntaxError::Positioned {
        message: "Must specify a table with from:".to_string(),
        position: marker.position,
        input: pine.inner.pine_string.to_string(),
    }
    .into())
}

#[cfg(test)]
mod test {
    use crate::pine_syntax::ast::{Node, Operation, Pine};
    use crate::query::naive_builder::neighbour_builder::build_neighbours_query;
    use crate::query::Renderable;

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
                        inner: Operation::ShowNeighbours(Default::default()),
                        position: Default::default(),
                    },
                ],
                pine_string: "",
            },
            position: Default::default(),
        };

        let result = build_neighbours_query(&pine);

        assert_eq!(
            result.unwrap(),
            Renderable::ShowNeighbours("table_name".to_string())
        );
    }
}
