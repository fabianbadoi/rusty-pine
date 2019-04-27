use ::pest::Parser;




use std::convert::From;
use ::pest::error::Error as PestError;
use self::pest::Rule;

#[derive(Debug)]
pub struct PineParseError(());
impl From<PestError<Rule>> for PineParseError {
    fn from(pest_error: PestError<Rule>) -> Self {
        panic!("{}", pest_error);
        // TODO this needs to be better
        PineParseError(())
    }
}

#[derive(Debug)]
pub struct Position {
    pub start: usize,
    pub end: usize
}

#[derive(Debug)]
pub struct Positioned<T> {
    pub item: T,
    pub position: Position
}

pub type TableName = Positioned<String>;
pub type ColumnName = Positioned<String>;

#[derive(Debug)]
pub enum Operation {
    From(TableName),
    Select(Vec<ColumnName>)
}

impl Operation {
    pub fn get_name(&self) -> &str {
        use Operation::*;

        match self {
            From(_) => "from",
            Select(_) => "select",
        }
    }
}


pub type Pine = Positioned<Vec<Positioned<Operation>>>;

pub trait PineParserTrait {
    fn parse(input: &str) -> Result<Pine, PineParseError>;
}

struct PineParser;
impl PineParserTrait for PineParser {
    fn parse(input: &str) -> Result<Pine, PineParseError> {
        let ast = pest::PinePestParser::parse(pest::Rule::pine, input)?.next()
            .expect("Pest should have failed to parse this input");

        let pine = pest_tree_translation::translate(ast);

        Ok(pine)
    }
}

mod pest_tree_translation {
    use super::Pine;
    use super::{Positioned, Position};
    use super::{Operation, TableName};
    use super::pest::Rule;
    use ::pest::iterators::Pair as PestPair;

    type Pair<'a> = PestPair<'a, Rule>;

    pub fn translate(root_pair: Pair) -> Pine {
        expect(Rule::pine, &root_pair);

        let position = pair_to_position(&root_pair);
        let operations : Vec<_> = root_pair.into_inner().map(translate_operation).collect();

        Pine { position, item: operations }
    }

    fn translate_operation(pair: Pair) -> Positioned<Operation> {
        expect(Rule::operation, &pair);
        
        let operation_pair = pair.into_inner().next()
            .expect("Pest should not have created an operation without an inner");

        let position = pair_to_position(&operation_pair);
        let operation = match operation_pair.as_rule() {
            Rule::from => translate_from(operation_pair),
            Rule::select => translate_select(operation_pair),
            _ => panic!("Expected a operation variant, got '{:?}'", operation_pair.as_rule())
        };

        Positioned { position, item: operation }
    }

    fn translate_from(pair: Pair) -> Operation {
        let table_name = translate_sql_name(
            pair.into_inner().next().expect("Found from without table name")
        );

        Operation::From(table_name)
    }

    fn translate_select(pair: Pair) -> Operation {
        let columns : Vec<_> = pair.into_inner().map(translate_sql_name).collect();

        Operation::Select(columns)
    }

    fn translate_sql_name(pair: Pair) -> TableName {
        expect_one_of(vec![Rule::column_name, Rule::table_name], &pair);

        let position = pair_to_position(&pair);

        TableName { item: pair.as_str().to_string(), position }
    }

    fn expect(expected_type: Rule, pair: &Pair) {
        if pair.as_rule() != expected_type {
            panic!("Token be a '{:?}' expression, found '{:?}'", expected_type, pair.as_rule());
        }
    }

    fn expect_one_of(expected_types: Vec<Rule>, pair: &Pair) {
        if !expected_types.contains(&pair.as_rule()) {
            panic!("Token be a one of {:?}, found '{:?}'", expected_types, pair.as_rule());
        }
    }

    fn pair_to_position(pair: &Pair) -> Position {
        let span = pair.as_span();

        Position {start: span.start(), end: span.end() }
    }
}

mod pest {
    #[derive(Parser)]
    #[grammar = "pine_syntax/pine.pest"]
    pub struct PinePestParser;

    #[cfg(test)]
    mod tests {
        use ::pest::Parser;
        use super::PinePestParser;
        use super::Rule;

        #[test]
        fn pest_syntax_is_ok() {
            let result = PinePestParser::parse(Rule::pine, "from: users | from: tests | from: x");

            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PineParser, PineParserTrait, Operation};

    #[test]
    fn parsing_simple_form_statement() {
        let pine = PineParser::parse("from: users | select: id, name").unwrap();

        assert_eq!("from", pine.item[0].item.get_name());
        assert_eq!("select", pine.item[1].item.get_name());

        if let Operation::From(ref table_name) = pine.item[0].item {
            assert_eq!("users", table_name.item);
        }
    }
}
