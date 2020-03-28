use super::{BuildResult, QueryBuilder};
use crate::error::Position;
use crate::error::SyntaxError;
use crate::pine_syntax::ast::{
    ColumnIdentifier as AstColumnIdentifier, Filter as AstFilter, Node, Operand,
    Operation as AstOperation, Order as AstOrder, Pine, ResultColumn as AstResultColumn,
    TableName as AstTableName, Value as AstValue,
};
use crate::query::{
    Filter as SqlFilter, Operand as SqlOperand, Order as SqlOrder, QualifiedColumnIdentifier,
    Query, ResultColumn as SqlResultColumn,
};
use log::{debug, info};

/// Has no concept of context, more complex queries will fail to build
#[derive(Debug)]
pub struct NaiveBuilder;

struct SingleUseQueryBuilder<'a> {
    /// The pine we're going through
    pine: &'a Node<Pine<'a>>,

    /// This is the query currenly being built.
    query: Query,

    /// Holds the initial table, pines that don't have one are invalid
    from_table: Option<String>,

    /// At the end of the build process, we may add a select for last_joined_table.*
    implicit_select_all_from_last_table: bool,
}

impl QueryBuilder for &NaiveBuilder {
    fn build(self, pine: &Node<Pine>) -> BuildResult {
        let builder = SingleUseQueryBuilder::new(pine);

        builder.build()
    }
}

