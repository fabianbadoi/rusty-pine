//! Function calls can be a bit complicated, so the code is split up in its own module.

use crate::engine::syntax::stage2::identifiers::translate_sql_name;
use crate::engine::syntax::stage2::translate_computation;
use crate::engine::syntax::{FunctionCall, Position};
use crate::engine::Rule;
use pest::iterators::Pair;

pub fn translate_fn_call(fn_call: Pair<Rule>) -> FunctionCall {
    assert_eq!(Rule::function_call, fn_call.as_rule());

    let position: Position = fn_call.as_span().into();

    let mut inners = fn_call.into_inner();

    let fn_name_pair = inners.next().expect("Has to be valid syntax");
    let fn_name = translate_sql_name(fn_name_pair);

    let mut params = Vec::new();
    for column_pair in inners {
        let column = translate_computation(column_pair);
        params.push(column);
    }

    FunctionCall {
        fn_name,
        params,
        position,
    }
}
