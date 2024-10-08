//! Stage 3 adds implicit information to Pines.
//! For example:
//!     "users | w: id = 3 | s: id"
//!     The where and select pines will also reference the previous table: users.
//!
//! Each pine should have all of the info from the input contained in itself, so future processing
//! does not have to look-backs.
use crate::engine::syntax::stage2::Stage2Rep;
use crate::engine::syntax::stage3::iterator::Stage3Iterator;
use crate::engine::syntax::stage4::{
    Stage4ComputationInput, Stage4Condition, Stage4Limit, Stage4Order, Stage4Selectable,
};
use crate::engine::syntax::{
    Stage4BinaryCondition, Stage4ColumnInput, Stage4Join, Stage4UnaryCondition, TableInput,
};
use crate::engine::Sourced;

/// The module covers iterating over our stage2 pines and converting them into stage3 pines
mod iterator;

pub struct Stage3Rep<'a> {
    // the iteration code got a bit complex here, so I split it off.
    pub pines: Stage3Iterator<'a>,
}

#[derive(Debug, Clone)]
pub enum Stage3Pine<'a> {
    From {
        table: Sourced<TableInput<'a>>,
        conditions: Vec<Sourced<Stage3Condition<'a>>>,
    },
    Select(Vec<Sourced<Stage3Selectable<'a>>>),
    Unselect(Vec<Sourced<Stage3ColumnInput<'a>>>),
    Filter(Vec<Sourced<Stage3Condition<'a>>>),
    Join(Sourced<Stage3Join<'a>>),
    Order(Vec<Sourced<Stage3Order<'a>>>),
    GroupBy(Vec<Sourced<Stage3Selectable<'a>>>),
    Limit(Sourced<Stage3Limit<'a>>),
    ShowNeighbors(Sourced<TableInput<'a>>),
    ShowColumns(Sourced<TableInput<'a>>),
}

// shh! keep these secret
pub type Stage3Selectable<'a> = Stage4Selectable<'a>;
pub type Stage3Condition<'a> = Stage4Condition<'a>;
pub type Stage3BinaryCondition<'a> = Stage4BinaryCondition<'a>;
pub type Stage3UnaryCondition<'a> = Stage4UnaryCondition<'a>;
pub type Stage3ColumnInput<'a> = Stage4ColumnInput<'a>;
pub type Stage3ComputationInput<'a> = Stage4ComputationInput<'a>;
pub type Stage3Join<'a> = Stage4Join<'a>;
pub type Stage3Order<'a> = Stage4Order<'a>;
pub type Stage3Limit<'a> = Stage4Limit<'a>;

impl<'a> From<Stage2Rep<'a>> for Stage3Rep<'a> {
    fn from(stage2: Stage2Rep<'a>) -> Self {
        let context = Stage3Iterator::new(stage2.pines);

        Stage3Rep { pines: context }
    }
}

#[cfg(test)]
mod test {
    use crate::engine::syntax::stage1::parse_stage1;
    use crate::engine::syntax::stage2::Stage2Rep;
    use crate::engine::syntax::stage3::{Stage3Pine, Stage3Rep};
    use crate::engine::syntax::{OptionalInput, SqlIdentifierInput, TableInput};
    use crate::engine::{Position, Source, Sourced};

    #[test]
    fn test_simple_convert() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let mut stage3: Stage3Rep = stage2.into();

        let first = stage3.pines.next().unwrap();
        assert_eq!(0..5, first.source);

        assert!(matches!(
            first.it,
            Stage3Pine::From {
                table: Sourced {
                    it: TableInput {
                        database: OptionalInput::Implicit,
                        table: Sourced {
                            it: SqlIdentifierInput { name: "table" },
                            source: Source::Input(Position { start: 0, end: 5 }),
                        },
                    },
                    source: Source::Input(Position { start: 0, end: 5 })
                },
                ..
            }
        ))
    }
}
