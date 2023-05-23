//! Stage 3 adds implicit information to Pines.
//! For example:
//!     "users | w: id = 3 | s: id"
//!     The where and select pines will also reference the previous table: users.
//!
//! Each pine should have all of the info from the input contained in itself, so future processing
//! does not have to look-backs.
use crate::engine::syntax::stage2::{PestIterator, Stage2Rep};
use crate::engine::syntax::stage3::iterator::Stage3OutputQueue;
use crate::engine::syntax::{Stage4ColumnInput, TableInput};

pub struct Stage3Rep<'a, T> {
    pub input: &'a str,
    /// Normally I would use the real type here, not a generic one. In this case, the type signature
    /// is really long, so this simplifies our lives a bit.
    /// In the end, both approaches should compile to the same code.
    pub pines: T,
}

pub enum Stage3Pine<'a> {
    From { table: TableInput<'a> },
    Select(Vec<Stage3ColumnInput<'a>>),
}

pub type Stage3ColumnInput<'a> = Stage4ColumnInput<'a>; // shh!

impl<'a> From<Stage2Rep<'a>>
    for Stage3Rep<'a, iterator::Stage3Iterator<'a, PestIterator<'a>, Stage3OutputQueue<'a>>>
//      👆 this is a really ugly type, it's about here that I start to get sick of looking at <'a>
{
    fn from(stage2: Stage2Rep<'a>) -> Self {
        let context = iterator::Stage3Iterator::new(stage2.pines);

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
        let mut stage3: Stage3Rep<_> = stage2.into();

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

mod iterator {
    use crate::engine::syntax::stage2::Stage2Pine;
    use crate::engine::syntax::stage3::{Stage3ColumnInput, Stage3Pine};
    use crate::engine::syntax::{ColumnInput, Position, Positioned, TableInput};
    use std::collections::VecDeque;

    pub type Stage3OutputQueue<'a> = VecDeque<Positioned<Stage3Pine<'a>>>;

    pub struct Stage3Iterator<'a, Source, OutputQueue> {
        source: Source,
        output_queue: OutputQueue,
        context: Context<'a>,
    }

    struct Context<'a> {
        previous_table: TableInput<'a>,
    }

    impl<'a, T> Iterator for Stage3Iterator<'a, T, Stage3OutputQueue<'a>>
    where
        T: Iterator<Item = Positioned<Stage2Pine<'a>>>,
    {
        type Item = Positioned<Stage3Pine<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.output_queue.is_empty() {
                self.consume_from_stage2();
            }

            self.output_queue.pop_front()
        }
    }

    impl<'a, T> Stage3Iterator<'a, T, Stage3OutputQueue<'a>>
    where
        T: Iterator<Item = Positioned<Stage2Pine<'a>>>,
    {
        pub fn new(mut source: T) -> Self {
            let base = source.next().expect("things must always have a base");
            let position = base.position;

            let base_table = match base.node {
                Stage2Pine::Base { table } => table,
                _ => panic!("Unknown starting pine, expected base"),
            };

            Self {
                source,
                context: Context {
                    previous_table: base_table,
                },
                output_queue: VecDeque::from([
                    position.holding(Stage3Pine::From { table: base_table })
                ]),
            }
        }

        fn consume_from_stage2(&mut self) {
            if let Some(next_stage2) = self.source.next() {
                let mut more_stage3_pines = self.process_stage_2_pine(next_stage2);
                self.output_queue.append(&mut more_stage3_pines);
            }
        }
    }

    impl<'a, T, O> Stage3Iterator<'a, T, O> {
        fn process_stage_2_pine(
            &mut self,
            stage2_pine: Positioned<Stage2Pine<'a>>,
        ) -> Stage3OutputQueue<'a> {
            let position = stage2_pine.position;

            let stage3_pines = match stage2_pine.node {
                Stage2Pine::Base { .. } => panic!("This was covered in the constructor"),
                Stage2Pine::Select(columns) => self.translate_select(position, columns),
            };

            VecDeque::from(stage3_pines)
        }

        fn translate_select(
            &mut self,
            position: Position,
            columns: Vec<ColumnInput<'a>>,
        ) -> Vec<Positioned<Stage3Pine<'a>>> {
            let columns = columns
                .iter()
                .map(|column| Stage3ColumnInput {
                    column: column.column,
                    position: column.position,
                    table: column.table.or(self.context.previous_table),
                })
                .collect();

            vec![position.holding(Stage3Pine::Select(columns))]
        }
    }
}
