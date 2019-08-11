use super::{BuildResult, QueryBuilder};
use crate::error::Position;
use crate::error::SyntaxError;
use crate::pine_syntax::ast::{
    ColumnName as AstColumnName, Filter as AstFilter, Operation as AstOperation, Pine,
    TableName as AstTableName, Value as AstValue, Operand, ColumnIdentifier, Node
};
use crate::query::{
    Filter as SqlFilter, QualifiedColumnIdentifier, Query, Operand as SqlOperand
};
use log::{info, debug};

/// Has no concept of context, more complex queries will fail to build
#[derive(Debug)]
pub struct NaiveBuilder;

struct SingleUseQueryBuilder<'a> {
    pine: &'a Node<Pine<'a>>,
    query: Query,
    current_table: Option<String>,
    from_table: Option<String>,
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
            current_table: None,
            from_table: None,
            query: Default::default(),
        }
    }

    fn build(mut self) -> BuildResult {
        info!("Building query object from initial representation");

        for operation_node in self.pine {
            debug!("Applying {:?}", operation_node);
            self.apply_operation(operation_node)?;
        }

        self.finalize()?;

        info!("Done");

        Ok(self.query)
    }

    fn apply_operation(&mut self, operation_node: &Node<AstOperation>) -> InternalResult {
        match operation_node.inner {
            AstOperation::From(ref table) => self.apply_from(table),
            AstOperation::Join(ref table) => self.apply_join(table),
            AstOperation::Select(ref selections) => self.apply_selections(selections)?,
            AstOperation::Filter(ref filters) => self.apply_filters(filters)?,
            AstOperation::Limit(ref limit) => self.apply_limit(limit)?,
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &Node<AstTableName>) {
        debug!("Found from: {:?}", table);

        self.current_table = Some(table.inner.to_string());

        if self.from_table.is_none() {
            self.from_table = Some(table.inner.to_string());
        }
    }

    fn apply_join(&mut self, table: &Node<AstTableName>) {
        debug!("Found join: {:?}", table);

        self.current_table = Some(table.inner.to_string());
        self.query.joins.push(table.inner.to_string());
    }

    fn apply_selections(&mut self, selections: &[Node<AstColumnName>]) -> InternalResult {
        debug!("Found select: {:?}", selections);

        if selections.is_empty() {
            return Ok(());
        }

        let table = self.require_table(selections[0].position)?;
        let mut selections: Vec<_> = selections
            .iter()
            .map(|name_node| name_node.inner.to_string())
            .map(|column| QualifiedColumnIdentifier {
                table: table.to_string(),
                column,
            })
            .collect();

        self.query.selections.append(&mut selections);

        Ok(())
    }

    fn apply_filters(&mut self, filters: &[Node<AstFilter>]) -> Result<(), SyntaxError> {
        debug!("Found where: {:?}", filters);

        if filters.is_empty() {
            return Ok(());
        }

        let table = self.require_table(filters[0].position)?;
        let mut filters: Vec<_> = filters
            .iter()
            .map(|filter_node| {
                translate_filter(filter_node, table)
            })
            .collect();

        self.query.filters.append(&mut filters);

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

    fn finalize(&mut self) -> Result<(), SyntaxError> {
        match self.from_table.clone() {
            Some(table) => {
                self.query.from = table;
                Ok(())
            }
            None => Err(SyntaxError::Positioned {
                message: "Missing a 'from:' statement".to_string(),
                position: self.pine.position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }

    fn require_table(&self, pine_position: Position) -> Result<&str, SyntaxError> {
        match &self.current_table {
            Some(table) => Ok(table.as_str()),
            None => Err(SyntaxError::Positioned {
                message: "Place a 'from:' statement in front fo this".to_string(),
                position: pine_position,
                input: self.pine.inner.pine_string.to_string(),
            }),
        }
    }
}

fn translate_filter(filter_node: &Node<AstFilter>, default_table: &str) -> SqlFilter {
    debug!("Found filter: {:?}", filter_node);

    match &filter_node.inner {
        AstFilter::Equals(rhs, lhs) => {
            let rhs = translate_operand(&rhs.inner, default_table);
            let lhs = translate_operand(&lhs.inner, default_table);

            SqlFilter::Equals(rhs, lhs)
        }
    }
}

fn translate_operand(operand: &Operand, default_table: &str) -> SqlOperand {
    match operand {
        Operand::Column(column_identifier) => {
            let column = translate_column_identifier(&column_identifier.inner, default_table);
            SqlOperand::Column(column)
        },
        Operand::Value(value) => SqlOperand::Value(value.inner.to_string()),
    }
}

fn translate_column_identifier(identifier: &ColumnIdentifier, default_table: &str) -> QualifiedColumnIdentifier {
    let (table, column) = match identifier {
        ColumnIdentifier::Implicit(column_name)             => (default_table.to_string(), column_name.to_string()),
        ColumnIdentifier::Explicit(table_name, column_name) => (table_name.to_string(),    column_name.to_string()),
    };

    QualifiedColumnIdentifier { table, column }
}

type InternalResult = Result<(), SyntaxError>;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::QualifiedColumnIdentifier;
    use super::{NaiveBuilder, QueryBuilder};
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
        let rhs = Operand::Column(node(ColumnIdentifier::Implicit(node("id"))));
        let lhs = Operand::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(query.filters[0], SqlFilter::Equals(("users", "id").into(), "3".into()));
    }

    #[test]
    fn build_filter_query_with_explicit_column() {
        let rhs = Operand::Column(node(ColumnIdentifier::Explicit(node("users"), node("id"))));
        let lhs = Operand::Value(node(Value::Numeric("3")));
        let pine = make_equals(rhs, lhs, "users");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.filters.len(), 1);

        assert_eq!(query.filters[0], SqlFilter::Equals(("users", "id").into(), "3".into()));
    }

    #[test]
    fn build_join_query() {
        let pine = join("users", "friends");

        let query_builder = NaiveBuilder {};
        let query = query_builder.build(&pine).unwrap();

        assert_eq!(query.from, "users");
        assert_eq!(query.joins[0], "friends");
    }

    fn make_equals(
        rhs: Operand<'static>,
        lhs: Operand<'static>,
        table: &'static str,
    ) -> Node<Pine<'static>> {
        let mut pine = from(table);

        let rhs = node(rhs);
        let lhs = node(lhs);
        let filter = node(Filter::Equals(rhs, lhs));

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
            AstOperation::Select(columns.iter().map(|c| node(*c)).collect()),
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
}
