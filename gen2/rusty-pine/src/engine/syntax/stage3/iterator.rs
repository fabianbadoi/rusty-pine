/// Walk through our stage 2 pines and convert them to stage3.
/// See more info about the stage 3 rep. in the parent module.
use crate::engine::syntax::stage2::{PestIterator, Stage2ExplicitJoin, Stage2Pine};
use crate::engine::syntax::stage3::{
    Stage3ColumnInput, Stage3ComputationInput, Stage3ExplicitJoin, Stage3Pine,
};
use crate::engine::syntax::stage4::Stage4FunctionCall;
use crate::engine::syntax::{ColumnInput, Computation, FunctionCall, TableInput};
use crate::engine::{Source, Sourced};
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
type Stage3Buffer<'a> = VecDeque<Sourced<Stage3Pine<'a>>>;

/// Each pine can implicitly reference the context created by the other pines.
/// For example, using a "select: column_name" will always refer to the previous table.
struct Context<'a> {
    previous_table: Sourced<TableInput<'a>>,
}

impl<'a> Iterator for Stage3Iterator<'a> {
    type Item = Sourced<Stage3Pine<'a>>;

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
    pub fn new(mut stage2_pines: PestIterator<'a>) -> Self {
        // Any input that does not have base is not valid syntax, so any PestIterator will have at
        // least 1 item. Unless I fucked up the grammar.
        let base = stage2_pines.next().expect("things must always have a base");
        let source = base.source;

        let base_table = match base.it {
            Stage2Pine::Base { table } => table,
            // Same as above, the grammar should guarantee this panic! never happens.
            _ => panic!("Unknown starting pine, expected base"),
        };

        Self {
            stage2_source: stage2_pines,
            stage3_buffer: VecDeque::from([source.holding(Stage3Pine::From { table: base_table })]),
            context: Context {
                previous_table: base_table,
            },
        }
    }

    /// Tries to generate religious victory points for the stage 3 buffer by cleansing filthy heretics
    /// from the stage 2 source.
    fn consume_from_stage2(&mut self) {
        if let Some(next_stage2) = self.stage2_source.next() {
            // As I add more pines, I will have to change the return type of process_stage_2_pine
            // to also return a new context.
            let mut more_stage3_pines = self.process_stage_2_pine(next_stage2);

            self.stage3_buffer.append(&mut more_stage3_pines);
        }
    }

    fn process_stage_2_pine(&mut self, stage2_pine: Sourced<Stage2Pine<'a>>) -> Stage3Buffer<'a> {
        let position = stage2_pine.source;

        let stage3_pines = match stage2_pine.it {
            Stage2Pine::Base { .. } => panic!("This was covered in the constructor"),
            Stage2Pine::Select(columns) => self.translate_select(position, columns),
            Stage2Pine::ExplicitJoin(explicit_join) => {
                self.process_explicit_join(position, explicit_join)
            }
        };

        stage3_pines
    }

    fn translate_select(
        &mut self,
        source: Source,
        columns: Vec<Sourced<Computation<'a>>>,
    ) -> Stage3Buffer<'a> {
        let columns = columns
            .iter()
            .map(|column| translate_computation(column, &self.context.previous_table))
            .collect();

        VecDeque::from([source.holding(Stage3Pine::Select(columns))])
    }

    fn process_explicit_join(
        &mut self,
        source: Source,
        join: Sourced<Stage2ExplicitJoin<'a>>,
    ) -> Stage3Buffer<'a> {
        let source_arg = translate_computation(&join.it.source_arg, &self.context.previous_table);
        let target_arg = translate_computation(&join.it.target_arg, &join.it.target_table);

        let Stage2ExplicitJoin {
            join_type,
            target_table,
            ..
        } = join.it;

        let stage3_join = source.holding(Stage3Pine::ExplicitJoin(join.source.holding(
            Stage3ExplicitJoin {
                join_type,
                source_table: self.context.previous_table,
                target_table,
                source_arg,
                target_arg,
            },
        )));

        // Future pines will implicitly reference this table
        self.context.previous_table = target_table;

        VecDeque::from([stage3_join])
    }
}

fn translate_computation<'a>(
    computation: &Sourced<Computation<'a>>,
    implicit_table: &Sourced<TableInput<'a>>,
) -> Sourced<Stage3ComputationInput<'a>> {
    computation.map_ref(|computation| match computation {
        Computation::Column(column) => translate_select_from_column(column, implicit_table),
        Computation::FunctionCall(fn_call) => {
            translate_select_from_fn_call(fn_call, implicit_table)
        }
    })
}

fn translate_select_from_column<'a>(
    column: &Sourced<ColumnInput<'a>>,
    implicit_table: &Sourced<TableInput<'a>>,
) -> Stage3ComputationInput<'a> {
    Stage3ComputationInput::Column(column.map_ref(|column| Stage3ColumnInput {
        column: column.column,
        table: column.table.or(*implicit_table),
    }))
}

fn translate_select_from_fn_call<'a>(
    fn_call: &Sourced<FunctionCall<'a>>,
    implicit_table: &Sourced<TableInput<'a>>,
) -> Stage3ComputationInput<'a> {
    Stage3ComputationInput::FunctionCall(fn_call.map_ref(|fn_call| {
        Stage4FunctionCall {
            fn_name: fn_call.fn_name,
            params: fn_call
                .params
                .iter()
                .map(|computation| translate_computation(computation, implicit_table))
                .collect(),
        }
    }))
}
