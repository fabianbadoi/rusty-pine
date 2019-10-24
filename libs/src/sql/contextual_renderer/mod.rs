use super::structure::Table;
use super::Renderer;
use crate::error::PineError;
use crate::query::Query;
use explicit_representation::{
    ExplicitColumn, ExplicitFilter, ExplicitJoin, ExplicitOperand, ExplicitOrder, ExplicitQuery,
    ExplicitQueryBuilder,
};
use log::info;

mod explicit_representation;

#[derive(Debug)]
pub struct SmartRenderer {
    tables: Vec<Table>,
}

impl Renderer<Query, String> for &SmartRenderer {
    fn render(self, query: &Query) -> Result<String, PineError> {
        info!("Rendering query");

        let explicit_query = self.build_explicit_query(query)?;

        let query = self.render_explicit_query(&explicit_query);

        info!("Rendering done");

        Ok(query)
    }
}

impl SmartRenderer {
    pub fn for_tables(tables: Vec<Table>) -> SmartRenderer {
        SmartRenderer { tables }
    }

    fn build_explicit_query<'a>(&'a self, query: &'a Query) -> Result<ExplicitQuery<'a>, String> {
        info!("Building render-ready intermediate representation");
        let mut builder = ExplicitQueryBuilder::new(&self.tables[..]);
        let result = builder.make_explicit_query(query);

        info!("Done building render-ready intermediate representation");

        result
    }