impl<'a> SingleUseQueryBuilder<'a> {
    fn new(pine: &'a Node<Pine>) -> SingleUseQueryBuilder<'a> {
        SingleUseQueryBuilder {
            pine,
            from_table: None,
            implicit_select_all_from_last_table: false,
            query: Default::default(),
        }
    }

    fn build(mut self) -> BuildResult {
        info!("Building query object from initial representation");

        for operation_node in self.pine {
            debug!("Applying {:?}", operation_node);
            self.apply_operation(operation_node)?;
        }

        self.finalize();

        info!("Done");

        Ok(self.query)
    }

    fn apply_operation(&mut self, operation_node: &Node<AstOperation>) -> InternalResult {
        match operation_node.inner {
            AstOperation::From(ref table) => self.apply_from(table),
            AstOperation::Join(ref table) => self.apply_join(table),
            AstOperation::Select(ref selections) => self.apply_selections(selections)?,
            AstOperation::Unselect(ref selections) => self.apply_unselections(selections)?,
            AstOperation::Filter(ref filters) => self.apply_filters(filters)?,
            AstOperation::GroupBy(ref group_by) => self.apply_group_by(group_by)?,
            AstOperation::Order(ref orders) => self.apply_orders(orders)?,
            AstOperation::Limit(ref limit) => self.apply_limit(limit)?,
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &Node<AstTableName>) {
        debug!("Found from: {:?}", table);

        if let Some(table) = &self.from_table {
            self.query.joins.push(table.clone());
        }

        self.from_table = Some(table.inner.to_string());
        self.implicit_select_all_from_last_table = true;
    }

    fn apply_join(&mut self, table: &Node<AstTableName>) {
        debug!("Found join: {:?}", table);

        self.apply_from(table);
    }

    fn apply_selections(&mut self, selections: &[Node<AstResultColumn>]) -> InternalResult {
        debug!("Found select: {:?}", selections);

        let mut selections = self.build_columns(selections)?;
        self.query.selections.append(&mut selections);

        // selecting only some tables MUST clear the implicit most_recent_table.* select
        self.implicit_select_all_from_last_table = false;

        Ok(())
    }

    fn apply_unselections(&mut self, unselect: &[Node<AstResultColumn>]) -> InternalResult {
        debug!("Found unselect: {:?}", unselect);

        let mut unselect = self.build_columns(unselect)?;
        self.query.unselections.append(&mut unselect);

        Ok(())
    }

    fn apply_filters(&mut self, filters: &[Node<AstFilter>]) -> Result<(), SyntaxError> {
        debug!("Found where: {:?}", filters);

        if filters.is_empty() {
            return Ok(());
        }

        let table = self.require_table(filters[0].position)?;
        let mut filters = filters
            .iter()
            .map(|filter_node| self.translate_filter(filter_node, table))
            .collect::<Result<Vec<_>, _>>()?;

        self.query.filters.append(&mut filters);

        Ok(())
    }

    fn apply_group_by(&mut self, groups: &[Node<AstResultColumn>]) -> Result<(), SyntaxError> {
        debug!("Found group_by: {:?}", groups);

        if groups.is_empty() {
            return Ok(());
        }

        self.add_group_by(groups)?;

        // this isn't that pretty, but it's the simplest solution
        // if we don't push this here, we have to analyze all selections in the finalize() function
        // to know if we should add it there.
        // if we remove this, you will only your group by column in the select
        if self.query.selections.is_empty() {
            self.push_implicit_select_all();
        }

        self.apply_selections(groups)
    }

    fn add_group_by(&mut self, groups: &[Node<AstResultColumn>]) -> Result<(), SyntaxError> {
        let mut group_by = self.build_columns(groups)?;
        self.query.group_by.append(&mut group_by);

        Ok(())
    }

    fn apply_orders(&mut self, orders: &[Node<AstOrder>]) -> Result<(), SyntaxError> {
        debug!("Found orders: {:?}", orders);

        if orders.is_empty() {
            return Ok(());
        }

        let table = self.require_table(orders[0].position)?;
        let mut order: Vec<_> = orders
            .iter()
            .map(|order_node| translate_order(order_node, table))
            .collect();

        self.query.order.append(&mut order);

        Ok(())
    }

    fn apply_limit(&mut self, value: &Node<AstValue>) -> Result<(), SyntaxError> {
        use std::str::FromStr;
        debug!("Found limit: {:?}", value);

        match usize::from_str(value.inner.as_str()) {
            Ok(limit) => {
                self.query.limit = limit;
                Ok(())
            }
            // Pest will make sure the values are actually numeric, but they may be
            // unrepresentable by usize.
            Err(parse_error) => Err(SyntaxError::Positioned {
                message: format!("{}", parse_error),
                position: value.position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }

    fn translate_filter(
        &self,
        filter_node: &Node<AstFilter>,
        default_table: &str,
    ) -> Result<SqlFilter, SyntaxError> {
        debug!("Found filter: {:?}", filter_node);

        Ok(match &filter_node.inner {
            AstFilter::Unary(operand, filter_type) => {
                let operand = translate_operand(&operand.inner, default_table);

                SqlFilter::Unary(operand, *filter_type)
            }
            AstFilter::Binary(lhs, rhs, filter_type) => {
                let lhs = self.translate_result_column(&lhs)?;
                let rhs = self.translate_result_column(&rhs)?;

                SqlFilter::Binary(lhs, rhs, *filter_type)
            }
        })
    }

    fn translate_result_column(
        &self,
        select_node: &Node<AstResultColumn>,
    ) -> Result<SqlResultColumn, SyntaxError> {
        let selection = match &select_node.inner {
            AstResultColumn::Value(value) => SqlResultColumn::Value(value.inner.to_string()),
            AstResultColumn::Column(column) => {
                let default_table = self.require_table(column.position)?;
                SqlResultColumn::Column(translate_column_identifier(&column.inner, default_table))
            }
            AstResultColumn::FunctionCall(function_name, column) => {
                let default_table = self.require_table(column.position)?;
                SqlResultColumn::FunctionCall(
                    function_name.inner.to_string(),
                    translate_column_identifier(&column.inner, default_table),
                )
            }
        };

        Ok(selection)
    }

    fn finalize(&mut self) {
        self.set_from_table();
        self.add_implicit_select_all();

        self.deduplicate_selections();
    }

    fn deduplicate_selections(&mut self) {
        // this only deduplicates *consecutive* entries, that's exactly what we want
        self.query.selections.dedup();
    }

    fn set_from_table(&mut self) {
        if let Some(table) = self.from_table.clone() {
            self.query.from = table;
        }
    }

    fn add_implicit_select_all(&mut self) {
        if self.implicit_select_all_from_last_table {
            self.push_implicit_select_all();
        }
    }

    fn push_implicit_select_all(&mut self) {
        let table = self.from_table.clone().unwrap();

        let magic_column = QualifiedColumnIdentifier {
            table,
            column: "*".to_string(),
        };

        self.query
            .selections
            .push(SqlResultColumn::Column(magic_column));
    }

    fn require_table(&self, pine_position: Position) -> Result<&str, SyntaxError> {
        match &self.from_table {
            Some(table) => Ok(table.as_str()),
            None => Err(SyntaxError::Positioned {
                message: "Place a 'from:' statement in front fo this".to_string(),
                position: pine_position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }

    fn build_columns(
        &self,
        columns: &[Node<AstResultColumn>],
    ) -> Result<Vec<SqlResultColumn>, SyntaxError> {
        let qualified_columns: Result<Vec<_>, _> = columns
            .iter()
            .map(|selection| self.translate_result_column(selection))
            .collect();

        Ok(qualified_columns?)
    }
}

fn translate_order(order_node: &Node<AstOrder>, default_table: &str) -> SqlOrder {
    debug!("Found order: {:?}", order_node);

    let operand = match &order_node.inner {
        AstOrder::Ascending(operand) | AstOrder::Descending(operand) => {
            translate_operand(&operand.inner, default_table)
        }
    };

    match &order_node.inner {
        AstOrder::Ascending(_) => SqlOrder::Ascending(operand),
        AstOrder::Descending(_) => SqlOrder::Descending(operand),
    }
}

fn translate_operand(operand: &Operand, default_table: &str) -> SqlOperand {
    match operand {
        Operand::Column(column_identifier) => {
            let column = translate_column_identifier(&column_identifier.inner, default_table);
            SqlOperand::Column(column)
        }
        Operand::Value(value) => SqlOperand::Value(value.inner.to_string()),
    }
}

fn translate_column_identifier(
    identifier: &AstColumnIdentifier,
    default_table: &str,
) -> QualifiedColumnIdentifier {
    let (table, column) = match identifier {
        AstColumnIdentifier::Implicit(column_name) => {
            (default_table.to_string(), column_name.to_string())
        }
        AstColumnIdentifier::Explicit(table_name, column_name) => {
            (table_name.to_string(), column_name.to_string())
        }
    };

    QualifiedColumnIdentifier { table, column }
}

type InternalResult = Result<(), SyntaxError>;

#[cfg(test)]
mod tests {
    use super::super::QualifiedColumnIdentifier;
    use super::*;
    use super::{NaiveBuilder, QueryBuilder};
    use crate::common::{BinaryFilterType, UnaryFilterType};
    use crate::pine_syntax::ast::*;

    #[test]
    fn build_from_query() {
        let pine = from("users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!("users", query.from);
    }

    #[test]
    fn build_with_limit() {
        let pine = with_limit("100");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(100, query.limit);
    }

    #[test]
    fn double_limits_overrides_previous_limit() {
        let mut pine = with_limit("100");
        append_operation(&mut pine, AstOperation::Limit(node(Value::Numeric("200"))));

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(200, query.limit);
    }

    #[test]
    fn build_select_query() {
        let pine = select(&["id", "name"], "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.selections[0], ("users", "id"));
        assert_eq!(query.selections[1], ("users", "name"));
    }

    #[test]
    fn build_filter_query() {
        let rhs = ResultColumn::Column(node(AstColumnIdentifier::Implicit(node("id"))));
        let lhs = ResultColumn::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

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
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(
            query.filters[0],
            SqlFilter::Unary(("users", "id").into(), UnaryFilterType::IsNull)
        );
    }

    #[test]
    fn build_filter_query_with_explicit_column() {
        let rhs = ResultColumn::Column(node(AstColumnIdentifier::Explicit(
            node("users"),
            node("id"),
        )));
        let lhs = ResultColumn::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

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
        let query = query_builder.build(&pine).unwrap();

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
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.order[0], SqlOrder::Ascending(("users", "id").into()));
        assert_eq!(query.order[1], SqlOrder::Descending(("3").into()));
    }

    fn make_equals(
        rhs: ResultColumn<'static>,
        lhs: ResultColumn<'static>,
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
                        node(ResultColumn::Column(node(AstColumnIdentifier::Implicit(
                            node(*c),
                        ))))
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

    use crate::query::structure::ResultColumn as QuerySelection;
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

    impl From<(&str, &str)> for SqlResultColumn {
        fn from(other: (&str, &str)) -> SqlResultColumn {
            SqlResultColumn::Column(QualifiedColumnIdentifier {
                table: other.0.to_string(),
                column: other.1.to_string(),
            })
        }
    }

    impl From<&str> for SqlResultColumn {
        fn from(other: &str) -> SqlResultColumn {
            SqlResultColumn::Value(other.to_string())
        }
    }
}
