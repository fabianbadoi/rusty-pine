use crate::engine::syntax::stage1::Rule;
use crate::engine::syntax::OptionalInput::{Implicit, Specified};
use crate::engine::syntax::{ColumnInput, OptionalInput, Position, SqlIdentifierInput, TableInput};
use pest::iterators::Pair;

pub fn translate_column(column: Pair<Rule>) -> ColumnInput {
    assert_eq!(Rule::column, column.as_rule());

    let mut inners = column.into_inner();
    let inner = inners.next().expect("Has to be valid syntax");
    assert!(inners.next().is_none());

    match inner.as_rule() {
        Rule::db_table_column_name => translate_db_table_column_name(inner),
        Rule::table_column_name => translate_table_column_name(inner),
        Rule::column_name => translate_column_name(inner),
        _ => panic!("Unknown column type {:#?}", inner.as_rule()),
    }
}

fn translate_column_name(pair: Pair<Rule>) -> ColumnInput {
    assert_eq!(Rule::column_name, pair.as_rule());

    let position: Position = pair.as_span().into();

    let mut inners = pair.into_inner();

    let table = Implicit;
    let column = translate_sql_name(inners.next().unwrap());

    assert!(inners.next().is_none());

    ColumnInput {
        table,
        column,
        position,
    }
}

fn translate_table_column_name(pair: Pair<Rule>) -> ColumnInput {
    assert_eq!(Rule::table_column_name, pair.as_rule());

    let position: Position = pair.as_span().into();

    let mut inners = pair.into_inner();

    let table_name = translate_sql_name(inners.next().unwrap());
    let table = Specified(TableInput {
        table: table_name,
        position: table_name.position,
        database: Implicit,
    });
    let column = translate_sql_name(inners.next().unwrap());

    ColumnInput {
        table,
        column,
        position,
    }
}

fn translate_db_table_column_name(pair: Pair<Rule>) -> ColumnInput {
    assert_eq!(Rule::db_table_column_name, pair.as_rule());

    let position: Position = pair.as_span().into();

    let mut inners = pair.into_inner();

    let table = Specified(translate_table(inners.next().unwrap()));
    let column = translate_sql_name(inners.next().unwrap());

    ColumnInput {
        table,
        column,
        position,
    }
}

pub fn translate_table(name_pair: Pair<Rule>) -> TableInput {
    let mut inners = name_pair.into_inner();
    let inner = inners.next().expect("No pairs should be invalid syntax");
    assert!(
        inners.next().is_none(),
        "Multiple pairs should be invalid syntax"
    );

    match inner.as_rule() {
        Rule::sql_name => translate_table_sql_name(inner),
        Rule::db_table_name => translate_db_table_name(inner),
        _ => panic!("Unsupported rule: {:?}", inner.as_rule()),
    }
}

fn translate_table_sql_name(pair: Pair<Rule>) -> TableInput {
    assert_eq!(Rule::sql_name, pair.as_rule());

    let position = pair.as_span().into();

    TableInput {
        database: OptionalInput::Implicit,
        table: translate_sql_name(pair),
        position,
    }
}

pub fn translate_sql_name(pair: Pair<Rule>) -> SqlIdentifierInput {
    assert_eq!(Rule::sql_name, pair.as_rule());

    let position = pair.as_span().into();

    SqlIdentifierInput {
        name: pair.as_str(),
        position,
    }
}

fn translate_db_table_name(pair: Pair<Rule>) -> TableInput {
    assert_eq!(Rule::db_table_name, pair.as_rule());

    let position = pair.as_span().into();

    let mut inners = pair.into_inner();
    let db_name_pair = inners.next().expect("No db should be invalid syntax");
    let table_name_pair = inners.next().expect("No table should be invalid syntax");

    TableInput {
        database: Specified(translate_sql_name(db_name_pair)),
        table: translate_sql_name(table_name_pair),
        position,
    }
}
