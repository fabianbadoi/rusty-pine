use super::Pine;
use super::{Positioned, Position};
use super::{Operation, TableName, Filter, FilterNode, Condition, ConditionNode, Value};
use super::pest::Rule;
use ::pest::iterators::Pair;

type Node<'a> = Pair<'a, Rule>;

pub fn translate(root_node: Node) -> Pine {
    expect(Rule::pine, &root_node);

    let position = node_to_position(&root_node);
    let operations : Vec<_> = root_node.into_inner().map(translate_operation).collect();

    Pine { position, inner: operations }
}

fn translate_operation(node: Node) -> Positioned<Operation> {
    expect(Rule::operation, &node);
    
    let operation_node = node.into_inner().next()
        .expect("Pest should not have created an operation without an inner");

    let position = node_to_position(&operation_node);
    let operation = match operation_node.as_rule() {
        Rule::from => translate_from(operation_node),
        Rule::select => translate_select(operation_node),
        Rule::filters => translate_filters(operation_node),
        _ => panic!("Expected a operation variant, got '{:?}'", operation_node.as_rule())
    };

    Positioned { position, inner: operation }
}

fn translate_from(node: Node) -> Operation {
    let table_name = translate_sql_name(
        node.into_inner().next().expect("Found from without table name")
    );

    Operation::From(table_name)
}

fn translate_select(node: Node) -> Operation {
    let columns : Vec<_> = node.into_inner().map(translate_sql_name).collect();

    Operation::Select(columns)
}

fn translate_filters(node: Node) -> Operation {
    let filters : Vec<_> = node.into_inner().map(translate_filter).collect();

    Operation::Filter(filters)
}

fn translate_filter(node: Node) -> FilterNode {
    expect(Rule::filter, &node);

    let position = node_to_position(&node);
    let mut parts : Vec<_> = node.into_inner().collect();

    if parts.len() != 2 {
        panic!("Filters must have 2 parts: a column and a condition. Found:\n{:#?}", parts);
    }

    let condition = parts.pop().unwrap();
    let condition = translate_condition(condition);

    let column = parts.pop().expect("First part of a filter must be the column");
    let column = translate_sql_name(column);

    let inner = Filter { column, condition };

    FilterNode { inner, position }
}

fn translate_condition(node: Node) -> ConditionNode {
    expect(Rule::equals, &node);

    let position = node_to_position(&node);
    let value = translate_value(node.into_inner().next().expect("For now, conditions must have a value"));
    let inner = Condition::Equals(value);

    ConditionNode { position, inner }
}

fn translate_value(node: Node) -> Value {
    expect(Rule::numeric_value, &node);

    let position = node_to_position(&node);
    let inner = node.as_str().trim().to_string();

    Value { inner, position }
}

fn translate_sql_name(node: Node) -> TableName {
    expect_one_of(vec![Rule::column_name, Rule::table_name], &node);

    let position = node_to_position(&node);

    TableName { inner: node.as_str().to_string(), position }
}

fn expect(expected_type: Rule, node: &Node) {
    if node.as_rule() != expected_type {
        panic!("node be a '{:?}' expression, found '{:?}'", expected_type, node.as_rule());
    }
}

fn expect_one_of(expected_types: Vec<Rule>, node: &Node) {
    if !expected_types.contains(&node.as_rule()) {
        panic!("node be a one of {:?}, found '{:?}'", expected_types, node.as_rule());
    }
}

fn node_to_position(node: &Node) -> Position {
    let span = node.as_span();

    Position {start: span.start(), end: span.end() }
}
