use crate::syntax::stage1::Stage1Rep;
use crate::syntax::{stage1, Position, SqlIdentifier};
use pest::iterators::Pair;
use pest::Token;

pub struct Stage2Rep<'a> {
    pub input: &'a str,
    pub root: Stage2Root<'a>,
}

pub struct Stage2Root<'a> {
    base: Stage2Base<'a>,
}

pub struct Stage2Base<'a> {
    table: SqlIdentifier<'a>,
}

impl From<Pair<'_, stage1::Rule>> for Stage2Root<'_> {
    fn from(stage1: Pair<'_, stage1::Rule>) -> Self {
        // println!("{}", stage1.)

        todo!()
    }
}

impl<'a> From<Stage1Rep<'a>> for Stage2Rep<'a> {
    fn from(mut stage1: Stage1Rep<'a>) -> Self {
        Self {
            input: stage1.input,
            root: stage1.pest.next().unwrap().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::stage1::{parse_stage1, Rule, Stage1Error};
    use crate::syntax::stage2::Stage2Rep;

    #[test]
    fn test_base() {
        let stage1_root = parse_stage1("root").unwrap();
        let stage2_rep: Stage2Rep = stage1_root.into();

        assert_eq!("root", stage2_rep.input);
        assert_eq!("root", stage2_rep.root.base.table.name);
    }
}
