#[derive(Parser)]
#[grammar = "pine_syntax/pine.pest"]
pub struct PinePestParser;

#[cfg(test)]
mod tests {
    use super::PinePestParser;
    use super::Rule;
    use ::pest::Parser;

    #[test]
    fn pest_syntax_is_ok() {
        let result = PinePestParser::parse(Rule::pine, "users | from: tests | from: x");

        assert!(result.is_ok());
    }

    #[test]
    fn shorthand_syntax_is_ok() {
        let result = PinePestParser::parse(Rule::pine, "users | s: tests | w: x = 0");

        assert!(result.is_ok());
    }
}
