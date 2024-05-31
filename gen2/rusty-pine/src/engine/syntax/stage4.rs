//! Stage 4 representation is a hierarchical structure representing what query the user constructed.
//! It will have all data present in the input, but will not have any data present that relates
//! to the actual state of the database:
//!     - how do to joins
//!     - can't tell if table is missing or name is mistyped
use crate::engine::syntax::stage3::{Stage3ComputationInput, Stage3Pine, Stage3Rep};
use crate::engine::syntax::{SqlIdentifierInput, TableInput};
use crate::engine::{JoinType, Limit};
use crate::engine::{LiteralValueHolder, Sourced};

pub struct Stage4Rep<'a> {
    pub input: &'a str,
    pub from: Sourced<TableInput<'a>>,
    pub joins: Vec<Sourced<Stage4ExplicitJoin<'a>>>,
    pub selected_columns: Vec<Sourced<Stage4ComputationInput<'a>>>,
    pub limit: Sourced<Limit>,
}

#[derive(Debug, Clone)]
pub struct Stage4ExplicitJoin<'a> {
    pub join_type: Sourced<JoinType>,
    pub source_table: Sourced<TableInput<'a>>,
    /// The table to join to.
    pub target_table: Sourced<TableInput<'a>>,
    /// The "source" of the join's ON query.
    ///
    /// All column names will default to referring to the previous table.
    pub source_arg: Sourced<Stage3ComputationInput<'a>>,
    /// The "target" of the join's ON query.
    ///
    /// All column names will default to referring to the target table.
    pub target_arg: Sourced<Stage3ComputationInput<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Stage4ColumnInput<'a> {
    pub table: Sourced<TableInput<'a>>, // we always know it because of SYNTAX
    pub column: Sourced<SqlIdentifierInput<'a>>,
}

#[derive(Debug, Clone)]
pub enum Stage4ComputationInput<'a> {
    Column(Sourced<Stage4ColumnInput<'a>>),
    FunctionCall(Sourced<Stage4FunctionCall<'a>>),
    Value(Sourced<Stage4LiteralValue<'a>>),
}

#[derive(Debug, Clone)]
pub struct Stage4FunctionCall<'a> {
    pub fn_name: Sourced<SqlIdentifierInput<'a>>,
    pub params: Vec<Sourced<Stage4ComputationInput<'a>>>,
}

pub type Stage4LiteralValue<'a> = LiteralValueHolder<&'a str>;

impl<'a> Stage4ExplicitJoin<'a> {
    pub fn switch(self) -> Self {
        Stage4ExplicitJoin {
            source_table: self.target_table,
            source_arg: self.target_arg,
            target_table: self.source_table,
            target_arg: self.source_arg,
            join_type: self.join_type,
        }
    }
}

impl<'a> From<Stage3Rep<'a>> for Stage4Rep<'a> {
    fn from(stage3: Stage3Rep<'a>) -> Self {
        let input = stage3.input;
        let mut from = None;
        let mut select = Vec::new();
        let mut joins = Vec::new();

        for pine in stage3.pines {
            match pine.it {
                Stage3Pine::From { table } => {
                    assert!(
                        from.is_none(),
                        "Our pest syntax forbids multiple from statements"
                    );
                    from = Some(table);
                }
                Stage3Pine::Select(columns) => {
                    select.append(&mut translate_columns(columns));
                }
                Stage3Pine::ExplicitJoin(join) => {
                    joins.push(join);
                }
            }
        }

        Stage4Rep {
            input,
            from: from.expect("Impossible: pines without a from are not valid pest syntax"),
            joins,
            selected_columns: select,
            limit: Sourced::implicit(Limit::Implicit()),
        }
    }
}

fn translate_columns(
    columns: Vec<Sourced<Stage3ComputationInput>>,
) -> Vec<Sourced<Stage4ComputationInput>> {
    columns.into_iter().collect()
}

#[cfg(test)]
mod test {
    use crate::engine::syntax::stage1::parse_stage1;
    use crate::engine::syntax::stage2::Stage2Rep;
    use crate::engine::syntax::stage3::Stage3Rep;
    use crate::engine::syntax::stage4::Stage4Rep;
    use crate::engine::syntax::OptionalInput::{Implicit, Specified};
    use crate::engine::syntax::{
        parse_to_stage4, OptionalInput, SqlIdentifierInput, Stage4ComputationInput, TableInput,
    };

