//! Stage 3 adds implicit information to Pines.
//! For example:
//!     "users | w: id = 3 | s: id"
//!     The where and select pines will also reference the previous table: users.
//!
//! Each pine should have all of the info from the input contained in itself, so future processing
//! does not have to look-backs.
use crate::syntax::stage2::{Stage2Pine, Stage2Rep};
use crate::syntax::{Positioned, Stage4ColumnInput, TableInput};

pub struct Stage3Rep<'a> {
    pub input: &'a str,
    pub pines: Vec<Positioned<Stage3Pine<'a>>>,
}

pub enum Stage3Pine<'a> {
    From { table: TableInput<'a> },
    Select(Stage3ColumnInput<'a>),
}

pub type Stage3ColumnInput<'a> = Stage4ColumnInput<'a>; // shh!

impl<'a> From<Stage2Rep<'a>> for Stage3Rep<'a> {
    fn from(stage2: Stage2Rep<'a>) -> Self {
        let (pines, _) = stage2
            .pines
            .into_iter()
            .fold(collector(), transform_stage_2_pine);

        Stage3Rep {
            input: stage2.input,
            pines,
        }
    }
}

type Stage3Pines<'a> = Vec<Positioned<Stage3Pine<'a>>>;

#[derive(Default)]
struct Context<'a> {
    last_table: Option<TableInput<'a>>,
}
fn collector<'a>() -> (Stage3Pines<'a>, Context<'a>) {
    (Vec::new(), Default::default())
}
type Stage2PineParam<'a> = Positioned<Stage2Pine<'a>>;

fn transform_stage_2_pine<'a>(
    (mut stage3_pines, mut context): (Stage3Pines<'a>, Context<'a>),
    stage2_pine: Stage2PineParam<'a>,
) -> (Vec<Positioned<Stage3Pine<'a>>>, Context<'a>) {
    let position = &stage2_pine.position;

    match &stage2_pine.node {
        Stage2Pine::Base { table } => {
            context.last_table.replace(*table);
            stage3_pines.push(position.holding(Stage3Pine::From { table: *table }))
        }
        Stage2Pine::Select(column) => stage3_pines.push(
            position.holding(Stage3Pine::Select(Stage3ColumnInput {
                column: column.column,
                position: column.position,
                table: column.table.or(context
                    .last_table
                    .expect("The base pine always has a table")),
            })),
        ),
    };

    (stage3_pines, context)
}

#[cfg(test)]
mod test {
    use crate::syntax::stage1::parse_stage1;
    use crate::syntax::stage2::Stage2Rep;
    use crate::syntax::stage3::{Stage3Pine, Stage3Rep};
    use crate::syntax::{OptionalInput, Position, SqlIdentifierInput, TableInput};

    #[test]
    fn test_simple_convert() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();

        assert_eq!("table", stage3.input);
        assert_eq!(1, stage3.pines.len());
        assert_eq!(0..5, stage3.pines[0].position);

        assert!(matches!(
            stage3.pines[0].node,
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
