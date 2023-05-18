use pest::iterators::Pairs;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "syntax/pine.pest"]
struct Stage1Parser;

pub fn parse_stage1(input: &str) -> Stage1Rep {
    
}

pub struct Stage1Rep<'a> {
    pub input: &'a str,
    pub pest: Pairs<'a, Rule>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn test_base() {
        let result = Stage1Parser::parse(Rule::root, "table");
        assert!(result.is_ok());

        let root = result.unwrap().peek();
        assert!(root.is_some());
    }
}
