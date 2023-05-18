use crate::syntax::stage1::Stage1Rep;
use crate::syntax::{stage1, Position, SqlIdentifier};
use pest::iterators::Pair;
use pest::Token;

pub struct Stage3Rep<'a> {
    pub input: &'a str,
    pub root: Stage3Root<'a>,
}

pub struct Stage3Root<'a> {
    base: Stage3Base<'a>,
}

pub struct Stage3Base<'a> {
    table: SqlIdentifier<'a>,
}
