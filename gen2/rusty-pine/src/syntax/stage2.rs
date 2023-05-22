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
use crate::syntax::{ColumnInput, Position, Positioned, TableInput};
use pest::iterators::{Pair, Pairs};
use pest::Span;

mod identifiers;

/// ```rust
/// # use crate::syntax::stage1::parse_stage1;
/// # let stage1_rep = parse_stage1("name").unwrap();
/// let stage2_rep = stage2_rep.into();
/// ```
///
pub struct Stage2Rep<'a> {
    pub input: &'a str,
    pub pines: PestIterator<'a>,
}

#[derive(Debug)]
pub enum Stage2Pine<'a> {
    Base { table: TableInput<'a> },
    Select(Vec<ColumnInput<'a>>),
}

impl<'a> From<Stage1Rep<'a>> for Stage2Rep<'a> {
    fn from(stage1: Stage1Rep<'a>) -> Self {
        let root_node = stage1.pest;
        let pines = translate_root(root_node);

        Stage2Rep {
            input: stage1.input,
            pines,
        }
    }
}

pub struct PestIterator<'a> {
    inners: Pairs<'a, Rule>,
    base_done: bool,
}

impl<'a> PestIterator<'a> {
    fn new(base: Pairs<'a, Rule>) -> PestIterator {
        Self {
            base_done: false,
            inners: base,
        }
    }
}

impl<'a> Iterator for PestIterator<'a> {
    type Item = Positioned<Stage2Pine<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inners.next();

        if !self.base_done {
            self.base_done = true;

            return Some(translate_base(next.expect("Guaranteed by syntax")));
        }

        match next {
            None => None,
            Some(pair) => translate_pine(pair),
        }
    }
}

fn translate_root(mut pairs: Pairs<Rule>) -> PestIterator {
    let root_pair = pairs.next().expect("Impossible due to pest parsing");

    assert_eq!(Rule::root, root_pair.as_rule());
    assert!(pairs.next().is_none());

    PestIterator::new(root_pair.into_inner())
}

fn translate_base(base_pair: Pair<Rule>) -> Positioned<Stage2Pine> {
    assert_eq!(Rule::base, base_pair.as_rule());

    let position: Position = base_pair.as_span().into();
    let table_name = identifiers::translate_table(base_pair.into_inner().next().unwrap());

    position.holding(Stage2Pine::Base { table: table_name })
}

fn translate_pine(pair: Pair<Rule>) -> Option<Positioned<Stage2Pine>> {
    match pair.as_rule() {
        Rule::select_pine => Some(translate_select(pair)),
        Rule::EOI => None,
        _ => panic!("Unknown pine {:#?}", pair),
    }
}

fn translate_select(select: Pair<Rule>) -> Positioned<Stage2Pine> {
    assert_eq!(Rule::select_pine, select.as_rule());

    let position: Position = select.as_span().into();
    let mut columns = Vec::new();

    for column_pair in select.into_inner() {
        let column = identifiers::translate_column(column_pair);
        columns.push(column);
    }

    position.holding(Stage2Pine::Select(columns))
}

impl From<Span<'_>> for Position {
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
        let mut stage2: Stage2Rep = stage1.into();

        assert_eq!("name", stage2.input);

        let base = &stage2.pines.next().unwrap();
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
        assert_eq!(0..4, base.position);
    }
}
