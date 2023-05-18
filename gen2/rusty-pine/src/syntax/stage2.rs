//! Stage 2 representation has a list of one node per "Pine"
//! For example "users 1 | s: id" would be represented by:
//!
//! # Examples
//! ```rust
//! # let (pine1, pine2) = (0,0); // ignore this
//! vec![
//!     pine1,
//!     pine2,
//! ]
//! ```
//!
//! The data is not yet hierarchical.
//!
//! Since this is just a more convenient way of representing the source Pest info, it's not possible
//! to fail to parse.
use crate::syntax::stage1::{Rule, Stage1Rep};
use crate::syntax::OptionalInput::Specified;
use crate::syntax::{OptionalInput, Position, Positioned, SqlIdentifierInput, TableInput};
use pest::iterators::{Pair, Pairs};
use pest::Span;

/// ```rust
/// # use crate::syntax::stage1::parse_stage1;
/// # let stage1_rep = parse_stage1("name").unwrap();
/// let stage2_rep = stage2_rep.into();
/// ```
///
pub struct Stage2Rep<'a> {
    pub input: &'a str,
    pub pines: Vec<Positioned<Stage2Pine<'a>>>,
}

pub enum Stage2Pine<'a> {
    Base { table: TableInput<'a> },
}

impl<'a> From<Stage1Rep<'a>> for Stage2Rep<'a> {
    fn from(stage1: Stage1Rep<'a>) -> Self {
        let root_node = stage1.pest;
        // println!("{:#?}", root_node);
        let pines = translate_root(root_node);

        return Stage2Rep {
            input: stage1.input,
            pines,
        };
    }
}

fn translate_root(mut pairs: Pairs<Rule>) -> Vec<Positioned<Stage2Pine>> {
    let first_pair = pairs.next().expect("Impossible due to pest parsing");
    assert_eq!(Rule::root, first_pair.as_rule());
    assert!(pairs.next().is_none());

    vec![translate_base(first_pair.into_inner())]
}

fn translate_base(mut pairs: Pairs<Rule>) -> Positioned<Stage2Pine> {
    let base_pair = pairs.next().expect("Impossible due to pest parsing");
    assert_eq!(Rule::base, base_pair.as_rule());
    assert_eq!(Rule::EOI, pairs.next().expect("Expect EOI").as_rule());

    let position: Position = base_pair.as_span().into();
    let table_name = translate_table(base_pair.into_inner());

    position.holding(Stage2Pine::Base { table: table_name })
}

fn translate_table(mut pairs: Pairs<Rule>) -> TableInput {
    let name_pair = pairs.next().expect("Expected sql_name");

    assert!(pairs.next().is_none());

    let mut inners = name_pair.into_inner();
    let inner = inners.next().expect("No pairs should be invalid syntax");
    assert!(
        inners.next().is_none(),
        "Multiple pairs should be invalid syntax"
    );

    match inner.as_rule() {
        Rule::sql_name => translate_table_sql_name(inner),
        Rule::db_and_table_names => translate_db_and_table_names(inner),
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

fn translate_sql_name(pair: Pair<Rule>) -> SqlIdentifierInput {
    assert_eq!(Rule::sql_name, pair.as_rule());

    let position = pair.as_span().into();

    SqlIdentifierInput {
        name: pair.as_str(),
        position,
    }
}

fn translate_db_and_table_names(pair: Pair<Rule>) -> TableInput {
    assert_eq!(Rule::db_and_table_names, pair.as_rule());

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

impl From<pest::Span<'_>> for Position {
    fn from(span: Span) -> Self {
        Position {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::syntax::stage1::parse_stage1;
    use crate::syntax::stage2::{Stage2Pine, Stage2Rep};
    use crate::syntax::{OptionalInput, Position, SqlIdentifierInput, TableInput};

    #[test]
    fn test_simple_parse() {
        let stage1 = parse_stage1("name").unwrap();
        // println!("{:#?}", stage1);
        let stage2: Stage2Rep = stage1.into();

        assert_eq!("name", stage2.input);
        assert_eq!(1, stage2.pines.len());

        let base = &stage2.pines[0];
        assert!(matches!(
            base.node,
            Stage2Pine::Base {
                table: TableInput {
                    database: OptionalInput::Implicit,
                    table: SqlIdentifierInput {
                        name: "name",
                        position: Position { start: 0, end: 4 }
                    },
                    position: Position { start: 0, end: 4 },
                }
            }
        ));
        assert_eq!(base.position, Position { start: 0, end: 4 });
    }
}
