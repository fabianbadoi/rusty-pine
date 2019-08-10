use super::ast::*;
use super::pest;
use super::pest::Rule;
use super::PineParser;
use crate::error::{Position, SyntaxError};
use ::pest::error::Error as PestError;
use ::pest::iterators::Pair;
use ::pest::Parser;
use std::convert::From;
use log::info;

// Look at the test at the end of this file to better understand
// the tree structures involved.

type PestNode<'a> = Pair<'a, Rule>;

#[derive(Debug)]
pub struct PestPineParser;

impl PineParser for &PestPineParser {
    fn parse(self, input: &str) -> Result<PineNode, SyntaxError> {
        let ast = pest::PinePestParser::parse(pest::Rule::pine, input)?
            .next()
            .expect("Pest should have failed to parse this input");

        let pine = translate(ast, input);

        Ok(pine)
    }
}

pub fn translate<'a>(root_node: PestNode<'a>, input: &'a str) -> PineNode<'a> {
    let translator = Translator::new();
    translator.translate(root_node, input)
}

struct Translator {
    has_from: bool,
}

impl Translator {
    pub fn new() -> Translator {
        Translator { has_from: false }
    }

    pub fn translate<'a>(mut self, root_node: PestNode<'a>, input: &'a str) -> PineNode<'a> {
        info!("Parsing pine query into first internal representation: {}", input);

        expect(Rule::pine, &root_node);

        let position = node_to_position(&root_node);
        let operations: Vec<_> = root_node
            .into_inner()
            .flat_map(|node| self.translate_operation(node))
            .collect();
        let inner = Pine {
            operations,
            pine_string: input,
        };

        info!("Parse done");
        PineNode { position, inner }
    }

    fn translate_operation<'a>(&mut self, node: PestNode<'a>) -> Vec<OperationNode<'a>> {
        let position = node_to_position(&node);
        let operations = match node.as_rule() {
            Rule::from => self.translate_from(node),
            Rule::select => self.translate_select(node),
            Rule::filters => self.translate_filters(node),
            Rule::compound_expression => self.translate_compound_expression(node),
            Rule::join => self.translate_join(node),
            Rule::limit => self.translate_limit(node),
            Rule::EOI => Vec::new(),
            _ => panic!("Expected a operation variant, got '{:?}'", node.as_rule()),
        };

        operations
            .into_iter()
            .map(|inner| OperationNode { position, inner })
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

    fn translate_select<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let columns: Vec<_> = node.into_inner().map(translate_sql_name).collect();

        vec![Operation::Select(columns)]
    }

    fn translate_limit<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let limit = translate_value(node.into_inner().next().unwrap());

        vec![Operation::Limit(limit)]
    }

    fn translate_filters<'a>(&self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let filters: Vec<_> = node.into_inner().map(translate_filter).collect();

        vec![Operation::Filter(filters)]
    }

    fn translate_compound_expression<'a>(&mut self, node: PestNode<'a>) -> Vec<Operation<'a>> {
        let inner = node.clone().into_inner();

        expect(Rule::table_name, &inner.peek().unwrap());

        let vec_with_from = if self.has_from {
            self.translate_join(node)
        } else {
            self.translate_from(node)
        };

        let mut operations = vec_with_from;

        let filters: Vec<_> = inner
            .skip(1)
            .map(|f| translate_filter_or_implicit_id(f))
            .collect();
        if filters.len() > 0 {
            operations.push(Operation::Filter(filters));
        }

        operations
    }
}

fn translate_filter(node: PestNode) -> FilterNode {
    expect(Rule::filter, &node);

    let position = node_to_position(&node);
    let mut parts: Vec<_> = node.into_inner().collect();

    if parts.len() != 2 {
        panic!(
            "Filters must have 2 parts: a column and a condition. Found:\n{:#?}",
            parts
        );
    }

    let condition = parts.pop().unwrap();
    let condition = translate_condition(condition);

    let column = parts
        .pop()
        .expect("First part of a filter must be the column");
    let column = translate_sql_name(column);

    let inner = Filter { column, condition };

    FilterNode { inner, position }
}

fn translate_condition(node: PestNode) -> ConditionNode {
    expect(Rule::equals, &node);

    let position = node_to_position(&node);
    let value = translate_value(
        node.into_inner()
            .next()
            .expect("For now, conditions must have a value"),
    );
    let inner = Condition::Equals(value);

    ConditionNode { position, inner }
}

fn translate_implicit_id_equals(node: PestNode) -> FilterNode {
    let position = node_to_position(&node);
    let filter = Filter {
        column: TableNameNode {
            position,
            inner: "id",
        },
        condition: ConditionNode {
            position,
            inner: Condition::Equals(translate_value(node)),
        },
    };

    FilterNode {
        position,
        inner: filter,
    }
}

fn translate_value(node: PestNode) -> ValueNode {
    expect(Rule::numeric_value, &node);

    let position = node_to_position(&node);
    let inner = node.as_str().trim();

    ValueNode { inner, position }
}

fn translate_sql_name(node: PestNode) -> TableNameNode {
    expect_one_of(vec![Rule::column_name, Rule::table_name], &node);

    let position = node_to_position(&node);

    TableNameNode {
        inner: node.as_str(),
        position,
    }
}

fn expect(expected_type: Rule, node: &PestNode) {
    if node.as_rule() != expected_type {
        panic!(
            "node be a '{:?}' expression, found '{:?}'",
            expected_type,
            node.as_rule()
        );
    }
}

fn expect_one_of(expected_types: Vec<Rule>, node: &PestNode) {
    if !expected_types.contains(&node.as_rule()) {
        panic!(
            "node be a one of {:?}, found '{:?}'",
            expected_types,
            node.as_rule()
        );
    }
}

fn node_to_position(node: &PestNode) -> Position {
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

fn translate_filter_or_implicit_id(node: PestNode) -> FilterNode {
    match node.as_rule() {
        Rule::numeric_value => translate_implicit_id_equals(node),
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
            .parse("from: users | select: id, name | where: id = 3 x = 4")
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
            .parse("from: users | select: id, name | limit: 5")
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
            .parse("users id = 3 | select: id, name | where: x = 4")
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
            .parse("users id = 3 | select: id, name | join: friends | where: x = 4")
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
            .parse("users id = 3 | select: id, name | friends stylish = 1 | where: x = 4")
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
}
