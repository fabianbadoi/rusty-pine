//! Translates the Pest AST into the first intermediary representation.
//!
//! Panicking in this module is accepted when the unexpected state should be illegal in terms of
//! the Pine grammar.
use super::ast::*;
use super::pest;
use super::pest::Rule;
use super::PineParser;
use crate::common::{BinaryFilterType, UnaryFilterType};
use crate::error::{Position, SyntaxError};
use ::pest::error::Error as PestError;
use ::pest::iterators::Pair;
use ::pest::Parser;
use log::info;
use std::convert::From;

// Look at the test at the end of this file to better understand
// the tree structures involved.

type PestNode<'a> = Pair<'a, Rule>;

#[derive(Debug)]
pub struct PestPineParser;

impl PineParser for &PestPineParser {
    fn parse(self, input: &str) -> Result<Node<Pine>, SyntaxError> {
        let ast = pest::PinePestParser::parse(pest::Rule::pine, input)?
            .next()
            .expect("Pest should have failed to parse this input");

        let pine = translate(ast, input);

        Ok(pine)
    }
}

pub fn translate<'a>(root_node: PestNode<'a>, input: &'a str) -> Node<Pine<'a>> {
    let translator = Translator::new();
    translator.translate(root_node, input)
}

/// Panics if the rule is not found.
/// This is a macro so we get the proper line number of errors.
macro_rules! expect_rule {
    ($rule:expr, $node:expr) => {{
        if $node.as_rule() != $rule {
            panic!("node be a '{:?}' expression, found '{:?}'", $rule, $node);
        }
    }};
}

/// Panics if node is not of those rules.
/// This is a macro so we get the proper line number of errors.
macro_rules! expect_rule_in {
    ($rules:expr, $node:expr) => {{
        if !$rules.contains(&$node.as_rule()) {
            panic!(
                "node be a one of {:?}, found '{:?}'",
                $rules,
                $node.as_rule()
            );
        }
    }};
}

struct Translator {
    has_from: bool,
}

impl Translator {
    pub fn new() -> Translator {
        Translator { has_from: false }
    }

    pub fn translate<'a>(mut self, root_node: PestNode<'a>, input: &'a str) -> Node<Pine<'a>> {
        info!(
            "Parsing pine query into first internal representation: {}",
            input
        );

        expect_rule!(Rule::pine, root_node);

        let position = position(&root_node);
        let operations: Vec<_> = root_node
            .into_inner()
            .flat_map(|node| self.translate_operation(node))
            .collect();
        let inner = Pine {
            operations,
            pine_string: input,
        };

        info!("Parse done");
        Node { position, inner }
    }

    fn translate_operation<'a>(&mut self, node: PestNode<'a>) -> Vec<Node<Operation<'a>>> {
        let position = position(&node);
        let operations = match node.as_rule() {
            Rule::from => self.translate_from(node),
            Rule::select => self.translate_selections(node),
            Rule::unselect => self.translate_unselect(node),
            Rule::filters => self.translate_filters(node),
            Rule::compound_expression => self.translate_compound_expression(node),
            Rule::join => self.translate_join(node),
            Rule::group_by => self.translate_group_by(node),
            Rule::order => self.translate_order(node),
            Rule::limit => self.translate_limit(node),
            Rule::EOI => Vec::new(),
            _ => panic!("Expected a operation variant, got '{:?}'", node.as_rule()),
        };

        operations
            .into_iter()
            .map(|inner| Node { position, inner })
            .collect()
    }

    fn translate_from<'a>(&mut self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        self.has_from = true;

        let table_name = translate_sql_name(
            node.into_inner()
                .next()
                .expect("Found from without table name"),
        );

        vec![Operation::From(table_name)]
    }

    fn translate_join<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let table_name = translate_sql_name(
            node.into_inner()
                .next()
                .expect("Found from without table name"),
        );

        vec![Operation::Join(table_name)]
    }

    fn translate_selections<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let columns: Vec<_> = node.into_inner().map(translate_result_column).collect();

        vec![Operation::Select(columns)]
    }

    fn translate_unselect<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let columns: Vec<_> = node.into_inner().map(translate_identified_column).collect();

        vec![Operation::Unselect(columns)]
    }

    fn translate_group_by<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let operands: Vec<_> = node.into_inner().map(translate_operand).collect();

        vec![Operation::GroupBy(operands)]
    }

    fn translate_order<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let orders = node.into_inner().map(translate_ordering).collect();

        vec![Operation::Order(orders)]
    }

    fn translate_limit<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let value_node = node.into_inner().next().unwrap();
        let limit = translate_numeric_value(value_node);

        vec![Operation::Limit(limit)]
    }

    fn translate_filters<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let filters: Vec<_> = node.into_inner().map(translate_filter).collect();

        vec![Operation::Filter(filters)]
    }

    fn translate_compound_expression<'a>(&mut self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let inner = node.clone().into_inner();

        expect_rule!(Rule::table_name, inner.peek().unwrap());

        let vec_with_from = if self.has_from {
            self.translate_join(node)
        } else {
            self.translate_from(node)
        };

        let mut operations = vec_with_from;

        let filters: Vec<_> = inner.skip(1).map(translate_filter_or_implicit_id).collect();

        if !filters.is_empty() {
            operations.push(Operation::Filter(filters));
        }

        operations
    }
}

