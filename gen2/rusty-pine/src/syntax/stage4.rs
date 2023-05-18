//! Stage 4 representation is a hierarchical structure representing what query the user would
//! constructed.
//! It will have all data present in the input, but will not have any data present that relates
//! to the actual state of the database:
//!     - how do to joins
//!     - can't tell if table is missing or name is mistyped
use crate::syntax::stage3::{Stage3Pine, Stage3Rep};
use crate::syntax::{Positioned, TableInput};

struct Stage4Rep<'a> {
    input: &'a str,
    from: Positioned<TableInput<'a>>,
}

impl<'a> From<Stage3Rep<'a>> for Stage4Rep<'a> {
    fn from(stage3: Stage3Rep<'a>) -> Self {
        let input = stage3.input;
        let mut from = None;

        for pine in stage3.pines {
            let position = pine.position;

            match pine.node {
                Stage3Pine::From { table } => {
                    assert!(
                        from.is_none(),
                        "Our pest syntax forbids multiple from statements"
                    );
                    from = Some(position.holding(table));
                }
            }
        }

        Stage4Rep {
            input,
            from: from.expect("Impossible: pines without a from are not valid pest syntax"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::syntax::stage1::parse_stage1;
    use crate::syntax::stage2::Stage2Rep;
    use crate::syntax::stage3::Stage3Rep;
    use crate::syntax::stage4::Stage4Rep;
    use crate::syntax::OptionalInput::{Implicit, Specified};
    use crate::syntax::{OptionalInput, Position, SqlIdentifierInput};

    #[test]
    fn test_simple_transform() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("table", stage4.input);
        assert_eq!(Position { start: 0, end: 5 }, stage4.from.position);
        assert_eq!(Position { start: 0, end: 5 }, stage4.from.node.position);
        assert_eq!("table", stage4.from.node.table.name);
        assert!(matches!(stage4.from.node.database, OptionalInput::Implicit));
    }

    #[test]
    fn test_transform_with_database() {
        let stage2: Stage2Rep = parse_stage1("database.table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("database.table", stage4.input);
        assert_eq!(Position { start: 0, end: 14 }, stage4.from.position);
        assert_eq!(Position { start: 0, end: 14 }, stage4.from.node.position);
        assert_eq!("table", stage4.from.node.table.name);
        assert_eq!(
            Position { start: 9, end: 14 },
            stage4.from.node.table.position
        );
        assert_eq!(Specified("database"), stage4.from.node.database);
        assert_eq!(Position { start: 0, end: 8 }, stage4.from.node.database);
    }

    #[test]
    fn test_examples_for_from() {
        use OptionalInput::{Implicit, Specified};

        let examples = vec![
            ("f: table", "table", Implicit),
            ("from: table", "table", Implicit),
            ("from: db.table", "table", Specified("db")),
        ];

        for (input, expected_table, expected_db) in examples {
            let output = parse(input);
            let from = output.from.node;

            assert_eq!(expected_table, from.table.name, "Parsing: {}", input);
            assert_eq!(expected_db, from.database, "Parsing: {}", input);
        }
    }

    fn parse(input: &str) -> Stage4Rep {
        let stage2: Stage2Rep = parse_stage1(input).unwrap().into();
        let stage3: Stage3Rep = stage2.into();

        stage3.into()
    }

    impl PartialEq<OptionalInput<SqlIdentifierInput<'_>>> for OptionalInput<&str> {
        fn eq(&self, other: &OptionalInput<SqlIdentifierInput<'_>>) -> bool {
            match (self, other) {
                (Implicit, Implicit) => true,
                (Specified(other_name), Specified(SqlIdentifierInput { name, .. })) => {
                    name == other_name
                }
                _ => false,
            }
        }
    }

    impl PartialEq<OptionalInput<SqlIdentifierInput<'_>>> for Position {
        fn eq(&self, other: &OptionalInput<SqlIdentifierInput<'_>>) -> bool {
            match other {
                Implicit => false,
                Specified(SqlIdentifierInput { position, .. }) => position == self,
            }
        }
    }
}
