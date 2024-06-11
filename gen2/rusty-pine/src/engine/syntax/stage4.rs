//! Stage 4 representation is a hierarchical structure representing what query the user constructed.
//! It will have all data present in the input, but will not have any data present that relates
//! to the actual state of the database:
//!     - how do to joins
//!     - can't tell if table is missing or name is mistyped
use crate::engine::syntax::stage3::{
    Stage3Condition, Stage3Order, Stage3Pine, Stage3Rep, Stage3Selectable,
};
use crate::engine::syntax::{SqlIdentifierInput, TableInput};
use crate::engine::{
    BinaryConditionHolder, ConditionHolder, JoinConditions, JoinType, LimitHolder, OrderHolder,
    SelectableHolder, UnaryConditionHolder,
};
use crate::engine::{LiteralValueHolder, Sourced};

pub struct Stage4Rep<'a> {
    pub input: &'a str,
    pub from: Sourced<TableInput<'a>>,
    pub filters: Vec<Sourced<Stage4Condition<'a>>>,
    pub joins: Vec<Sourced<Stage4Join<'a>>>,
    pub selected_columns: Vec<Sourced<Stage4Selectable<'a>>>,
    pub orders: Vec<Sourced<Stage4Order<'a>>>,
    pub group_by: Vec<Sourced<Stage4Selectable<'a>>>,
    pub limit: Sourced<Stage4Limit<'a>>,
}

pub type Stage4Selectable<'a> = SelectableHolder<Stage4Condition<'a>, Stage4ComputationInput<'a>>;
pub type Stage4Condition<'a> = ConditionHolder<Stage4ComputationInput<'a>>;
pub type Stage4BinaryCondition<'a> = BinaryConditionHolder<Stage4ComputationInput<'a>>;
pub type Stage4UnaryCondition<'a> = UnaryConditionHolder<Stage4ComputationInput<'a>>;
pub type Stage4Order<'a> = OrderHolder<Stage4Selectable<'a>>;
pub type Stage4Limit<'a> = LimitHolder<Stage4LiteralValue<'a>>;

#[derive(Clone, Debug)]
pub struct Stage4Join<'a> {
    pub join_type: Sourced<JoinType>,
    pub source_table: Sourced<TableInput<'a>>,
    /// The table to join to.
    pub target_table: Sourced<TableInput<'a>>,
    pub conditions: Stage4JoinConditions<'a>,
}

pub type Stage4JoinConditions<'a> = JoinConditions<Stage4Condition<'a>>;

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

impl<'a> From<Stage3Rep<'a>> for Stage4Rep<'a> {
    fn from(stage3: Stage3Rep<'a>) -> Self {
        let input = stage3.input;
        let mut from = None;
        let mut last_table = None;
        let mut selected_columns = Vec::new();
        let mut joins = Vec::new();
        let mut filters = Vec::new();
        let mut orders = Vec::new();
        let mut group_by = Vec::new();
        let mut limit = Sourced::implicit(LimitHolder::Implicit());

        for pine in stage3.pines {
            match pine.it {
                Stage3Pine::From { table, conditions } => {
                    assert!(
                        from.is_none(),
                        "Our pest syntax forbids multiple from statements"
                    );
                    from = Some(table);
                    last_table = Some(table);
                    filters.append(&mut translate_conditions(conditions));
                }
                Stage3Pine::Select(selectables) => {
                    // If we add a select, we drop any implicit wildcards so far. This is just
                    // what I find convenient.
                    purge_implicit_wildcards(
                        &mut selected_columns,
                        last_table.expect("Should have a table before any selects"),
                    );

                    selected_columns.append(&mut translate_selectables(selectables));
                }
                Stage3Pine::Limit(new_limit) => limit = new_limit.into(),
                Stage3Pine::Order(new_orders) => {
                    orders.append(&mut translate_orders(new_orders));
                }
                Stage3Pine::GroupBy(selectables) => {
                    let mut selectables = translate_selectables(selectables);
                    let mut group_selectables = selectables.clone();

                    if selected_columns.is_empty() {
                        // We add an implicit * wildcard select because I think it's convenient.
                        // This is just what I find convenient.
                        selectables.push(implicit_wildcard_select(
                            last_table.expect("pines must start with a from: pine"),
                        ))
                    }

                    selected_columns.append(&mut selectables);
                    group_by.append(&mut group_selectables);
                }
                Stage3Pine::Filter(conditions) => {
                    filters.append(&mut translate_conditions(conditions))
                }
                Stage3Pine::Join(join) => {
                    last_table = Some(join.it.target_table);
                    joins.push(join);
                }
            }
        }

        Stage4Rep {
            input,
            from: from.expect("Impossible: pines without a from are not valid pest syntax"),
            filters,
            joins,
            selected_columns,
            orders,
            group_by,
            limit,
        }
    }
}

fn implicit_wildcard_select(last_table: Sourced<TableInput>) -> Sourced<Stage4Selectable> {
    Sourced::implicit(Stage4Selectable::Computation(Sourced::implicit(
        Stage4ComputationInput::Column(Sourced::implicit(Stage4ColumnInput {
            table: last_table,
            column: Sourced::implicit(SqlIdentifierInput { name: "*" }),
        })),
    )))
}

fn purge_implicit_wildcards(
    selectables: &mut Vec<Sourced<Stage4Selectable>>,
    implicit_table: Sourced<TableInput>,
) {
    selectables.retain(|selectable| {
        let comp = match &selectable.it {
            Stage4Selectable::Computation(comp) => comp,
            _ => return true,
        };

        let column = match comp.it {
            Stage4ComputationInput::Column(column) => column,
            _ => return true,
        };

        !(column.it.table == implicit_table && column.it.column.it.name == "*")
    })
}

fn translate_selectables(
    selectables: Vec<Sourced<Stage3Selectable>>,
) -> Vec<Sourced<Stage4Selectable>> {
    selectables
}

fn translate_conditions(
    conditions: Vec<Sourced<Stage3Condition>>,
) -> Vec<Sourced<Stage4Condition>> {
    conditions
}

fn translate_orders(conditions: Vec<Sourced<Stage3Order>>) -> Vec<Sourced<Stage4Order>> {
    conditions
}

#[cfg(test)]
mod test {
    use crate::engine::syntax::stage1::parse_stage1;
    use crate::engine::syntax::stage2::Stage2Rep;
    use crate::engine::syntax::stage3::Stage3Rep;
    use crate::engine::syntax::stage4::Stage4Rep;
    use crate::engine::syntax::OptionalInput::{Implicit, Specified};
    use crate::engine::syntax::{
        parse_to_stage4, OptionalInput, SqlIdentifierInput, Stage4ComputationInput,
        Stage4Selectable, TableInput,
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
                Sourced{ it: Stage4Selectable::Computation(Sourced { it:
                    Stage4ComputationInput::Column(column), ..},), ..}
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
                Source::Input(position) => self == position,
                _ => false,
            }
        }
    }
}
