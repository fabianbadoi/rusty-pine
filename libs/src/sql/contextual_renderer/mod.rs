use super::structure::Table;
use super::Renderer;
use crate::common::{BinaryFilterType, UnaryFilterType};
use crate::error::PineError;
use crate::query::Query;
use crate::sql::contextual_renderer::explicit_representation::ExplicitResultColumn;
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
        let group = render_group_by(&query.group_by[..]);
        let order = render_orders(&query.order[..]);
        let limit = render_limit(&query);

        info!("Done rendering reander-ready representation");

        vec![select, from, join, filter, group, order, limit]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_select(query: &ExplicitQuery) -> String {
    let columns = if query.selections.len() > 0 {
        render_results_columns(&query.selections[..])
    } else {
        render_wildcard_select(&query)
    };

    format!("SELECT {}", columns)
}

fn render_results_columns(columns: &[ExplicitResultColumn]) -> String {
    columns
        .iter()
        .map(|selection| match selection {
            ExplicitResultColumn::Value(value) => render_value(value),
            ExplicitResultColumn::Column(column) => render_column(column),
            ExplicitResultColumn::FunctionCall(function_name, column) => {
                render_function_call(function_name, column)
            }
        })
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

fn render_function_call(function_name: &str, column: &ExplicitColumn) -> String {
    format!("{}({})", function_name, render_column(column))
}

fn render_from(table: &str) -> String {
    if table.is_empty() {
        "".to_string()
    } else {
        format!("FROM {}", table)
    }
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
        Unary(operand, filter_type) => {
            let operand = render_operand(operand);

            format!("{} {}", operand, filter_type)
        }
        Binary(lhs, rhs, BinaryFilterType::Equals) => render_smart_equals(lhs, rhs),
        Binary(lhs, rhs, filter_type) => {
            let lhs = render_operand(lhs);
            let rhs = render_operand(rhs);

            format!("{} {} {}", lhs, filter_type, rhs)
        }
    }
}

fn render_smart_equals(lhs: &ExplicitOperand, rhs: &ExplicitOperand) -> String {
    use ExplicitOperand::*;

    let operator = match rhs {
        Value(value) if value.contains('%') => "LIKE",
        _ => "=",
    };

    format!(
        "{} {} {}",
        render_operand(lhs),
        operator,
        render_operand(rhs)
    )
}

fn render_group_by(groups: &[ExplicitResultColumn]) -> String {
    if groups.is_empty() {
        return "".to_string();
    }

    let groups = render_results_columns(groups);

    format!("GROUP BY {}", groups)
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

fn render_limit(query: &ExplicitQuery) -> String {
    if query.from.is_empty() {
        "".to_string()
    } else {
        format!("LIMIT {}", query.limit)
    }
}

/// Used to simplify rendering
impl std::fmt::Display for UnaryFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryFilterType::IsNull => write!(f, "IS NULL"),
            UnaryFilterType::IsNotNull => write!(f, "IS NOT NULL"),
        }
    }
}

/// Used to simplify rendering
impl std::fmt::Display for BinaryFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFilterType::LesserThan => write!(f, "<"),
            BinaryFilterType::LesserThanOrEquals => write!(f, "<="),
            BinaryFilterType::Equals => write!(f, "="),
            BinaryFilterType::NotEquals => write!(f, "!="),
            BinaryFilterType::GreaterThan => write!(f, ">"),
            BinaryFilterType::GreaterThanOrEquals => write!(f, ">="),
        }
    }
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
                Filter::Binary("users.id", "1", BinaryFilterType::Equals),
                Filter::Binary("users.mojo", "'great'", BinaryFilterType::Equals),
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
