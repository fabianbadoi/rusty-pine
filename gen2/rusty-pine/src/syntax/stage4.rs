//! Stage 4 representation is a hierarchical structure representing what query the user would
//! constructed.
//! It will have all data present in the input, but will not have any data present that relates
//! to the actual state of the database:
//!     - how do to joins
//!     - can't tell if table is missing or name is mistyped
use crate::syntax::stage3::{Stage3Pine, Stage3Rep};
use crate::syntax::{ColumnInput, Position, Positioned, TableInput};

struct Stage4Rep<'a> {
    input: &'a str,
    from: Positioned<TableInput<'a>>,
    selected_columns: Vec<ColumnInput<'a>>, // TODO stage 4 columns always have tables, unlike the input
}

impl<'a> From<Stage3Rep<'a>> for Stage4Rep<'a> {
    fn from(stage3: Stage3Rep<'a>) -> Self {
        let input = stage3.input;
        let mut from = None;
        let mut select = Vec::new();

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
                Stage3Pine::Select(column) => {
                    select.push(column);
                }
            }
        }

        Stage4Rep {
            input,
            from: from.expect("Impossible: pines without a from are not valid pest syntax"),
            selected_columns: select,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::syntax::stage1::{parse_stage1, Stage1Error};
    use crate::syntax::stage2::Stage2Rep;
    use crate::syntax::stage3::Stage3Rep;
    use crate::syntax::stage4::Stage4Rep;
    use crate::syntax::OptionalInput::{Implicit, Specified};
    use crate::syntax::{OptionalInput, Position, SqlIdentifierInput, TableInput};
    use std::ops::Range;

    #[test]
    fn test_simple_transform() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("table", stage4.input);
        assert_eq!(0..5, stage4.from.position);
        assert_eq!(0..5, stage4.from.node.position);
        assert_eq!("table", stage4.from.node.table.name);
        assert!(matches!(stage4.from.node.database, OptionalInput::Implicit));
    }

    #[test]
    fn test_transform_with_database() {
        let stage2: Stage2Rep = parse_stage1("database.table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("database.table", stage4.input);
        assert_eq!(0..14, stage4.from.position);
        assert_eq!(0..14, stage4.from.node.position);
        assert_eq!("table", stage4.from.node.table.name);
        assert_eq!(
            Position { start: 9, end: 14 },
            stage4.from.node.table.position
        );
        assert_eq!(Specified("database"), stage4.from.node.database);
        assert_eq!(0..8, stage4.from.node.database);
    }

    #[test]
    fn test_examples_for_select() {
        struct Example<'a> {
            input: &'a str,
            expected_column: SqlIdentifierInput<'a>,
            expected_table: OptionalInput<TableInput<'a>>,
        }

        let examples = vec![
            Example {
                input: "table | s: id",
                expected_column: SqlIdentifierInput {
                    name: "id",
                    position: (11..13).into(),
                },
                expected_table: Implicit,
            },
            Example {
                input: "table | s: table.id",
                expected_column: SqlIdentifierInput {
                    name: "id",
                    position: (17..19).into(),
                },
                expected_table: Specified(TableInput {
                    table: SqlIdentifierInput {
                        name: "table",
                        position: (11..16).into(),
                    },
                    database: Implicit,
                    position: (11..16).into(),
                }),
            },
            Example {
                input: "table | s: db.table.id",
                expected_column: SqlIdentifierInput {
                    name: "id",
                    position: (20..22).into(),
                },
                expected_table: Specified(TableInput {
                    table: SqlIdentifierInput {
                        name: "table",
                        position: (14..19).into(),
                    },
                    database: Specified(SqlIdentifierInput {
                        name: "db",
                        position: (11..13).into(),
                    }),
                    position: (11..19).into(),
                }),
            },
        ];

        for example in examples {
            let Example {
                input,
                expected_column,
                expected_table,
            } = example;
            let output = parse(input).unwrap();

            assert_eq!(1, output.selected_columns.len());
            assert_eq!(expected_column, output.selected_columns[0].column);
            assert_eq!(expected_column, output.selected_columns[0].column);

            let table = output.selected_columns[0].table;
            assert_eq!(expected_table, table);
        }
    }

    #[test]
    fn test_examples_for_from() {
        use OptionalInput::{Implicit, Specified};

        let examples = vec![
            ("table", "table", Implicit),
            ("f: table", "table", Implicit),
            ("from: table", "table", Implicit),
            ("from: db.table", "table", Specified("db")),
        ];

        for (input, expected_table, expected_db) in examples {
            let output = parse(input).unwrap();
            let from = output.from.node;

            assert_eq!(expected_table, from.table.name, "Parsing: {}", input);
            assert_eq!(expected_db, from.database, "Parsing: {}", input);
        }
    }

    fn parse(input: &str) -> Result<Stage4Rep, Stage1Error> {
        parse_stage1(input).map(|stage1| {
            let stage2: Stage2Rep = stage1.into();
            let stage3: Stage3Rep = stage2.into();

            stage3.into()
        })
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

    impl PartialEq<OptionalInput<SqlIdentifierInput<'_>>> for Range<usize> {
        fn eq(&self, other: &OptionalInput<SqlIdentifierInput<'_>>) -> bool {
            match other {
                Implicit => false,
                Specified(SqlIdentifierInput { position, .. }) => self == position,
            }
        }
    }
}
