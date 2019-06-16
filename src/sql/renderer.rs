use super::Renderer;
use crate::query::Condition;
use crate::query::QualifiedColumnIdentifier;
use crate::query::Query;
use crate::error::PineError;

pub struct DumbRenderer {}

impl Renderer<Query, String> for &DumbRenderer {
    fn render(self, query: &Query) -> Result<String, PineError> {
        let select = render_select(&query);
        let from = render_from(&query);
        let filters = render_filters(&query);

        Ok(format!("SELECT {}\nFROM {}\nWHERE {}", select, from, filters))
    }
}

pub fn render_select(query: &Query) -> String {
    let column_renderer = ColumnRenderer::new(query);

    let fields = if query.selections.is_empty() {
        "*".to_string()
    } else {
        query
            .selections
            .iter()
            .map(|field| column_renderer.render(field))
            .collect::<Vec<_>>()
            .join(", ")
    };

    fields
}

pub fn render_from(query: &Query) -> String {
    query.from.clone()
}

pub fn render_filters(query: &Query) -> String {
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

pub fn render_limit(query: &Query) -> String {
    format!("LIMIT {}", query.limit)
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
        if self.query.joins.is_empty() {
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
    fn dumb_render() {
        let renderer = DumbRenderer {};
        let query = QueryShorthand(
            Select(&["id", "name"]),
            From("users"),
            &[Filter::Equals("id", "1"), Filter::Equals("mojo", "great")],
        );

        let rendering = renderer.render(&query.into()).unwrap();

        assert_eq!(
            "SELECT id, name\nFROM users\nWHERE id = \"1\" AND mojo = \"great\"",
            rendering
        );
    }
}
