use crate::query::Condition;
use crate::query::QualifiedColumnIdentifier;
use crate::query::Query;
use super::Renderer;

pub struct StringRenderer {}

impl Renderer<Query, String> for &StringRenderer {
    fn render(self, query: &Query) -> String {
        let select = self.render_select(&query);
        let from = self.render_from(&query);
        let filters = self.render_filters(&query);

        format!("SELECT {}\nFROM {}\nWHERE {}", select, from, filters)
    }
}

impl StringRenderer {
    fn render_select(&self, query: &Query) -> String {
        let column_renderer = ColumnRenderer::new(query);

        let fields = query
            .selections
            .iter()
            .map(|field| column_renderer.render(field))
            .collect::<Vec<_>>()
            .join(", ");

        fields
    }

    fn render_from(&self, query: &Query) -> String {
        query.from.clone()
    }

    fn render_filters(&self, query: &Query) -> String {
        let column_renderer = ColumnRenderer::new(query);

        let filters = query
            .filters
            .iter()
            .map(|filter| {
                let column = column_renderer.render(&filter.column);
                match &filter.condition {
                    Condition::Equals(value) => format!("{} = \"{}\"", column, value),
                }
            })
            .collect::<Vec<_>>()
            .join(" AND ");

        filters
    }
}

struct ColumnRenderer<Q> {
    query: Q,
}

impl<Q> ColumnRenderer<Q> {
    fn new(query: Q) -> Self {
        ColumnRenderer { query }
    }
}

impl ColumnRenderer<&Query> {
    fn render(&self, id: &QualifiedColumnIdentifier) -> String {
        if *self.query.from == id.table {
            id.column.to_string()
        } else {
            format!("{}.{}", id.table, id.column)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::shorthand::*;
    use super::*;

    #[test]
    fn simple_render() {
        let renderer = StringRenderer {};
        let query = QueryShorthand(
            Select(&["id", "name"]),
            From("users"),
            &[Filter::Equals("id", "1"), Filter::Equals("mojo", "great")],
        );

        let rendering = renderer.render(&query.into());

        assert_eq!(
            "SELECT id, name\nFROM users\nWHERE id = \"1\" AND mojo = \"great\"",
            rendering
        );
    }
}
