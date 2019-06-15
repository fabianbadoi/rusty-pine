mod sql_reflect;
mod structure;

use super::Renderer;
use super::renderer::{ render_select, render_from, render_filters };
use crate::query::Query;
use structure::*;
use crate::error::PineError;

pub struct SmartRenderer {
    tables: Vec<Table>,
}

impl Renderer<Query, String> for &SmartRenderer {
    fn render(self, query: &Query) -> Result<String, PineError> {
        let render_operation = RenderOperation::new(&self.tables, query);

        render_operation.render()
    }
}

impl SmartRenderer {
    pub fn for_tables(tables: Vec<Table>) -> SmartRenderer {
        SmartRenderer { tables }
    }
}

struct RenderOperation<'a> {
    tables: &'a [Table],
    last_table: &'a str,
    query: &'a Query,
}

impl<'a> RenderOperation<'a> {
    pub fn new(tables: &'a [Table], query: &'a Query) -> RenderOperation<'a> {
        RenderOperation {
            tables,
            query,
            last_table: &query.from
        }
    }

    pub fn render(mut self) -> Result<String, PineError> {
        let select = render_select(self.query);
        let from = render_from(self.query);
        let joins = self.render_joins()?;
        let filters = render_filters(self.query);

        Ok(format!(
            "SELECT {}\nFROM {}\n{}\nWHERE {}",
            select,
            from,
            joins,
            filters
        ))
    }

    fn render_joins(&mut self) -> Result<String, String> {
        let mut joins: Vec<String> = Vec::new();

        for join_table in &self.query.joins {
            joins.push(self.render_join(self.last_table, join_table)?);

            self.last_table = join_table;
        }

        Ok(joins.join("\n"))
    }

    fn render_join(&self, left_table: &str, join_table: &str) -> Result<String, String> {
        let (left_column, right_column) = self.find_foreign_key_columns(join_table)?;

        Ok(format!("LEFT JOIN friends ON {}.{} = {}.{}", left_table, left_column, join_table, right_column))
    }

    fn find_foreign_key_columns(&self, to_table: &str) -> Result<(&str, &str), String> {
        let table = self.get_last_table()?;
        let find_fk = table.get_foreign_key(to_table);

        match find_fk {
            Some(ref fk) => Ok((&fk.from_column.0, &fk.to_column.0)),
            None => Err(self.make_cannot_find_fk_error(to_table)),
        }
    }

    fn get_last_table(&self) -> Result<&'a Table, String> {
        let find_table = self.tables
            .iter()
            .find(|table| {
                // maybe having a HashMap instead of a vector would be better, but tables don't
                // usually have that much data
                table.name == self.last_table
            });

        match find_table {
            Some(ref table) => Ok(table),
            None => Err(self.make_cannot_find_table_error()),
        }
    }

    fn make_cannot_find_table_error(&self) -> String {
            format!(
                "Unkown table `{}`. Try:\n{}",
                self.last_table,
                self.tables.iter().map(|table| table.name.as_ref()).collect::<Vec<_>>().join("\n")
            )
    }

    fn make_cannot_find_fk_error(&self, to_table: &str) -> String {
        format!(
            "Couldn't find foreign key from `{}` to `{}`. Try:\n{}",
            self.last_table,
            to_table,
            self.get_last_table()
                .unwrap()
                .foreign_keys
                .iter()
                .map(|fk| (&fk.to_table).into())
                .collect::<Vec<&str>>()
                .join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::sql::shorthand::*;
    use super::*;

    #[test]
    fn smart_render() {
        let renderer = make_renderer();
        let query = make_join_query();

        let rendering = renderer.render(&query).unwrap();

        assert_eq!(
            "SELECT users.id, users.name\nFROM users\nLEFT JOIN friends ON users.friendId = friends.id\nWHERE users.id = \"1\" AND users.mojo = \"great\"",
            rendering
        );
    }

    #[test]
    fn join_to_unknown_table() {
        let renderer = make_renderer();
        let mut query = make_join_query();
        query.joins[0] = "missing".to_string();

        let error = renderer.render(&query).unwrap_err();

        assert_eq!(
            "Couldn't find foreign key from `users` to `missing`. Try:\nfriends",
            format!("{}", error)
        );
    }

    #[test]
    fn select_from_unknown_table() {
        let renderer = make_renderer();
        let mut query = make_join_query();
        query.from = "missing".to_string();

        let error = renderer.render(&query).unwrap_err();

        println!("{}", error);
        assert_eq!(
            "Unkown table `missing`. Try:\nusers\nfriends",
            format!("{}", error)
        );
    }

    fn make_join_query() -> Query {
        let query = QueryShorthand(
            Select(&["id", "name"]),
            From("users"),
            &[Filter::Equals("id", "1"), Filter::Equals("mojo", "great")],
        );
        let mut query: Query = query.into();
        query.joins.push("friends".to_string());

        query
    }

    fn make_renderer() -> SmartRenderer {
        let tables = vec![
            Table {
                name: "users".into(),
                columns: Vec::new(),
                foreign_keys: vec![
                    ForeignKey::from(&("friendId", ("friends", "id"))),
                ],
            },
            Table {
                name: "friends".into(),
                columns: Vec::new(),
                foreign_keys: Vec::new(),
            },
        ];

        SmartRenderer::for_tables(tables)
    }
}