    fn render_explicit_query(&self, query: &ExplicitQuery) -> String {
        info!("Rendering reander-ready representation");

        let select = render_select(&query);
        let from = render_from(query.from);
        let join = render_joins(&query.joins[..]);
        let filter = render_filters(&query.filters[..]);
        let order = render_orders(&query.order[..]);
        let limit = render_limit(query.limit);

        info!("Done rendering reander-ready representation");

        vec![select, from, join, filter, order, limit]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_select(query: &ExplicitQuery) -> String {
    let columns = if query.selections.len() > 0 {
        render_columns(&query.selections[..])
    } else {
        render_wildcard_select(&query)
    };

    format!("SELECT {}", columns)
}

fn render_columns(columns: &[ExplicitColumn]) -> String {
    columns
        .iter()
        .map(render_column)
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_wildcard_select(query: &ExplicitQuery) -> String {
    // always get data from the last table used
    match query.joins.last() {
        Some(join) => format!("{}.*", join.to_table),
        None => "*".to_string(),
    }
}

fn render_column(column: &ExplicitColumn) -> String {
    use ExplicitColumn::*;

    match column {
        Simple(column_name) => column_name.to_string(),
        FullyQualified(table_name, column_name) => format!("{}.{}", table_name, column_name),
    }
}

fn render_from(table: &str) -> String {
    format!("FROM {}", table)
}

fn render_joins(joins: &[ExplicitJoin]) -> String {
    joins.iter().map(render_join).collect::<Vec<_>>().join("\n")
}

fn render_join(join: &ExplicitJoin) -> String {
    format!(
        "LEFT JOIN {} ON {}.{} = {}.{}",
        join.to_table, join.to_table, join.to_column, join.from_table, join.from_column
    )
}

fn render_filters(filters: &[ExplicitFilter]) -> String {
    if filters.is_empty() {
        return "".to_owned();
    }

    let filters = filters
        .iter()
        .map(render_filter)
        .collect::<Vec<_>>()
        .join(" AND ");

    format!("WHERE {}", filters)
}

fn render_filter(filter: &ExplicitFilter) -> String {
    use ExplicitFilter::*;

    match filter {
        Equals(lhs, rhs) => render_smart_equals(lhs, rhs),
        IsNull(operand) => format!("{} IS NULL", render_operand(operand)),
        IsNotNull(operand) => format!("{} IS NOT NULL", render_operand(operand)),
    }
}

fn render_smart_equals(lhs: &ExplicitOperand, rhs: &ExplicitOperand) -> String {
    use ExplicitOperand::*;

    let operator = match rhs {
        Value(value) if value.contains('%') => "LIKE",
        _ => "=",
    };

    format!("{} {} {}", render_operand(lhs), operator, render_operand(rhs))
}

fn render_orders(orders: &[ExplicitOrder]) -> String {
    if orders.is_empty() {
        return "".to_owned();
    }

    let orders = orders
        .iter()
        .map(render_order)
        .collect::<Vec<_>>()
        .join(", ");

    format!("ORDER BY {}", orders)
}

fn render_order(order: &ExplicitOrder) -> String {
    let operand = match order {
        ExplicitOrder::Ascending(operand) | ExplicitOrder::Descending(operand) => {
            render_operand(operand)
        }
    };

    let direction = match order {
        ExplicitOrder::Ascending(_) => "",
        ExplicitOrder::Descending(_) => " DESC",
    };

    format!("{}{}", operand, direction)
}

fn render_operand(operand: &ExplicitOperand) -> String {
    use ExplicitOperand::*;

    match operand {
        Column(column) => render_column(column),
        Value(value) => render_value(value),
    }
}

fn render_value(value: &str) -> String {
    format!("{}", value)
}

fn render_limit(limit: usize) -> String {
    format!("LIMIT {}", limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::shorthand::*;
    use crate::sql::structure::ForeignKey;

    #[test]
    fn smart_render() {
        let renderer = make_renderer();
        let query = make_join_query();

        let rendering = renderer.render(&query).unwrap();

        assert_eq!(
            "SELECT users.id, users.name\nFROM users\nLEFT JOIN friends ON friends.id = users.friendId\nWHERE users.id = 1 AND users.mojo = 'great'\nLIMIT 10",
            rendering
        );
    }

    #[test]
    fn join_to_unknown_table() {
        let renderer = make_renderer();
        let mut query = make_join_query();
        query.joins[0] = "missing".to_string();

        let error = renderer.render(&query).unwrap_err();

        assert_eq!("Table missing not found.", format!("{}", error));
    }

    #[test]
    fn limits() {
        let renderer = make_renderer();
        let mut query = make_join_query();
        query.limit = 2;

        let rendering = renderer.render(&query).unwrap();

        let limit_is_2 = rendering.find("LIMIT 2").is_some();
        assert!(limit_is_2);
    }

    #[test]
    fn order() {
        use crate::query::{Operand, Order, QualifiedColumnIdentifier};

        let renderer = make_renderer();
        let mut query = make_query();
        query.order.push(Order::Descending(Operand::Column(
            QualifiedColumnIdentifier {
                table: "users".to_owned(),
                column: "id".to_owned(),
            },
        )));
        query
            .order
            .push(Order::Ascending(Operand::Value("3".to_owned())));

        let rendering = renderer.render(&query).unwrap();

        assert_eq!(
            "SELECT *\nFROM users\nORDER BY id DESC, 3\nLIMIT 10",
            rendering
        );
    }

    #[test]
    fn order_with_explict_column() {
        use crate::query::{Operand, Order, QualifiedColumnIdentifier};

        let renderer = make_renderer();
        let mut query = make_query();
        query.joins.push("friends".to_owned());
        query.order.push(Order::Descending(Operand::Column(
            QualifiedColumnIdentifier {
                table: "users".to_owned(),
                column: "id".to_owned(),
            },
        )));

        let rendering = renderer.render(&query).unwrap();

        assert_eq!("SELECT friends.*\nFROM users\nLEFT JOIN friends ON friends.id = users.friendId\nORDER BY users.id DESC\nLIMIT 10", rendering);
    }

    #[test]
    fn select_from_unknown_table() {
        let renderer = make_renderer();
        let mut query = make_join_query();
        query.from = "rusers".to_string();

        let error = renderer.render(&query).unwrap_err();

        println!("{}", error);
        assert_eq!("Table rusers not found, try: users", format!("{}", error));
    }

    fn make_join_query() -> Query {
        let query = QueryShorthand(
            Select(&["id", "name"]),
            From("users"),
            &[
                Filter::Equals("users.id", "1"),
                Filter::Equals("users.mojo", "'great'"),
            ],
        );
        let mut query: Query = query.into();
        query.joins.push("friends".to_string());

        query
    }

    fn make_query() -> Query {
        let query = QueryShorthand(Select(&[]), From("users"), &[]);

        query.into()
    }

    fn make_renderer() -> SmartRenderer {
        let tables = vec![
            Table {
                name: "users".into(),
                columns: Vec::new(),
                foreign_keys: vec![ForeignKey::from(&("friendId", ("friends", "id")))],
            },
            Table {
                name: "friends".into(),
                columns: Vec::new(),
                foreign_keys: Vec::new(),
            },
        ];

        SmartRenderer::for_tables(tables)
    }
}
