use super::Condition;
use super::QualifiedColumnIdentifier;
use super::Query;

pub trait Renderer<O, Q> {
    fn render(self, query: &Q) -> O;
}

pub struct StringRenderer {}

impl Renderer<String, Query<'_>> for &StringRenderer {
    fn render<'a>(self, query: &'a Query<'a>) -> String {
        let select = self.render_select(&query);
        let from = self.render_from(&query);
        let filters = self.render_filters(&query);

        format!("SELECT {}\nFROM {}\nWHERE {}", select, from, filters)
    }
}

impl StringRenderer {
    fn render_select<'a>(&self, query: &'a Query<'a>) -> String {
        let column_renderer = ColumnRenderer::new(query);

        let fields = query
            .selections
            .iter()
            .map(|field| column_renderer.render(field))
            .collect::<Vec<_>>()
            .join(", ");

        fields
    }

    fn render_from<'a>(&self, query: &'a Query<'a>) -> &'a str {
        query.from.unwrap()
    }

    fn render_filters<'a>(&self, query: &'a Query<'a>) -> String {
        let column_renderer = ColumnRenderer::new(query);

        let filters = query
            .filters
            .iter()
            .map(|filter| {
                let column = column_renderer.render(&filter.column);
                match filter.condition {
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

impl<'a> ColumnRenderer<&'a Query<'a>> {
    fn render(&self, id: &'a QualifiedColumnIdentifier) -> String {
        if self.query.from.unwrap() == id.table {
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
