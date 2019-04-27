use super::ast::*;
use super::pest::Rule;
use ::pest::iterators::Pair;

// Look at the test to better understand the tree structures involved

type PestNode<'a> = Pair<'a, Rule>;

pub fn translate(root_node: PestNode) -> PineNode {
    expect(Rule::pine, &root_node);

    let position = node_to_position(&root_node);
    let operations : Vec<_> = root_node.into_inner().map(translate_operation).collect();
    let inner = Pine { operations };

    PineNode { position, inner }
}

fn translate_operation(node: PestNode) -> OperationNode {
    let position = node_to_position(&node);
    let operation = match node.as_rule() {
        Rule::from => translate_from(node),
        Rule::select => translate_select(node),
        Rule::filters => translate_filters(node),
        _ => panic!("Expected a operation variant, got '{:?}'", node.as_rule())
    };

    OperationNode { position, inner: operation }
}

fn translate_from(node: PestNode) -> Operation {
    let table_name = translate_sql_name(
        node.into_inner().next().expect("Found from without table name")
    );

    Operation::From(table_name)
}

fn translate_select(node: PestNode) -> Operation {
    let columns : Vec<_> = node.into_inner().map(translate_sql_name).collect();

    Operation::Select(columns)
}

fn translate_filters(node: PestNode) -> Operation {
    let filters : Vec<_> = node.into_inner().map(translate_filter).collect();

    Operation::Filter(filters)
}

fn translate_filter(node: PestNode) -> FilterNode {
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

fn translate_condition(node: PestNode) -> ConditionNode {
    expect(Rule::equals, &node);

    let position = node_to_position(&node);
    let value = translate_value(node.into_inner().next().expect("For now, conditions must have a value"));
    let inner = Condition::Equals(value);

    ConditionNode { position, inner }
}

fn translate_value(node: PestNode) -> ValueNode {
    expect(Rule::numeric_value, &node);

    let position = node_to_position(&node);
    let inner = node.as_str().trim().to_string();

    ValueNode { inner, position }
}

fn translate_sql_name(node: PestNode) -> TableNameNode {
    expect_one_of(vec![Rule::column_name, Rule::table_name], &node);

    let position = node_to_position(&node);

    TableNameNode { inner: node.as_str().to_string(), position }
}

fn expect(expected_type: Rule, node: &PestNode) {
    if node.as_rule() != expected_type {
        panic!("node be a '{:?}' expression, found '{:?}'", expected_type, node.as_rule());
    }
}

fn expect_one_of(expected_types: Vec<Rule>, node: &PestNode) {
    if !expected_types.contains(&node.as_rule()) {
        panic!("node be a one of {:?}, found '{:?}'", expected_types, node.as_rule());
    }
}

fn node_to_position(node: &PestNode) -> Position {
    let span = node.as_span();

    Position {start: span.start(), end: span.end() }
}

#[cfg(test)]
mod tests {
    use ::pest::Parser;
    use super::super::pest::PinePestParser;
    use super::super::pest::Rule;

    /// Run this test with `--nocapture` to see a demo of the tree structures involved
    /// For example: `cargo test pine_syntax::pest_tree_translation::tests::show_tree_structures --nocapture`
    #[test]
    fn show_tree_structures() {
        let pine_string = "from: users | select: id";
        let ast = PinePestParser::parse(Rule::pine, pine_string).unwrap().next()
            .expect("Pest should have failed to parse this input");

        let pine = super::translate(ast.clone());

        println!("Pine string: {}", pine_string);
        println!("Pest AST:\n{:#?}", ast);
        println!("-------------------------------------------------------");
        println!("Pine internal pepresentation:\n{:#?}", pine);
    }
}