fn translate_result_column(node: PestNode) -> Node<Selection> {
    expect_rule!(Rule::result_column, node);

    let inner = node.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::identified_column => translate_select_column(inner),
        Rule::function_call => translate_function_call(inner),
        Rule::value => translate_select_value(inner),
        _ => panic!("Unexpected select rule: {:?}", inner.as_rule()),
    }
}

fn translate_select_column(node: PestNode) -> Node<Selection> {
    expect_rule!(Rule::identified_column, node);

    let position = position(&node);
    let column = translate_identified_column(node);
    let inner = Selection::Column(column);

    Node { position, inner }
}

fn translate_function_call(node: PestNode) -> Node<Selection> {
    let position = position(&node);

    let mut parts = node.into_inner();

    let function_name = translate_sql_name(parts.next().unwrap());
    let column = translate_identified_column(parts.next().unwrap());

    let inner = Selection::FunctionCall(function_name, column);

    Node { position, inner }
}

fn translate_select_value(node: PestNode) -> Node<Selection> {
    expect_rule!(Rule::value, node);

    let position = position(&node);
    let value  = translate_value(node);
    let inner = Selection::Value(value);

    Node { position, inner }
}

fn translate_filter(node: PestNode) -> Node<Filter> {
    let inner = node.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::binary_filter => translate_binary_filter(inner),
        Rule::unary_filter => translate_unary_filter(inner),
        _ => panic!("Unexpected condition rule: {:?}", inner.as_rule()),
    }
}

fn translate_binary_filter(node: PestNode) -> Node<Filter> {
    expect_rule!(Rule::binary_filter, node);

    let position = position(&node);

    let mut parts = node.into_inner();

    let lhs = translate_operand(parts.next().unwrap());
    let operator = parts.next().unwrap();
    let rhs = translate_operand(parts.next().unwrap());

    let filter_type = match operator.as_rule() {
        Rule::optr_lt => BinaryFilterType::LesserThan,
        Rule::optr_lte => BinaryFilterType::LesserThanOrEquals,
        Rule::optr_eq => BinaryFilterType::Equals,
        Rule::optr_ne => BinaryFilterType::NotEquals,
        Rule::optr_gt => BinaryFilterType::GreaterThan,
        Rule::optr_gte => BinaryFilterType::GreaterThanOrEquals,
        _ => panic!("Unexpected rule: {:?}", operator.as_rule()),
    };

    let inner = Filter::Binary(lhs, rhs, filter_type);

    Node { position, inner }
}

fn translate_unary_filter(node: PestNode) -> Node<Filter> {
    expect_rule!(Rule::unary_filter, node);

    let position = position(&node);
    let inner = node.into_inner().next().unwrap();
    let filter_type = match inner.as_rule() {
        Rule::filter_is_null => UnaryFilterType::IsNull,
        Rule::filter_is_not_null => UnaryFilterType::IsNotNull,
        _ => panic!("Unexpected unary filter rule: {:?}", inner.as_rule()),
    };
    let operand = translate_operand(inner.into_inner().next().unwrap());

    let filter = Filter::Unary(operand, filter_type);

    Node {
        position,
        inner: filter,
    }
}

