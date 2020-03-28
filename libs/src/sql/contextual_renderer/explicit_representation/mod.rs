//! The Query parameter renderers receive does not contain enough
//! information to render a query.
//! This module takes care of stuff like:
//!     - determining if we will select 'table.column' or just 'column'
//!     - figuring out how to exactly to do joins
use crate::common::{BinaryFilterType, UnaryFilterType};
use crate::query::*;
use crate::sql::structure::{Column, ForeignKey, Table};
use join_finder::JoinFinder;
use log::info;

mod join_finder;

#[derive(Debug)]
pub struct ExplicitQuery<'a> {
    pub selections: Vec<ExplicitResultColumn<'a>>,
    pub from: &'a str,
    pub joins: Vec<ExplicitJoin<'a>>,
    pub filters: Vec<ExplicitFilter<'a>>,
    pub group_by: Vec<ExplicitResultColumn<'a>>,
    pub order: Vec<ExplicitOrder<'a>>,
    pub limit: usize,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExplicitResultColumn<'a> {
    Value(&'a str),
    Column(ExplicitColumn),
    FunctionCall(&'a str, ExplicitColumn),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ExplicitColumn {
    Simple(String),
    FullyQualified(String, String),
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExplicitOperand<'a> {
    Column(ExplicitColumn),
    Value(&'a str),
}

impl<'a> ExplicitColumn {
    pub fn is_wildcard_of(&self, table: &str) -> bool {
        use ExplicitColumn::*;

        let column = match self {
            Simple(column) => column,
            FullyQualified(self_table, column) if self_table == table => column,
            _ => return false,
        };

        column == "*"
    }

    pub fn table_is(&self, table: &str) -> bool {
        match self {
            ExplicitColumn::Simple(_) => true,
            ExplicitColumn::FullyQualified(self_table, _) if self_table == table => true,
            _ => false,
        }
    }

    fn column_names_match(a: &str, b: &str) -> bool {
        a == "*" || b == "*" || a == b
    }

    #[cfg(test)]
    fn is_simple(&self) -> bool {
        match self {
            ExplicitColumn::Simple(_) => true,
            _ => false,
        }
    }

    #[cfg(test)]
    fn is_explicit(&self) -> bool {
        !self.is_simple()
    }
}

impl PartialEq<QualifiedColumnIdentifier> for ExplicitColumn {
    fn eq(&self, other: &QualifiedColumnIdentifier) -> bool {
        use ExplicitColumn::*;

        match self {
            Simple(column) => Self::column_names_match(column, &other.column),
            FullyQualified(table, column) if table == &other.table => {
                Self::column_names_match(column, &other.column)
            }
            _ => false,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExplicitFilter<'a> {
    Unary(ExplicitResultColumn<'a>, UnaryFilterType),
    Binary(
        ExplicitResultColumn<'a>,
        ExplicitResultColumn<'a>,
        BinaryFilterType,
    ),
}

#[derive(Debug)]
pub struct ExplicitJoin<'a> {
    pub from_table: &'a str,
    pub from_column: &'a str,
    pub to_table: &'a str,
    pub to_column: &'a str,
}

impl ExplicitJoin<'_> {
    pub fn for_fk<'a>(table: &'a str, fk: &'a ForeignKey) -> ExplicitJoin<'a> {
        ExplicitJoin {
            from_table: table,
            from_column: fk.from_column.0.as_ref(),
            to_table: fk.to_table.0.as_ref(),
            to_column: fk.to_column.0.as_ref(),
        }
    }

    pub fn reversed(self) -> Self {
        ExplicitJoin {
            from_table: self.to_table,
            from_column: self.to_column,
            to_table: self.from_table,
            to_column: self.from_column,
        }
    }
}

#[derive(Debug)]
pub enum ExplicitOrder<'a> {
    Ascending(ExplicitOperand<'a>),
    Descending(ExplicitOperand<'a>),
}

pub struct ExplicitQueryBuilder<'t> {
    tables: &'t [Table],
    working_with_single_table: bool,
}

impl<'t> ExplicitQueryBuilder<'t> {
    pub fn new(tables: &[Table]) -> ExplicitQueryBuilder {
        ExplicitQueryBuilder {
            tables,
            working_with_single_table: false,
        }
    }

    pub fn make_explicit_query(&mut self, query: &'t Query) -> Result<ExplicitQuery<'t>, String> {
        info!("Preparing query for rendering");

        self.working_with_single_table = query.joins.is_empty();

        let joins = self.translate_joins(&query.from[..], &query.joins[..])?;
        let selections = self.translate_selection(&query.selections[..], &query.unselections[..]);

        Ok(ExplicitQuery {
            selections,
            from: query.from.as_ref(),
            joins,
            filters: self.translate_filters(&query.filters[..]),
            group_by: self.translate_group_by(&query.group_by[..]),
            order: self.translate_orders(&query.order[..]),
            limit: query.limit,
        })
    }

    fn translate_selection(
        &self,
        selections: &'t [ResultColumn],
        unselections: &'t [ResultColumn],
    ) -> Vec<ExplicitResultColumn<'t>> {
        let selections = selections
            .iter()
            .map(|select| self.make_results_column(select))
            .collect::<Vec<_>>();

        unselections
            .iter()
            .fold(selections, |selections, unselect| {
                self.process_unselection(selections, unselect)
            })
            .into_iter()
            .collect()
    }

    fn process_unselection(
        &self,
        selections: Vec<ExplicitResultColumn<'t>>,
        unselect: &'t ResultColumn,
    ) -> Vec<ExplicitResultColumn<'t>> {
        let full_selections = self.expand_wildcard(selections, unselect);

        full_selections
            .into_iter()
            .filter(|select| select != unselect)
            .collect()
    }

    fn expand_wildcard(
        &self,
        selections: Vec<ExplicitResultColumn<'t>>,
        unselect: &'t ResultColumn,
    ) -> Vec<ExplicitResultColumn<'t>> {
        let unselect_column = if let ResultColumn::Column(column) = unselect {
            column
        } else {
            return selections;
        };

        let found_wildcard = selections
            .iter()
            .filter_map(|selection| match selection {
                ExplicitResultColumn::Column(column) => Some(column),
                _ => None,
            })
            .find(|select| select.is_wildcard_of(&unselect_column.table))
            .is_some();

        if found_wildcard {
            self.expand_selections(selections, &unselect_column.table)
        } else {
            selections
        }
    }

    fn expand_selections(
        &self,
        selections: Vec<ExplicitResultColumn<'t>>,
        table: &'t str,
    ) -> Vec<ExplicitResultColumn<'t>> {
        selections
            .iter()
            .flat_map(|selection| match selection {
                ExplicitResultColumn::Column(column) if column.table_is(table) => {
                    self.get_columns_of(table)
                }
                _ => vec![selection.clone()],
            })
            .fold(Vec::new(), |mut deduplicated, selection| {
                if !deduplicated.contains(&selection) {
                    deduplicated.push(selection);
                }

                deduplicated
            })
    }

    fn translate_filters(&self, filters: &'t [Filter]) -> Vec<ExplicitFilter<'t>> {
        filters
            .iter()
            .map(|filter| self.translate_filter(filter))
            .collect()
    }

    fn translate_filter(&self, filter: &'t Filter) -> ExplicitFilter<'t> {
        match filter {
            Filter::Unary(operand, filter_type) => {
                let operand = self.make_results_column(operand);

                ExplicitFilter::Unary(operand, *filter_type)
            }
            Filter::Binary(lhs, rhs, filter_type) => {
                let lhs = self.make_results_column(lhs);
                let rhs = self.make_results_column(rhs);

                ExplicitFilter::Binary(lhs, rhs, *filter_type)
            }
        }
    }

    fn translate_joins(
        &self,
        from: &'t str,
        joins: &'t [String],
    ) -> Result<Vec<ExplicitJoin<'t>>, String> {
        // queries without a from: operation can just skip this step
        if from.is_empty() {
            return Ok(Vec::with_capacity(0));
        }

        self.ensure_all_join_tables_exist(from, joins)?;

        let finder = JoinFinder::new(&self.tables[..]);
        let to: Vec<_> = joins
            .iter()
            .map(|table_name| table_name.as_ref())
            .rev()
            .collect();

        Ok(finder.find(from, to.as_ref())?)
    }

    fn translate_group_by(&self, groups: &'t [ResultColumn]) -> Vec<ExplicitResultColumn<'t>> {
        groups
            .iter()
            .map(|group| self.make_results_column(group))
            .collect()
    }

    fn translate_orders(&self, orders: &'t [Order]) -> Vec<ExplicitOrder<'t>> {
        orders
            .iter()
            .map(|order| self.translate_order(order))
            .collect()
    }

    fn translate_order(&self, order: &'t Order) -> ExplicitOrder<'t> {
        let operand = match order {
            Order::Ascending(operand) | Order::Descending(operand) => self.make_operand(operand),
        };

        match order {
            Order::Ascending(_) => ExplicitOrder::Ascending(operand),
            Order::Descending(_) => ExplicitOrder::Descending(operand),
        }
    }

    fn make_operand(&self, operand: &'t Operand) -> ExplicitOperand<'t> {
        match operand {
            Operand::Column(column) => ExplicitOperand::Column(self.make_explicit_column(column)),
            Operand::Value(value) => ExplicitOperand::Value(value.as_ref()),
        }
    }

    fn make_results_column(&self, select: &'t ResultColumn) -> ExplicitResultColumn<'t> {
        match select {
            ResultColumn::Value(value) => ExplicitResultColumn::Value(value.as_ref()),
            ResultColumn::Column(column) => {
                ExplicitResultColumn::Column(self.make_explicit_column(column))
            }
            ResultColumn::FunctionCall(function_name, column) => {
                ExplicitResultColumn::FunctionCall(
                    function_name.as_str(),
                    self.make_explicit_column(column),
                )
            }
        }
    }

    fn make_explicit_column(&self, column: &'t QualifiedColumnIdentifier) -> ExplicitColumn {
        if self.working_with_single_table {
            ExplicitColumn::Simple(column.column.clone())
        } else {
            ExplicitColumn::FullyQualified(column.table.clone(), column.column.clone())
        }
    }

    fn get_columns_of(&self, table_name: &str) -> Vec<ExplicitResultColumn<'t>> {
        self.tables
            .iter()
            .filter(|table| table.name == table_name)
            .flat_map(|table| &table.columns)
            .map(|column| self.make_selection_column(table_name, column))
            .collect()
    }

    fn make_selection_column(&self, table_name: &str, column: &Column) -> ExplicitResultColumn<'t> {
        ExplicitResultColumn::Column(if self.working_with_single_table {
            ExplicitColumn::Simple(column.name.clone())
        } else {
            ExplicitColumn::FullyQualified(table_name.to_string(), column.name.clone())
        })
    }

    /// Knowing if we can't find a table because it's misspelled or because it doesn't exist can
    /// make working with queries much simpler.
    fn ensure_all_join_tables_exist(&self, from: &str, joins: &[String]) -> Result<(), String> {
        self.ensure_table_exists(from)?;
        joins
            .iter()
            .map(|join| self.ensure_table_exists(join))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    fn ensure_table_exists(&self, table_name: &str) -> Result<(), String> {
        let exists = self.tables.iter().any(|table| table.name == table_name);

        if exists {
            Ok(())
        } else {
            let all_tables = self
                .tables
                .iter()
                .map(|table| table.name.as_ref())
                .filter(|existing_table_name| {
                    strsim::normalized_damerau_levenshtein(table_name, existing_table_name) > 0.75
                })
                .collect::<Vec<_>>();

            let message = if all_tables.is_empty() {
                format!("Table {} not found.", table_name)
            } else {
                format!(
                    "Table {} not found, try: {}",
                    table_name,
                    all_tables.join(", ")
                )
            };

            Err(message)
        }
    }
}

impl PartialEq<ResultColumn> for ExplicitResultColumn<'_> {
    fn eq(&self, other: &ResultColumn) -> bool {
        use ExplicitResultColumn as S; // for self
        use ResultColumn as O; // for other

        match (self, other) {
            (S::Column(column), O::Column(other)) => column == other,
            (S::FunctionCall(s_function, s_column), O::FunctionCall(o_function, o_column)) => {
                s_function == o_function && s_column == o_column
            }
            (S::Value(s_value), O::Value(o_value)) => s_value == o_value,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_build_simple_selections() {
        let selections = vec![
            ResultColumn::Column(QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column1".into(),
            }),
            ResultColumn::Column(QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column2".into(),
            }),
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: true,
        };

        let better_selections = builder.translate_selection(&selections[..], &[]);

        assert_eq!(2, better_selections.len());
        assert_eq!(
            ExplicitColumn::Simple("column1".to_string()),
            better_selections[0]
        );
        assert_eq!(
            ExplicitColumn::Simple("column2".to_string()),
            better_selections[1]
        );
    }

    #[test]
    fn can_build_complex_selections() {
        let selections = vec![
            ResultColumn::Column(QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column1".into(),
            }),
            ResultColumn::Column(QualifiedColumnIdentifier {
                table: "friends".into(),
                column: "column2".into(),
            }),
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: false,
        };

        let better_selections = builder.translate_selection(&selections[..], &[]);

        assert_eq!(2, better_selections.len());
        assert_eq!(
            ExplicitColumn::FullyQualified("users".to_string(), "column1".to_string()),
            better_selections[0]
        );
        assert_eq!(
            ExplicitColumn::FullyQualified("friends".to_string(), "column2".to_string()),
            better_selections[1]
        );
    }

    #[test]
    fn can_build_simple_filters() {
        let filters = vec![
            Filter::Binary(
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                }),
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                }),
                BinaryFilterType::Equals,
            ),
            Filter::Binary(
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column2".into(),
                }),
                ResultColumn::Value("3".to_owned()),
                BinaryFilterType::Equals,
            ),
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: true,
        };

        let better_filters = builder.translate_filters(&filters[..]);

        assert_eq!(2, better_filters.len());
        assert!(better_filters[0].rhs().as_column().is_simple());
        assert!(better_filters[1].rhs().as_column().is_simple());
    }

    #[test]
    fn can_build_complex_filters() {
        let filters = vec![
            Filter::Binary(
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                }),
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                }),
                BinaryFilterType::Equals,
            ),
            Filter::Binary(
                ResultColumn::Column(QualifiedColumnIdentifier {
                    table: "friends".into(),
                    column: "column2".into(),
                }),
                ResultColumn::Value("3".to_owned()),
                BinaryFilterType::Equals,
            ),
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: false,
        };

        let better_filters = builder.translate_filters(&filters[..]);

        assert_eq!(2, better_filters.len());
        assert!(better_filters[0].rhs().as_column().is_explicit());
        assert!(better_filters[1].rhs().as_column().is_explicit());
    }

    #[test]
    fn can_build_direct_joins() {
        let tables = vec![
            Table {
                name: "users".to_owned(),
                columns: vec!["id".into(), "name".into()],
                foreign_keys: Vec::new(),
            },
            Table {
                name: "friends".to_owned(),
                columns: vec!["id".into(), "userId".into(), "name".into()],
                foreign_keys: vec![(&("userId", ("users", "id"))).into()],
            },
        ];
        let joins = vec!["friends".to_owned()];
        let builder = ExplicitQueryBuilder {
            tables: &tables[..],
            working_with_single_table: false,
        };

        let better_joins = builder.translate_joins("users", &joins[..]);

        assert_eq!(
            ExplicitJoin::new("users", "id", "friends", "userId"),
            better_joins.unwrap()[0]
        );
    }

    #[test]
    fn can_build_order() {
        let orders = vec![
            Order::Ascending(Operand::Column(QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column1".into(),
            })),
            Order::Descending(Operand::Value("3".to_owned())),
        ];

        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: false,
        };

        let better_orders = builder.translate_orders(&orders[..]);

        assert_eq!(2, better_orders.len());
    }

    // this is used when testing if we can find joins, the might fail when chaning the find*
    // methods if we use the naive implementation
    impl PartialEq for ExplicitJoin<'_> {
        fn eq(&self, other: &Self) -> bool {
            return (self.from_table, self.from_column, self.to_table, self.to_column) ==  // are the same
                (other.from_table, other.from_column, other.to_table, other.to_column)
                || (self.to_table, self.to_column, self.from_table, self.from_column) ==     // are reversed
                (other.from_table, other.from_column, other.to_table, other.to_column);
        }
    }

    impl PartialEq<ExplicitResultColumn<'_>> for ExplicitColumn {
        fn eq(&self, other: &ExplicitResultColumn) -> bool {
            match other {
                ExplicitResultColumn::Column(column) => self == column,
                _ => false,
            }
        }
    }

    impl Eq for ExplicitJoin<'_> {}

    impl ExplicitJoin<'_> {
        pub fn new<'a>(
            from_table: &'a str,
            from_column: &'a str,
            to_table: &'a str,
            to_column: &'a str,
        ) -> ExplicitJoin<'a> {
            ExplicitJoin {
                from_table,
                from_column,
                to_table,
                to_column,
            }
        }
    }

    impl ExplicitResultColumn<'_> {
        pub fn as_column(&self) -> &ExplicitColumn {
            match self {
                ExplicitResultColumn::Column(column) => column,
                _ => panic!("Can't use operand as column"),
            }
        }
    }

    impl ExplicitFilter<'_> {
        pub fn rhs(&self) -> &ExplicitResultColumn {
            #[allow(unreachable_patterns)]
            match self {
                ExplicitFilter::Binary(rhs, _, _) => rhs,
                _ => panic!("Filter doesn't have a rhs"),
            }
        }
    }
}
