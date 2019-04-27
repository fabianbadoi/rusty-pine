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