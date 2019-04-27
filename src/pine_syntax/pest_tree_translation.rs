use super::Pine;
use super::{Positioned, Position};
use super::{Operation, TableName, Filter, FilterNode, Condition, ConditionNode, Value};
use super::pest::Rule;
use ::pest::iterators::Pair as PestPair;

type Pair<'a> = PestPair<'a, Rule>;

pub fn translate(root_pair: Pair) -> Pine {
    expect(Rule::pine, &root_pair);

    let position = pair_to_position(&root_pair);
    let operations : Vec<_> = root_pair.into_inner().map(translate_operation).collect();

    Pine { position, item: operations }
}

fn translate_operation(pair: Pair) -> Positioned<Operation> {
    expect(Rule::operation, &pair);
    
    let operation_pair = pair.into_inner().next()
        .expect("Pest should not have created an operation without an inner");

    let position = pair_to_position(&operation_pair);
    let operation = match operation_pair.as_rule() {
        Rule::from => translate_from(operation_pair),
        Rule::select => translate_select(operation_pair),
        Rule::filters => translate_filters(operation_pair),
        _ => panic!("Expected a operation variant, got '{:?}'", operation_pair.as_rule())
    };

    Positioned { position, item: operation }
}

fn translate_from(pair: Pair) -> Operation {
    let table_name = translate_sql_name(
        pair.into_inner().next().expect("Found from without table name")
    );

    Operation::From(table_name)
}

fn translate_select(pair: Pair) -> Operation {
    let columns : Vec<_> = pair.into_inner().map(translate_sql_name).collect();

    Operation::Select(columns)
}

fn translate_filters(pair: Pair) -> Operation {
    let filters : Vec<_> = pair.into_inner().map(translate_filter).collect();

    Operation::Filter(filters)
}

fn translate_filter(pair: Pair) -> FilterNode {
    expect(Rule::filter, &pair);

    let position = pair_to_position(&pair);
    let mut parts : Vec<_> = pair.into_inner().collect();

    if parts.len() != 2 {
        panic!("Filters must have 2 parts: a column and a condition. Found:\n{:#?}", parts);
    }

    let condition = parts.pop().unwrap();
    let condition = translate_condition(condition);

    let column = parts.pop().expect("First part of a filter must be the column");
    let column = translate_sql_name(column);

    let item = Filter { column, condition };

    FilterNode { item, position }
}

fn translate_condition(pair: Pair) -> ConditionNode {
    expect(Rule::equals, &pair);

    let position = pair_to_position(&pair);
    let value = translate_value(pair.into_inner().next().expect("For now, conditions must have a value"));
    let item = Condition::Equals(value);

    ConditionNode { position, inner }
}

fn translate_value(pair: Pair) -> Value {
    expect(Rule::numeric_value, &pair);

    let position = pair_to_position(&pair);
    let item = pair.as_str().trim().to_string();

    Value { item, position }
}

fn translate_sql_name(pair: Pair) -> TableName {
    expect_one_of(vec![Rule::column_name, Rule::table_name], &pair);

    let position = pair_to_position(&pair);

    TableName { item: pair.as_str().to_string(), position }
}

fn expect(expected_type: Rule, pair: &Pair) {
    if pair.as_rule() != expected_type {
        panic!("Token be a '{:?}' expression, found '{:?}'", expected_type, pair.as_rule());
    }
}

fn expect_one_of(expected_types: Vec<Rule>, pair: &Pair) {
    if !expected_types.contains(&pair.as_rule()) {
        panic!("Token be a one of {:?}, found '{:?}'", expected_types, pair.as_rule());
    }
}

fn pair_to_position(pair: &Pair) -> Position {
    let span = pair.as_span();

    Position {start: span.start(), end: span.end() }
}
