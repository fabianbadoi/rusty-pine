/// Walk through our stage 2 pines and convert them to stage3.
/// See more info about the stage 3 rep. in the parent module.
use crate::engine::syntax::stage2::{PestIterator, Stage2Pine};
use crate::engine::syntax::stage3::{Stage3ColumnInput, Stage3Pine};
use crate::engine::syntax::{ColumnInput, Position, Positioned, TableInput};
use std::collections::VecDeque;

pub struct Stage3Iterator<'a> {
    stage2_source: PestIterator<'a>,
    /// A single stage 2 pine can actually lead to multiple stage 3 pines.
    /// In order to handle that, we will put translation output in a buffer to be produced
    /// by the iterator.
    stage3_buffer: Stage3Buffer<'a>,
    /// Unlike previous steps, we will need to flesh out stage 3 pines some contextual data
    /// derived from the processing of previous pines.
    context: Context<'a>,
}

/// Type aliases make our code cleaner.
/// We are using VecDeq instead of Vec because we are using this type like a buffer: pushing onto
/// its back, and popping from the front. While doable with Vec::drain() or some nightly features,
/// it's not as convenient.
type Stage3Buffer<'a> = VecDeque<Positioned<Stage3Pine<'a>>>;

/// Each pine can implicitly reference the context created by the other pines.
/// For example, using a "select: column_name" will always refer to the previous table.
struct Context<'a> {
    previous_table: TableInput<'a>,
}

impl<'a> Iterator for Stage3Iterator<'a> {
    type Item = Positioned<Stage3Pine<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // The buffer will be empty when this Iterator returns enough Items.
        // When that happens, we'll try to consume next stage 2 pine to produce more Items.
        // If we're all out of stage 2 pines, the output buffer will remain empty.
        if self.stage3_buffer.is_empty() {
            self.consume_from_stage2();
        }

        // The consume_from_stage2() call might have not put anything into the buffer. This can
        // happen when stage2_source is empty.
        // In such cases, we will return None, signaling that the Iterator is finished.
        //
        // If we were using Vec instead of VecDeq, we would have used:
        // ```rust
        // if self.stage3_buffer.is_empty() { // check needed so .drain() does not panic!
        //      None
        // } else {
        //      self.stage3_buffer.drain(..1).next()
        // }
        // ```
        self.stage3_buffer.pop_front()
    }
}

impl<'a> Stage3Iterator<'a> {
    pub fn new(mut source: PestIterator<'a>) -> Self {
        // Any input that does not have base is not valid syntax, so any PestIterator will have at
        // least 1 item. Unless I fucked up the grammar.
        let base = source.next().expect("things must always have a base");
        let position = base.position;

        let base_table = match base.node {
            Stage2Pine::Base { table } => table,
            // Same as above, the grammar should guarantee this panic! never happens.
            _ => panic!("Unknown starting pine, expected base"),
        };

        Self {
            stage2_source: source,
            stage3_buffer: VecDeque::from([
                position.holding(Stage3Pine::From { table: base_table })
            ]),
            context: Context {
                previous_table: base_table,
            },
        }
    }

    /// Tries generate religious victory points for the stage 3 buffer by cleansing filthy heretics
    /// from the stage 2 source.
    fn consume_from_stage2(&mut self) {
        if let Some(next_stage2) = self.stage2_source.next() {
            // As I add more pines, I will have to change the return type of process_stage_2_pine
            // to also return a new context.
            let mut more_stage3_pines = self.process_stage_2_pine(next_stage2);

            self.stage3_buffer.append(&mut more_stage3_pines);
        }
    }

    fn process_stage_2_pine(
        &mut self,
        stage2_pine: Positioned<Stage2Pine<'a>>,
    ) -> Stage3Buffer<'a> {
        let position = stage2_pine.position;

        let stage3_pines = match stage2_pine.node {
            Stage2Pine::Base { .. } => panic!("This was covered in the constructor"),
            Stage2Pine::Select(columns) => self.translate_select(position, columns),
        };

        stage3_pines
    }

    fn translate_select(
        &mut self,
        position: Position,
        columns: Vec<ColumnInput<'a>>,
    ) -> Stage3Buffer<'a> {
        let columns = columns
            .iter()
            .map(|column| Stage3ColumnInput {
                column: column.column,
                position: column.position,
                // our amazing context in action 💪
                table: column.table.or(self.context.previous_table),
            })
            .collect();

        VecDeque::from([position.holding(Stage3Pine::Select(columns))])
    }
}
