//! Stage 3 adds implicit information to Pines.
//! For example:
//!     "users | w: id = 3 | s: id"
//!     The where and select pines will also reference the previous table: users.
//!
//! Each pine should have all of the info from the input contained in itself, so future processing
//! does not have to look-backs.
use crate::engine::syntax::stage2::Stage2Rep;
use crate::engine::syntax::stage3::iterator::Stage3Iterator;
use crate::engine::syntax::{Stage4ColumnInput, TableInput};

/// The module covers iterating over our stage2 pines and converting them into stage3 pines
mod iterator;

pub struct Stage3Rep<'a> {
    pub input: &'a str,
    // the iteration code got a bit complex here, so I split it off.
    pub pines: Stage3Iterator<'a>,
}

pub enum Stage3Pine<'a> {
    From { table: TableInput<'a> },
    Select(Vec<Stage3ColumnInput<'a>>),
}

pub type Stage3ColumnInput<'a> = Stage4ColumnInput<'a>; // shh! keep this secret

impl<'a> From<Stage2Rep<'a>> for Stage3Rep<'a> {
    fn from(stage2: Stage2Rep<'a>) -> Self {
        let context = Stage3Iterator::new(stage2.pines);

        Stage3Rep {
            input: stage2.input,
            pines: context,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::engine::syntax::stage1::parse_stage1;
    use crate::engine::syntax::stage2::Stage2Rep;
    use crate::engine::syntax::stage3::{Stage3Pine, Stage3Rep};
    use crate::engine::syntax::{OptionalInput, Position, SqlIdentifierInput, TableInput};

    #[test]
    fn test_simple_convert() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let mut stage3: Stage3Rep = stage2.into();

        assert_eq!("table", stage3.input);

        let first = stage3.pines.next().unwrap();
        assert_eq!(0..5, first.position);

        assert!(matches!(
            first.node,
            Stage3Pine::From {
                table: TableInput {
                    database: OptionalInput::Implicit,
                    table: SqlIdentifierInput {
                        name: "table",
                        position: Position { start: 0, end: 5 },
                    },
                    position: Position { start: 0, end: 5 },
                },
            }
        ))
    }
}
