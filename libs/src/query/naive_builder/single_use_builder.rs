use super::BuildResult;
use crate::error::Position;
use crate::error::SyntaxError;
use crate::pine_syntax::ast::{
    ColumnIdentifier as AstColumnIdentifier, ColumnName, Filter as AstFilter,
    FunctionOperand as AstFunctionOperand, MetaOperation, Node, Operand as AstOperand,
    Operation as AstOperation, Order as AstOrder, Pine, TableName as AstTableName, TableName,
    Value as AstValue,
};
use crate::query::{
    Filter as SqlFilter, FunctionOperand, Join, JoinSpec, Operand as SqlOperand, Order as SqlOrder,
    QualifiedColumnIdentifier, Query, Renderable,
};
use log::{debug, info};

pub struct SingleUseQueryBuilder<'a> {
    /// The pine we're going through
    pine: &'a Node<Pine<'a>>,

    /// This is the query currenly being built.
    query: Query,

    /// Holds the initial table, pines that don't have one are invalid
    from_table: Option<String>,

    /// At the end of the build process, we may add a select for last_joined_table.*
    implicit_select_all_from_last_table: bool,
}

impl<'a> SingleUseQueryBuilder<'a> {
    pub fn new(pine: &'a Node<Pine>) -> SingleUseQueryBuilder<'a> {
        SingleUseQueryBuilder {
            pine,
            from_table: None,
            implicit_select_all_from_last_table: false,
            query: Default::default(),
        }
    }

    pub fn build(mut self) -> BuildResult {
        info!("Building query object from initial representation");

        for operation_node in self.pine {
            debug!("Applying {:?}", operation_node);
            self.apply_operation(operation_node)?;
        }

        self.finalize();

        info!("Done");

        Ok(Renderable::Query(self.query))
    }

    fn apply_operation(&mut self, operation_node: &Node<AstOperation>) -> InternalResult {
        match operation_node.inner {
            AstOperation::From(ref table) => self.apply_from(table),
            AstOperation::Join(ref table) => self.apply_join(table),
            AstOperation::ExplicitJoin(ref table, ref column) => {
                self.apply_explicit_join(table, column)
            }
            AstOperation::Select(ref selections) => self.apply_selections(selections)?,
            AstOperation::Unselect(ref selections) => self.apply_unselections(selections)?,
            AstOperation::Filter(ref filters) => self.apply_filters(filters)?,
            AstOperation::GroupBy(ref group_by) => self.apply_group_by(group_by)?,
            AstOperation::Order(ref orders) => self.apply_orders(orders)?,
            AstOperation::Limit(ref limit) => self.apply_limit(limit)?,

            AstOperation::Meta(ref meta) => self.apply_meta_operation(meta),
        };

        Ok(())
    }

    fn apply_from(&mut self, table: &Node<AstTableName>) {
        debug!("Found from: {:?}", table);

        if let Some(table) = &self.from_table {
            self.query.joins.push(Join::Auto(table.clone()));
        }

        self.from_table = Some(table.inner.to_string());
        self.implicit_select_all_from_last_table = true;
    }

    fn apply_join(&mut self, table: &Node<AstTableName>) {
        debug!("Found join: {:?}", table);

        self.apply_from(table);
    }

    fn apply_explicit_join(&mut self, new_from_table: &Node<TableName>, column: &Node<ColumnName>) {
        debug!("Found explicit join: {:?}.{:?}", new_from_table, column);

        let to_table = self
            .from_table
            .replace(new_from_table.inner.to_string())
            .expect("Pines with explicit joins but no from should not be possible");

        self.query.joins.push(Join::Explicit(JoinSpec {
            from: new_from_table.inner.to_string(),
            from_foreign_key: column.inner.to_string(),
            to: to_table,
        }));

        self.implicit_select_all_from_last_table = true;
    }

    fn apply_selections(&mut self, selections: &[Node<AstOperand>]) -> InternalResult {
        debug!("Found select: {:?}", selections);

        let mut selections = self.build_columns(selections)?;
        self.query.selections.append(&mut selections);

        // selecting only some tables MUST clear the implicit most_recent_table.* select
        self.implicit_select_all_from_last_table = false;

        Ok(())
    }

    fn apply_unselections(&mut self, unselect: &[Node<AstOperand>]) -> InternalResult {
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

        let mut filters = filters
            .iter()
            .map(|filter_node| self.translate_filter(filter_node))
            .collect::<Result<Vec<_>, _>>()?;

        self.query.filters.append(&mut filters);

        Ok(())
    }

    fn apply_group_by(&mut self, groups: &[Node<AstOperand>]) -> Result<(), SyntaxError> {
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

    fn add_group_by(&mut self, groups: &[Node<AstOperand>]) -> Result<(), SyntaxError> {
        let mut group_by = self.build_columns(groups)?;
        self.query.group_by.append(&mut group_by);

        Ok(())
    }

    fn apply_orders(&mut self, orders: &[Node<AstOrder>]) -> Result<(), SyntaxError> {
        debug!("Found orders: {:?}", orders);

        if orders.is_empty() {
            return Ok(());
        }

        let mut order = orders
            .iter()
            .map(|order_node| self.translate_order(order_node))
            .collect::<Result<Vec<_>, _>>()?;

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

    fn apply_meta_operation(&self, _: &MetaOperation) {
        // we ignore meta operations here
        // they only have effects as the last part of the Pine
    }

    fn translate_filter(&self, filter_node: &Node<AstFilter>) -> Result<SqlFilter, SyntaxError> {
        debug!("Found filter: {:?}", filter_node);

        Ok(match &filter_node.inner {
            AstFilter::Unary(operand, filter_type) => {
                let operand = self.translate_operand(&operand)?;

                SqlFilter::Unary(operand, *filter_type)
            }
            AstFilter::Binary(lhs, rhs, filter_type) => {
                let lhs = self.translate_operand(&lhs)?;
                let rhs = self.translate_operand(&rhs)?;

                SqlFilter::Binary(lhs, rhs, *filter_type)
            }
        })
    }

    fn translate_operand(&self, operand: &Node<AstOperand>) -> Result<SqlOperand, SyntaxError> {
        let selection = match &operand.inner {
            AstOperand::Value(value) => SqlOperand::Value(value.inner.to_string()),
            AstOperand::Column(column) => {
                let default_table = self.require_table(column.position)?;
                SqlOperand::Column(translate_column_identifier(&column.inner, default_table))
            }
            AstOperand::FunctionCall(function_name, column) => {
                let default_table = self.require_table(column.position)?;
                let function_operand = translate_function_operand(&column.inner, default_table);

                SqlOperand::FunctionCall(
                    function_name.inner.to_string(),
                    function_operand, // translate_column_identifier(&column.inner, default_table),
                )
            }
        };

        Ok(selection)
    }

    fn translate_order(&self, order_node: &Node<AstOrder>) -> Result<SqlOrder, SyntaxError> {
        debug!("Found order: {:?}", order_node);

        let operand = match &order_node.inner {
            AstOrder::Ascending(operand) | AstOrder::Descending(operand) => {
                self.translate_operand(&operand)?
            }
        };

        Ok(match &order_node.inner {
            AstOrder::Ascending(_) => SqlOrder::Ascending(operand),
            AstOrder::Descending(_) => SqlOrder::Descending(operand),
        })
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

        self.query.selections.push(SqlOperand::Column(magic_column));
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

    fn build_columns(&self, columns: &[Node<AstOperand>]) -> Result<Vec<SqlOperand>, SyntaxError> {
        let qualified_columns: Result<Vec<_>, _> = columns
            .iter()
            .map(|selection| self.translate_operand(selection))
            .collect();

        Ok(qualified_columns?)
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

fn translate_function_operand(
    operand: &AstFunctionOperand,
    default_table: &str,
) -> FunctionOperand {
    match operand {
        AstFunctionOperand::Identifier(identifier) => FunctionOperand::Column(
            translate_column_identifier(&identifier.inner, default_table),
        ),
        AstFunctionOperand::Constant(constant) => {
            FunctionOperand::Constant(constant.inner.to_string())
        }
    }
}

type InternalResult = Result<(), SyntaxError>;
