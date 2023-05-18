use crate::syntax::stage1::Stage1Rep;
use crate::syntax::SqlIdentifier;

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

impl From<Stage1Rep> for Stage2Rep {
    fn from(value: Stage1Rep) -> Self {
        todo!()
    }
}

//
// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_base() {
//         let n = &"fdasfa"[0..2];
//         n.char_indices()
//     }
// }