fn translate_identified_column(node: PestNode) -> Node<ColumnIdentifier> {
    expect_rule!(Rule::identified_column, node);

    let inner = node.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::column_name => translate_implicit_column(inner),
        Rule::fully_qualified_column => translate_explicit_column(inner),
        _ => panic!("Unexpected rule: {:?}", inner.as_rule()),
    }
}

fn translate_ordering(node: PestNode) -> Node<Order> {
    expect_rule!(Rule::ordering, node);

    let position = position(&node);
    let inner = node.into_inner().next().unwrap();

    let ordering_type_rule = inner.as_rule();

    let operand = inner.into_inner().next().unwrap();
    let operand = translate_operand(operand);

    let ordering = match ordering_type_rule {
        Rule::ordering_asc => Order::Ascending(operand),
        Rule::ordering_desc => Order::Descending(operand),
        _ => unimplemented!("Unexpected node type {:?}", ordering_type_rule),
    };

    Node {
        inner: ordering,
        position,
    }
}

fn translate_operand(node: PestNode) -> Node<Operand> {
    expect_rule!(Rule::operand, node);

    let inner = node.into_inner().next().unwrap();
    let position = position(&inner);

    let operand = match inner.as_rule() {
        Rule::value => Operand::Value(translate_value(inner)),
        Rule::identified_column => Operand::Column(translate_identified_column(inner)),
        _ => panic!("Unexpected rule: {:?}", inner.as_rule()),
    };

    Node {
        position,
        inner: operand,
    }
}

fn translate_implicit_id_equals(node: PestNode) -> Node<Filter> {
    let position = position(&node);

    let column = ColumnIdentifier::Implicit(Node {
        position,
        inner: "id",
    });
    let rhs = Operand::Column(Node {
        position,
        inner: column,
    });
    let rhs = Node {
        position,
        inner: rhs,
    };

    let lhs = Node {
        position,
        inner: Operand::Value(translate_value(node)),
    };

    let filter = Filter::Binary(lhs, rhs, BinaryFilterType::Equals);

    Node {
        position,
        inner: filter,
    }
}

fn translate_implicit_column(node: PestNode) -> Node<ColumnIdentifier> {
    expect_rule!(Rule::column_name, node);

    let position = position(&node);
    let inner = ColumnIdentifier::Implicit(translate_sql_name(node));

    Node { position, inner }
}

fn translate_explicit_column(node: PestNode) -> Node<ColumnIdentifier> {
    expect_rule!(Rule::fully_qualified_column, node);

    let position = position(&node);

    let mut parts = node.into_inner();

    let table = translate_sql_name(parts.next().unwrap());
    let column = translate_sql_name(parts.next().unwrap());
    let inner = ColumnIdentifier::Explicit(table, column);

    Node { position, inner }
}

fn translate_value(node: PestNode) -> Node<Value> {
    expect_rule!(Rule::value, node);

    let inner = node.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::numeric_value => translate_numeric_value(inner),
        Rule::string_value => translate_string_value(inner),
        _ => {
            expect_rule_in!([Rule::numeric_value, Rule::string_value], inner);
            panic!("previous statement should have panicked")
        }
    }
}

fn translate_string_value(node: PestNode) -> Node<Value> {
    let inner = node
        .into_inner()
        .next()
        .expect("String values MUST have child nodes");
    expect_rule_in!(
        [Rule::apostrophe_string_value, Rule::quote_string_value],
        inner
    );

    let position = position(&inner);
    let inner = Value::String(inner.as_str().trim());

    Node { inner, position }
}

fn translate_numeric_value(node: PestNode) -> Node<Value> {
    expect_rule!(Rule::numeric_value, node);

    let position = position(&node);
    let inner = Value::Numeric(node.as_str().trim());

    Node { inner, position }
}

fn translate_sql_name(node: PestNode) -> Node<TableName> {
    expect_rule_in!(
        [Rule::column_name, Rule::table_name, Rule::function_name],
        node
    );

    let position = position(&node);

    Node {
        inner: node.as_str(),
        position,
    }
}

fn position(node: &PestNode) -> Position {
    let span = node.as_span();

    Position {
        start: span.start(),
        end: span.end(),
    }
}

impl From<PestError<Rule>> for SyntaxError {
    fn from(pest_error: PestError<Rule>) -> Self {
        let message = format!("{}", pest_error);

        SyntaxError::Detailed(message)
    }
}

