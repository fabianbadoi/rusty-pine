//! Stage 2 representation has a list of one node per "Pine"
//! For example "users 1 | s: id" would be represented by:
//!
//! ```rust
//! # let (pine1, pine2) = (0,0); // ignore this
//! vec![
//!     pine1,
//!     pine2,
//! ]
//!
//! The data is not yet hierarchical.
//!
//! Since this is just a more convenient way of representing the source Pest info, it's not possible
//! to fail to parse.
use crate::syntax::stage1::{Rule, Stage1Rep};
use crate::syntax::{Position, SqlIdentifier};
use pest::iterators::{Pair, Pairs};
use pest::Span;

pub struct Stage2Rep<'a> {
    pub input: &'a str,
    pub pines: Vec<Stage2Pine<'a>>,
}

pub struct Stage2Pine<'a> {
    pub pine: Stage2PineNode<'a>,
    pub position: Position,
}

pub enum Stage2PineNode<'a> {
    Base { table: SqlIdentifier<'a> },
}

impl<'a> From<Stage1Rep<'a>> for Stage2Rep<'a> {
    fn from(mut stage1: Stage1Rep<'a>) -> Self {
        let root_node = stage1.pest;
        let pines = translate_root(root_node);

        return Stage2Rep {
            input: stage1.input,
            pines,
        };
    }
}

fn translate_root(mut pairs: Pairs<Rule>) -> Vec<Stage2Pine> {
    let first_pair = pairs.next().expect("Impossible due to pest parsing");
    assert_eq!(Rule::root, first_pair.as_rule());
    assert!(pairs.next().is_none());

    vec![translate_base(first_pair.into_inner())]
}

fn translate_base(mut pairs: Pairs<Rule>) -> Stage2Pine {
    let base_pair = pairs.next().expect("Impossible due to pest parsing");
    assert_eq!(Rule::base, base_pair.as_rule());
    assert_eq!(Rule::EOI, pairs.next().expect("Expect EOI").as_rule());

    let position = base_pair.as_span().into();
    let table_name = translate_table(base_pair.into_inner());

    Stage2Pine {
        pine: Stage2PineNode::Base { table: table_name },
        position,
    }
}

fn translate_table(mut pairs: Pairs<Rule>) -> SqlIdentifier {
    let name_pair = pairs.next().expect("Expected sql_name");
    assert_eq!(Rule::sql_name, name_pair.as_rule());
    assert!(pairs.next().is_none());

    SqlIdentifier {
        name: name_pair.as_str(),
        position: name_pair.as_span().into(),
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
    use crate::syntax::stage1::{parse_stage1, Stage1Rep};
    use crate::syntax::stage2::{Stage2PineNode, Stage2Rep};
    use crate::syntax::{Position, SqlIdentifier};

    #[test]
    fn test_simple_parse() {
        let stage1 = parse_stage1("name").unwrap();
        // println!("{:#?}", stage1);
        let stage2: Stage2Rep = stage1.into();

        assert_eq!("name", stage2.input);
        assert_eq!(1, stage2.pines.len());

        let base = &stage2.pines[0];
        assert!(matches!(
            base.pine,
            Stage2PineNode::Base {
                table: SqlIdentifier {
                    name: "name",
                    position: Position { start: 0, end: 4 }
                }
            }
        ));
        assert_eq!(base.position, Position { start: 0, end: 4 });
    }
}
