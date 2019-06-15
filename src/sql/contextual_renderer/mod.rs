mod sql_reflect;
mod structure;

use super::Renderer;
use super::renderer::{ render_select, render_from, render_filters };
use crate::query::Query;
use structure::*;
use crate::error::PineError;

struct SmartRenderer {
    tables: Vec<Table>,
}

impl Renderer<Query, String> for &SmartRenderer {
    // TODO return RESULT<_, _>
    fn render(self, query: &Query) -> Result<String, PineError> {
        let select = render_select(&query);
        let from = render_from(&query);
        let joins = self.render_joins(&query);
        let filters = render_filters(&query);

        Ok(format!(
            "SELECT {}\nFROM {}\n{}\nWHERE {}",
            select,
            from,
            joins,
            filters
        ))
    }
}

impl SmartRenderer {
    pub fn for_tables(tables: Vec<Table>) -> SmartRenderer {
        SmartRenderer { tables }
    }

    fn render_joins(&self, query: &Query) -> String {
        let joins = query
            .joins
            .iter()
            .map(|table_name| {
                let left_table = "users";
                let (left_column, right_table, right_column) = self.find_foreign_key(table_name).unwrap();

                format!("LEFT JOIN friends ON {}.{} = {}.{}", left_table, left_column, right_table, right_column)
            })
            .collect::<Vec<_>>()
            .join("\n");

        joins
    }

    fn find_foreign_key(&self, to_table: &str) -> Result<ShortHandForeignKey, String> {
        let find_fk = self
            .tables
            .iter()
            .find_map(|table| {
                table.foreign_keys.iter()
                    .find(|foreign_key| foreign_key.to_table == to_table)
            });

        match find_fk {
            Some(ref fk) => Ok((
                &fk.from_column.0,
                &fk.to_table.0,
                &fk.to_column.0,
            )),
            None => Err("Couldn't find foreign key".to_string()),
        }
    }
}

type ShortHandForeignKey<'a> = (&'a str, &'a str, &'a str);

#[cfg(test)]
mod tests {
    use crate::sql::shorthand::*;
    use super::*;

    // TODO add error tests
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