fn translate_filter_or_implicit_id(node: PestNode) -> Node<Filter> {
    match node.as_rule() {
        Rule::value => translate_implicit_id_equals(node),
        Rule::filter => translate_filter(node),
        _ => panic!("can't treat this node as a filter: {:#?}", node),
    }
}

#[cfg(test)]
mod tests {
    use super::super::pest::PinePestParser as AstParser;
    use super::super::pest::Rule;
    use super::{Operation, PestPineParser, PineParser};
    use ::pest::Parser;

    /// Run this test with `--nocapture` to see a demo of the tree structures involved
    /// For example: `cargo test pine_syntax::pest_tree_translation::tests::show_tree_structures -- --nocapture`
    #[test]
    fn show_tree_structures() {
        let pine_string = "from: users | select: id";
        let ast = AstParser::parse(Rule::pine, pine_string)
            .unwrap()
            .next()
            .expect("Pest should have failed to parse this input");

        let pine = super::translate(ast.clone(), pine_string);

        println!("Pine string: {}", pine_string);
        println!("Pest AST:\n{:#?}", ast);
        println!("-------------------------------------------------------");
        println!("Pine internal pepresentation:\n{:#?}", pine);
    }

    #[test]
    fn parsing_simple_form_statement() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("from: users | select: id name | where: id = 3 x = 4")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[2].inner.get_name());

        if let Operation::From(ref table_name) = pine_node.inner.operations[0].inner {
            assert_eq!("users", table_name.inner);
        }
    }

    #[test]
    fn parse_simple_compound_statement() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users | friends").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("join", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn parse_compound_statement() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users | friends id = 1").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("join", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[2].inner.get_name());
    }

    #[test]
    fn parse_complex_compound_statement() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users 1 parent=33 | friends id = 1").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("join", pine_node.inner.operations[2].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[3].inner.get_name());

        if let Operation::Filter(ref filters) = pine_node.inner.operations[1].inner {
            assert_eq!(2, filters.len());
        }
    }

    #[test]
    fn parse_limit_expression() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("from: users | select: id name | limit: 5")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("limit", pine_node.inner.operations[2].inner.get_name());

        if let Operation::From(ref limit) = pine_node.inner.operations[2].inner {
            assert_eq!("5", limit.inner);
        }
    }

    #[test]
    fn parsing_compound_expression() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("users id = 3 | select: id name | where: x = 4")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[2].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[3].inner.get_name());

        if let Operation::From(ref table_name) = pine_node.inner.operations[0].inner {
            assert_eq!("users", table_name.inner);
        }
    }

    #[test]
    fn parsing_join_expression() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("users id = 3 | select: id name | join: friends | where: x = 4")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[2].inner.get_name());
        assert_eq!("join", pine_node.inner.operations[3].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[4].inner.get_name());

        if let Operation::Join(ref table_name) = pine_node.inner.operations[3].inner {
            assert_eq!("friends", table_name.inner);
        }
    }

    #[test]
    fn parsing_compound_join_expression() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("users id = 3 | select: id name | friends stylish = 1 | where: x = 4")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("select", pine_node.inner.operations[2].inner.get_name());
        assert_eq!("join", pine_node.inner.operations[3].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[4].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[5].inner.get_name());

        if let Operation::Join(ref table_name) = pine_node.inner.operations[3].inner {
            assert_eq!("friends", table_name.inner);
        }
    }

    #[test]
    fn parse_quote_string_value() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users id = \"a string\"").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn parse_apostrophe_string_value() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users id = 'a string'").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn compare_two_columns() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users id = parentId").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn compare_two_columns_from_different_tables() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users id = other.parentId").unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn order() {
        let parser = PestPineParser {};
        let pine_node = parser
            .parse("users | order: id asc, users.id, friends.friendId DESC, 3-, u+")
            .unwrap();

        assert_eq!("from", pine_node.inner.operations[0].inner.get_name());
        assert_eq!("order", pine_node.inner.operations[1].inner.get_name());
    }

    #[test]
    fn is_null() {
        let parser = PestPineParser {};
        let pine_node = parser.parse("users parent? | w: parent!?").unwrap();

        assert_eq!("filter", pine_node.inner.operations[1].inner.get_name());
        assert_eq!("filter", pine_node.inner.operations[2].inner.get_name());
    }
}