    use crate::engine::{Position, Source, Sourced};
    use std::ops::Range;

    #[test]
    fn test_simple_transform() {
        let stage2: Stage2Rep = parse_stage1("table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("table", stage4.input);
        assert_eq!(0..5, stage4.from.source);
        assert_eq!(0..5, stage4.from.source);
        assert_eq!("table", stage4.from.it.table.it.name);
        assert!(matches!(stage4.from.it.database, OptionalInput::Implicit));
    }

    #[test]
    fn test_transform_with_database() {
        let stage2: Stage2Rep = parse_stage1("database.table").unwrap().into();
        let stage3: Stage3Rep = stage2.into();
        let stage4: Stage4Rep = stage3.into();

        assert_eq!("database.table", stage4.input);
        assert_eq!(0..14, stage4.from.source);
        assert_eq!(0..14, stage4.from.source);
        assert_eq!("table", stage4.from.it.table.it.name);
        assert_eq!(Position { start: 9, end: 14 }, stage4.from.it.table.source);
        assert_eq!("database", stage4.from.it.database.unwrap().it.name);
        assert_eq!(0..8, stage4.from.it.database.unwrap().source);
    }

    #[test]
    fn test_examples_for_select() {
        struct Example<'a> {
            input: &'a str,
            expected_column: SqlIdentifierInput<'a>,
            expected_table: TableInput<'a>,
        }

        let examples = vec![
            Example {
                input: "table | s: id",
                expected_column: SqlIdentifierInput { name: "id" },
                expected_table: TableInput {
                    database: Implicit,
                    table: Sourced::from_input(0..5, SqlIdentifierInput { name: "table" }),
                },
            },
            Example {
                input: "table | s: table.id",
                expected_column: SqlIdentifierInput { name: "id" },
                expected_table: TableInput {
                    table: Sourced::from_input(11..16, SqlIdentifierInput { name: "table" }),
                    database: Implicit,
                },
            },
            Example {
                input: "table | s: db.table.id",
                expected_column: SqlIdentifierInput { name: "id" },
                expected_table: TableInput {
                    table: Sourced::from_input(14..19, SqlIdentifierInput { name: "table" }),
                    database: Specified(Sourced::from_input(
                        11..13,
                        SqlIdentifierInput { name: "db" },
                    )),
                },
            },
        ];

        for example in examples {
            let Example {
                input,
                expected_column,
                expected_table,
            } = example;
            let output = parse_to_stage4(input).unwrap();

            assert_eq!(1, output.selected_columns.len());
            assert!(matches!(
                &output.selected_columns[0],
                Sourced{ it: Stage4ComputationInput::Column(column), ..}
                    if column.it.column.it == expected_column && column.it.table.it == expected_table
            ));
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
            let output = parse_to_stage4(input).unwrap();
            let from = output.from;

            assert_eq!(expected_table, from.it.table.it.name, "Parsing: {}", input);
            assert_eq!(expected_db, from.it.database, "Parsing: {}", input);
        }
    }

    #[test]
    fn test_multiple_selects() {
        let output = parse_to_stage4("table | s: id | s: id2").unwrap();

        assert_eq!(2, output.selected_columns.len());
    }

    #[test]
    fn test_selecting_multiple_columns() {
        let output = parse_to_stage4("table | s: id id2").unwrap();

        assert_eq!(2, output.selected_columns.len());
    }

    impl PartialEq<OptionalInput<Sourced<SqlIdentifierInput<'_>>>> for &str {
        fn eq(&self, other: &OptionalInput<Sourced<SqlIdentifierInput<'_>>>) -> bool {
            match other {
                Implicit => false,
                Specified(value) => &value.it.name == other,
            }
        }
    }

    impl PartialEq<OptionalInput<Sourced<SqlIdentifierInput<'_>>>> for OptionalInput<&str> {
        fn eq(&self, other: &OptionalInput<Sourced<SqlIdentifierInput<'_>>>) -> bool {
            match (self, other) {
                (Implicit, Implicit) => true,
                (Specified(left), Specified(right)) => left == &right.it.name,
                _ => false,
            }
        }
    }

    impl PartialEq<Source> for Range<usize> {
        fn eq(&self, other: &Source) -> bool {
            match other {
                Source::Implicit => false,
                Source::Input(position) => self == position,
            }
        }
    }
}
