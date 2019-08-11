//! The Query parameter renderers receive does not contain enough
//! information to render a query.
//! This module takes care of stuff like:
//!     - determining if we will select 'table.column' or just 'column'
//!     - figuring out how to exactly to do joins
use crate::query::*;
use crate::sql::structure::{ForeignKey, Table};
use join_finder::JoinFinder;
use log::info;

mod join_finder;

#[derive(Debug)]
pub struct ExplicitQuery<'a> {
    pub selections: Vec<ExplicitColumn<'a>>,
    pub from: &'a str,
    pub joins: Vec<ExplicitJoin<'a>>,
    pub filters: Vec<ExplicitFilter<'a>>,
    pub limit: usize,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ExplicitColumn<'a> {
    Simple(&'a str),
    FullyQualified(&'a str, &'a str),
}

impl<'a> ExplicitColumn<'a> {
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

#[derive(PartialEq, Eq, Debug)]
pub struct ExplicitFilter<'a> {
    pub column: ExplicitColumn<'a>,
    pub condition: &'a Condition,
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

        Ok(ExplicitQuery {
            selections: self.translate_selection(&query.selections[..]),
            from: query.from.as_ref(),
            joins,
            filters: self.translate_filters(&query.filters[..]),
            limit: query.limit,
        })
    }

    fn translate_selection(
        &self,
        selections: &'t [QualifiedColumnIdentifier],
    ) -> Vec<ExplicitColumn<'t>> {
        selections
            .iter()
            .map(|select| self.make_explicit_column(select))
            .collect()
    }

    fn translate_filters(&self, selections: &'t [Filter]) -> Vec<ExplicitFilter<'t>> {
        selections
            .iter()
            .map(|filter| {
                let column = self.make_explicit_column(&filter.column);

                ExplicitFilter {
                    column,
                    condition: &filter.condition,
                }
            })
            .collect()
    }

    fn translate_joins(
        &self,
        from: &'t str,
        joins: &'t [String],
    ) -> Result<Vec<ExplicitJoin<'t>>, String> {
        self.ensure_all_join_tables_exist(from, joins)?;

        let finder = JoinFinder::new(&self.tables[..]);
        let to: Vec<_> = joins.iter().map(|table_name| table_name.as_ref()).collect();

        Ok(finder.find(from, to.as_ref())?)
    }

    fn make_explicit_column(&self, column: &'t QualifiedColumnIdentifier) -> ExplicitColumn<'t> {
        if self.working_with_single_table {
            ExplicitColumn::Simple(column.column.as_ref())
        } else {
            ExplicitColumn::FullyQualified(column.table.as_ref(), column.column.as_ref())
        }
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
                .filter(|existing_table_name| strsim::normalized_damerau_levenshtein(table_name, existing_table_name) > 0.75)
                .collect::<Vec<_>>();

            let message = if all_tables.is_empty() {
                format!("Table {} not found.", table_name)
            } else {
                format!(
                    "Table {} not found, try: {}",
                    table_name, all_tables.join(", ")
                )
            };

            Err(message)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_build_simple_selections() {
        let selections = vec![
            QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column1".into(),
            },
            QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column2".into(),
            },
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: true,
        };

        let better_selections = builder.translate_selection(&selections[..]);

        assert_eq!(2, better_selections.len());
        assert_eq!(ExplicitColumn::Simple("column1"), better_selections[0]);
        assert_eq!(ExplicitColumn::Simple("column2"), better_selections[1]);
    }

    #[test]
    fn can_build_complex_selections() {
        let selections = vec![
            QualifiedColumnIdentifier {
                table: "users".into(),
                column: "column1".into(),
            },
            QualifiedColumnIdentifier {
                table: "friends".into(),
                column: "column2".into(),
            },
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: false,
        };

        let better_selections = builder.translate_selection(&selections[..]);

        assert_eq!(2, better_selections.len());
        assert_eq!(
            ExplicitColumn::FullyQualified("users", "column1"),
            better_selections[0]
        );
        assert_eq!(
            ExplicitColumn::FullyQualified("friends", "column2"),
            better_selections[1]
        );
    }

    #[test]
    fn can_build_simple_filters() {
        let filters = vec![
            Filter {
                column: QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                },
                condition: Condition::Equals("3".to_owned()),
            },
            Filter {
                column: QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column2".into(),
                },
                condition: Condition::Equals("3".to_owned()),
            },
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: true,
        };

        let better_filters = builder.translate_filters(&filters[..]);

        assert_eq!(2, better_filters.len());
        assert!(better_filters[0].column.is_simple());
        assert!(better_filters[1].column.is_simple());
    }

    #[test]
    fn can_build_complex_filters() {
        let filters = vec![
            Filter {
                column: QualifiedColumnIdentifier {
                    table: "users".into(),
                    column: "column1".into(),
                },
                condition: Condition::Equals("3".to_owned()),
            },
            Filter {
                column: QualifiedColumnIdentifier {
                    table: "friends".into(),
                    column: "column2".into(),
                },
                condition: Condition::Equals("3".to_owned()),
            },
        ];
        let builder = ExplicitQueryBuilder {
            tables: &[],
            working_with_single_table: false,
        };

        let better_filters = builder.translate_filters(&filters[..]);

        assert_eq!(2, better_filters.len());
        assert!(better_filters[0].column.is_explicit());
        assert!(better_filters[1].column.is_explicit());
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
}
