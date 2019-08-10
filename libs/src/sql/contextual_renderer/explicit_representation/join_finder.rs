use super::ExplicitJoin;
use crate::sql::structure::Table;
use std::error::Error;
use std::fmt::Display;
use log::info;

pub struct JoinFinder<'tables> {
    tables: &'tables [Table],
}

impl<'t> JoinFinder<'t> {
    pub fn new(tables: &'t [Table]) -> JoinFinder {
        JoinFinder { tables }
    }

    pub fn find(
        &self,
        from: &'t str,
        to: &[&'t str],
    ) -> Result<Vec<ExplicitJoin<'t>>, JoinsNotFound> {
        info!("Finding joins between {}, {:?}", from, to);

        /*
         * join priority:
         *  - previous table
         *  - shortest path (not relevant now)
         *  - direct path
         *  - reverse path
         *  - don't try to look for join posibilities with other tables
         *
         * How:
         * Having a list of joins: j1, j2, j3, etc.; we only want to join consecutive joins.
         * That means our joins will be (from, j1), (j1, j2), (j2, j3), etc.
         *
         * Here's another way of looking at it:
         *   join_sources:   from, j1, j2, [j3], j4, j5, ...
         *   join_targets:   j1,   j2, j3, [j4], j5, ...
         * Using [] to denote the join currently being processed: (j3, j4)
         */
        let from_as_array = [from]; // we need this to help the borrow checker;

        let join_targets = to.iter();
        let join_sources = from_as_array.iter().chain(to.iter());
        let join_table_pairs = join_sources.zip(join_targets);

        let joins = join_table_pairs.map(move |(table1, table2)| {
            self.find_join_for_tables(table1, table2)
                // use (t1, t2) as an error type so we can construct a nice error message
                .ok_or((*table1, *table2))
        });

        Self::potential_joins_to_result(joins)
    }

    fn find_join_for_tables(&self, table1: &'t str, table2: &'t str) -> Option<ExplicitJoin<'t>> {
        info!("Finding join {} to {}", table1, table2);

        let direct = self.find_direct_join_for(table1, table2);
        let direct_or_inverse = direct.or_else(move || {
            info!("Trying inverse: {} to {}", table2, table1);

            self.find_direct_join_for(table2, table1)
        });

        direct_or_inverse
    }

    fn find_direct_join_for(&self, source: &'t str, dest: &str) -> Option<ExplicitJoin<'t>> {
        self.tables
            .iter()
            .find(|table| table.name == source)
            .and_then(|table| table.get_foreign_key(dest))
            .map(|foreign_key| ExplicitJoin::for_fk(source, foreign_key))
    }

    fn potential_joins_to_result(
        potential_joins: impl Iterator<Item = IntermediateResult<'t>>,
    ) -> Result<Vec<ExplicitJoin<'t>>, JoinsNotFound> {
        let (joins, failed_joins): (Vec<_>, Vec<_>) = potential_joins.partition(Result::is_ok);

        if failed_joins.len() > 0 {
            Err(JoinsNotFound::new(failed_joins))
        } else {
            Ok(joins.into_iter().map(Result::unwrap).collect())
        }
    }
}

// TODO: add data about what joins ARE available?
#[derive(Debug, Clone)]
pub struct JoinsNotFound {
    joins: Vec<(String, String)>,
}

impl JoinsNotFound {
    fn new(joins: Vec<Result<ExplicitJoin, (&str, &str)>>) -> JoinsNotFound {
        let joins = joins
            .into_iter()
            .map(Result::unwrap_err)
            .map(|(table1, table2)| (table1.to_owned(), table2.to_owned()))
            .collect();

        JoinsNotFound { joins }
    }
}

impl Display for JoinsNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let failed_join_list = self
            .joins
            .iter()
            .map(|(table1, table2)| format!("- {} and {}", table1, table2))
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Can't figure out how to join these tables:\n{}",
            failed_join_list
        )
    }
}

impl Error for JoinsNotFound {}

impl From<JoinsNotFound> for String {
    fn from(other: JoinsNotFound) -> String {
        format!("{}", other)
    }
}

/// Err() is the two tables for which we did not find a join
type IntermediateResult<'a> = Result<ExplicitJoin<'a>, (&'a str, &'a str)>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_joins() {
        let from = "users";
        let to = [];
        let tables = make_debug_tables();
        let join_finder = JoinFinder::new(&tables[..]);

        let joins = join_finder.find(from, &to);

        assert!(joins.is_ok());
        assert!(joins.unwrap().is_empty());
    }

    #[test]
    fn simple_join() {
        let from = "users";
        let to = ["friends"];
        let tables = make_debug_tables();
        let join_finder = JoinFinder::new(&tables[..]);

        let joins = join_finder.find(from, &to);

        assert!(joins.is_ok());

        let joins = joins.unwrap();

        assert_eq!(joins.len(), 1);

        let expected_join = ExplicitJoin::new("users", "id", "friends", "user_id");
        assert_eq!(expected_join, joins[0]);
    }

    #[test]
    fn not_found() {
        let from = "users";
        let to = ["not_found1", "not_found2", "friends"];
        let tables = make_debug_tables();
        let join_finder = JoinFinder::new(&tables[..]);

        let joins = join_finder.find(from, &to);

        assert!(joins.is_err());

        let joins_not_found = joins.unwrap_err();

        // this will make sure we have nice error messages
        assert!(joins_not_found
            .joins
            .contains(&("users".to_owned(), "not_found1".to_owned())));
        assert!(
            joins_not_found
                .joins
                .contains(&("not_found1".to_owned(),
                "not_found2".to_owned()))
        );

        assert!(
            !joins_not_found
                .joins
                .contains(&("users".to_owned(), "friends".to_owned()))
        );
    }

    #[test]
    fn complex_join() {
        let from = "friends";
        let to = ["users", "customers", "customer_settings"];
        let tables = make_debug_tables();
        let join_finder = JoinFinder::new(&tables[..]);

        let joins = join_finder.find(from, &to);

        assert!(joins.is_ok());

        let joins = joins.unwrap();
        assert_eq!(joins.len(), 3);
        assert!(joins.contains(&ExplicitJoin::new("users", "id", "friends", "user_id")));
        assert!(joins.contains(&ExplicitJoin::new(
            "users",
            "customer_id",
            "customers",
            "id"
        )));
        assert!(joins.contains(&ExplicitJoin::new(
            "customers",
            "id",
            "customer_settings",
            "customer_id"
        )));
    }

    fn make_debug_tables() -> Vec<Table> {
        let tables = [
            (
                "users",
                [
                    ("id", ("friends", "user_id")),
                    ("customer_id", ("customers", "id")),
                ]
                .as_ref(),
            ),
            ("friends", &[]),
            ("customers", &[]),
            ("customer_settings", &[("customer_id", ("customers", "id"))]),
        ];

        tables.into_iter().map(make_table).collect()
    }

    fn make_table(proto: &(&str, &[(&str, (&str, &str))])) -> Table {
        Table {
            name: proto.0.into(),
            columns: Vec::new(),
            foreign_keys: proto.1.iter().map(From::from).collect(),
        }
    }
}
